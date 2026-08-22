use super::*;

pub(super) fn route_content_warnings(method: &str, args_json: &str) -> Outcome {
    match method {
        "contentWarningUrl" => Ok(Value::String(content_warnings::content_warning_url(
            &arg_str(args_json, "imdbId")?,
        ))),
        "buildContentWarnings" => {
            opt_json(content_warnings::build_content_warnings_json(args_json))
        }
        _ => Err(unknown_method()),
    }
}
