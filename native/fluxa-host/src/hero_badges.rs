use std::collections::{HashMap, HashSet};
use std::sync::mpsc::Receiver;

use fluxa_ui::SourceBadge;
use serde_json::Value;

use crate::{RendererState, Route};

#[derive(Default)]
pub(crate) struct HeroBadges {
    found: HashMap<String, Vec<SourceBadge>>,
    requested: HashSet<String>,
    fetching: Option<(String, Receiver<Option<Value>>)>,
    enabled: bool,
}

fn lines(stream: &Value, key: &str) -> String {
    stream
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

fn badges_of(stream: &Value) -> Vec<SourceBadge> {
    let video_size = stream
        .pointer("/behaviorHints/videoSize")
        .and_then(Value::as_i64);
    let filename = stream
        .pointer("/behaviorHints/filename")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let raw = format!(
        "{}\n{}\n{}\n{filename}",
        lines(stream, "name"),
        lines(stream, "description"),
        lines(stream, "title")
    );
    crate::stream_badges::badges_for(&raw, video_size).0
}

pub(crate) fn tick(state: &mut RendererState) {
    let enabled = state.settings.bool_value("streamBadgesEnabled");
    if enabled != state.heroes.enabled {
        state.heroes.enabled = enabled;
        state.heroes.found.clear();
        state.heroes.requested.clear();
        state.heroes.fetching = None;
        for slide in &mut state.home.hero_slides {
            slide.badges.clear();
        }
    }
    if !enabled || state.route != Route::Home {
        return;
    }
    if let Some((id, receiver)) = state.heroes.fetching.as_ref() {
        let Ok(stream) = receiver.try_recv() else {
            return;
        };
        let id = id.clone();
        state.heroes.fetching = None;
        let badges = stream.as_ref().map(badges_of).unwrap_or_default();
        state.heroes.found.insert(id, badges);
    }
    for slide in &mut state.home.hero_slides {
        if let Some(badges) = slide
            .item_id
            .as_ref()
            .and_then(|id| state.heroes.found.get(id))
            && slide.badges.len() != badges.len()
        {
            slide.badges = badges.clone();
        }
    }
    let Some(gpu) = state.gpu.as_ref() else {
        return;
    };
    let Some(slide) = fluxa_ui::active_home_hero(&gpu.egui_context, &state.home) else {
        return;
    };
    let (Some(id), Some(content_type)) = (slide.item_id.clone(), slide.item_type.clone()) else {
        return;
    };
    if state.heroes.requested.contains(&id) {
        return;
    }
    let Some(session) = state.session.as_ref() else {
        return;
    };
    state.heroes.requested.insert(id.clone());
    let receiver = session
        .executor()
        .fetch_hero_stream(id.clone(), content_type);
    state.heroes.fetching = Some((id, receiver));
}
