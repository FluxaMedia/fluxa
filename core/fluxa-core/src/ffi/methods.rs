use super::*;

type Router = fn(&str, &str) -> Outcome;

pub(super) fn router_for(method: &str) -> Option<Router> {
    METHODS
        .binary_search_by(|(name, _)| name.cmp(&method))
        .ok()
        .map(|index| METHODS[index].1)
}

static METHODS: &[(&str, Router)] = &[
    ("accountSource", route_profile_contract),
    ("activeProfilePlan", route_profile_contract),
    ("addonResourceRequestPlan", route_resource_plan),
    ("anime4kShaderChain", route_player_policy),
    ("annotateCatalogItems", route_library_state),
    ("baseUrl", route_addon_protocol),
    ("buildContentWarnings", route_content_warnings),
    ("buildContinueWatchingFromProgress", route_library_state),
    ("buildHomeCollectionShelves", route_library_state),
    ("buildMetadataFeedOptions", route_search_plan),
    ("buildResourceUrl", route_addon_protocol),
    ("castAirplayBody", route_cast),
    ("castChromecastDecode", route_cast),
    ("castChromecastEncode", route_cast),
    ("castContentType", route_cast),
    ("castDlnaDevice", route_cast),
    ("castDlnaLoadArgs", route_cast),
    ("castDlnaSeekArgs", route_cast),
    ("castDlnaSoap", route_cast),
    ("castFcastDecode", route_cast),
    ("castFcastEncode", route_cast),
    ("castFcastPlayBody", route_cast),
    ("castFcastSeekBody", route_cast),
    ("castFcastUpdate", route_cast),
    ("castFcastVersionBody", route_cast),
    ("castLanUrl", route_cast),
    ("castRokuLaunch", route_cast),
    ("castRokuName", route_cast),
    ("castValidUrl", route_cast),
    ("connectedProviders", route_provider_library),
    ("contentImdbId", route_content_identity),
    ("contentWarningUrl", route_content_warnings),
    ("continueWatchingSourcePlan", route_library_state),
    ("createProfilePlan", route_profile_contract),
    ("cs3CatalogFeedKey", route_content_identity),
    ("cs3MetadataFeedOptions", route_content_identity),
    ("cs3PluginFeedKey", route_content_identity),
    ("desktopCalendarReadPlan", route_calendar),
    ("detailStreamResultPlan", route_headless_adapter_plan),
    ("discoverCatalogOptions", route_search_plan),
    ("discoverSourceRequests", route_search_plan),
    ("effectiveAddonsOwnerId", route_addon_store),
    ("filterEnabledAddons", route_addon_store),
    ("homeHeroPlan", route_library_state),
    ("homeMetadataFeedPlan", route_library_state),
    ("identity", route_addon_protocol),
    ("libraryCommandPlan", route_watchlist),
    ("libraryViewPlan", route_watchlist),
    ("manifestFetchPlan", route_addon_protocol),
    ("mdblistMediaInfoBatchPlan", route_mdblist),
    ("mediaServerParse", route_mediaserver),
    ("mediaServerRequest", route_mediaserver),
    ("mergeDiscoverSources", route_search_plan),
    ("mergeSearchSources", route_search_plan),
    ("normalizeAddonDescriptor", route_addon_protocol),
    ("normalizeLibraryDocument", route_library_state),
    ("normalizeTrailerSubtitleUrl", route_trailer_subtitles),
    ("nuvioAddonSnapshotPlan", route_nuvio),
    ("nuvioApplyDeltaSync", route_nuvio),
    ("nuvioApplyProgressSync", route_nuvio),
    ("nuvioApplyRemoteProfiles", route_nuvio),
    ("nuvioDeltaSyncRequestPlan", route_nuvio),
    ("nuvioEffectiveProfileScopes", route_nuvio),
    ("nuvioHomeLayout", route_nuvio),
    ("nuvioMapCollections", route_nuvio),
    ("nuvioProgressMetaNeeds", route_nuvio),
    ("nuvioProviderLibrarySnapshot", route_nuvio),
    ("nuvioWriteRequests", route_nuvio),
    ("parseAndPlanAddonResource", route_resource_plan),
    ("parseManifest", route_addon_protocol),
    ("playbackClosePlan", route_player_policy),
    ("playbackPreferencesPlan", route_player_policy),
    ("playbackPreparePlan", route_resource_plan),
    ("playbackProgressWritePlan", route_watchlist),
    ("playerMediaCommandPlan", route_media_session),
    ("playerMediaSessionPlan", route_media_session),
    ("playerOverlayPlan", route_player_overlay),
    ("playerSegmentsStep", route_segments),
    ("playerSegmentsSubmitPlan", route_segments),
    ("playerTickPlan", route_player_overlay),
    ("playerToastPlan", route_player_overlay),
    ("playerTrackPlan", route_player_overlay),
    ("primaryProfileId", route_profile_contract),
    ("profileAvatarPackCatalog", route_profile_avatar_pack),
    ("profileAvatarPackDiscoveryPlan", route_profile_avatar_pack),
    ("profileAvatarPackManifestPlan", route_profile_avatar_pack),
    ("profileAvatarPackParse", route_profile_avatar_pack),
    ("profileAvatarPackRepositoryPlan", route_profile_avatar_pack),
    ("profileLocalAddonsKey", route_addon_store),
    ("profileMutationPlan", route_profile_contract),
    ("profilePinHash", route_profile_contract),
    ("profilePinMatches", route_profile_contract),
    ("profileSettingsMigrationPlan", route_profile_contract),
    ("providerAuthCallback", route_provider_library),
    ("providerAuthOutcome", route_provider_library),
    ("providerAuthRequest", route_provider_library),
    ("providerAuthorizeUrl", route_provider_library),
    ("providerAvailabilityPlan", route_headless_adapter_plan),
    ("providerCalendarItems", route_external_sync),
    ("providerCalendarPlan", route_provider_library),
    ("providerLibraryRequests", route_provider_library),
    ("providerLibrarySnapshot", route_provider_library),
    ("providerRegistry", route_provider_library),
    ("providerScrobbleRequest", route_provider_library),
    ("providerWriteRequests", route_provider_library),
    (
        "publicmetadbAnimeSeasonsDeleteChunkPlan",
        route_publicmetadb,
    ),
    (
        "publicmetadbAnimeSeasonsDeleteMappingPlan",
        route_publicmetadb,
    ),
    (
        "publicmetadbEpisodeRatingsBatchCreatePlan",
        route_publicmetadb,
    ),
    (
        "publicmetadbEpisodeRatingsBatchDeletePlan",
        route_publicmetadb,
    ),
    ("recommendationOutroPlan", route_library_state),
    ("replaceExternalContinueWatching", route_external_sync),
    ("resolveManifestAssets", route_addon_protocol),
    ("resolveNextEpisode", route_library_state),
    ("resourceFetchExecutionPolicy", route_resource_plan),
    ("resourceFetchPlan", route_resource_plan),
    ("resourceKindToResource", route_resource_plan),
    ("searchResultGrouping", route_search_plan),
    ("sha256VerificationStatus", route_addon_store),
    ("shortcutAssign", route_shortcuts),
    ("shortcutBindings", route_shortcuts),
    ("shortcutResolve", route_shortcuts),
    ("shortenSynopsis", route_content_identity),
    ("shouldAttemptAnimeTracking", route_anime_detection),
    ("shuffleEpisodePick", route_player_policy),
    ("simklCalendarPlan", route_simkl),
    ("simklSyncApply", route_simkl),
    ("simklSyncPlan", route_simkl),
    ("streamRequestIds", route_content_identity),
    ("streamSubtitlesResult", route_player_policy),
    ("subtitleTracks", route_addon_resource),
    ("terminalRecommendationEligibility", route_library_state),
    ("terminalRecommendationPlan", route_library_state),
    ("tmdbBuiltinCatalogUrl", route_tmdb),
    ("tmdbBuiltinManifest", route_tmdb),
    ("tmdbBulkMetas", route_tmdb),
    ("tmdbBulkVideosToTrailers", route_tmdb),
    ("tmdbDetailRequestPlan", route_tmdb),
    ("tmdbDetailRequestUrlsFromFind", route_tmdb),
    ("tokenMergePlan", route_profile_contract),
    ("torrentRuntimeInfo", route_stream_policy),
    ("trailerSubtitleSelectionPlan", route_trailer_subtitles),
    ("trailerYoutubeVideoIds", route_trailer_subtitles),
    ("traktRemapVideoIds", route_trakt),
    ("traktShowIdFromEpisodeId", route_trakt),
];

#[cfg(test)]
mod tests {
    use super::METHODS;

    const SOURCES: &[&str] = &[
        include_str!("../services/auth/routes.rs"),
        include_str!("../services/tracking/external_sync/routes.rs"),
        include_str!("../services/sync/fluxa/routes.rs"),
        include_str!("../addons/routes/discovery.rs"),
        include_str!("../addons/routes/plan.rs"),
        include_str!("../addons/routes/plugins.rs"),
        include_str!("../addons/routes/protocol.rs"),
        include_str!("../addons/routes/resource.rs"),
        include_str!("../addons/routes/store.rs"),
        include_str!("../catalog/routes/anime.rs"),
        include_str!("../catalog/routes/identity.rs"),
        include_str!("../catalog/routes/search.rs"),
        include_str!("../catalog/routes/warnings.rs"),
        include_str!("../headless_engine/routes.rs"),
        include_str!("../library/routes/calendar.rs"),
        include_str!("../library/routes/state.rs"),
        include_str!("../library/routes/watchlist.rs"),
        include_str!("../player/routes/cast.rs"),
        include_str!("../player/routes/flow.rs"),
        include_str!("../player/routes/media_session.rs"),
        include_str!("../player/routes/overlay.rs"),
        include_str!("../player/routes/segments.rs"),
        include_str!("../player/routes/policy.rs"),
        include_str!("../player/routes/scrobble.rs"),
        include_str!("../player/routes/stream_badges.rs"),
        include_str!("../player/routes/stream_policy.rs"),
        include_str!("../player/routes/trailer_subtitles.rs"),
        include_str!("../profile/routes.rs"),
        include_str!("../services/tracking/anilist/routes.rs"),
        include_str!("../services/tracking/mdblist/routes.rs"),
        include_str!("../services/mediaserver/routes.rs"),
        include_str!("../services/sync/nuvio/routes.rs"),
        include_str!("../services/provider_routes.rs"),
        include_str!("../services/metadata/publicmetadb/routes.rs"),
        include_str!("../services/tracking/simkl/routes.rs"),
        include_str!("../services/metadata/tmdb/routes.rs"),
        include_str!("../services/tracking/trakt/routes.rs"),
        include_str!("../settings/routes/data_policy.rs"),
        include_str!("../settings/routes/discord_presence.rs"),
        include_str!("../settings/routes/shortcuts.rs"),
        include_str!("../settings/routes/version_policy.rs"),
    ];

    fn arm_methods(line: &str) -> Vec<&str> {
        let Some(rest) = line.strip_prefix("        ") else {
            return Vec::new();
        };
        let mut rest = rest.strip_prefix("| ").unwrap_or(rest);
        let mut found = Vec::new();
        while let Some(body) = rest.strip_prefix('"') {
            let Some(end) = body.find('"') else {
                return Vec::new();
            };
            found.push(&body[..end]);
            rest = &body[end + 1..];
            match rest.strip_prefix(" | ") {
                Some(next) if next.starts_with('"') => rest = next,
                _ => break,
            }
        }
        match rest.trim_end() {
            "" | " |" => found,
            tail if tail.starts_with(" =>") => found,
            _ => Vec::new(),
        }
    }

    #[test]
    fn table_lists_every_router_arm() {
        let mut arms = Vec::new();
        for source in SOURCES {
            let mut in_router = false;
            for line in source.lines() {
                if line.contains(" fn route") && line.contains("(method: &str") {
                    in_router = true;
                } else if line.starts_with('}') {
                    in_router = false;
                } else if in_router {
                    arms.extend(arm_methods(line));
                }
            }
        }
        arms.sort_unstable();
        let names = METHODS.iter().map(|(name, _)| *name).collect::<Vec<_>>();
        assert!(names.windows(2).all(|pair| pair[0] < pair[1]));
        assert_eq!(arms, names);
    }
}
