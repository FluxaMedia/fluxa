use crate::ffi::*;

pub(crate) fn route_addon_resource(method: &str, args_json: &str) -> Outcome {
    match method {
        "subtitleTracks" => opt_json(crate::addons::streams::resource::subtitle_tracks_json(
            args_json,
        )),
        _ => Err(unknown_method()),
    }
}
