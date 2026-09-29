use super::*;
use crate::library;

pub(super) fn route_provider_library(method: &str, args_json: &str) -> Outcome {
    match method {
        "providerAuthCallback" => {
            opt_json(library::provider::provider_auth_callback_json(args_json))
        }
        "providerAuthorizeUrl" => {
            opt_json(library::provider::provider_authorize_url_json(args_json))
        }
        "providerAuthRequest" => opt_json(library::provider::provider_auth_request_json(args_json)),
        "providerAuthOutcome" => opt_json(library::provider::provider_auth_outcome_json(args_json)),
        "providerLibraryRequests" => {
            opt_json(library::provider::provider_library_requests_json(args_json))
        }
        "providerLibrarySnapshot" => {
            opt_json(library::provider::provider_library_snapshot_json(args_json))
        }
        "simklSyncPlan" => opt_json(library::simkl_sync::simkl_sync_plan_json(args_json)),
        "mdblistCalendarPlan" => opt_json(library::provider::mdblist_calendar_plan_json(args_json)),
        "traktCalendarPlan" => opt_json(library::provider::trakt_calendar_plan_json(args_json)),
        "simklCalendarPlan" => opt_json(library::simkl_sync::simkl_calendar_plan_json(args_json)),
        "simklSyncApply" => opt_json(library::simkl_sync::simkl_sync_apply_json(args_json)),
        "providerWriteRequests" => {
            opt_json(library::provider::provider_write_requests_json(args_json))
        }
        "providerScrobbleRequest" => {
            opt_json(library::provider::provider_scrobble_request_json(args_json))
        }
        _ => Err(unknown_method()),
    }
}
