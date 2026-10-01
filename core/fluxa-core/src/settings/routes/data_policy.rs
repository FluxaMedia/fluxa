use crate::ffi::*;

pub(crate) fn route_data_policy(method: &str, _args_json: &str) -> Outcome {
    match method {
        // args_json IS the request object for all of these
        _ => Err(unknown_method()),
    }
}

pub(crate) fn route_device_resource(method: &str, _args_json: &str) -> Outcome {
    match method {
        // args_json IS the request object
        _ => Err(unknown_method()),
    }
}
