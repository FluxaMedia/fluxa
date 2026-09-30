use super::{EffectEnvelope, EffectKind};
use serde_json::json;

#[test]
fn as_str_and_from_str_roundtrip_for_every_variant() {
    let all = [
        EffectKind::ClearPlaybackProgress,
        EffectKind::EnqueueOfflineDownload,
        EffectKind::EnqueueTraktScrobble,
        EffectKind::ExchangeAuthCode,
        EffectKind::ExecutePlugin,
        EffectKind::FetchAddonManifest,
        EffectKind::FetchAddonResource,
        EffectKind::FetchCatalogPage,
        EffectKind::FetchDiscoverPage,
        EffectKind::FetchDetailSecondary,
        EffectKind::FetchDetailStreams,
        EffectKind::FetchIntroSegments,
        EffectKind::FetchMetaDetail,
        EffectKind::FetchMetaDetailLookup,
        EffectKind::FetchPluginManifest,
        EffectKind::FetchSeasonEpisodes,
        EffectKind::FetchSubtitles,
        EffectKind::FetchYoutubeTrailerPlayer,
        EffectKind::FetchYoutubeTrailerPlayerScript,
        EffectKind::FetchYoutubeTrailerWatchConfig,
        EffectKind::LoadStreams,
        EffectKind::NotifyReleasedEpisodes,
        EffectKind::PrefetchDetailStreams,
        EffectKind::PrefetchNextEpisodeStreams,
        EffectKind::PrepareDirectPlayback,
        EffectKind::ReadCalendarMonth,
        EffectKind::ReadDetailLocalState,
        EffectKind::ReadDiscoverCatalogFilters,
        EffectKind::ReadHomeBootstrap,
        EffectKind::ReadLibraryState,
        EffectKind::ReadPlaybackProgress,
        EffectKind::RefreshAuthToken,
        EffectKind::RefreshInstalledAddons,
        EffectKind::ReplaceExternalContinueWatching,
        EffectKind::ResolveIntroImdbId,
        EffectKind::RunAuthFlow,
        EffectKind::RunDiscover,
        EffectKind::RunSearch,
        EffectKind::StartTorrentStream,
        EffectKind::StopTorrent,
        EffectKind::SyncWatchedState,
        EffectKind::UpdateCalendarWidget,
        EffectKind::WriteFeedback,
        EffectKind::WriteLibraryCommand,
        EffectKind::WritePlaybackProgress,
        EffectKind::WriteSettings,
    ];
    for kind in all {
        assert_eq!(EffectKind::from_str(kind.as_str()), Some(kind));
    }
}

#[test]
fn from_str_rejects_unknown_value() {
    assert_eq!(EffectKind::from_str("notAnEffect"), None);
}

#[test]
fn effect_kind_serde_uses_the_wire_names() {
    let encoded = serde_json::to_string(&EffectKind::FetchDetailStreams).unwrap();
    assert_eq!(encoded, "\"fetchDetailStreams\"");
    let decoded: EffectKind = serde_json::from_str(&encoded).unwrap();
    assert_eq!(decoded, EffectKind::FetchDetailStreams);
}

#[test]
fn scheduled_effect_dedupe_keys_hash_the_payload() {
    let first = EffectEnvelope::new(
        "fx-1".to_string(),
        EffectKind::FetchAddonResource,
        7,
        json!({ "url": "https://example.com/one", "token": "secret" }),
    );
    let duplicate = EffectEnvelope::new(
        "fx-2".to_string(),
        EffectKind::FetchAddonResource,
        7,
        json!({ "url": "https://example.com/one", "token": "secret" }),
    );
    let different = EffectEnvelope::new(
        "fx-3".to_string(),
        EffectKind::FetchAddonResource,
        7,
        json!({ "url": "https://example.com/two", "token": "secret" }),
    );

    assert_eq!(first.dedupe_key, duplicate.dedupe_key);
    assert_ne!(first.dedupe_key, different.dedupe_key);
    assert!(
        !first
            .dedupe_key
            .as_deref()
            .unwrap_or_default()
            .contains("secret")
    );
}
