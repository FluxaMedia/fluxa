use std::collections::HashMap;
use std::sync::Arc;

use serde_json::{Value, json};

use crate::{RendererState, profile_language};

#[derive(Default)]
pub(crate) struct EpisodeDetails {
    requested: Option<(String, Option<i64>, Option<i64>)>,
    found: HashMap<String, Value>,
    published: usize,
}

fn base_id(id: &str) -> &str {
    id.split(':').next().unwrap_or(id)
}

pub(crate) fn tick(state: &mut RendererState) {
    let Some(gpu) = state.gpu.as_ref() else {
        return;
    };
    let Some(session) = state.session.as_ref() else {
        return;
    };
    let details = &mut state.episode_details;
    let poster = fluxa_ui::previewed_poster(&gpu.egui_context);
    if let Some(poster) = poster.as_ref() {
        let video = poster
            .raw
            .get("lastVideoId")
            .and_then(Value::as_str)
            .filter(|video| !video.is_empty());
        let has_overview = poster
            .raw
            .get("lastEpisodeOverview")
            .and_then(Value::as_str)
            .is_some_and(|text| !text.trim().is_empty());
        let key = video.map(|video| video.to_owned());
        if let Some(video) = key
            && !has_overview
            && !details.found.contains_key(&video)
            && details.requested.as_ref().map(|(id, _, _)| id.as_str()) != Some(video.as_str())
        {
            let number = |key: &str| poster.raw.get(key).and_then(Value::as_i64);
            details.requested = Some((
                video.clone(),
                number("lastEpisodeSeason"),
                number("lastEpisodeNumber"),
            ));
            let content_type = poster
                .raw
                .get("type")
                .and_then(Value::as_str)
                .unwrap_or("series");
            let profile = session.active_profile();
            let _ = session.dispatch(json!({
                "type": "metaDetailRequested",
                "contentType": content_type,
                "id": base_id(&poster.id),
                "language": profile_language(&profile),
                "profile": profile,
            }));
        }
    }
    let Some((requested, season, number)) = details.requested.clone() else {
        return;
    };
    let Some(snapshot) = state.core_snapshot.as_ref() else {
        return;
    };
    let Some(detail) = snapshot.pointer("/lookup/metaDetail") else {
        return;
    };
    let meta = detail.get("meta").unwrap_or(detail);
    let same = meta
        .get("id")
        .and_then(Value::as_str)
        .is_some_and(|id| base_id(id) == base_id(&requested));
    if !same {
        return;
    }
    let episode = meta
        .get("videos")
        .and_then(Value::as_array)
        .and_then(|videos| {
            let by_id = videos
                .iter()
                .find(|video| video.get("id").and_then(Value::as_str) == Some(requested.as_str()));
            by_id.or_else(|| {
                let (season, number) = (season?, number?);
                videos.iter().find(|video| {
                    video.get("season").and_then(Value::as_i64) == Some(season)
                        && video.get("episode").and_then(Value::as_i64) == Some(number)
                })
            })
        });
    details.requested = None;
    let text = |keys: &[&str]| {
        episode.and_then(|video| {
            keys.iter()
                .filter_map(|key| video.get(*key).and_then(Value::as_str))
                .find(|text| !text.trim().is_empty())
        })
    };
    details.found.insert(
        requested,
        json!({
            "overview": text(&["overview", "description"]),
            "thumbnail": text(&["thumbnail"]),
        }),
    );
    if details.found.len() != details.published {
        details.published = details.found.len();
        fluxa_ui::set_episode_details(&gpu.egui_context, Arc::new(details.found.clone()));
        gpu.egui_context.request_repaint();
    }
}
