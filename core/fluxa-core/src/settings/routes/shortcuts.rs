use crate::ffi::*;

pub(crate) fn route_shortcuts(method: &str, args_json: &str) -> Outcome {
    match method {
        "shortcutAssign" => opt_json(crate::settings::shortcuts::assign_json(args_json)),
        "shortcutBindings" => opt_json(crate::settings::shortcuts::bindings_json(args_json)),
        "shortcutResolve" => opt_json(crate::settings::shortcuts::resolve_json(args_json)),
        _ => Err(unknown_method()),
    }
}
