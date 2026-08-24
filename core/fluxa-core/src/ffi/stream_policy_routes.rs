use super::*;

pub(super) fn route_stream_policy(method: &str, args_json: &str) -> Outcome {
    match method {
        // args_json IS the stream/request JSON
        "streamPlaybackInfo" => opt_json(stream_policy::stream_playback_info_json(args_json)),
        "torrentRuntimeInfo" => opt_json(stream_policy::torrent_runtime_info_json(args_json)),
        "torrentStatusInfo" => opt_json(stream_policy::torrent_status_info_json(args_json)),
        "torrentReadyBudget" => into_json(stream_policy::torrent_ready_budget_json()),
        "streamRequestHeaders" => opt_json(stream_policy::stream_request_headers_json(&arg_str(
            args_json,
            "headersJson",
        )?)),
        "streamRequestReferer" => opt_json(stream_policy::stream_request_referer(&arg_str(
            args_json, "url",
        )?)),
        "selectStreamIndex" => {
            let args = object(args_json)?;
            let saved_url = args.get("savedUrl").and_then(Value::as_str);
            let saved_title = args.get("savedTitle").and_then(Value::as_str);
            let regex_pattern = args.get("regexPattern").and_then(Value::as_str);
            let preferred_binge_group = args.get("preferredBingeGroup").and_then(Value::as_str);
            Ok(json!(stream_policy::select_stream_index(
                field_str(&args, "streamsJson")?,
                field_str(&args, "currentVideoId")?,
                field(&args, "initialStreamIndex")?
                    .as_i64()
                    .ok_or_else(|| fail(
                        ErrorKind::InvalidArgs,
                        "initialStreamIndex must be a number"
                    ))? as i32,
                saved_url,
                saved_title,
                field_str(&args, "sourceSelectionMode")?.into(),
                regex_pattern,
                preferred_binge_group,
            )))
        }
        "playerTrackState" => opt_json(stream_policy::player_track_state_json(args_json)),
        "externalAudioOptions" => {
            opt_json(stream_policy::external_audio_options_json(args_json))
        }
        "resolvePreferredAudioLanguage" => {
            let args = object(args_json)?;
            let last = args.get("lastAudioLanguage").and_then(Value::as_str);
            let preferred = args.get("preferredAudioLanguage").and_then(Value::as_str);
            let original = args.get("originalLanguage").and_then(Value::as_str);
            Ok(Value::String(
                stream_policy::resolve_preferred_audio_language(last, preferred, original),
            ))
        }
        "subtitleLanguageMatches" => {
            let args = object(args_json)?;
            let language = args.get("language").and_then(Value::as_str);
            Ok(json!(stream_policy::subtitle_language_matches(
                field_str(&args, "label")?,
                language,
                field_str(&args, "preferredLanguage")?,
            )))
        }
        "findPreferredSubtitleIndex" => {
            let args = object(args_json)?;
            let last = args
                .get("lastSubtitleLanguage")
                .and_then(Value::as_str)
                .map(str::to_string);
            let preferred = args
                .get("preferredSubtitleLanguage")
                .and_then(Value::as_str)
                .map(str::to_string);
            let secondary = args
                .get("secondarySubtitleLanguage")
                .and_then(Value::as_str)
                .map(str::to_string);
            Ok(json!(stream_policy::find_preferred_subtitle_index(
                field_str(&args, "tracks")?,
                last.as_deref(),
                preferred.as_deref(),
                secondary.as_deref(),
            )))
        }

        _ => Err(unknown_method()),
    }
}
