use crate::ffi::*;

pub(crate) fn route_data_policy(_method: &str, _args_json: &str) -> Outcome {
    Err(unknown_method())
}

pub(crate) fn route_device_resource(_method: &str, _args_json: &str) -> Outcome {
    Err(unknown_method())
}
