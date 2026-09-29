use crate::ffi::*;

pub(crate) fn route_provider_library(method: &str, args_json: &str) -> Outcome {
    match method {
        "connectedProviders" => opt_json(crate::services::registry::connected_providers_json(
            args_json,
        )),
        "providerRegistry" => opt_json(Some(crate::services::registry::provider_registry_json())),
        "providerAuthCallback" => opt_json(crate::services::provider_auth_callback_json(args_json)),
        "providerAuthorizeUrl" => opt_json(crate::services::provider_authorize_url_json(args_json)),
        "providerAuthRequest" => opt_json(crate::services::provider_auth_request_json(args_json)),
        "providerAuthOutcome" => opt_json(crate::services::provider_auth_outcome_json(args_json)),
        "providerLibraryRequests" => {
            opt_json(crate::services::provider_library_requests_json(args_json))
        }
        "providerLibrarySnapshot" => {
            opt_json(crate::services::provider_library_snapshot_json(args_json))
        }
        "providerCalendarPlan" => opt_json(crate::services::provider_calendar_plan_json(args_json)),
        "providerWriteRequests" => {
            opt_json(crate::services::provider_write_requests_json(args_json))
        }
        "providerScrobbleRequest" => {
            opt_json(crate::services::provider_scrobble_request_json(args_json))
        }
        _ => Err(unknown_method()),
    }
}
