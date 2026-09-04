package com.fluxa.app.core.rust

/** Central routing table for Android headless-core effects. */
internal suspend fun FluxaAndroidHeadlessEnvironment.dispatchEffect(
    effect: NativeHeadlessEffect,
): HeadlessEffectCompletion = when (effect.type) {
    FluxaHeadlessEffectType.FetchMetaDetail, FluxaHeadlessEffectType.FetchMetaDetailLookup -> fetchMetaDetail(effect)
    FluxaHeadlessEffectType.ReadPlaybackProgress -> readPlaybackProgress(effect)
    FluxaHeadlessEffectType.ReadDetailLocalState -> readDetailLocalState(effect)
    FluxaHeadlessEffectType.FetchDetailSecondary -> fetchDetailSecondary(effect)
    FluxaHeadlessEffectType.PrefetchDetailStreams -> prefetchDetailStreams(effect)
    FluxaHeadlessEffectType.FetchDetailStreams -> fetchDetailStreams(effect)
    FluxaHeadlessEffectType.PrepareDirectPlayback -> prepareDirectPlayback(effect)
    FluxaHeadlessEffectType.FetchIntroSegments -> fetchIntroSegments(effect)
    FluxaHeadlessEffectType.ResolveIntroImdbId -> resolveIntroImdbId(effect)

    FluxaHeadlessEffectType.LoadStreams -> loadStreams(effect)
    FluxaHeadlessEffectType.EnqueueTraktScrobble -> enqueueTraktScrobble(effect)
    FluxaHeadlessEffectType.StartTorrentStream -> startTorrentStream(effect)
    FluxaHeadlessEffectType.StopTorrent -> stopTorrent(effect)
    FluxaHeadlessEffectType.ClearPlaybackProgress -> clearPlaybackProgress(effect)
    FluxaHeadlessEffectType.SyncWatchedState -> syncWatchedState(effect)
    FluxaHeadlessEffectType.WritePlaybackProgress -> writePlaybackProgress(effect)

    FluxaHeadlessEffectType.FetchAddonManifest -> fetchAddonManifest(effect)
    FluxaHeadlessEffectType.FetchPluginManifest -> fetchPluginManifest(effect)
    FluxaHeadlessEffectType.RefreshInstalledAddons -> refreshInstalledAddons(effect)
    FluxaHeadlessEffectType.FetchAddonResource -> fetchAddonResource(effect)

    FluxaHeadlessEffectType.ReadHomeBootstrap -> readHomeBootstrap(effect)
    FluxaHeadlessEffectType.ReadLibraryState -> readLibraryState(effect)
    FluxaHeadlessEffectType.WriteLibraryCommand -> writeLibraryCommand(effect)
    FluxaHeadlessEffectType.WriteFeedback -> writeFeedback(effect)

    FluxaHeadlessEffectType.RunSearch -> runSearch(effect)
    FluxaHeadlessEffectType.RunDiscover -> runDiscover(effect)
    FluxaHeadlessEffectType.ReadDiscoverCatalogFilters -> readDiscoverCatalogFilters(effect)
    FluxaHeadlessEffectType.FetchCatalogPage, FluxaHeadlessEffectType.FetchDiscoverPage -> fetchCatalogPage(effect)
    FluxaHeadlessEffectType.FetchSeasonEpisodes -> fetchSeasonEpisodes(effect)
    FluxaHeadlessEffectType.FetchSubtitles -> fetchSubtitles(effect)

    FluxaHeadlessEffectType.RunExternalSync,
    FluxaHeadlessEffectType.RunAuthFlow,
    FluxaHeadlessEffectType.ExchangeAuthCode,
    FluxaHeadlessEffectType.RefreshAuthToken,
    FluxaHeadlessEffectType.SyncExternalIntegration -> authEffectHandler.execute(effect)

    FluxaHeadlessEffectType.ReadCalendarMonth,
    FluxaHeadlessEffectType.ReplaceExternalContinueWatching,
    FluxaHeadlessEffectType.UpdateCalendarWidget,
    FluxaHeadlessEffectType.NotifyReleasedEpisodes -> calendarEffectHandler.execute(effect)

    FluxaHeadlessEffectType.EnqueueOfflineDownload -> offlineEffectHandler.enqueue(effect)
    FluxaHeadlessEffectType.WriteSettings -> writeSettings(effect)

    FluxaHeadlessEffectType.FetchYoutubeTrailerWatchConfig,
    FluxaHeadlessEffectType.FetchYoutubeTrailerPlayer,
    FluxaHeadlessEffectType.FetchYoutubeTrailerPlayerScript -> executeTrailerHttpEffect(effect)

    else -> error(effect, "unsupported_effect")
}
