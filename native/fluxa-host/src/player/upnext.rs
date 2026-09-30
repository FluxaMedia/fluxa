use super::*;

pub(super) fn tick_recommendations(
    player: &mut PlayerSession,
    session: &fluxa_effects::SessionHandle,
    settings: &SettingsModel,
    snapshot: &Value,
) {
    if let Some(items) = player
        .recommendations_rx
        .as_ref()
        .and_then(|rx| rx.try_recv().ok())
    {
        player.recommendations_rx = None;
        player.recommendations = terminal_plan(&player.meta, items, snapshot);
    }
    if player.outro_reached || player.status.duration <= 0.0 {
        return;
    }
    let series = player.meta.get("type").and_then(Value::as_str) == Some("series");
    let threshold = settings
        .values
        .get(if series {
            "seriesRecommendationOutroPercent"
        } else {
            "movieRecommendationOutroPercent"
        })
        .and_then(|value| value.as_f64().or_else(|| value.as_str()?.parse().ok()))
        .unwrap_or(85.0);
    let reached = core_value(
        "recommendationOutroPlan",
        json!({
            "positionSeconds": player.status.position,
            "durationSeconds": player.status.duration,
            "thresholdPercent": threshold,
            "alreadyShown": false,
        }),
    )
    .and_then(|plan| plan.get("shouldShow").and_then(Value::as_bool))
    .unwrap_or(false);
    if !reached {
        return;
    }
    player.outro_reached = true;
    if series && !series_finished(&player.meta, snapshot) {
        return;
    }
    let endpoints = [
        (
            "recommendations",
            settings.bool_value("tmdbRecommendationsEnabled"),
        ),
        ("similar", settings.bool_value("tmdbSimilarResultsEnabled")),
    ]
    .into_iter()
    .filter_map(|(endpoint, on)| on.then_some(endpoint))
    .collect();
    let api_key = settings
        .str_value("tmdbApiKey")
        .unwrap_or_default()
        .to_owned();
    player.recommendations_rx = Some(session.executor().fetch_similar(
        player.meta.clone(),
        api_key,
        player.language.clone(),
        endpoints,
    ));
}

pub(super) fn series_finished(meta: &Value, snapshot: &Value) -> bool {
    let Some(videos) = meta.get("videos").and_then(Value::as_array) else {
        return false;
    };
    let Some(current) = snapshot
        .pointer("/player/currentVideoId")
        .and_then(Value::as_str)
        .and_then(|id| {
            videos
                .iter()
                .find(|video| video.get("id").and_then(Value::as_str) == Some(id))
        })
    else {
        return false;
    };
    let number = |key: &str| current.get(key).and_then(Value::as_i64);
    core_value(
        "terminalRecommendationEligibility",
        json!({
            "contentType": "series",
            "videos": videos,
            "currentSeason": number("season").unwrap_or(0),
            "currentEpisode": number("episode").or_else(|| number("number")).unwrap_or(0),
            "nowMs": web_time::SystemTime::now()
                .duration_since(web_time::UNIX_EPOCH)
                .map(|elapsed| elapsed.as_millis() as i64)
                .unwrap_or(0),
        }),
    )
    .and_then(|plan| plan.get("eligible").and_then(Value::as_bool))
    .unwrap_or(false)
}

pub(super) fn terminal_plan(meta: &Value, candidates: Vec<Value>, snapshot: &Value) -> Vec<Value> {
    let watched: Vec<&str> = snapshot
        .pointer("/library/watched")
        .and_then(Value::as_object)
        .map(|watched| {
            watched
                .iter()
                .filter(|(_, value)| value.as_bool() == Some(true))
                .map(|(id, _)| id.as_str())
                .collect()
        })
        .unwrap_or_default();
    let Some(plan) = core_value(
        "terminalRecommendationPlan",
        json!({
            "current": {"id": meta.get("id"), "type": meta.get("type")},
            "candidates": candidates,
            "hasNextEpisode": false,
            "watchedIds": watched,
            "limit": fluxa_ui::PLAYER_RECOMMENDATION_LIMIT,
        }),
    ) else {
        return Vec::new();
    };
    if plan.get("showRecommendations").and_then(Value::as_bool) != Some(true) {
        return Vec::new();
    }
    plan.get("items")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
}

pub(super) fn episode_title(meta: &Value, snapshot: &Value) -> Option<String> {
    let id = snapshot
        .pointer("/player/currentVideoId")
        .and_then(Value::as_str)?;
    meta.get("videos")?
        .as_array()?
        .iter()
        .find(|video| video.get("id").and_then(Value::as_str) == Some(id))?
        .get("name")
        .or_else(|| meta.get("title"))
        .and_then(Value::as_str)
        .filter(|title| !title.trim().is_empty())
        .map(ToOwned::to_owned)
}

pub(super) fn content_warning_url(meta: &Value, snapshot: &Value) -> Option<String> {
    let candidates = [
        meta.get("id").and_then(Value::as_str),
        snapshot
            .pointer("/player/currentVideoId")
            .and_then(Value::as_str),
    ];
    let imdb = candidates.into_iter().flatten().find_map(|id| {
        core_value("contentImdbId", json!({"id": id}))?
            .as_str()
            .filter(|id| !id.is_empty())
            .map(ToOwned::to_owned)
    })?;
    core_value("contentWarningUrl", json!({"imdbId": imdb}))?
        .as_str()
        .map(ToOwned::to_owned)
}

pub(super) fn tick_warnings(player: &mut PlayerSession) {
    let delta = player.last_pump.elapsed().as_secs_f32();
    player.last_pump = Instant::now();
    if let Some(response) = player
        .warnings_rx
        .as_ref()
        .and_then(|rx| rx.try_recv().ok())
    {
        player.warnings_rx = None;
        player.warnings = response
            .and_then(|response| build_warnings(&response, &player.language))
            .unwrap_or_default();
    }
    if player.warnings.is_empty() || !player.status.has_frame {
        return;
    }
    let clock = player.warnings_clock.get_or_insert(0.0);
    if !player.status.paused {
        *clock += delta.min(0.25);
    }
    if *clock > fluxa_ui::content_warning_duration(player.warnings.len()) {
        player.warnings.clear();
        player.warnings_clock = None;
    }
}

pub(super) fn build_warnings(response: &Value, language: &str) -> Option<Vec<(String, String)>> {
    let label = |key: &str| fluxa_ui::localized(&format!("content_warning.{key}"), language);
    let labels = [
        "nudity",
        "violence",
        "profanity",
        "alcohol",
        "frightening",
        "severe",
        "moderate",
        "mild",
    ]
    .into_iter()
    .map(|key| (key.to_owned(), Value::String(label(key))))
    .collect::<serde_json::Map<_, _>>();
    let result = core_value(
        "buildContentWarnings",
        json!({"responseJson": response.to_string(), "labels": labels}),
    )?;
    Some(
        result
            .get("warnings")?
            .as_array()?
            .iter()
            .filter_map(|warning| {
                Some((
                    warning.get("label")?.as_str()?.to_owned(),
                    warning.get("severity")?.as_str()?.to_owned(),
                ))
            })
            .collect(),
    )
}
