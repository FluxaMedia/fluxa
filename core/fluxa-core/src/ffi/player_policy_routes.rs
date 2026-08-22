use super::*;

pub(super) fn route_player_policy(method: &str, args_json: &str) -> Outcome {
    match method {
        "dvProxyPlan" => opt_json(player_policy::dv_proxy_plan_json(args_json)),
        "torrentFallbackFilePolicy" => {
            opt_json(player_policy::torrent_fallback_file_policy_json(args_json))
        }
        // args_json IS the request object for single-arg methods
        "playerBackendSelection" => {
            opt_json(player_policy::player_backend_selection_json(args_json))
        }
        "playerBufferTargets" => opt_json(player_policy::player_buffer_targets_json(args_json)),
        "playerRetryPolicy" => opt_json(player_policy::player_retry_policy_json(args_json)),
        "nextRetrySourcePlan" => opt_json(player_policy::next_retry_source_plan_json(args_json)),
        "playbackClosePlan" => opt_json(player_policy::playback_close_plan_json(args_json)),
        "playbackPreferencesPlan" => {
            opt_json(player_policy::playback_preferences_plan_json(args_json))
        }
        "streamShellPlan" => opt_json(player_policy::stream_shell_plan_json(args_json)),
        "orderStreamsPlan" => opt_json(player_policy::order_streams_plan_json(args_json)),
        "playerSourceSidebarPlan" => {
            opt_json(player_policy::player_source_sidebar_plan_json(args_json))
        }
        "canPrefetchNextEpisode" => {
            let args = object(args_json)?;
            Ok(json!(player_policy::can_prefetch_next_episode_json(
                field_str(&args, "prefsJson")?,
                field_str(&args, "streamJson")?,
            )))
        }
        "selectNextEpisodeStream" => {
            let args = object(args_json)?;
            opt_json(player_policy::select_next_episode_stream_json(
                field_str(&args, "streamsJson")?,
                field_str(&args, "currentStreamJson")?,
                field_str(&args, "prefsJson")?,
                field_str(&args, "nextVideoId")?,
            ))
        }
        "chapterSkipSegments" => {
            let args = object(args_json)?;
            into_json(desktop_playback::chapter_skip_segments_json(field_str(
                &args,
                "chaptersJson",
            )?))
        }

        _ => Err(unknown_method()),
    }
}
