use super::HeadlessEngine;
use super::home;
use super::library;
use crate::constants::GUEST_PROFILE_ID;
use crate::types::Profile;
use serde::{Deserialize, Serialize};
use serde_json::Value;

// Ambient/shared state read by almost every feature module (active_profile_id is
// the fallback identity used whenever a dispatch doesn't carry an explicit profile).
// Unlike feature-owned state below, its fields are pub(super) for direct reads, but
// writes should go through activate()/update_active() so the home/library mirrors
// they keep in sync don't drift out from under those modules.
#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", default)]
pub(super) struct ProfileState {
    pub(super) active: Value,
    pub(super) active_profile_id: Value,
}

pub(super) fn activate(engine: &mut HeadlessEngine, profile: Profile) {
    let id = profile
        .id
        .as_deref()
        .filter(|value| !value.is_empty())
        .unwrap_or(GUEST_PROFILE_ID)
        .to_string();
    let profile_value = profile.to_value();
    engine.state.profile.active = profile_value.clone();
    engine.state.profile.active_profile_id = Value::String(id.clone());
    library::set_active_profile_id(engine, &id);
    *engine.state.home = home::HomeState::default();
    home::mirror_active_profile(engine, profile_value);
}

pub(super) fn update_active(engine: &mut HeadlessEngine, profile: Value) {
    engine.state.profile.active = profile.clone();
    home::mirror_active_profile(engine, profile);
}
