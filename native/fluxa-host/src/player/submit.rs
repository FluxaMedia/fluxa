use std::sync::mpsc::Receiver;

use fluxa_ui::{PanelRow, PlayerPanel, SegmentSpan, SkipKind, localized};
use serde_json::{Value, json};

use super::{PlayerSession, overlay};
use crate::RendererState;

const KINDS: [&str; 3] = ["intro", "recap", "outro"];
const PROVIDERS: [(&str, &str); 3] = [
    ("introdb", "introDbApiKey"),
    ("theintrodb", "theIntroDbApiKey"),
    ("skipdb", "skipDbApiKey"),
];
const MIN_SECONDS: f64 = 5.0;
const MAX_SECONDS: f64 = 300.0;
const MAX_OUTRO_SECONDS: f64 = 900.0;

#[derive(Default)]
pub(super) struct Submit {
    kind: usize,
    start: Option<f64>,
    end: Option<f64>,
    pending: Vec<Receiver<bool>>,
    sent: usize,
    accepted: usize,
    message: Option<&'static str>,
}

impl Submit {
    pub(super) fn poll(&mut self) {
        let mut accepted = 0;
        self.pending.retain(|receiver| match receiver.try_recv() {
            Ok(ok) => {
                accepted += usize::from(ok);
                false
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => true,
            Err(std::sync::mpsc::TryRecvError::Disconnected) => false,
        });
        self.accepted += accepted;
        if self.sent > 0
            && self.pending.is_empty()
            && self.message == Some("player.mark_segment_submitting")
        {
            self.message = Some(if self.accepted > 0 {
                "player.mark_segment_success"
            } else {
                "player.mark_segment_error_unknown"
            });
        }
    }

    pub(super) fn preview(&self, position: f64, duration: f64) -> Option<SegmentSpan> {
        if duration <= 0.0 {
            return None;
        }
        let start = self.start?;
        let end = self.end.unwrap_or(position).max(start);
        let kind = match KINDS[self.kind] {
            "intro" => SkipKind::Intro,
            "recap" => SkipKind::Recap,
            _ => SkipKind::Outro,
        };
        Some(SegmentSpan {
            start_fraction: (start / duration).clamp(0.0, 1.0) as f32,
            end_fraction: (end / duration).clamp(0.0, 1.0) as f32,
            kind,
        })
    }

    pub(super) fn model(&self, language: &str) -> PlayerPanel {
        let text = |key: &str| localized(key, language);
        let time = |value: Option<f64>| value.map(|seconds| fluxa_ui::format_time(seconds));
        let message = match self.message {
            Some(key) => text(key),
            None => text("player.mark_segment_help"),
        };
        PlayerPanel {
            title: text("player.mark_segment_title"),
            message: Some(message),
            rows: vec![
                PanelRow {
                    label: text(&format!("player.mark_segment_type_{}", KINDS[self.kind])),
                    value: None,
                    primary: false,
                    ..Default::default()
                },
                PanelRow {
                    label: text("player.mark_segment_start"),
                    value: time(self.start),
                    primary: false,
                    ..Default::default()
                },
                PanelRow {
                    label: text("player.mark_segment_stop"),
                    value: time(self.end),
                    primary: false,
                    ..Default::default()
                },
                PanelRow {
                    label: text("player.mark_segment_submit"),
                    value: None,
                    primary: true,
                    ..Default::default()
                },
            ],
            ..Default::default()
        }
    }
}

pub(super) fn open(player: &mut PlayerSession) {
    player.panel = Some(super::Panel::Submit(Submit::default()));
}

pub(super) fn activate_row(state: &mut RendererState, row: usize) {
    let Some(player) = state.player.as_mut() else {
        return;
    };
    let position = player.status.position;
    let Some(super::Panel::Submit(submit)) = player.panel.as_mut() else {
        return;
    };
    match row {
        0 => submit.kind = (submit.kind + 1) % KINDS.len(),
        1 => submit.start = Some(position),
        2 => submit.end = Some(position),
        3 => send(state),
        _ => {}
    }
}

fn send(state: &mut RendererState) {
    let Some(session) = state.session.as_ref() else {
        return;
    };
    let snapshot = session.snapshot();
    let keys: Vec<(&str, String)> = PROVIDERS
        .iter()
        .filter_map(|(provider, setting)| {
            let key = state.settings.str_value(setting)?.trim();
            (!key.is_empty()).then(|| (*provider, key.to_owned()))
        })
        .collect();
    let Some(player) = state.player.as_mut() else {
        return;
    };
    let context = overlay::lookup_context(player, &snapshot);
    let Some(super::Panel::Submit(submit)) = player.panel.as_mut() else {
        return;
    };
    let (Some(start), Some(end)) = (submit.start, submit.end) else {
        submit.message = Some("player.mark_segment_empty");
        return;
    };
    let limit = if KINDS[submit.kind] == "outro" {
        MAX_OUTRO_SECONDS
    } else {
        MAX_SECONDS
    };
    if end - start < MIN_SECONDS || end - start > limit {
        submit.message = Some("player.mark_segment_duration_error");
        return;
    }
    let Some(context) = context else {
        submit.message = Some("player.mark_segment_no_metadata");
        return;
    };
    let mut plans = Vec::new();
    for (provider, api_key) in keys {
        let mut input = json!({
            "provider": provider,
            "apiKey": api_key,
            "segmentType": KINDS[submit.kind],
            "startMs": (start * 1000.0) as i64,
            "endMs": (end * 1000.0) as i64,
        });
        for field in [
            "imdbId",
            "tmdbId",
            "mediaType",
            "season",
            "episode",
            "durationMs",
        ] {
            input[field] = context.get(field).cloned().unwrap_or(Value::Null);
        }
        if let Some(plan) = crate::core_value("playerSegmentsSubmitPlan", input) {
            plans.push(plan);
        }
    }
    if plans.is_empty() {
        submit.message = Some("player.mark_segment_error_invalid_key");
        return;
    }
    submit.sent = plans.len();
    submit.accepted = 0;
    submit.message = Some("player.mark_segment_submitting");
    submit.pending = plans
        .into_iter()
        .map(|plan| session.request_ok(plan))
        .collect();
}
