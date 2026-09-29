mod collection_and_detail;
mod library_and_prefs;
mod playback_prepare;
mod resource_fetch;
mod resource_parse;

pub(crate) use collection_and_detail::{
    addon_collection_mutation_plan_json, detail_episode_plan_json, mark_seasons_action_plan_json,
    season_watched_plan_json,
};
pub(crate) use library_and_prefs::{
    apply_preference_update_json, library_local_state_plan_json, preferences_schema_json,
};
pub(crate) use playback_prepare::playback_prepare_plan_json;
pub(crate) use resource_fetch::{resource_fetch_execution_policy_json, resource_fetch_plan_json};
pub(crate) use resource_parse::{
    parse_and_plan_addon_resource_json, resource_kind_to_resource, resource_parse_plan_json,
};

#[cfg(test)]
mod tests;
