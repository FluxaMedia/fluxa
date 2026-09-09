use super::*;
use crate::version_policy;

pub(super) fn route_version_policy(method: &str, args_json: &str) -> Outcome {
    match method {
        "versionIsNewer" => {
            let args = object(args_json)?;
            Ok(json!(version_policy::is_newer(
                field_str(&args, "remote")?,
                field_str(&args, "current")?,
            )))
        }
        _ => Err(unknown_method()),
    }
}
