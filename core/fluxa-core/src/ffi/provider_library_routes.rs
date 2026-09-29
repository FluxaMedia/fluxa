use super::*;
use crate::library;

pub(super) fn route_provider_library(method: &str, args_json: &str) -> Outcome {
    match method {
        "providerAuthRequest" => opt_json(library::provider::provider_auth_request_json(args_json)),
        "providerAuthOutcome" => opt_json(library::provider::provider_auth_outcome_json(args_json)),
        "providerLibraryRequests" => {
            opt_json(library::provider::provider_library_requests_json(args_json))
        }
        "providerLibrarySnapshot" => {
            opt_json(library::provider::provider_library_snapshot_json(args_json))
        }
        "providerWriteRequests" => {
            opt_json(library::provider::provider_write_requests_json(args_json))
        }
        "providerScrobbleRequest" => {
            opt_json(library::provider::provider_scrobble_request_json(args_json))
        }
        _ => Err(unknown_method()),
    }
}
