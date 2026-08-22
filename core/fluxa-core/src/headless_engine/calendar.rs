use super::helpers::{active_profile_id, normalize_error};
use super::state::GenerationKey;
use super::{EffectResultInput, HeadlessEngine};
use crate::runtime::{EffectEnvelope, EffectKind};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", default)]
pub(super) struct CalendarState {
    year: i32,
    month: i32,
    is_loading: bool,
    items: Value,
    local_items: Value,
    external_items: Value,
    error: Value,
    generation: u64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ReadCalendarMonthPayload {
    profile_id: String,
    profile: Value,
    year: i32,
    month: i32,
    planned_items: Value,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CalendarItemsPayload {
    profile: Value,
    items: Value,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ReplaceExternalContinueWatchingPayload {
    profile_id: Value,
    items: Value,
}

pub(super) fn dispatch(
    engine: &mut HeadlessEngine,
    profile: Option<Value>,
    year: i32,
    month: i32,
    planned_items: Option<Value>,
) -> Vec<EffectEnvelope> {
    let generation = engine.bump_generation(GenerationKey::Calendar);
    let profile_value = profile.unwrap_or_else(|| engine.state.profile.active.clone());
    let profile_id = active_profile_id(&engine.state, &profile_value);
    *engine.state.calendar = CalendarState {
        year,
        month,
        is_loading: true,
        items: serde_json::json!([]),
        local_items: Value::Null,
        external_items: Value::Null,
        error: Value::Null,
        generation,
    };
    vec![engine.effect(
        EffectKind::ReadCalendarMonth,
        generation,
        ReadCalendarMonthPayload {
            profile_id,
            profile: profile_value,
            year,
            month: month.clamp(1, 12),
            planned_items: planned_items.unwrap_or_else(|| serde_json::json!([])),
        },
    )]
}

pub(super) fn complete(
    engine: &mut HeadlessEngine,
    generation: u64,
    result: &EffectResultInput,
    effect: &EffectEnvelope,
) -> Vec<EffectEnvelope> {
    if generation != engine.state.runtime.get(GenerationKey::Calendar) {
        return vec![];
    }
    engine.state.calendar.is_loading = false;
    if result.status.is_ok() {
        let items = result
            .value
            .get("items")
            .cloned()
            .unwrap_or_else(|| result.value.clone());
        let local_items = result
            .value
            .get("localItems")
            .cloned()
            .unwrap_or_else(|| serde_json::json!([]));
        let external_items = result
            .value
            .get("externalItems")
            .cloned()
            .unwrap_or_else(|| serde_json::json!([]));
        engine.state.calendar.items = items.clone();
        engine.state.calendar.local_items = local_items;
        engine.state.calendar.external_items = external_items.clone();
        engine.state.calendar.error = Value::Null;
        let profile = effect
            .payload
            .get("profile")
            .cloned()
            .unwrap_or(Value::Null);
        let profile_id = effect
            .payload
            .get("profileId")
            .cloned()
            .unwrap_or(Value::Null);
        let mut follow_up = vec![
            engine.effect(
                EffectKind::UpdateCalendarWidget,
                generation,
                CalendarItemsPayload {
                    profile: profile.clone(),
                    items: items.clone(),
                },
            ),
            engine.effect(
                EffectKind::NotifyReleasedEpisodes,
                generation,
                CalendarItemsPayload { profile, items },
            ),
        ];
        if external_items.as_array().is_some_and(|i| !i.is_empty()) {
            follow_up.push(engine.effect(
                EffectKind::ReplaceExternalContinueWatching,
                generation,
                ReplaceExternalContinueWatchingPayload {
                    profile_id,
                    items: external_items,
                },
            ));
        }
        follow_up
    } else {
        engine.state.calendar.error = normalize_error(result.error.clone());
        vec![]
    }
}
