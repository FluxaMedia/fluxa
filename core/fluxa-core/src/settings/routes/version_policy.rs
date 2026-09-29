use crate::ffi::*;
use serde_json::json;

pub(crate) fn route_version_policy(method: &str, args_json: &str) -> Outcome {
    match method {
        "versionIsNewer" => {
            let args = object(args_json)?;
            Ok(json!(crate::settings::version::is_newer(
                field_str(&args, "remote")?,
                field_str(&args, "current")?,
            )))
        }
        _ => Err(unknown_method()),
    }
}
