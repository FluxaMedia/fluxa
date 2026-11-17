use super::*;
use crate::dp::Dp;
use crate::flex::{self, Align, Insets};

use crate::draw_text;

pub(super) const ACCOUNT_SERVICES: [(&str, &str); 4] = [
    ("Trakt", "traktAccessToken"),
    ("Simkl", "simklAccessToken"),
    ("AniList", "anilistAccessToken"),
    ("MDBList", "mdblistAccessToken"),
];
pub const SOURCE_TOKENS: [(&str, &str); 6] = [
    ("nuvio", "nuvioAccessToken"),
    ("trakt", "traktAccessToken"),
    ("simkl", "simklAccessToken"),
    ("anilist", "anilistAccessToken"),
    ("mdblist", "mdblistAccessToken"),
    ("stremio", "stremioAuthKey"),
];

pub const ACCOUNT_PROVIDERS: [&str; 4] = ["trakt", "simkl", "mdblist", "anilist"];
pub(super) const ACCOUNT_SOURCES: [(&str, &str); 3] = [
    (
        "settings.integration_library_source",
        "integrationLibrarySource",
    ),
    (
        "settings.continue_watching_source",
        "continueWatchingSource",
    ),
    ("settings.mdblist_library_list", "mdblistLibraryListId"),
];

pub(super) const SYNC_SOURCES: usize = 2;
pub(super) const MDBLIST_LIST_SOURCE: usize = 2;







pub(super) fn source_label(source: &str, language: &str) -> String {
    match source {
        "mdblist" => localized("settings.service.mdblist", language),
        _ => localized_or(&format!("settings.option.{source}"), source, language),
    }
}

pub fn account_source_state(
    settings: &SettingsModel,
    key: &str,
) -> (Vec<(String, String)>, String, String) {
    let language = settings.language();
    let options = if key == "trickplaySource" {
        trickplay_sources(settings)
    } else if key == "heroBadgeSource" || key == "externalAudioSource" {
        addon_sources(settings)
    } else if key == "mdblistLibraryListId" {
        if account_connected(settings, "mdblistAccessToken")
            || account_connected(settings, "mdblistApiKey")
        {
            std::iter::once((
                String::new(),
                localized("settings.mdblist_watchlist", language),
            ))
            .chain(settings.mdblist_lists.iter().cloned())
            .collect()
        } else {
            Vec::new()
        }
    } else {
        SOURCE_TOKENS
            .into_iter()
            .filter(|(_, token_key)| account_connected(settings, token_key))
            .map(|(source, _)| (source.to_owned(), source_label(source, language)))
            .collect::<Vec<_>>()
    };
    let current = settings
        .value(key)
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default()
        .to_owned();
    let label = match options.iter().find(|(source, _)| *source == current) {
        Some((_, label)) => label.clone(),
        None if matches!(
            key,
            "trickplaySource" | "heroBadgeSource" | "externalAudioSource"
        ) =>
        {
            localized("settings.option.none", language)
        }
        None if options.is_empty() => localized("settings.not_connected", language),
        None => localized("settings.option.none", language),
    };
    (options, current, label)
}

pub fn account_source(index: usize) -> Option<(&'static str, &'static str)> {
    ACCOUNT_SOURCES
        .get(index)
        .copied()
        .or((index == TRICKPLAY_SOURCE_INDEX).then_some(TRICKPLAY_SOURCE))
        .or((index == HERO_BADGE_SOURCE_INDEX).then_some(HERO_BADGE_SOURCE))
        .or((index == EXTERNAL_AUDIO_SOURCE_INDEX).then_some(EXTERNAL_AUDIO_SOURCE))
}

const TRICKPLAY_SOURCE_INDEX: usize = 3;
const TRICKPLAY_SOURCE: (&str, &str) = ("settings.trickplay_source", "trickplaySource");
pub const HERO_BADGE_SOURCE_INDEX: usize = 4;
const HERO_BADGE_SOURCE: (&str, &str) = ("settings.hero_badge_source", "heroBadgeSource");
pub const EXTERNAL_AUDIO_SOURCE_INDEX: usize = 5;
const EXTERNAL_AUDIO_SOURCE: (&str, &str) =
    ("settings.external_audio_source", "externalAudioSource");


fn addon_sources(settings: &SettingsModel) -> Vec<(String, String)> {
    let text = |value: &serde_json::Value, key: &str| {
        value
            .get(key)
            .and_then(serde_json::Value::as_str)
            .filter(|text| !text.trim().is_empty())
            .map(str::to_owned)
    };
    let addons = settings
        .addons
        .iter()
        .filter(|addon| addon_enabled(addon))
        .filter_map(|addon| {
            let url = addon_transport_url(addon)?;
            let manifest = addon.get("manifest").unwrap_or(addon);
            let name = text(manifest, "name").unwrap_or_else(|| url.to_owned());
            Some((format!("addon:{url}"), name))
        });
    let plugins = settings
        .plugins
        .get("scrapers")
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|scraper| {
            let id = text(scraper, "id")?;
            let name = text(scraper, "name").unwrap_or_else(|| id.clone());
            Some((format!("plugin:{id}"), name))
        });
    addons.chain(plugins).collect()
}

fn trickplay_sources(settings: &SettingsModel) -> Vec<(String, String)> {
    let language = settings.language();
    std::iter::once((
        "stream".to_owned(),
        localized("settings.trickplay_source_stream", language),
    ))
    .chain(std::iter::once((
        "server".to_owned(),
        localized("settings.trickplay_source_server", language),
    )))
    .chain(addon_sources(settings))
    .collect()
}

pub(super) fn account_connected(settings: &SettingsModel, key: &str) -> bool {
    settings
        .profile
        .get(key)
        .or_else(|| settings.value(key))
        .and_then(serde_json::Value::as_str)
        .is_some_and(|token| !token.is_empty())
}



