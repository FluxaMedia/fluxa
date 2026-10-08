use std::collections::HashSet;
use std::sync::{Arc, mpsc::Receiver};

use serde_json::{Map, Value, json};

use crate::{RendererState, Route};

#[derive(Default)]
pub(crate) struct Logos {
    fetched: Map<String, Value>,
    fetched_ids: Vec<Value>,
    requested: HashSet<String>,
    fetching: Option<Receiver<Vec<(String, String)>>>,
    targets: Vec<Value>,
}

impl Logos {
    pub(crate) fn inputs(&self) -> Arc<Value> {
        Arc::new(json!({"fetched": self.fetched, "ids": self.fetched_ids}))
    }

    pub(crate) fn set_targets(&mut self, targets: Vec<Value>) {
        for target in targets {
            self.add_target(target);
        }
    }

    fn add_target(&mut self, target: Value) {
        let Some(id) = target.get("id").and_then(Value::as_str) else {
            return;
        };
        if self.requested.contains(id)
            || self
                .targets
                .iter()
                .any(|queued| queued.get("id") == target.get("id"))
        {
            return;
        }
        self.targets.push(target);
    }

    pub(crate) fn fetched(&self) -> &Map<String, Value> {
        &self.fetched
    }
}

fn queue_focused(state: &mut RendererState) {
    if state.route != Route::Home
        || !(state.settings.bool_value("followFocusHero")
            || state.settings.bool_value("expandedPosters"))
    {
        return;
    }
    let Some(node) = state.ui.focused() else {
        return;
    };
    if !(fluxa_ui::NODE_CARD_BASE..fluxa_ui::NODE_PREVIEW_BASE).contains(&node) {
        return;
    }
    let Some(card) = state
        .home
        .card_at((node - fluxa_ui::NODE_CARD_BASE) as usize)
    else {
        return;
    };
    let (Some(id), Some(content_type)) = (card.id.clone(), card.item_type.clone()) else {
        return;
    };
    if card.logo_url.is_some() {
        return;
    }
    state
        .logos
        .add_target(json!({"id": id, "type": content_type}));
}

pub(crate) fn tick(state: &mut RendererState) {
    queue_focused(state);
    if let Some(receiver) = state.logos.fetching.as_ref() {
        let Ok(found) = receiver.try_recv() else {
            return;
        };
        state.logos.fetching = None;
        for (id, logo) in found {
            if !logo.is_empty() {
                state.logos.fetched.insert(id.clone(), Value::String(logo));
            }
            state.logos.fetched_ids.push(Value::String(id));
        }
        state.home.apply_logos(&state.logos.fetched);
        state.last_snapshot_revision = None;
        return;
    }
    if state.logos.targets.is_empty() {
        return;
    }
    let Some(session) = state.session.as_ref() else {
        return;
    };
    let targets: Vec<Value> = std::mem::take(&mut state.logos.targets)
        .into_iter()
        .take(6)
        .collect();
    for target in &targets {
        if let Some(id) = target.get("id").and_then(Value::as_str) {
            state.logos.requested.insert(id.to_owned());
        }
    }
    state.logos.fetching = Some(session.executor().fetch_logos(targets));
}
