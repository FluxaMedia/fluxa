import { useCallback } from 'react';
import { open as shellOpen } from '@tauri-apps/plugin-shell';
import { coreCanPrefetchNextEpisode, coreDetectAnimePlayback, coreInvoke, corePlaybackIntroLookupContentId, corePlaybackPreparePlan, coreResolveNextEpisode, coreSelectNextEpisodeStream, coreStreamShellPlan, coreTorrentReadyBudget, coreTorrentStatusInfo } from '../core/engine';
import { fetchMetaVideos, fetchPlaybackSkipSegments, fetchStreamsForEpisode } from '../core/effectRunner';
import { fetchContentLogo } from '../core/detailEffects';
import { loadEnabledAddons } from '../core/libraryOps';
import { appPrefs, prefString } from '../core/appPrefs';
import { getLanguage, t } from '../i18n';
import { formatNextEpisodeSubtitle } from '../core/playerUtils';
import type { PlaybackPreparePlan } from '../core/playerUtils';
import { resolvePlaybackSubtitles, type ResolvedSubtitles } from '../core/subtitles';
import { embeddedMpvSetLoadingArtwork, embeddedMpvStatus, playerClearChapters, playerClearSkipInfo, playerSetEpisodes, playerSetSkipInfo, playerTorrentStats, startTorrentStream } from '../core/mpvPlayer';
import type { AddonDescriptor, Meta, Stream, Video } from '../core/types';

export function usePlayerPlaybackStart(options: any) {
  const { stateRef, onEpisodePlaybackFailed, playbackScope, scrobbleStartedRef, scrobbleStoppedRef, scrobbleWasPausedRef, setPlayerPlaybackError, setPlayerSubtitleWarning, openSourcePickerOnFailureRef, setPlayerUrl, playingSourceCandidatesRef, attemptedSourceKeysRef, setPlayerUsesTorrent, prefetchedNextEpRef, playingMetaRef, playingEpisodeRef, playingNextEpisodeRef, playingStreamRef, lastResumeAtSecondsRef, lastTotalDurationSecondsRef, setPlayerEpisode, playerDisplayTitle, playerArtwork, setPlayerPosterUrl, setPlayerLogoUrl, setPlayerMetaId, setPlayerStreamHeaders, artworkPrefetchRef, prefetchPlayerArtwork, showPlayerLoading, pendingArtworkRef, inNativePlayerRef, setPlayerLoadingOverlay, setLoadingStatus, playerLoadingOverlayRef, playInEmbeddedMpv, nextRetrySource, failPlayerLoading, debugLog, playbackErrorMessage, setSkipSegmentCoverage } = options;
  const handlePlay = useCallback(async (
    stream: Stream,
    meta?: Meta,
    episode?: Video | null,
    resumeAtSeconds?: number,
    totalDurationSeconds?: number,
    sourceCandidates?: Stream[],
    openSourcePickerOnFailure = false,
  ) => {
    debugLog('handlePlay:start');
    scrobbleStartedRef.current = false;
    scrobbleStoppedRef.current = false;
    scrobbleWasPausedRef.current = false;
    setPlayerPlaybackError(null);
    setPlayerSubtitleWarning(null);
    try {
    const generation = playbackScope.advance();
    const isCancelled = () => !playbackScope.isCurrent(generation);

    const playbackPlan = await corePlaybackPreparePlan({
      stream,
      meta,
      episode,
      preferredPlayer: prefString(appPrefs(stateRef.current), 'preferredPlayer', 'mpv'),
    }) as PlaybackPreparePlan | null;
    if (isCancelled()) return;
    if (playbackPlan?.mode === 'external') {
      if (playbackPlan.url) await shellOpen(playbackPlan.url).catch(() => undefined);
      return;
    }

    openSourcePickerOnFailureRef.current = openSourcePickerOnFailure;
    setPlayerUrl(null);
    const streamPlan = await coreStreamShellPlan(stream);
    const currentStreamKey = streamPlan?.identityKey ?? '';
    const candidatePlans = await Promise.all(playingSourceCandidatesRef.current.map(coreStreamShellPlan));
    setPlayerUsesTorrent(streamPlan?.isTorrent === true);
    if (sourceCandidates?.length) {
      playingSourceCandidatesRef.current = sourceCandidates;
      attemptedSourceKeysRef.current = new Set();
    } else if (!candidatePlans.some((candidate) => candidate?.identityKey === currentStreamKey)) {
      playingSourceCandidatesRef.current = [stream];
      attemptedSourceKeysRef.current = new Set();
    }
    if (currentStreamKey) attemptedSourceKeysRef.current.add(currentStreamKey);
    prefetchedNextEpRef.current = null;
    if (meta) playingMetaRef.current = meta;
    playingEpisodeRef.current = episode ?? null;
    playingStreamRef.current = stream;
    lastResumeAtSecondsRef.current = resumeAtSeconds;
    lastTotalDurationSecondsRef.current = totalDurationSeconds;
    setPlayerEpisode(episode ?? null);


    const earlyTitle = playerDisplayTitle(meta, episode, stream);
    const earlyArtwork = playerArtwork(meta, episode);
    setPlayerPosterUrl(earlyArtwork.background ?? meta?.poster);
    setPlayerLogoUrl(earlyArtwork.logo ?? undefined);
    setPlayerMetaId(meta?.id);
    setPlayerStreamHeaders(streamPlan?.requestHeaders);
    artworkPrefetchRef.current = prefetchPlayerArtwork(earlyArtwork.background, earlyArtwork.logo).catch(() => undefined);
    let loadingArtworkPromise = showPlayerLoading(generation, earlyTitle, earlyArtwork, stream);

    if (!earlyArtwork.logo && meta?.id && meta?.type) {
      const logoPrefs = appPrefs(stateRef.current);
      const tmdbApiKey = prefString(logoPrefs, 'tmdbApiKey');
      const fanartApiKey = prefString(logoPrefs, 'fanartApiKey');
      void fetchContentLogo(meta.id, meta.type, getLanguage(), tmdbApiKey, fanartApiKey)
        .then((logo) => {
          if (!logo || isCancelled()) return;
          if (playingMetaRef.current) playingMetaRef.current = { ...playingMetaRef.current, logo };
          setPlayerLogoUrl(logo);
      setPlayerLoadingOverlay((prev: any) => (prev ? { ...prev, logo } : prev));
        })
        .catch(() => undefined);
    }

    const effectiveTotalDuration = totalDurationSeconds
      ?? (meta?.id ? (stateRef.current.library.lastWrite?.progress as Record<string, import('../core/types').LibraryItem> | undefined)?.[meta.id]?.duration : undefined);
    lastTotalDurationSecondsRef.current = effectiveTotalDuration;

    const retryNextOrFail = async (message: string) => {
      const nextSource = await nextRetrySource(stream, message === t('player.torrent_no_peers') || message === t('player.torrent_too_slow'));
      if (nextSource && meta && !isCancelled()) {
        setLoadingStatus(t('player.status_trying_next_source'));
        await handlePlay(nextSource, meta, episode, resumeAtSeconds, effectiveTotalDuration, undefined, openSourcePickerOnFailure);
        return;
      }
      if (openSourcePickerOnFailure && meta && episode && onEpisodePlaybackFailed) {
        await onEpisodePlaybackFailed(meta, episode, message);
        return;
      }
      if (!isCancelled()) await failPlayerLoading(message);
    };

    debugLog('handlePlay:resolving next episode');
    const nextEp = episode
      ? (await coreResolveNextEpisode(JSON.stringify(meta?.videos ?? []), episode.season ?? 0, episode.episode ?? episode.number ?? 0, Date.now(), true)) as Video | null
      : null;
    playingNextEpisodeRef.current = nextEp;

    debugLog(`handlePlay:plan ready mode=${playbackPlan?.mode} url=${(playbackPlan?.url ?? stream.playableUrl ?? stream.url)?.slice(0, 80)}`);
    if (isCancelled()) return;

    const url = playbackPlan?.url ?? stream.playableUrl ?? stream.url;
    if (!url) {
      await retryNextOrFail(t('player.no_playable_url'));
      return;
    }
    if (playbackPlan?.mode === 'reject') {
      await retryNextOrFail(playbackPlan.rejectReason === 'incompatible_stream'
        ? t('player.incompatible_desktop_stream')
        : t('player.no_playable_url'));
      return;
    }

    const title = playbackPlan?.title ?? earlyTitle;
    if (playbackPlan?.artwork) {
      pendingArtworkRef.current = playbackPlan.artwork;
      setPlayerLogoUrl(playbackPlan.artwork.logo ?? undefined);
      setPlayerLoadingOverlay((prev: any) =>
        prev ? { ...prev, background: playbackPlan.artwork!.background, logo: playbackPlan.artwork!.logo } : prev,
      );
      if (inNativePlayerRef.current) {
        loadingArtworkPromise = embeddedMpvSetLoadingArtwork(
          title.contentTitle ?? 'Fluxa',
          title.episodeLine,
          playbackPlan.artwork.background,
          playbackPlan.artwork.logo,
        ).catch(() => undefined);
      }
    }

    const playbackAddons = await loadEnabledAddons().catch(() => stateRef.current.addons.installed ?? [] as AddonDescriptor[]);
    const subtitlesPromise = resolvePlaybackSubtitles(
      stream,
      meta,
      episode,
      playbackPlan?.subtitleExtraArgs,
      playbackAddons,
    ).catch(() => ({ subtitles: [], failedAddons: [] } as ResolvedSubtitles));

    await playerClearSkipInfo();
    const skipPrefs = appPrefs(stateRef.current);
    const playbackPrefs = await coreInvoke<{
      nextEpisodeThresholdPercent: number;
      autoPlayNextEpisode: boolean;
      autoPlayCountdownSecs: number;
      autoSkipIntro: boolean;
      useSkipSegments: boolean;
      useAnimeSkip: boolean;
      animeSkipClientId: string;
    }>('playbackPreferencesPlan', JSON.stringify(skipPrefs));
    if (!playbackPrefs) throw new Error();
    const skipThreshold = playbackPrefs.nextEpisodeThresholdPercent;
    const skipAutoPlay = playbackPrefs.autoPlayNextEpisode;
    const skipCountdown = playbackPrefs.autoPlayCountdownSecs;
    const playableInitialNextEp = nextEp;
    await playerSetSkipInfo(
      '[]',
      playableInitialNextEp ? formatNextEpisodeSubtitle(playableInitialNextEp) : undefined,
      skipThreshold,
      skipAutoPlay,
      skipCountdown,
      playbackPrefs.autoSkipIntro,
    );
    setSkipSegmentCoverage({});
    void playerClearChapters();
    const episodeList = meta?.videos ?? [];
    void playerSetEpisodes(JSON.stringify(episodeList));
    const animeDetection = await coreDetectAnimePlayback(
      meta ?? null,
      episode ?? null,
      stream ?? null,
      stateRef.current.addons.installed ?? [],
    );
    debugLog(`handlePlay:anime detection confidence=${animeDetection.confidence} isAnime=${animeDetection.isAnime} reasons=${animeDetection.reasons.join(', ')}`);
    const skipSegmentsPromise = (async () => {
      const useSkipSegments = playbackPrefs.useSkipSegments;
      const useAnimeSkip = playbackPrefs.useAnimeSkip;
      if ((!useSkipSegments && !useAnimeSkip) || !episode) return { segments: [], coverage: {} };
      const resolvedId = useSkipSegments && meta?.id ? await corePlaybackIntroLookupContentId(meta.id) : '';
      const imdbId = resolvedId.startsWith('tt') ? resolvedId : '';
      const tmdbId = !imdbId && /^\d+$/.test(resolvedId) ? Number(resolvedId) : undefined;
      const season = episode.season ?? 1;
      const epNum = episode.episode ?? episode.number ?? 1;
      return fetchPlaybackSkipSegments({ imdbId, tmdbId, season, episode: epNum, title: meta?.name ?? '', useSkipSegments, useAnimeSkip, animeSkipClientId: playbackPrefs.animeSkipClientId });
    })();
    void skipSegmentsPromise.then(({ segments, coverage }) => {
      if (isCancelled()) return;
      setSkipSegmentCoverage(coverage);
      if (segments.length === 0) return;
      return playerSetSkipInfo(
        JSON.stringify(segments),
        playableInitialNextEp ? formatNextEpisodeSubtitle(playableInitialNextEp) : undefined,
        skipThreshold,
        skipAutoPlay,
        skipCountdown,
        playbackPrefs.autoSkipIntro,
      );
    }).catch(() => undefined);

    let loadingStatusPollActive = true;
    const pollMpvLoadingStatus = async () => {
      while (loadingStatusPollActive && !isCancelled() && playerLoadingOverlayRef.current && !playerLoadingOverlayRef.current.error) {
        const status = await embeddedMpvStatus().catch(() => null);
        if (!loadingStatusPollActive || isCancelled() || !playerLoadingOverlayRef.current || playerLoadingOverlayRef.current.error) return;
        if (!status?.loaded) {
          setLoadingStatus(t('player.status_connecting_source'));
        } else if (status.pausedForCache === 'yes') {
          const pct = Math.round(parseFloat(status.cacheBufferingState ?? '') || 0);
          setLoadingStatus(pct > 0 ? t('player.status_buffering_percent', pct) : t('player.status_buffering'));
        } else if (
          !status.hasVideoTrack ||
          status.voConfigured !== 'yes' ||
          status.framesRendered < 2 ||
          (parseFloat(status.width ?? '0') || 0) <= 0 ||
          (parseFloat(status.height ?? '0') || 0) <= 0
        ) {
          setLoadingStatus(t('player.status_connecting_source'));
        } else {
          setLoadingStatus(t('player.status_starting_playback'));
        }
        await new Promise((r) => setTimeout(r, 500));
      }
    };

    if (playbackPlan?.mode === 'torrent') {
      const budget = await coreTorrentReadyBudget();
      const retryCandidatePlans = await Promise.all(playingSourceCandidatesRef.current.map(coreStreamShellPlan));
      const MAX_PEER_RETRIES = retryCandidatePlans.some((candidate) => candidate?.identityKey !== currentStreamKey)
        ? budget.maxPeerRetriesWithAlternatives
        : budget.maxPeerRetriesSingleSource;
      const TORRENT_READY_FIRST_ATTEMPT_MS = budget.firstAttemptMs;
      const TORRENT_READY_RETRY_BUDGET_MS = budget.retryBudgetMs;
      const TORRENT_READY_PER_RETRY_MS = MAX_PEER_RETRIES > 0 ? Math.floor(TORRENT_READY_RETRY_BUDGET_MS / MAX_PEER_RETRIES) : 0;
      let statusPollActive = true;
      const retrySuffix = (retryIndex: number) => (retryIndex > 0 ? ` ${t('player.status_retry_attempt', retryIndex, MAX_PEER_RETRIES)}` : '');
      const pollTorrentStatus = async (retryIndex: number) => {
        while (statusPollActive && !isCancelled()) {
          const ts = await playerTorrentStats().catch(() => null);
          if (!statusPollActive || isCancelled()) return;
          const percent = ts && typeof ts.preload === 'number' ? Math.max(0, Math.min(100, Math.round(ts.preload))) : 0;
          if (ts && ts.active_peers > 0) {
            setLoadingStatus(t('player.status_fetching_peers', ts.active_peers, percent) + retrySuffix(retryIndex));
          } else {
            setLoadingStatus(t('player.status_fetching_torrent', percent) + retrySuffix(retryIndex));
          }
          await new Promise((r) => setTimeout(r, 700));
        }
      };
      const TORRENT_READY_HARD_LIMIT_MS = budget.hardLimitMs;
      const waitForTorrentReady = async (budgetMs: number) => {
        const startedAt = Date.now();
        let deadline = startedAt + budgetMs;
        let lastLoaded = 0;
        let sawPeers = false;
        while (Date.now() < Math.min(deadline, startedAt + TORRENT_READY_HARD_LIMIT_MS)) {
          if (isCancelled()) return;
          const ts = await playerTorrentStats().catch(() => null);
          if (ts?.stat === -1) throw new Error(ts.error?.trim() || t('player.torrent_no_peers'));
          if (ts) {
            const info = await coreTorrentStatusInfo(ts).catch(() => null);
            if (info?.isPlayableEnough) return;
            if (ts.active_peers > 0) sawPeers = true;
            if (ts.loaded_size > lastLoaded) {
              lastLoaded = ts.loaded_size;
              deadline = Date.now() + budgetMs;
            } else if (ts.active_peers > 0 || ts.resolving) {
              deadline = Math.max(deadline, Date.now() + budget.stallExtensionMs);
            }
          }
          await new Promise((r) => setTimeout(r, 700));
        }
        throw new Error(t(sawPeers ? 'player.torrent_too_slow' : 'player.torrent_no_peers'));
      };
      try {
        let localUrl: string | null = null;
        for (let retryIndex = 0; retryIndex <= MAX_PEER_RETRIES; retryIndex++) {
          try {
            debugLog(`handlePlay:starting torrent stream retryIndex=${retryIndex}`);
            statusPollActive = true;
            setLoadingStatus(t('player.status_starting_torrent') + retrySuffix(retryIndex));
            void pollTorrentStatus(retryIndex);
            localUrl = await startTorrentStream(JSON.stringify(stream), title.contentTitle, appPrefs(stateRef.current), effectiveTotalDuration * 1000);
            debugLog(`handlePlay:torrent stream started localUrl=${localUrl?.slice(0, 80)}`);
            if (isCancelled()) { statusPollActive = false; return; }
            await waitForTorrentReady(retryIndex === 0 ? TORRENT_READY_FIRST_ATTEMPT_MS : TORRENT_READY_PER_RETRY_MS);
            statusPollActive = false;
            break;
          } catch (retryErr) {
            statusPollActive = false;
            if (isCancelled()) return;
            if (retryIndex >= MAX_PEER_RETRIES) throw retryErr;
            debugLog(`handlePlay:torrent retry ${retryIndex} failed, retrying`);
            localUrl = null;
          }
        }
        if (isCancelled() || !localUrl) return;
        setLoadingStatus(t('player.status_loading_stream'));
        void pollMpvLoadingStatus();
        await playInEmbeddedMpv(generation, localUrl, title, true, subtitlesPromise, loadingArtworkPromise, resumeAtSeconds, effectiveTotalDuration, undefined, animeDetection.isAnime);
        debugLog('handlePlay:playInEmbeddedMpv (torrent) resolved');
      } catch (err) {
        statusPollActive = false;
        loadingStatusPollActive = false;
        debugLog(`handlePlay:torrent path FAILED ${err instanceof Error ? `${err.message}\n${err.stack}` : String(err)}`);
        await retryNextOrFail(playbackErrorMessage(err, t('player.playback_error') || 'Playback failed'));
        return;
      }
    } else {
      try {
        debugLog('handlePlay:calling playInEmbeddedMpv');
        setLoadingStatus(t('player.status_loading_stream'));
        void pollMpvLoadingStatus();
        await playInEmbeddedMpv(generation, url, title, false, subtitlesPromise, loadingArtworkPromise, resumeAtSeconds, effectiveTotalDuration, streamPlan?.requestHeaders, animeDetection.isAnime);
        debugLog('handlePlay:playInEmbeddedMpv resolved');
      } catch (err) {
        loadingStatusPollActive = false;
        debugLog(`handlePlay:direct path FAILED ${err instanceof Error ? `${err.message}\n${err.stack}` : String(err)}`);
        await retryNextOrFail(playbackErrorMessage(err, t('player.playback_error') || 'Playback failed'));
        return;
      }
    }

    void (async () => {
      try {
        const prefs = appPrefs(stateRef.current);
        const needVideos = !nextEp && !!meta?.id && !!meta?.type && !!episode;

        const [{ segments: segmentResult, coverage: segmentCoverage }, fetchedVideos] = await Promise.all([
          skipSegmentsPromise,
          needVideos ? fetchMetaVideos(meta!.id, meta!.type) : Promise.resolve([] as Video[]),
        ]);
        setSkipSegmentCoverage(segmentCoverage);

        if (fetchedVideos.length > 0) {
          void playerSetEpisodes(JSON.stringify(fetchedVideos));
          if (playingMetaRef.current) playingMetaRef.current = { ...playingMetaRef.current, videos: fetchedVideos };
        }

        const videoList = fetchedVideos.length > 0 ? fetchedVideos : episodeList;
        let resolvedNextEp = nextEp;
        if (fetchedVideos.length > 0 && episode) {
          resolvedNextEp = (await coreResolveNextEpisode(JSON.stringify(videoList), episode.season ?? 0, episode.episode ?? episode.number ?? 0, Date.now(), true)) as Video | null;
          playingNextEpisodeRef.current = resolvedNextEp;
        }

        const resolvedPlayableNextEp = resolvedNextEp;
        if (segmentResult.length === 0 && !resolvedPlayableNextEp) return;

        await playerSetSkipInfo(
          JSON.stringify(segmentResult),
          resolvedPlayableNextEp ? formatNextEpisodeSubtitle(resolvedPlayableNextEp) : undefined,
          playbackPrefs.nextEpisodeThresholdPercent,
          playbackPrefs.autoPlayNextEpisode,
          playbackPrefs.autoPlayCountdownSecs,
          playbackPrefs.autoSkipIntro,
        );

        if (resolvedPlayableNextEp && await coreCanPrefetchNextEpisode(JSON.stringify(prefs), JSON.stringify(stream))) {
          void (async () => {
            try {
              const result = await fetchStreamsForEpisode(resolvedPlayableNextEp.id, meta?.type ?? 'series');
              const streams = result.streams as Stream[];
              if (streams.length > 0) {
                const chosen = (await coreSelectNextEpisodeStream(JSON.stringify(streams), JSON.stringify(stream), JSON.stringify(prefs), resolvedPlayableNextEp.id)) as Stream | null;
                if (chosen) prefetchedNextEpRef.current = { episodeId: resolvedPlayableNextEp.id, stream: chosen };
              }
            } catch {}
          })();
        }
      } catch {}
    })();
    } catch (err) {
      debugLog(`handlePlay:FATAL ${err instanceof Error ? `${err.message}\n${err.stack}` : String(err)}`);
      if (openSourcePickerOnFailure && meta && episode && onEpisodePlaybackFailed) {
        await onEpisodePlaybackFailed(meta, episode, playbackErrorMessage(err, t('player.playback_error')));
        return;
      }
      await failPlayerLoading(playbackErrorMessage(err, t('player.playback_error') || 'Playback failed'));
    }
  }, [stateRef, showPlayerLoading, failPlayerLoading, playInEmbeddedMpv, nextRetrySource, setLoadingStatus, onEpisodePlaybackFailed]);

  return handlePlay;
}
