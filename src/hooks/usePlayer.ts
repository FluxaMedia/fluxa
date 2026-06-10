import { useCallback, useEffect, useRef, useState } from 'react';
import { dispatchAction, corePlaybackIntroLookupContentId, corePlaybackPreparePlan } from '../core/engine';
import {
  embeddedMpvAddSubtitle,
  embeddedMpvApplyPreferences,
  destroyEmbeddedMpv,
  embeddedMpvLoad,
  embeddedMpvHide,
  embeddedMpvShowLoading,
  embeddedMpvSetLoadingArtwork,
  embeddedMpvSetTitle,
  embeddedMpvStatus,
  embeddedMpvStop,
  prefetchPlayerArtwork,
  initEmbeddedMpv,
  playerClearChapters,
  playerClearEpisodes,
  playerClearSkipInfo,
  playerSetEpisodes,
  playerSetSkipInfo,
  startTorrentStream,
  stopTorrentStream,
} from '../core/mpvPlayer';
import { fetchPlaybackSkipSegments, fetchStreamsForEpisode, fetchMetaVideos, pumpEffects } from '../core/effectRunner';
import { appPrefs, prefBool, prefString } from '../core/appPrefs';
import { getLanguage, t } from '../i18n';
import {
  playerDisplayTitle,
  playerArtwork,
  formatNextEpisodeSubtitle,
  resolveNextEpisode,
  isEpisodeReleasedForPlayback,
  canPrefetchNextEpisode,
  selectNextEpisodeStream,
  withCloseTimeout,
} from '../core/playerUtils';
import type { PlayerDisplayTitle, PlayerArtwork, PlaybackPreparePlan, PlayerSubtitleSource } from '../core/playerUtils';
import { traktScrobbleOnClose, simklScrobbleOnClose } from '../core/scrobble';
import { resolvePlaybackSubtitles } from '../core/subtitles';
import { persistLastPlaybackSource } from '../core/libraryStorage';
import type { AppState, Meta, Video, Stream, AddonDescriptor, UserProfile } from '../core/types';
import { usePlayerNativeEvents } from './usePlayerNativeEvents';

export type PlayerLoadingOverlayState = {
  background?: string | null;
  logo?: string | null;
  title?: string;
  episodeLine?: string;
};

interface UsePlayerOptions {
  stateRef: React.MutableRefObject<AppState>;
  activeProfile: UserProfile | null;
  updateState: (s: AppState) => void;
}

interface UsePlayerResult {
  playerLoadingOverlay: PlayerLoadingOverlayState | null;
  handlePlay: (stream: Stream, meta?: Meta, episode?: Video | null, resumeAtSeconds?: number, totalDurationSeconds?: number) => Promise<void>;
  closePlayer: () => Promise<void>;
}

export function usePlayer({ stateRef, activeProfile, updateState }: UsePlayerOptions): UsePlayerResult {
  const [playerUrl, setPlayerUrl] = useState<string | null>(null);
  const [playerTitle, setPlayerTitle] = useState<string | undefined>();
  const [playerUsesTorrent, setPlayerUsesTorrent] = useState(false);
  const [playerLoadingOverlay, setPlayerLoadingOverlay] = useState<PlayerLoadingOverlayState | null>(null);

  const activeProfileRef = useRef<UserProfile | null>(null);
  const mpvInitializedRef = useRef(false);
  const closingPlayerRef = useRef(false);
  const playGenerationRef = useRef(0);
  const artworkPrefetchRef = useRef<Promise<unknown> | null>(null);
  const inNativePlayerRef = useRef(false);
  const pendingArtworkRef = useRef<PlayerArtwork | null>(null);
  const playingMetaRef = useRef<Meta | null>(null);
  const playingEpisodeRef = useRef<Video | null>(null);
  const playingStreamRef = useRef<Stream | null>(null);
  const playingNextEpisodeRef = useRef<Video | null>(null);
  const prefetchedNextEpRef = useRef<{ episodeId: string; stream: Stream } | null>(null);
  const playerUsesTorrentRef = useRef(false);

  useEffect(() => { activeProfileRef.current = activeProfile; }, [activeProfile]);
  useEffect(() => { playerUsesTorrentRef.current = playerUsesTorrent; }, [playerUsesTorrent]);

  const abortPlayerLoading = useCallback(async () => {
    setPlayerUrl(null);
    setPlayerTitle(undefined);
    setPlayerUsesTorrent(false);
    setPlayerLoadingOverlay(null);
    inNativePlayerRef.current = false;
    await embeddedMpvHide().catch(() => undefined);
  }, []);

  const showPlayerLoading = useCallback((
    generation: number,
    title: PlayerDisplayTitle,
    artwork: PlayerArtwork,
  ): Promise<unknown> => {
    const isCancelled = () => playGenerationRef.current !== generation;
    setPlayerTitle(title.contentTitle);
    pendingArtworkRef.current = artwork;

    if (!inNativePlayerRef.current) {
      setPlayerLoadingOverlay({
        background: artwork.background,
        logo: artwork.logo,
        title: title.contentTitle,
        episodeLine: title.episodeLine,
      });
      void (async () => {
        if (isCancelled()) return;
        if (!mpvInitializedRef.current) {
          await initEmbeddedMpv();
          if (!isCancelled()) mpvInitializedRef.current = true;
        }
      })();
      return Promise.resolve();
    }

    if (!isCancelled()) {
      void embeddedMpvSetLoadingArtwork(
        title.contentTitle ?? 'Fluxa',
        title.episodeLine,
        artwork.background,
        artwork.logo,
      ).catch(() => undefined);
    }
    const ready = (async () => {
      if (isCancelled()) return;
      await embeddedMpvShowLoading(title.contentTitle, title.episodeLine);
    })();
    void ready.then(async () => {
      if (!mpvInitializedRef.current && !isCancelled()) {
        await initEmbeddedMpv();
        mpvInitializedRef.current = true;
      }
    }).catch(() => undefined);
    return ready;
  }, []);

  const playInEmbeddedMpv = useCallback(async (
    generation: number,
    url: string,
    title: PlayerDisplayTitle | undefined,
    usesTorrent: boolean,
    subtitlesPromise: Promise<PlayerSubtitleSource[]>,
    artworkPromise: Promise<unknown> | undefined,
    resumeAtSeconds?: number,
    totalDurationSeconds?: number,
  ) => {
    const isCancelled = () => playGenerationRef.current !== generation;
    if (isCancelled()) return;
    setPlayerTitle(title?.contentTitle);
    setPlayerUrl(url);
    setPlayerUsesTorrent(usesTorrent);
    if (!mpvInitializedRef.current) {
      await initEmbeddedMpv();
      mpvInitializedRef.current = true;
    }
    if (isCancelled()) return;
    const prefs = appPrefs(stateRef.current);
    await embeddedMpvApplyPreferences({ ...prefs, isTorrentPlayback: usesTorrent }).catch(() => undefined);
    await embeddedMpvSetTitle(title?.contentTitle, title?.episodeLine).catch(() => undefined);
    if (isCancelled()) return;
    if (!inNativePlayerRef.current) {
      if (artworkPrefetchRef.current) {
        await Promise.race([artworkPrefetchRef.current, new Promise<void>((r) => setTimeout(r, 2000))]);
      }
      if (isCancelled()) return;
      inNativePlayerRef.current = true;
      setPlayerLoadingOverlay(null);
      await embeddedMpvShowLoading(title?.contentTitle ?? 'Fluxa', title?.episodeLine);
      void embeddedMpvSetLoadingArtwork(
        title?.contentTitle ?? 'Fluxa',
        title?.episodeLine,
        pendingArtworkRef.current?.background,
        pendingArtworkRef.current?.logo,
      ).catch(() => undefined);
      if (isCancelled()) return;
    } else if (artworkPromise) {
      await Promise.race([artworkPromise, new Promise<void>((r) => setTimeout(r, 2000))]);
      if (isCancelled()) return;
    }
    await embeddedMpvLoad(url, resumeAtSeconds, totalDurationSeconds);
    const subtitles = await subtitlesPromise.catch(() => [] as PlayerSubtitleSource[]);
    if (isCancelled()) return;
    await Promise.all(
      subtitles.map((subtitle) =>
        embeddedMpvAddSubtitle(subtitle.url, subtitle.label, subtitle.lang).catch(() => undefined),
      ),
    );
  }, [stateRef]);

  const closePlayer = useCallback(async () => {
    if (closingPlayerRef.current) return;
    closingPlayerRef.current = true;
    ++playGenerationRef.current;
    const captureMeta = playingMetaRef.current;
    const captureEpisode = playingEpisodeRef.current;
    const captureStream = playingStreamRef.current;
    const shouldStopTorrent = playerUsesTorrentRef.current;
    setPlayerUrl(null);
    setPlayerTitle(undefined);
    setPlayerUsesTorrent(false);
    setPlayerLoadingOverlay(null);
    inNativePlayerRef.current = false;
    void playerClearSkipInfo();
    void playerClearChapters();
    void playerClearEpisodes();
    try {
      await withCloseTimeout(embeddedMpvHide(), 400).catch(() => undefined);
      const status = await withCloseTimeout(embeddedMpvStatus(), 700).catch(() => null);
      await withCloseTimeout(embeddedMpvStop(), 900).catch(() => undefined);
      await withCloseTimeout(destroyEmbeddedMpv(), 900).catch(() => undefined);
      closingPlayerRef.current = false;
      if (status && captureMeta) {
        const timePos = parseFloat(status.timePos ?? '0');
        const duration = parseFloat(status.duration ?? '0');
        if (timePos > 30 && duration > 0) {
          traktScrobbleOnClose(activeProfileRef.current, captureMeta, captureEpisode, timePos, duration);
          simklScrobbleOnClose(activeProfileRef.current, captureMeta, captureEpisode, timePos, duration);
          try {
            const closePrefs = appPrefs(stateRef.current);
            const watchedThreshold = (Number(prefString(closePrefs, 'watchedThresholdPercent', '90')) || 90) / 100;
            const isWatched = duration > 0 && timePos / duration >= watchedThreshold;
            const saveResult = await dispatchAction(JSON.stringify({
              type: 'savePlaybackProgressRequested',
              meta: captureMeta,
              timeOffset: Math.floor(timePos),
              duration: Math.floor(duration),
              lastVideoId: captureEpisode?.id ?? null,
              lastStreamIndex: stateRef.current.player.currentStreamIndex ?? null,
              lastEpisodeName: captureEpisode?.name ?? captureEpisode?.title ?? null,
              lastEpisodeSeason: captureEpisode?.season ?? null,
              lastEpisodeNumber: captureEpisode?.episode ?? captureEpisode?.number ?? null,
              lastEpisodeThumbnail: captureEpisode?.thumbnail ?? null,
              lastStreamUrl: captureStream?.playableUrl ?? captureStream?.url ?? null,
              lastStreamTitle: captureStream?.title ?? captureStream?.name ?? null,
              lastAudioLanguage: null,
              lastSubtitleLanguage: null,
              scrobbleTraktPause: true,
            }));
            if (saveResult) {
              updateState(saveResult.state);
              if (saveResult.effects.length > 0) await pumpEffects(saveResult.effects, updateState);
              await persistLastPlaybackSource(captureMeta, captureStream);
              if (isWatched) {
                const videoId = captureEpisode?.id ?? captureMeta.id;
                const watchedResult = await dispatchAction(JSON.stringify({
                  type: 'markWatchedRequested',
                  seriesId: captureMeta.id,
                  videoIds: [videoId],
                  watched: true,
                  meta: captureMeta,
                  episodes: captureEpisode ? [{
                    id: captureEpisode.id,
                    name: captureEpisode.name ?? captureEpisode.title,
                    season: captureEpisode.season,
                    number: captureEpisode.episode ?? captureEpisode.number,
                    thumbnail: captureEpisode.thumbnail,
                  }] : [],
                })).catch(() => null);
                if (watchedResult) {
                  updateState(watchedResult.state);
                  if (watchedResult.effects.length > 0) await pumpEffects(watchedResult.effects, updateState);
                }
                const nextEp = playingNextEpisodeRef.current;
                if (nextEp && captureMeta.type === 'series') {
                  const upNextResult = await dispatchAction(JSON.stringify({
                    type: 'savePlaybackProgressRequested',
                    meta: captureMeta,
                    timeOffset: 1,
                    duration: 99999,
                    lastVideoId: nextEp.id ?? null,
                    lastStreamIndex: null,
                    lastEpisodeName: nextEp.name ?? nextEp.title ?? null,
                    lastEpisodeSeason: nextEp.season ?? null,
                    lastEpisodeNumber: nextEp.episode ?? nextEp.number ?? null,
                    lastEpisodeThumbnail: nextEp.thumbnail ?? null,
                    lastStreamUrl: null,
                    lastStreamTitle: null,
                    lastAudioLanguage: null,
                    lastSubtitleLanguage: null,
                    scrobbleTraktPause: false,
                  })).catch(() => null);
                  if (upNextResult) {
                    updateState(upNextResult.state);
                    if (upNextResult.effects.length > 0) await pumpEffects(upNextResult.effects, updateState);
                  }
                }
              }
              void dispatchAction(JSON.stringify({ type: 'homeLoadRequested', language: getLanguage(), force: true }))
                .then((reloadResult) => {
                  if (reloadResult) {
                    updateState(reloadResult.state);
                    if (reloadResult.effects.length > 0) void pumpEffects(reloadResult.effects, updateState);
                  }
                }).catch(() => undefined);
            }
          } catch {}
        }
      }
    } finally {
      if (shouldStopTorrent) {
        await stopTorrentStream().catch(() => false);
      }
      closingPlayerRef.current = false;
    }
    playingMetaRef.current = null;
    playingEpisodeRef.current = null;
    playingStreamRef.current = null;
  }, [stateRef, updateState]);

  const handlePlay = useCallback(async (
    stream: Stream,
    meta?: Meta,
    episode?: Video | null,
    resumeAtSeconds?: number,
    totalDurationSeconds?: number,
  ) => {
    const generation = ++playGenerationRef.current;
    const isCancelled = () => generation !== playGenerationRef.current;
    prefetchedNextEpRef.current = null;
    if (meta) playingMetaRef.current = meta;
    playingEpisodeRef.current = episode ?? null;
    playingStreamRef.current = stream;

    const earlyTitle = playerDisplayTitle(meta, episode, stream);
    const earlyArtwork = playerArtwork(meta, episode);
    artworkPrefetchRef.current = prefetchPlayerArtwork(earlyArtwork.background, earlyArtwork.logo).catch(() => undefined);
    let loadingArtworkPromise = showPlayerLoading(generation, earlyTitle, earlyArtwork);

    const effectiveTotalDuration = totalDurationSeconds
      ?? (meta?.id ? (stateRef.current.library.lastWrite?.progress as Record<string, import('../core/types').LibraryItem> | undefined)?.[meta.id]?.duration : undefined);

    const nextEp = resolveNextEpisode(meta?.videos ?? [], episode);
    playingNextEpisodeRef.current = nextEp;

    const playbackPlan = await corePlaybackPreparePlan({
      stream,
      meta,
      episode,
      preferredPlayer: prefString(appPrefs(stateRef.current), 'preferredPlayer', 'mpv'),
    }) as PlaybackPreparePlan | null;
    if (isCancelled()) return;

    const url = playbackPlan?.url ?? stream.playableUrl ?? stream.url;
    if (!url) {
      if (!isCancelled()) await abortPlayerLoading();
      alert(t('player.no_playable_url'));
      return;
    }
    if (playbackPlan?.mode === 'reject') {
      if (!isCancelled()) await abortPlayerLoading();
      alert(playbackPlan.rejectReason === 'incompatible_stream'
        ? t('player.incompatible_desktop_stream')
        : t('player.no_playable_url'));
      return;
    }

    const title = playbackPlan?.title ?? earlyTitle;
    if (playbackPlan?.artwork) {
      if (!inNativePlayerRef.current) {
        pendingArtworkRef.current = playbackPlan.artwork;
        setPlayerLoadingOverlay((prev) =>
          prev ? { ...prev, background: playbackPlan.artwork!.background, logo: playbackPlan.artwork!.logo } : prev,
        );
      } else {
        loadingArtworkPromise = embeddedMpvSetLoadingArtwork(
          title.contentTitle ?? 'Fluxa',
          title.episodeLine,
          playbackPlan.artwork.background,
          playbackPlan.artwork.logo,
        ).catch(() => undefined);
      }
    }

    const subtitlesPromise = resolvePlaybackSubtitles(
      stream,
      meta,
      episode,
      playbackPlan?.subtitleExtraArgs,
      stateRef.current.addons.installed ?? [] as AddonDescriptor[],
    ).catch(() => [] as PlayerSubtitleSource[]);

    void playerClearSkipInfo();
    void playerClearChapters();
    const episodeList = meta?.videos ?? [];
    void playerSetEpisodes(JSON.stringify(episodeList));

    if (playbackPlan?.mode === 'torrent') {
      try {
        const localUrl = await startTorrentStream(JSON.stringify(stream), title.contentTitle, appPrefs(stateRef.current));
        if (isCancelled()) return;
        await playInEmbeddedMpv(generation, localUrl, title, true, subtitlesPromise, loadingArtworkPromise, resumeAtSeconds, effectiveTotalDuration);
      } catch {
        if (!isCancelled()) await abortPlayerLoading();
        alert(t('player.playback_error') || 'Playback failed');
        return;
      }
    } else {
      try {
        await playInEmbeddedMpv(generation, url, title, false, subtitlesPromise, loadingArtworkPromise, resumeAtSeconds, effectiveTotalDuration);
      } catch {
        if (!isCancelled()) await abortPlayerLoading();
        alert(t('player.playback_error') || 'Playback failed');
        return;
      }
    }

    const playableNextEp = nextEp && isEpisodeReleasedForPlayback(nextEp) ? nextEp : null;
    if (playableNextEp) {
      const prefs0 = appPrefs(stateRef.current);
      const threshold0 = Number(prefString(prefs0, 'nextEpisodeThresholdPercent', '85')) || 85;
      const autoPlay0 = prefBool(prefs0, 'autoPlayNextEpisode', true);
      void playerSetSkipInfo('[]', formatNextEpisodeSubtitle(playableNextEp), threshold0, autoPlay0);
    }

    // Background: fetch skip segments + prefetch next episode stream
    void (async () => {
      try {
        const prefs = appPrefs(stateRef.current);
        const useIntroDb = prefBool(prefs, 'useIntroDb', true);
        const useAniSkip = prefBool(prefs, 'useAniSkip', true);
        const needVideos = !episodeList.length && !!meta?.id && !!meta?.type && !!episode;

        const [segmentResult, fetchedVideos] = await Promise.all([
          (async () => {
            if ((!useIntroDb && !useAniSkip) || !meta?.id || !episode) return [];
            const imdbId = await corePlaybackIntroLookupContentId(meta.id);
            if (!imdbId) return [];
            const season = episode.season ?? 1;
            const epNum = episode.episode ?? episode.number ?? 1;
            return fetchPlaybackSkipSegments({ imdbId, season, episode: epNum, title: meta.name, useIntroDb, useAniSkip });
          })(),
          needVideos ? fetchMetaVideos(meta!.id, meta!.type) : Promise.resolve([] as Video[]),
        ]);

        if (fetchedVideos.length > 0) void playerSetEpisodes(JSON.stringify(fetchedVideos));

        const videoList = fetchedVideos.length > 0 ? fetchedVideos : episodeList;
        let resolvedNextEp = nextEp;
        if (fetchedVideos.length > 0 && episode) {
          resolvedNextEp = resolveNextEpisode(videoList, episode);
          playingNextEpisodeRef.current = resolvedNextEp;
        }

        const resolvedPlayableNextEp = resolvedNextEp && isEpisodeReleasedForPlayback(resolvedNextEp) ? resolvedNextEp : null;
        if (segmentResult.length === 0 && !resolvedPlayableNextEp) return;

        const threshold = Number(prefString(prefs, 'nextEpisodeThresholdPercent', '85')) || 85;
        const autoPlay = prefBool(prefs, 'autoPlayNextEpisode', true);
        await playerSetSkipInfo(
          JSON.stringify(segmentResult),
          resolvedPlayableNextEp ? formatNextEpisodeSubtitle(resolvedPlayableNextEp) : undefined,
          threshold,
          autoPlay,
        );

        if (resolvedPlayableNextEp && canPrefetchNextEpisode(prefs, stream)) {
          void (async () => {
            try {
              const result = await fetchStreamsForEpisode(resolvedPlayableNextEp.id, meta?.type ?? 'series');
              const streams = result.streams as Stream[];
              if (streams.length > 0) {
                const chosen = selectNextEpisodeStream(streams, stream, prefs) ?? streams[0];
                prefetchedNextEpRef.current = { episodeId: resolvedPlayableNextEp.id, stream: chosen };
              }
            } catch {}
          })();
        }
      } catch {}
    })();
  }, [stateRef, showPlayerLoading, abortPlayerLoading, playInEmbeddedMpv]);

  usePlayerNativeEvents({
    stateRef,
    closingPlayerRef,
    playingMetaRef,
    playingStreamRef,
    playingEpisodeRef,
    playingNextEpisodeRef,
    prefetchedNextEpRef,
    closePlayer,
    handlePlay,
  });

  return { playerLoadingOverlay, handlePlay, closePlayer };
}
