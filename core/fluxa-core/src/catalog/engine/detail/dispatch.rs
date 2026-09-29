use super::*;

pub(crate) fn dispatch_load(
    engine: &mut HeadlessEngine,
    content_type: String,
    id: String,
    language: Option<String>,
    source_addon_transport_url: Option<String>,
    source_addon_catalog_type: Option<String>,
    profile: Option<Value>,
    preview: Option<Value>,
) -> Vec<EffectEnvelope> {
    let generation = engine.bump_generation(GenerationKey::Detail);
    let language = language.unwrap_or_else(|| "en".to_string());
    *engine.state.detail = DetailState {
        content_type: content_type.clone(),
        id: id.clone(),
        language: language.clone(),
        profile: profile.clone().unwrap_or(Value::Null),
        is_loading: true,
        generation,
        meta: preview.filter(Value::is_object).unwrap_or_default(),
        ..DetailState::default()
    };
    vec![
        engine.effect(
            EffectKind::FetchMetaDetail,
            generation,
            FetchMetaDetailPayload {
                content_type,
                id: id.clone(),
                language,
                source_addon_transport_url: source_addon_transport_url.unwrap_or_default(),
                source_addon_catalog_type: source_addon_catalog_type.unwrap_or_default(),
                profile: profile.unwrap_or(Value::Null),
            },
        ),
        engine.effect(
            EffectKind::ReadPlaybackProgress,
            generation,
            ReadPlaybackProgressPayload { id },
        ),
    ]
}

pub(crate) fn dispatch_local_state(
    engine: &mut HeadlessEngine,
    primary_id: String,
    fallback_id: Option<String>,
    content_type: String,
    profile: Option<Value>,
) -> Vec<EffectEnvelope> {
    let generation = engine.state.runtime.get(GenerationKey::Detail);
    vec![engine.effect(
        EffectKind::ReadDetailLocalState,
        generation,
        ReadDetailLocalStatePayload {
            primary_id,
            fallback_id,
            content_type,
            profile: profile.unwrap_or(Value::Null),
        },
    )]
}

pub(crate) fn dispatch_secondary(
    engine: &mut HeadlessEngine,
    content_type: String,
    id: String,
    language: Option<String>,
    profile: Option<Value>,
    similar_titles_source: Option<String>,
) -> Vec<EffectEnvelope> {
    let generation = engine.state.runtime.get(GenerationKey::Detail);
    vec![engine.effect(
        EffectKind::FetchDetailSecondary,
        generation,
        FetchDetailSecondaryPayload {
            content_type,
            id,
            language: language.unwrap_or_else(|| "en".to_string()),
            profile: profile.unwrap_or(Value::Null),
            similar_titles_source,
        },
    )]
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn dispatch_prefetch(
    engine: &mut HeadlessEngine,
    content_type: String,
    id: String,
    stream_lookup_id: String,
    title: Option<String>,
    original_name: Option<String>,
    year: Option<i32>,
    language: Option<String>,
    profile: Option<Value>,
) -> Vec<EffectEnvelope> {
    let generation = engine.state.runtime.get(GenerationKey::Detail);
    vec![engine.effect(
        EffectKind::PrefetchDetailStreams,
        generation,
        PrefetchDetailStreamsPayload {
            content_type,
            id,
            stream_lookup_id,
            title: title.unwrap_or_default(),
            original_name,
            year,
            language: language.unwrap_or_else(|| "en".to_string()),
            profile: profile.unwrap_or(Value::Null),
        },
    )]
}

pub(crate) fn dispatch_streams(
    engine: &mut HeadlessEngine,
    content_type: String,
    request_ids: Vec<String>,
    detail: Option<Value>,
    season_episodes: Option<Vec<Value>>,
    language: Option<String>,
    profile: Option<Value>,
) -> Vec<EffectEnvelope> {
    let generation = engine.bump_generation(GenerationKey::DetailStreams);
    engine.state.detail.is_loading_streams = true;
    engine.state.detail.streams = serde_json::json!([]);
    engine.state.detail.visible_streams = serde_json::json!([]);
    engine.state.detail.selected_addon = Value::Null;
    engine.state.detail.available_addons = serde_json::json!([]);
    engine.state.detail.loading_addon_names = serde_json::json!([]);
    engine.state.detail.failed_addons = serde_json::json!([]);
    vec![engine.effect(
        EffectKind::FetchDetailStreams,
        generation,
        FetchDetailStreamsPayload {
            content_type,
            request_ids,
            detail: detail.unwrap_or(Value::Null),
            season_episodes: season_episodes.unwrap_or_default(),
            language: language.unwrap_or_else(|| "en".to_string()),
            profile: profile.unwrap_or(Value::Null),
        },
    )]
}

pub(crate) fn dispatch_streams_appended(
    engine: &mut HeadlessEngine,
    streams: Vec<Value>,
    available_addons: Vec<String>,
    generation: Option<u64>,
) -> Vec<EffectEnvelope> {
    // Late plugin results may arrive after the addon request has already
    // completed. Generation still protects against stale episode results; an
    // unscoped append is only accepted while the request is loading.
    if !engine.state.detail.is_loading_streams && generation.is_none() {
        return vec![];
    }
    if let Some(generation) = generation
        && generation != engine.state.runtime.get(GenerationKey::DetailStreams)
    {
        return vec![];
    }
    let mut merged: Vec<Value> = engine
        .state
        .detail
        .streams
        .as_array()
        .cloned()
        .unwrap_or_default();
    merged.extend(streams);
    engine.state.detail.streams = serde_json::json!(merged);
    engine.state.detail.visible_streams = visible_streams(
        &engine.state.detail.streams,
        engine.state.detail.selected_addon.as_str(),
    );
    let mut all_addons: Vec<String> = engine
        .state
        .detail
        .available_addons
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default();
    for addon in available_addons {
        if !all_addons.contains(&addon) {
            all_addons.push(addon);
        }
    }
    engine.state.detail.available_addons = serde_json::json!(all_addons);
    engine.state.detail.has_stream_providers =
        Value::Bool(!value_array_is_empty(&engine.state.detail.streams));
    vec![]
}

pub(crate) fn dispatch_selected_addon_changed(
    engine: &mut HeadlessEngine,
    addon: Option<String>,
) -> Vec<EffectEnvelope> {
    let selected = addon.and_then(|value| {
        let trimmed = value.trim().to_string();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        }
    });
    engine.state.detail.selected_addon = selected
        .as_ref()
        .map(|value| Value::String(value.clone()))
        .unwrap_or(Value::Null);
    engine.state.detail.visible_streams =
        visible_streams(&engine.state.detail.streams, selected.as_deref());
    vec![]
}

pub(crate) fn dispatch_meta_detail(
    engine: &mut HeadlessEngine,
    content_type: String,
    id: String,
    language: Option<String>,
    profile: Option<Value>,
) -> Vec<EffectEnvelope> {
    let generation = engine.bump_generation(GenerationKey::Lookup);
    vec![engine.effect(
        EffectKind::FetchMetaDetailLookup,
        generation,
        FetchMetaDetailLookupPayload {
            content_type,
            id,
            language: language.unwrap_or_else(|| "en".to_string()),
            profile: profile.unwrap_or(Value::Null),
        },
    )]
}

pub(crate) fn dispatch_season(
    engine: &mut HeadlessEngine,
    series_id: String,
    season: i32,
    profile: Option<Value>,
    language: Option<String>,
) -> Vec<EffectEnvelope> {
    let generation = engine.state.runtime.get(GenerationKey::Detail);
    let profile_value = profile.unwrap_or_else(|| engine.state.profile.active.clone());
    let profile_id = active_profile_id(&engine.state, &profile_value);
    engine.state.detail.season_loading = Value::from(season);
    vec![engine.effect(
        EffectKind::FetchSeasonEpisodes,
        generation,
        FetchSeasonEpisodesPayload {
            series_id,
            season: season.max(0),
            profile_id,
            profile: profile_value,
            language: language.unwrap_or_else(|| "en".to_string()),
        },
    )]
}
