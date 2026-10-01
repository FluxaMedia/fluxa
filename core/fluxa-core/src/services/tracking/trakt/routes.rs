use crate::ffi::*;
use crate::services;
use serde_json::Value;

pub(crate) fn route_trakt(method: &str, args_json: &str) -> Outcome {
    match method {
        "traktShowIdFromEpisodeId" => Ok(Value::String(
            services::trakt::trakt_show_id_from_episode_id(&arg_str(args_json, "videoId")?),
        )),
        // args_json IS the items array for single-array-arg methods
        "traktRemapVideoIds" => opt_json(services::trakt::trakt_remap_video_ids_json(args_json)),
        _ => Err(unknown_method()),
    }
}
