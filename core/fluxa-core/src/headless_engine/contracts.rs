use crate::action_contract::{MarkWatchedAction, SavePlaybackProgressAction};
use crate::runtime::EffectEnvelope;
use crate::types::{MetaItem, Profile, Stream, Video};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;

#[derive(Clone, Debug, Deserialize)]
#[serde(
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    tag = "type"
)]
pub(super) enum AppAction {
    #[serde(rename = "navigationRequested")]
    NavigationRequested {
        route: String,
        params: Option<Value>,
    },
    #[serde(rename = "detailLoadRequested")]
    DetailLoadRequested {
        content_type: String,
        id: String,
        language: Option<String>,
        source_addon_transport_url: Option<String>,
        source_addon_catalog_type: Option<String>,
        profile: Option<Value>,
    },
    #[serde(rename = "detailLocalStateRequested")]
    DetailLocalStateRequested {
        primary_id: String,
        fallback_id: Option<String>,
        content_type: String,
        profile: Option<Value>,
    },
    #[serde(rename = "detailSecondaryRequested")]
    DetailSecondaryRequested {
        content_type: String,
        id: String,
        language: Option<String>,
        profile: Option<Value>,
        similar_titles_source: Option<String>,
    },
    #[serde(rename = "detailPrefetchRequested")]
    DetailPrefetchRequested {
        content_type: String,
        id: String,
        stream_lookup_id: String,
        title: Option<String>,
        original_name: Option<String>,
        year: Option<i32>,
        language: Option<String>,
        profile: Option<Value>,
    },
    #[serde(rename = "detailStreamsRequested")]
    DetailStreamsRequested {
        content_type: String,
        request_ids: Vec<String>,
        detail: Option<MetaItem>,
        season_episodes: Option<Vec<Video>>,
        language: Option<String>,
        profile: Option<Profile>,
    },
    #[serde(rename = "detailStreamsAppended")]
    DetailStreamsAppended {
        streams: Vec<Stream>,
        available_addons: Vec<String>,
        generation: Option<u64>,
    },
    #[serde(rename = "detailSelectedAddonChanged")]
    DetailSelectedAddonChanged { addon: Option<String> },
    #[serde(rename = "metaDetailRequested")]
    MetaDetailRequested {
        content_type: String,
        id: String,
        language: Option<String>,
        profile: Option<Value>,
    },
    #[serde(rename = "directPlaybackRequested")]
    DirectPlaybackRequested {
        meta: Value,
        language: Option<String>,
        profile: Option<Value>,
    },
    #[serde(rename = "introSegmentsRequested")]
    IntroSegmentsRequested {
        imdb_id: String,
        season: i32,
        episode: i32,
        title: Option<String>,
        use_intro_db: bool,
        use_ani_skip: bool,
    },
    #[serde(rename = "introImdbIdRequested")]
    IntroImdbIdRequested {
        meta: Value,
        video_id: Option<String>,
        language: Option<String>,
    },
    #[serde(rename = "playerLoadStreamsRequested")]
    PlayerLoadStreamsRequested {
        content_type: String,
        id: String,
        current_video_id: Option<String>,
        initial_video_id: Option<String>,
        initial_streams: Option<Vec<Stream>>,
        initial_stream_index: Option<i32>,
        saved_url: Option<String>,
        saved_title: Option<String>,
        source_selection_mode: Option<SourceSelectionMode>,
        regex_pattern: Option<String>,
        preferred_binge_group: Option<String>,
        title: Option<String>,
        original_name: Option<String>,
        year: Option<i32>,
        language: Option<String>,
        profile: Option<Profile>,
        outgoing_progress: Option<Value>,
    },
    #[serde(rename = "playerStreamsLoaded")]
    PlayerStreamsLoaded {
        streams: Vec<Stream>,
        current_video_id: Option<String>,
        initial_stream_index: Option<i32>,
        saved_url: Option<String>,
        saved_title: Option<String>,
        source_selection_mode: Option<SourceSelectionMode>,
        regex_pattern: Option<String>,
        preferred_binge_group: Option<String>,
    },
    #[serde(rename = "playerStreamsFailed")]
    PlayerStreamsFailed { error_code: Option<String> },
    #[serde(rename = "playerResolvePlaybackRequested")]
    PlayerResolvePlaybackRequested {
        url: String,
        stream: Option<Stream>,
        current_video_id: Option<String>,
        title: Option<String>,
    },
    #[serde(rename = "scrobbleRequested")]
    ScrobbleRequested {
        token: String,
        meta_type: String,
        item_id: String,
        progress: f64,
        action_name: String,
        profile: Option<Value>,
    },
    #[serde(rename = "profileActivated")]
    ProfileActivated { profile: Profile },
    #[serde(rename = "homeLoadRequested")]
    HomeLoadRequested {
        profile: Option<Value>,
        language: Option<String>,
        force: Option<bool>,
    },
    #[serde(rename = "refreshContinueWatchingRequested")]
    RefreshContinueWatchingRequested {
        profile: Option<Value>,
        language: Option<String>,
        source: Option<String>,
    },
    #[serde(rename = "libraryHydrateRequested")]
    LibraryHydrateRequested { profile_id: Option<String> },
    #[serde(rename = "toggleWatchlistRequested")]
    ToggleWatchlistRequested { item: Value, profile: Option<Value> },
    #[serde(rename = "toggleLibraryStatusRequested")]
    ToggleLibraryStatusRequested { list: String, item: Value },
    #[serde(rename = "setFeedbackRequested")]
    SetFeedbackRequested {
        id: String,
        value: Option<bool>,
        meta: Value,
    },
    #[serde(rename = "clearPlaybackProgressRequested")]
    ClearPlaybackProgressRequested { profile: Option<Value>, meta: Value },
    #[serde(rename = "savePlaybackProgressRequested")]
    SavePlaybackProgressRequested {
        #[serde(flatten)]
        action: SavePlaybackProgressAction,
    },
    #[serde(rename = "markWatchedRequested")]
    MarkWatchedRequested {
        #[serde(flatten)]
        action: MarkWatchedAction,
    },
    #[serde(rename = "addonInstallRequested")]
    AddonInstallRequested {
        transport_url: String,
        force_refresh: Option<bool>,
    },
    #[serde(rename = "addonsRefreshRequested")]
    AddonsRefreshRequested {
        profile: Option<Value>,
        force_refresh: Option<bool>,
    },
    #[serde(rename = "addonResourceRequested")]
    AddonResourceRequested {
        transport_url: String,
        resource: String,
        content_type: String,
        id: String,
        extra: Option<Value>,
    },
    #[serde(rename = "searchRequested")]
    SearchRequested {
        query: String,
        profile: Option<Value>,
        language: Option<String>,
    },
    #[serde(rename = "discoverRequested")]
    DiscoverRequested {
        content_type: String,
        filters: Option<Value>,
        profile: Option<Value>,
        language: Option<String>,
    },
    #[serde(rename = "discoverCatalogFiltersRequested")]
    DiscoverCatalogFiltersRequested {
        content_type: String,
        selected_catalog_key: Option<String>,
        profile: Option<Value>,
        language: Option<String>,
    },
    #[serde(rename = "discoverPageRequested")]
    DiscoverPageRequested {
        transport_url: Option<String>,
        content_type: String,
        catalog_id: String,
        skip: Option<i32>,
        genre: Option<String>,
    },
    #[serde(rename = "catalogPageRequested")]
    CatalogPageRequested {
        category_id: String,
        transport_url: Option<String>,
        content_type: String,
        catalog_id: String,
        skip: Option<i32>,
        genre: Option<String>,
        search: Option<String>,
        remote_source: Option<Value>,
        profile: Option<Value>,
    },
    #[serde(rename = "detailSeasonRequested")]
    DetailSeasonRequested {
        series_id: String,
        season: i32,
        profile: Option<Value>,
        language: Option<String>,
    },
    #[serde(rename = "playerNextEpisodeCardShown")]
    PlayerNextEpisodeCardShown {
        content_type: String,
        series_id: String,
        next_video_id: String,
        title: Option<String>,
        original_name: Option<String>,
        year: Option<i32>,
        language: Option<String>,
        profile: Option<Value>,
    },
    #[serde(rename = "subtitleLoadRequested")]
    SubtitleLoadRequested {
        stream: Value,
        content_type: String,
        id: String,
        extra_args: Option<String>,
    },
    #[serde(rename = "externalSyncRequested")]
    ExternalSyncRequested {
        provider: String,
        profile: Option<Value>,
        language: Option<String>,
    },
    #[serde(rename = "authFlowRequested")]
    AuthFlowRequested { provider: String, mode: String },
    #[serde(rename = "authExchangeRequested")]
    AuthExchangeRequested {
        provider: String,
        code: String,
        code_verifier: Option<String>,
        profile: Option<Value>,
    },
    #[serde(rename = "authRefreshRequested")]
    AuthRefreshRequested { provider: String, profile: Value },
    #[serde(rename = "externalIntegrationSyncRequested")]
    ExternalIntegrationSyncRequested {
        provider: String,
        profile: Value,
        language: Option<String>,
    },
    #[serde(rename = "settingsChanged")]
    SettingsChanged { key: String, value: Value },
    #[serde(rename = "calendarMonthRequested")]
    CalendarMonthRequested {
        profile: Option<Value>,
        year: i32,
        month: i32,
        #[serde(rename = "plannedItems")]
        planned_items: Option<Value>,
    },
    #[serde(rename = "offlineDownloadRequested")]
    OfflineDownloadRequested {
        meta: Value,
        stream: Value,
        video_id: Option<String>,
        video: Option<Value>,
        subtitle: Option<Value>,
        profile_id: Option<String>,
        language: Option<String>,
    },
    #[serde(rename = "trailerResolveRequested")]
    TrailerResolveRequested {
        request_id: String,
        video_id: String,
        max_height: Option<u32>,
    },
    #[serde(rename = "trailerPrewarmRequested")]
    TrailerPrewarmRequested,
    #[serde(rename = "pluginRepositoryAddRequested")]
    PluginRepositoryAddRequested { manifest_url: String },
    #[serde(rename = "pluginRepositoryRemoveRequested")]
    PluginRepositoryRemoveRequested { manifest_url: String },
    #[serde(rename = "pluginScraperToggled")]
    PluginScraperToggled { scraper_id: String, enabled: bool },
    #[serde(rename = "pluginScraperSettingsUpdated")]
    PluginScraperSettingsUpdated { scraper_id: String, settings: Value },
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) enum SourceSelectionMode {
    Manual,
    First,
    Regex,
}

impl SourceSelectionMode {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Manual => "manual",
            Self::First => "first",
            Self::Regex => "regex",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) enum EffectStatus {
    Ok,
    Error,
    Cancelled,
}

impl EffectStatus {
    pub(crate) fn is_ok(self) -> bool {
        matches!(self, EffectStatus::Ok)
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct EffectResultInput {
    pub effect_id: String,
    pub status: EffectStatus,
    #[serde(default)]
    pub value: Value,
    #[serde(default)]
    pub error: Value,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct DispatchResult {
    pub revision: u64,
    pub state: StatePatch,
    pub effects: Vec<EffectEnvelope>,
}

// Only domains that actually changed since the snapshot taken before this dispatch
// are `Some` here — the platform merges this onto its existing state instead of
// replacing it wholesale, since serializing the full EngineState on every action
// scales with everything the user has ever loaded, not with what changed.
#[derive(Clone, Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct StatePatch {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub navigation: Option<Arc<super::navigation::NavigationState>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub home: Option<Arc<super::home::HomeState>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search: Option<Arc<super::search::SearchState>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discover: Option<Arc<super::discover::DiscoverState>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<Arc<super::detail::DetailState>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub player: Option<Arc<super::player::PlayerState>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub library: Option<Arc<super::library::LibraryState>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile: Option<Arc<super::profile::ProfileState>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub settings: Option<Arc<super::settings::SettingsState>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub calendar: Option<Arc<super::calendar::CalendarState>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub addons: Option<Arc<super::addons::AddonsState>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth: Option<Arc<super::auth::AuthState>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sync: Option<Arc<super::sync::SyncState>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lookup: Option<Arc<super::detail::LookupState>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offline: Option<Arc<super::offline::OfflineState>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trailer: Option<Arc<super::trailer::TrailerState>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plugins: Option<Arc<super::plugins::PluginsState>>,
}

#[cfg(test)]
mod tests {
    use super::{AppAction, SourceSelectionMode};

    #[test]
    fn source_selection_mode_is_closed_at_the_wire_boundary() {
        let action: AppAction = serde_json::from_str(
            r#"{"type":"playerStreamsLoaded","streams":[],"sourceSelectionMode":"regex"}"#,
        )
        .unwrap();
        assert!(matches!(
            action,
            AppAction::PlayerStreamsLoaded {
                source_selection_mode: Some(SourceSelectionMode::Regex),
                ..
            }
        ));

        assert!(serde_json::from_str::<AppAction>(
            r#"{"type":"playerStreamsLoaded","streams":[],"sourceSelectionMode":"typo"}"#,
        )
        .is_err());
    }
}
