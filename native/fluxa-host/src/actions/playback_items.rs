use super::*;

pub(super) fn known_logo(state: &RendererState, item: &Value) -> Option<String> {
    let id = item.get("id").and_then(Value::as_str)?;
    let home = &state.home;
    home.hero_slides
        .iter()
        .find(|hero| hero.item_id.as_deref() == Some(id))
        .and_then(|hero| hero.logo_url.clone())
        .or_else(|| {
            (home.item_id.as_deref() == Some(id))
                .then(|| home.logo_url.clone())
                .flatten()
        })
        .or_else(|| {
            home.cards
                .iter()
                .chain(home.rows.iter().flat_map(|row| row.cards.iter()))
                .find(|card| card.id.as_deref() == Some(id))
                .and_then(|card| card.logo_url.clone())
        })
        .or_else(|| {
            (state.detail.id == id)
                .then(|| state.detail.logo_url.clone())
                .flatten()
        })
}

pub(crate) fn playback_item(
    item: &Value,
    resume: Option<&HomeCard>,
    first_video: Option<String>,
) -> Value {
    let mut item = item.clone();
    let Some(fields) = item.as_object_mut() else {
        return item;
    };
    if let Some(resume) = resume.and_then(|card| card.raw.as_object()) {
        for (key, value) in resume {
            if key.starts_with("last") || matches!(key.as_str(), "timeOffset" | "duration") {
                fields.insert(key.clone(), value.clone());
            }
        }
        if !fields.contains_key("lastVideoId")
            && let Some(video) = resume.get("videoId")
        {
            fields.insert("lastVideoId".to_owned(), video.clone());
        }
    } else if let Some(video) = first_video {
        fields.insert("lastVideoId".to_owned(), video.into());
    }
    let foreign = fields
        .get("lastVideoId")
        .and_then(Value::as_str)
        .is_some_and(|video| !player::video_belongs_to(video, &Value::Object(fields.clone())));
    if foreign {
        let episode = fields
            .get("id")
            .and_then(Value::as_str)
            .zip(fields.get("lastEpisodeSeason").and_then(Value::as_i64))
            .zip(fields.get("lastEpisodeNumber").and_then(Value::as_i64))
            .map(|((id, season), number)| format!("{id}:{season}:{number}"));
        fields.retain(|key, _| {
            !key.starts_with("last")
                && !matches!(
                    key.as_str(),
                    "timeOffset" | "duration" | "videoId" | "video_id"
                )
        });
        if let Some(video) = episode {
            fields.insert("lastVideoId".to_owned(), video.into());
        }
    }
    item
}

pub(crate) fn refresh_library_view(state: &mut RendererState) {
    state.library.query = state.library_query.clone();
    state.library.sort_by = state.library_sort.clone();
    state.library.content_type = state.library_type.clone();
    state.library.list_view = state.library_list;
    state.library.downloads_open = state.library_downloads;
    state.library.downloads_group = state.library_download_group.clone();
    if let Some(snapshot) = state.core_snapshot.as_ref() {
        let library = snapshot
            .get("library")
            .cloned()
            .unwrap_or_else(|| json!({}));
        if let Some(plan) = core_value(
            "libraryViewPlan",
            json!({
                "watchlist": library.get("watchlist"),
                "watching": library.get("continueWatching"),
                "completed": library.get("completed"),
                "dropped": library.get("dropped"),
                "onHold": library.get("onHold"),
                "favorites": library.get("liked"),
                "progress": library.get("progress"),
                "tab": state.library_tab.core_tab_key(),
                "query": state.library_query,
                "sortBy": state.library_sort,
                "type": state.library_type,
                "source": state.library.source.clone(),
            }),
        ) {
            state.library.apply_core_plan(&plan, state.library_tab);
        }
    }
}
