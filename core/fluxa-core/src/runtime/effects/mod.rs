use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Exhaustive catalog of all effect types the headless engine can emit.
///
/// This is the single source of truth for effect type names — the string
/// representations produced by `as_str()` are the ones the platform (Kotlin,
/// JS, etc.) matches against in its effect dispatcher.
///
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum EffectKind {
    ClearPlaybackProgress,
    EnqueueOfflineDownload,
    EnqueueTraktScrobble,
    ExchangeAuthCode,
    ExecutePlugin,
    FetchAddonManifest,
    FetchAddonResource,
    FetchCatalogPage,
    FetchDiscoverPage,
    FetchDetailSecondary,
    FetchDetailStreams,
    FetchIntroSegments,
    FetchMetaDetail,
    FetchMetaDetailLookup,
    FetchPluginManifest,
    FetchSeasonEpisodes,
    FetchSubtitles,
    FetchYoutubeTrailerPlayer,
    FetchYoutubeTrailerPlayerScript,
    FetchYoutubeTrailerWatchConfig,
    LoadStreams,
    NotifyReleasedEpisodes,
    PrefetchDetailStreams,
    PrefetchNextEpisodeStreams,
    PrepareDirectPlayback,
    ReadCalendarMonth,
    ReadDetailLocalState,
    ReadDiscoverCatalogFilters,
    ReadHomeBootstrap,
    RefreshContinueWatching,
    ReadLibraryState,
    ReadPlaybackProgress,
    RefreshAuthToken,
    RefreshInstalledAddons,
    ReplaceExternalContinueWatching,
    ResolveIntroImdbId,
    RunAuthFlow,
    RunDiscover,
    RunExternalSync,
    RunSearch,
    StartTorrentStream,
    StopTorrent,
    SyncExternalIntegration,
    SyncWatchedState,
    UpdateCalendarWidget,
    WriteFeedback,
    WriteLibraryCommand,
    WritePlaybackProgress,
    WriteSettings,
}

impl EffectKind {
    pub const ALL: &[Self] = &[
        Self::ClearPlaybackProgress,
        Self::EnqueueOfflineDownload,
        Self::EnqueueTraktScrobble,
        Self::ExchangeAuthCode,
        Self::ExecutePlugin,
        Self::FetchAddonManifest,
        Self::FetchAddonResource,
        Self::FetchCatalogPage,
        Self::FetchDiscoverPage,
        Self::FetchDetailSecondary,
        Self::FetchDetailStreams,
        Self::FetchIntroSegments,
        Self::FetchMetaDetail,
        Self::FetchMetaDetailLookup,
        Self::FetchPluginManifest,
        Self::FetchSeasonEpisodes,
        Self::FetchSubtitles,
        Self::FetchYoutubeTrailerPlayer,
        Self::FetchYoutubeTrailerPlayerScript,
        Self::FetchYoutubeTrailerWatchConfig,
        Self::LoadStreams,
        Self::NotifyReleasedEpisodes,
        Self::PrefetchDetailStreams,
        Self::PrefetchNextEpisodeStreams,
        Self::PrepareDirectPlayback,
        Self::ReadCalendarMonth,
        Self::ReadDetailLocalState,
        Self::ReadDiscoverCatalogFilters,
        Self::ReadHomeBootstrap,
        Self::RefreshContinueWatching,
        Self::ReadLibraryState,
        Self::ReadPlaybackProgress,
        Self::RefreshAuthToken,
        Self::RefreshInstalledAddons,
        Self::ReplaceExternalContinueWatching,
        Self::ResolveIntroImdbId,
        Self::RunAuthFlow,
        Self::RunDiscover,
        Self::RunExternalSync,
        Self::RunSearch,
        Self::StartTorrentStream,
        Self::StopTorrent,
        Self::SyncExternalIntegration,
        Self::SyncWatchedState,
        Self::UpdateCalendarWidget,
        Self::WriteFeedback,
        Self::WriteLibraryCommand,
        Self::WritePlaybackProgress,
        Self::WriteSettings,
    ];
    pub fn as_str(self) -> &'static str {
        match self {
            EffectKind::ClearPlaybackProgress => "clearPlaybackProgress",
            EffectKind::EnqueueOfflineDownload => "enqueueOfflineDownload",
            EffectKind::EnqueueTraktScrobble => "enqueueTraktScrobble",
            EffectKind::ExchangeAuthCode => "exchangeAuthCode",
            EffectKind::ExecutePlugin => "executePlugin",
            EffectKind::FetchAddonManifest => "fetchAddonManifest",
            EffectKind::FetchAddonResource => "fetchAddonResource",
            EffectKind::FetchCatalogPage => "fetchCatalogPage",
            EffectKind::FetchDiscoverPage => "fetchDiscoverPage",
            EffectKind::FetchDetailSecondary => "fetchDetailSecondary",
            EffectKind::FetchDetailStreams => "fetchDetailStreams",
            EffectKind::FetchIntroSegments => "fetchIntroSegments",
            EffectKind::FetchMetaDetail => "fetchMetaDetail",
            EffectKind::FetchMetaDetailLookup => "fetchMetaDetailLookup",
            EffectKind::FetchPluginManifest => "fetchPluginManifest",
            EffectKind::FetchSeasonEpisodes => "fetchSeasonEpisodes",
            EffectKind::FetchSubtitles => "fetchSubtitles",
            EffectKind::FetchYoutubeTrailerPlayer => "fetchYoutubeTrailerPlayer",
            EffectKind::FetchYoutubeTrailerPlayerScript => "fetchYoutubeTrailerPlayerScript",
            EffectKind::FetchYoutubeTrailerWatchConfig => "fetchYoutubeTrailerWatchConfig",
            EffectKind::LoadStreams => "loadStreams",
            EffectKind::NotifyReleasedEpisodes => "notifyReleasedEpisodes",
            EffectKind::PrefetchDetailStreams => "prefetchDetailStreams",
            EffectKind::PrefetchNextEpisodeStreams => "prefetchNextEpisodeStreams",
            EffectKind::PrepareDirectPlayback => "prepareDirectPlayback",
            EffectKind::ReadCalendarMonth => "readCalendarMonth",
            EffectKind::ReadDetailLocalState => "readDetailLocalState",
            EffectKind::ReadDiscoverCatalogFilters => "readDiscoverCatalogFilters",
            EffectKind::ReadHomeBootstrap => "readHomeBootstrap",
            EffectKind::RefreshContinueWatching => "refreshContinueWatching",
            EffectKind::ReadLibraryState => "readLibraryState",
            EffectKind::ReadPlaybackProgress => "readPlaybackProgress",
            EffectKind::RefreshAuthToken => "refreshAuthToken",
            EffectKind::RefreshInstalledAddons => "refreshInstalledAddons",
            EffectKind::ReplaceExternalContinueWatching => "replaceExternalContinueWatching",
            EffectKind::ResolveIntroImdbId => "resolveIntroImdbId",
            EffectKind::RunAuthFlow => "runAuthFlow",
            EffectKind::RunDiscover => "runDiscover",
            EffectKind::RunExternalSync => "runExternalSync",
            EffectKind::RunSearch => "runSearch",
            EffectKind::StartTorrentStream => "startTorrentStream",
            EffectKind::StopTorrent => "stopTorrent",
            EffectKind::SyncExternalIntegration => "syncExternalIntegration",
            EffectKind::SyncWatchedState => "syncWatchedState",
            EffectKind::UpdateCalendarWidget => "updateCalendarWidget",
            EffectKind::WriteFeedback => "writeFeedback",
            EffectKind::WriteLibraryCommand => "writeLibraryCommand",
            EffectKind::WritePlaybackProgress => "writePlaybackProgress",
            EffectKind::WriteSettings => "writeSettings",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        serde_json::from_value(serde_json::Value::String(value.to_owned())).ok()
    }
}

/// Wire format for an effect emitted by the headless engine.
///
/// Matches the `NativeHeadlessEffect` data class on the Kotlin side:
/// ```kotlin
/// data class NativeHeadlessEffect(
///     val id: String,
///     val type: String,
///     val generation: Long,
///     val payload: Map<String, Any?>
/// )
/// ```
///
/// `id` is a monotonically-increasing opaque string (`"fx-N"`).
/// `generation` lets the platform discard stale completions.
/// `payload` carries effect-specific parameters as a JSON object.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EffectEnvelope {
    pub id: String,
    #[serde(rename = "type")]
    pub kind: EffectKind,
    pub generation: u64,
    pub payload: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group_id: Option<String>,
    #[serde(
        default = "default_priority",
        skip_serializing_if = "is_default_priority"
    )]
    pub priority: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dedupe_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_policy: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout_ms: Option<u64>,
}

impl EffectEnvelope {
    pub fn new(id: String, kind: EffectKind, generation: u64, payload: serde_json::Value) -> Self {
        let (group_id, priority, cache_policy, timeout_ms) = effect_schedule(kind);
        Self {
            dedupe_key: group_id
                .as_ref()
                .and_then(|_| payload_dedupe_key(kind, generation, &payload)),
            id,
            kind,
            generation,
            payload,
            group_id,
            priority,
            cache_policy,
            timeout_ms,
        }
    }
}

fn payload_dedupe_key(
    kind: EffectKind,
    generation: u64,
    payload: &serde_json::Value,
) -> Option<String> {
    let payload = serde_json::to_string(payload).ok()?;
    let digest = Sha256::digest(payload.as_bytes());
    let hash = digest
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    Some(format!("{}:{generation}:{hash}", kind.as_str()))
}

fn default_priority() -> u8 {
    100
}

fn is_default_priority(priority: &u8) -> bool {
    *priority == default_priority()
}

fn effect_schedule(kind: EffectKind) -> (Option<String>, u8, Option<String>, Option<u64>) {
    match kind {
        EffectKind::FetchAddonResource
        | EffectKind::FetchCatalogPage
        | EffectKind::FetchDetailStreams
        | EffectKind::PrefetchDetailStreams
        | EffectKind::PrefetchNextEpisodeStreams
        | EffectKind::RefreshInstalledAddons => (
            Some("addon".to_string()),
            50,
            Some("default".to_string()),
            Some(15_000),
        ),
        EffectKind::RunSearch | EffectKind::RunDiscover => (
            Some("addon".to_string()),
            40,
            Some("default".to_string()),
            Some(15_000),
        ),
        EffectKind::ReadHomeBootstrap => (None, 100, None, Some(20_000)),
        EffectKind::ExecutePlugin => (
            Some("plugin".to_string()),
            60,
            Some("no-store".to_string()),
            Some(10_000),
        ),
        _ => (None, 100, None, None),
    }
}

#[cfg(test)]
mod tests;
