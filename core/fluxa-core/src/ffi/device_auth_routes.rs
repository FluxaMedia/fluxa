use super::*;

pub(super) fn route_device_auth(method: &str, args_json: &str) -> Outcome {
    match method {
        "deviceAuthStart" => opt_json(crate::accounts::device_auth::start_json(args_json)),
        "deviceAuthTransition" => {
            opt_json(crate::accounts::device_auth::transition_json(args_json))
        }
        _ => Err(unknown_method()),
    }
}
