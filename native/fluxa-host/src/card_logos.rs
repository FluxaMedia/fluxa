use std::collections::HashMap;
use std::sync::mpsc::{Receiver, TryRecvError};

use serde_json::{Value, json};

use crate::profiles::{read, write};
use crate::{RendererState, profile_language};

const STORAGE_KEY: &str = "card_logos";
const BATCH: usize = 8;

#[derive(Default)]
pub(crate) struct CardLogos {
    started: bool,
    receiver: Option<Receiver<Vec<(String, String)>>>,
}

fn stored(state: &RendererState) -> HashMap<String, String> {
    let Some(session) = state.session.as_ref() else {
        return HashMap::new();
    };
    read(session.storage(), STORAGE_KEY)
        .and_then(|value| serde_json::from_value(value).ok())
        .unwrap_or_default()
}

pub(crate) fn tick(state: &mut RendererState) {
    if state.session.is_none() {
        return;
    }
    if !state.card_logos.started {
        state.card_logos.started = true;
        fluxa_ui::enable_card_logos(stored(state));
    }
    if let Some(receiver) = state.card_logos.receiver.as_ref() {
        match receiver.try_recv() {
            Ok(entries) => {
                state.card_logos.receiver = None;
                let known = fluxa_ui::resolve_card_logos(entries);
                if let Some(session) = state.session.as_ref() {
                    write(session.storage(), STORAGE_KEY, &json!(known));
                }
                if let Some(gpu) = state.gpu.as_ref() {
                    gpu.egui_context.request_repaint();
                }
            }
            Err(TryRecvError::Empty) => return,
            Err(TryRecvError::Disconnected) => state.card_logos.receiver = None,
        }
    }
    let wanted = fluxa_ui::take_wanted_logos(BATCH);
    if wanted.is_empty() {
        return;
    }
    let Some(session) = state.session.as_ref() else {
        return;
    };
    let items = wanted
        .into_iter()
        .map(|(id, kind)| json!({"id": id, "type": kind}))
        .collect::<Vec<Value>>();
    let api_key = state
        .settings
        .str_value("tmdbApiKey")
        .unwrap_or_default()
        .trim()
        .to_owned();
    let language = profile_language(&session.active_profile());
    state.card_logos.receiver = Some(session.executor().fetch_logos(items, api_key, language));
}
