package com.fluxa.app.core.rust

/** Central routing table for Android headless-core effects. */
internal suspend fun FluxaAndroidHeadlessEnvironment.dispatchEffect(
    effect: NativeHeadlessEffect,
): HeadlessEffectCompletion = when (effect.type) {
    "fetchMetaDetail", "fetchMetaDetailLookup" -> fetchMetaDetail(effect)
    "readPlaybackProgress" -> readPlaybackProgress(effect)
    "readDetailLocalState" -> readDetailLocalState(effect)
    "fetchDetailSecondary" -> fetchDetailSecondary(effect)
    "prefetchDetailStreams" -> prefetchDetailStreams(effect)
    "fetchDetailStreams" -> fetchDetailStreams(effect)
    "prepareDirectPlayback" -> prepareDirectPlayback(effect)
    "fetchIntroSegments" -> fetchIntroSegments(effect)
    "resolveIntroImdbId" -> resolveIntroImdbId(effect)

    "loadStreams" -> loadStreams(effect)
    "enqueueTraktScrobble" -> enqueueTraktScrobble(effect)
    "startTorrentStream" -> startTorrentStream(effect)
    "stopTorrent" -> stopTorrent(effect)
    "clearPlaybackProgress" -> clearPlaybackProgress(effect)
    "syncWatchedState" -> syncWatchedState(effect)
    "writePlaybackProgress" -> writePlaybackProgress(effect)

    "fetchAddonManifest" -> fetchAddonManifest(effect)
    "fetchPluginManifest" -> fetchPluginManifest(effect)
    "refreshInstalledAddons" -> refreshInstalledAddons(effect)
    "fetchAddonResource" -> fetchAddonResource(effect)

    "readHomeBootstrap" -> readHomeBootstrap(effect)
    "readLibraryState" -> readLibraryState(effect)
    "writeLibraryCommand" -> writeLibraryCommand(effect)
    "writeFeedback" -> writeFeedback(effect)

    "runSearch" -> runSearch(effect)
    "runDiscover" -> runDiscover(effect)
    "readDiscoverCatalogFilters" -> readDiscoverCatalogFilters(effect)
    "fetchCatalogPage", "fetchDiscoverPage" -> fetchCatalogPage(effect)
    "fetchSeasonEpisodes" -> fetchSeasonEpisodes(effect)
    "fetchSubtitles" -> fetchSubtitles(effect)

    "runExternalSync",
    "runAuthFlow",
    "exchangeAuthCode",
    "refreshAuthToken",
    "syncExternalIntegration" -> authEffectHandler.execute(effect)

    "readCalendarMonth",
    "replaceExternalContinueWatching",
    "updateCalendarWidget",
    "notifyReleasedEpisodes" -> calendarEffectHandler.execute(effect)

    "enqueueOfflineDownload" -> offlineEffectHandler.enqueue(effect)
    "writeSettings" -> writeSettings(effect)

    "fetchYoutubeTrailerWatchConfig",
    "fetchYoutubeTrailerPlayer",
    "fetchYoutubeTrailerPlayerScript" -> executeTrailerHttpEffect(effect)

    else -> error(effect, "unsupported_effect")
}
