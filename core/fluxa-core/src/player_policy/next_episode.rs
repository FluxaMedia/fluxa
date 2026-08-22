use serde_json::Value;

pub(crate) fn can_prefetch_next_episode_json(prefs_json: &str, stream_json: &str) -> bool {
    let prefs: Value = serde_json::from_str(prefs_json).unwrap_or(Value::Null);
    let stream: Value = serde_json::from_str(stream_json).unwrap_or(Value::Null);
    let try_binge = prefs
        .get("tryBingeGroup")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let mode = prefs
        .get("streamSourceSelectionMode")
        .and_then(Value::as_str)
        .unwrap_or("manual");
    let has_binge_group = stream
        .get("behaviorHints")
        .and_then(|h| h.get("bingeGroup"))
        .and_then(Value::as_str)
        .is_some_and(|s| !s.is_empty());
    (try_binge && has_binge_group) || mode != "manual"
}

/// Selects the best stream from `streams_json` for the next episode given the
/// current stream and playback preferences. Returns the selected stream as JSON,
/// or `null` if none qualifies.
pub(crate) fn select_next_episode_stream_json(
    streams_json: &str,
    current_stream_json: &str,
    prefs_json: &str,
    next_video_id: &str,
) -> Option<String> {
    let streams: Vec<Value> = serde_json::from_str(streams_json).ok()?;
    if streams.is_empty() {
        return None;
    }
    let current: Value = serde_json::from_str(current_stream_json).ok()?;
    let prefs: Value = serde_json::from_str(prefs_json).unwrap_or(Value::Null);

    let episode_ok = |s: &Value| -> bool {
        let field = |key: &str| -> String {
            s.get(key)
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string()
        };
        let behavior_hints = s.get("behaviorHints");
        let filename = behavior_hints
            .and_then(|h| h.get("filename"))
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        crate::content_identity::stream_matches_episode(
            next_video_id,
            &[
                field("title"),
                field("name"),
                field("description"),
                filename,
            ],
        )
    };

    let try_binge = prefs
        .get("tryBingeGroup")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let mode = prefs
        .get("streamSourceSelectionMode")
        .and_then(Value::as_str)
        .unwrap_or("manual");
    let regex_pat = prefs
        .get("streamSourceRegexPattern")
        .and_then(Value::as_str)
        .unwrap_or("");
    let cur_binge = current
        .get("behaviorHints")
        .and_then(|h| h.get("bingeGroup"))
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty());

    if try_binge && let Some(group) = cur_binge {
        let matched = streams.iter().find(|s| {
            s.get("behaviorHints")
                .and_then(|h| h.get("bingeGroup"))
                .and_then(Value::as_str)
                == Some(group)
                && episode_ok(s)
        });
        if let Some(s) = matched {
            return serde_json::to_string(s).ok();
        }
    }

    if mode == "regex"
        && !regex_pat.is_empty()
        && let Ok(re) = regex::RegexBuilder::new(regex_pat)
            .case_insensitive(true)
            .build()
    {
        let stream_text = |s: &Value| -> String {
            [
                s.get("name"),
                s.get("title"),
                s.get("description"),
                s.get("url"),
                s.get("playableUrl"),
                s.get("infoHash"),
            ]
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .collect::<Vec<_>>()
            .join(" ")
        };
        if let Some(matched) = streams
            .iter()
            .find(|s| re.is_match(&stream_text(s)) && episode_ok(s))
        {
            return serde_json::to_string(matched).ok();
        }
    }

    if let Some(addon_name) = current
        .get("addonName")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        && let Some(matched) = streams.iter().find(|stream| {
            stream.get("addonName").and_then(Value::as_str) == Some(addon_name)
                && episode_ok(stream)
        })
    {
        return serde_json::to_string(matched).ok();
    }

    streams
        .iter()
        .find(|s| episode_ok(s))
        .and_then(|s| serde_json::to_string(s).ok())
}
