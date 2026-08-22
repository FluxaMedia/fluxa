use crate::headless_engine::HeadlessEngine;
use crate::headless_engine::state::GenerationKey;
use crate::runtime::{EffectEnvelope, EffectKind};
use serde::Serialize;
use serde_json::Value;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct FetchIntroSegmentsPayload {
    imdb_id: String,
    season: i32,
    episode: i32,
    title: Option<String>,
    use_intro_db: bool,
    use_ani_skip: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ResolveIntroImdbIdPayload {
    meta: Value,
    video_id: Option<String>,
    language: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct FetchSubtitlesPayload {
    stream: Value,
    content_type: String,
    id: String,
    extra_args: String,
}

pub(in crate::headless_engine) fn dispatch_intro_segments(
    engine: &mut HeadlessEngine,
    imdb_id: String,
    season: i32,
    episode: i32,
    title: Option<String>,
    use_intro_db: bool,
    use_ani_skip: bool,
) -> Vec<EffectEnvelope> {
    let generation = engine.bump_generation(GenerationKey::Intro);
    vec![engine.effect(
        EffectKind::FetchIntroSegments,
        generation,
        FetchIntroSegmentsPayload {
            imdb_id,
            season,
            episode,
            title,
            use_intro_db,
            use_ani_skip,
        },
    )]
}

pub(in crate::headless_engine) fn dispatch_intro_imdb_id(
    engine: &mut HeadlessEngine,
    meta: Value,
    video_id: Option<String>,
    language: Option<String>,
) -> Vec<EffectEnvelope> {
    let generation = engine.bump_generation(GenerationKey::Intro);
    vec![engine.effect(
        EffectKind::ResolveIntroImdbId,
        generation,
        ResolveIntroImdbIdPayload {
            meta,
            video_id,
            language: language.unwrap_or_else(|| "en".to_string()),
        },
    )]
}

pub(in crate::headless_engine) fn dispatch_subtitle_load(
    engine: &mut HeadlessEngine,
    stream: Value,
    content_type: String,
    id: String,
    extra_args: Option<String>,
) -> Vec<EffectEnvelope> {
    let generation = engine.bump_generation(GenerationKey::Player);
    engine.state.player.subtitle_loading = true;
    vec![engine.effect(
        EffectKind::FetchSubtitles,
        generation,
        FetchSubtitlesPayload {
            stream,
            content_type,
            id,
            extra_args: extra_args.unwrap_or_default(),
        },
    )]
}
