use super::*;

pub(super) fn route_addon_uptime(method: &str, args_json: &str) -> Outcome {
    match method {
        "addonUptimeMatchPlan" => opt_json(addon_uptime::addon_uptime_match_plan_json(args_json)),
        _ => Err(fail(
            ErrorKind::UnknownMethod,
            "unknown addon uptime method",
        )),
    }
}

pub(super) fn route_trailer_subtitles(method: &str, args_json: &str) -> Outcome {
    match method {
        "trailerSubtitleSelectionPlan" => opt_json(
            trailer_subtitles::trailer_subtitle_selection_plan_json(args_json),
        ),
        "normalizeTrailerSubtitleUrl" => opt_json(
            trailer_subtitles::normalize_trailer_subtitle_url_json(args_json),
        ),
        "trailerPlaybackPolicy" => {
            opt_json(trailer_subtitles::trailer_playback_policy_json(args_json))
        }
        "trailerYoutubeVideoIds" => {
            opt_json(trailer_subtitles::trailer_youtube_video_ids_json(args_json))
        }
        "trailerDirectSelection" => {
            opt_json(trailer_subtitles::trailer_direct_selection_json(args_json))
        }
        "parseTrailerSubtitleCues" => opt_json(
            trailer_subtitles::parse_trailer_subtitle_cues_json(args_json),
        ),
        _ => Err(fail(
            ErrorKind::UnknownMethod,
            "unknown trailer subtitle method",
        )),
    }
}
