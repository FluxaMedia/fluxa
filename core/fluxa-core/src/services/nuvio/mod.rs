pub(crate) mod pin;
mod routes;
pub(crate) use routes::*;
mod addon_priority;
mod collections;
mod continue_watching;
pub(crate) use continue_watching::continue_watching_json;
mod delta_state;
mod export_push;
mod helpers;
mod home_layout;
mod library_snapshot;
mod plugin_content;
mod profiles;
mod progress_sync;
mod reconciliation;
mod write_requests;

pub(crate) use addon_priority::{
    addon_snapshot_plan_json, addon_state_json, sort_addons_by_priority_json,
};
pub(crate) use collections::map_collections_json;
pub(crate) use delta_state::{
    apply_delta_sync_json, apply_progress_sync_json, delta_sync_request_plan_json,
    progress_sync_request_plan_json,
};
pub(crate) use export_push::{
    collection_request_json, export_push_plan_json, library_item_request_json,
    playback_progress_request_json, watched_items_request_json,
};
pub(crate) use helpers::canonical_content_type;
pub(crate) use home_layout::home_layout_json;
pub(crate) use library_snapshot::provider_library_snapshot_json;
pub(crate) use plugin_content::{candidate_content_types, plugin_content_id, plugin_content_type};
pub(crate) use profiles::{
    apply_remote_profiles_json, build_local_profiles_json, effective_profile_scopes_json,
};
pub(crate) use progress_sync::{
    import_merge_plan_json, library_to_watchlist_json, progress_meta_needs_json,
    progress_presentation_json, resolve_continue_watching_json,
};
pub(crate) use reconciliation::{addon_reconciliation_plan_json, library_mutation_plan_json};
pub(crate) use write_requests::write_requests_json;

#[cfg(test)]
mod tests;
