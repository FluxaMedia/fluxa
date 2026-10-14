use crate::ffi::*;

pub(crate) fn route_player_policy(method: &str, args_json: &str) -> Outcome {
    match method {
        // args_json IS the request object for single-arg methods
        "anime4kShaderChain" => opt_json(
            crate::player::playback::policy::anime4k_shader_chain_json(args_json),
        ),
        "playerEpisodes" => opt_json(crate::player::playback::policy::player_episodes_json(
            args_json,
        )),
        "playbackClosePlan" => opt_json(crate::player::playback::policy::playback_close_plan_json(
            args_json,
        )),
        "playbackPreferencesPlan" => {
            opt_json(crate::player::playback::policy::playback_preferences_plan_json(args_json))
        }
        "shuffleEpisodePick" => opt_json(
            crate::player::playback::policy::shuffle_episode_pick_json(args_json),
        ),
        "subtitleStylePlan" => opt_json(crate::player::playback::policy::subtitle_style_plan_json(
            args_json,
        )),
        "videoConfigPlan" => opt_json(crate::player::playback::policy::video_config_plan_json(
            args_json,
        )),
        "trickplayPickSource" => opt_json(crate::player::trickplay::pick_source_json(args_json)),
        "trickplayServerManifest" => opt_json(crate::player::trickplay::server_manifest_url_json(
            args_json,
        )),
        "trickplayServerSource" => {
            opt_json(crate::player::trickplay::server_source_json(args_json))
        }
        "trickplayPlan" => opt_json(crate::player::trickplay::plan_json(args_json)),
        "streamSubtitlesResult" => {
            opt_json(crate::player::playback::policy::stream_subtitles_result_json(args_json))
        }
        _ => Err(unknown_method()),
    }
}
