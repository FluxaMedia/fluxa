use super::*;

pub(super) fn route_provider_library(method: &str, args_json: &str) -> Outcome {
    match method {
        "connectedProviders" => opt_json(services::registry::connected_providers_json(args_json)),
        "providerRegistry" => opt_json(Some(services::registry::provider_registry_json())),
        "providerAuthCallback" => opt_json(services::provider_auth_callback_json(args_json)),
        "providerAuthorizeUrl" => opt_json(services::provider_authorize_url_json(args_json)),
        "providerAuthRequest" => opt_json(services::provider_auth_request_json(args_json)),
        "providerAuthOutcome" => opt_json(services::provider_auth_outcome_json(args_json)),
        "providerLibraryRequests" => opt_json(services::provider_library_requests_json(args_json)),
        "providerLibrarySnapshot" => opt_json(services::provider_library_snapshot_json(args_json)),
        "simklSyncPlan" => opt_json(services::simkl::simkl_sync_plan_json(args_json)),
        "providerCalendarPlan" => opt_json(services::provider_calendar_plan_json(args_json)),
        "anilistCalendarPlan" => opt_json(services::anilist::anilist_calendar_plan_json(args_json)),
        "mdblistCalendarPlan" => opt_json(services::mdblist_calendar_plan_json(args_json)),
        "traktCalendarPlan" => opt_json(services::trakt::trakt_calendar_plan_json(args_json)),
        "simklCalendarPlan" => opt_json(services::simkl::simkl_calendar_plan_json(args_json)),
        "simklSyncApply" => opt_json(services::simkl::simkl_sync_apply_json(args_json)),
        "providerWriteRequests" => opt_json(services::provider_write_requests_json(args_json)),
        "providerScrobbleRequest" => opt_json(services::provider_scrobble_request_json(args_json)),
        _ => Err(unknown_method()),
    }
}
