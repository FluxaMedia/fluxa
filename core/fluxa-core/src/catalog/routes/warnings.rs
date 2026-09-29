use crate::ffi::*;
use serde_json::Value;

pub(crate) fn route_content_warnings(method: &str, args_json: &str) -> Outcome {
    match method {
        "contentWarningUrl" => Ok(Value::String(
            crate::catalog::warnings::content_warning_url(&arg_str(args_json, "imdbId")?),
        )),
        "buildContentWarnings" => opt_json(crate::catalog::warnings::build_content_warnings_json(
            args_json,
        )),
        _ => Err(unknown_method()),
    }
}
