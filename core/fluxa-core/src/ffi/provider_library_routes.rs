use super::*;
use crate::provider_library;

pub(super) fn route_provider_library(method: &str, args_json: &str) -> Outcome {
    match method {
        "providerAuthRequest" => opt_json(provider_library::provider_auth_request_json(args_json)),
        "providerAuthOutcome" => opt_json(provider_library::provider_auth_outcome_json(args_json)),
        "providerLibraryRequests" => {
            opt_json(provider_library::provider_library_requests_json(args_json))
        }
        "providerLibrarySnapshot" => {
            opt_json(provider_library::provider_library_snapshot_json(args_json))
        }
        "providerWriteRequests" => {
            opt_json(provider_library::provider_write_requests_json(args_json))
        }
        "providerScrobbleRequest" => {
            opt_json(provider_library::provider_scrobble_request_json(args_json))
        }
        _ => Err(unknown_method()),
    }
}
