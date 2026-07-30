import { useCallback, useEffect, useRef, useState, type RefObject } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { open as shellOpen } from '@tauri-apps/plugin-shell';
import * as Sentry from '@sentry/react';
import { dispatchAction, coreDetectAnimePlayback, coreInvoke, corePlaybackIntroLookupContentId, corePlaybackPreparePlan, coreResolveNextEpisode, coreCanPrefetchNextEpisode, coreSelectNextEpisodeStream, coreStreamShellPlan, coreTorrentStatusInfo, coreTorrentReadyBudget } from '../core/engine';

function debugLog(msg: string) {
  void invoke('debug_log', { msg }).catch(() => {});
}
import {
  type EmbeddedMpvStatus,
  destroyEmbeddedMpv,
  embeddedMpvHide,
  embeddedMpvShowLoading,
  embeddedMpvSetLoadingArtwork,
  embeddedMpvStatus,
  embeddedMpvStop,
  prefetchPlayerArtwork,
  playerClearChapters,
  playerClearEpisodes,
  playerClearSkipInfo,
  playerSetEpisodes,
  playerSetSkipInfo,
  playerTorrentStats,
  startTorrentStream,
  stopTorrentStream,
} from '../core/mpvPlayer';
import { fetchPlaybackSkipSegments, fetchStreamsForEpisode, fetchMetaVideos, pumpEffects } from '../core/effectRunner';
import { fetchContentLogo } from '../core/detailEffects';
import { loadAddons } from '../core/libraryOps';
import { appPrefs, prefBool, prefString } from '../core/appPrefs';
import { getLanguage, t } from '../i18n';
import {
  playerDisplayTitle,
  playerArtwork,
  formatNextEpisodeSubtitle,
  withCloseTimeout,
} from '../core/playerUtils';
import type { PlayerDisplayTitle, PlayerArtwork, PlaybackPreparePlan } from '../core/playerUtils';
import { resolvePlaybackSubtitles } from '../core/subtitles';
import type { ResolvedSubtitles } from '../core/subtitles';
import { persistLastPlaybackSource } from '../core/libraryStorage';
import type { AppState, Meta, Video, Stream, AddonDescriptor, UserProfile } from '../core/types';
import { usePlayerNativeEvents } from './usePlayerNativeEvents';
import { usePlayerMpvLifecycle } from './usePlayerMpvLifecycle';
import { usePlayerScrobbling } from './usePlayerScrobbling';
import { usePlayerProgressPersistence } from './usePlayerProgressPersistence';
import { applyPlayerCloseActions } from './playerCloseActions';
import { usePlayerRetry } from './usePlayerRetry';
import { usePlayerPlaybackStart } from './usePlayerPlaybackStart';

function playbackErrorMessage(error: unknown, fallback: string): string {
  if (error instanceof Error && error.message.trim()) return error.message.trim();
  if (typeof error === 'string' && error.trim()) return error.trim();
  if (error && typeof error === 'object' && 'message' in error) {
    const message = String((error as { message?: unknown }).message ?? '').trim();
    if (message) return message;
  }
  return fallback;
}
export type PlayerLoadingOverlayState = {
  background?: string | null;
  logo?: string | null;
  title?: string;
  episodeLine?: string;
  status?: string;
  error?: string | null;
  source?: {
    title?: string;
    addon?: string;
    filename?: string;
    fileIdx?: number;
    infoHash?: string;
    sources?: string[];
  };
};

interface UsePlayerOptions {
  stateRef: React.MutableRefObject<AppState>;
  activeProfile: UserProfile | null;
  updateState: (s: Partial<AppState>) => void;
  onProfileUpdated?: (profile: UserProfile) => void;
  onEpisodePlaybackFailed?: (meta: Meta, episode: Video, message: string) => Promise<void> | void;
}

interface UsePlayerResult {
  playerLoadingOverlay: PlayerLoadingOverlayState | null;
  playerUrl: string | null;
  playerTitle: string | undefined;
  playerEpisodeTitle: string | undefined;
  playerEpisode: Video | null;
  playerUsesTorrent: boolean;
  playerPosterUrl: string | undefined;
  playerLogoUrl: string | undefined;
  playerMetaId: string | undefined;
  playerSubtitleUrl: string | undefined;
  playerStreamHeaders: Record<string, string> | undefined;
  playingStreamRef: RefObject<Stream | null>;
  playingMetaRef: RefObject<Meta | null>;
  playerPlaybackError: string | null;
  playerSubtitleWarning: string[] | null;
  dismissSubtitleWarning: () => void;
  handlePlay: (stream: Stream, meta?: Meta, episode?: Video | null, resumeAtSeconds?: number, totalDurationSeconds?: number, sourceCandidates?: Stream[], openSourcePickerOnFailure?: boolean) => Promise<void>;
  closePlayer: () => Promise<void>;
  notifyFirstFrame: () => void;
  flushProgressOnQuit: () => Promise<void>;
}

export function usePlayer({ stateRef, activeProfile, updateState, onProfileUpdated, onEpisodePlaybackFailed }: UsePlayerOptions): UsePlayerResult {
  const [playerUrl, setPlayerUrl] = useState<string | null>(null);
  const [playerTitle, setPlayerTitle] = useState<string | undefined>();
  const [playerEpisodeTitle, setPlayerEpisodeTitle] = useState<string | undefined>();
  const [playerEpisode, setPlayerEpisode] = useState<Video | null>(null);
  const [playerPosterUrl, setPlayerPosterUrl] = useState<string | undefined>();
  const [playerLogoUrl, setPlayerLogoUrl] = useState<string | undefined>();
  const [playerMetaId, setPlayerMetaId] = useState<string | undefined>();
  const [playerSubtitleUrl, setPlayerSubtitleUrl] = useState<string | undefined>();
  const [playerStreamHeaders, setPlayerStreamHeaders] = useState<Record<string, string> | undefined>();
  const [playerUsesTorrent, setPlayerUsesTorrent] = useState(false);
  const [playerLoadingOverlay, setPlayerLoadingOverlay] = useState<PlayerLoadingOverlayState | null>(null);
  const [playerPlaybackError, setPlayerPlaybackError] = useState<string | null>(null);
  const [playerSubtitleWarning, setPlayerSubtitleWarning] = useState<string[] | null>(null);

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
  const playingSourceCandidatesRef = useRef<Stream[]>([]);
  const attemptedSourceKeysRef = useRef<Set<string>>(new Set());
  const lastResumeAtSecondsRef = useRef<number | undefined>(undefined);
  const lastTotalDurationSecondsRef = useRef<number | undefined>(undefined);
  const playingNextEpisodeRef = useRef<Video | null>(null);
  const prefetchedNextEpRef = useRef<{ episodeId: string; stream: Stream } | null>(null);
  const playerUsesTorrentRef = useRef(false);
  const lastPlaybackStatusRef = useRef<EmbeddedMpvStatus | null>(null);
  const openSourcePickerOnFailureRef = useRef(false);
  const firstFrameHandoffPendingRef = useRef(false);
  const scrobbleStartedRef = useRef(false);
  const scrobbleStoppedRef = useRef(false);
  const scrobbleWasPausedRef = useRef(false);

  const playerLoadingOverlayRef = useRef<PlayerLoadingOverlayState | null>(null);

  useEffect(() => { activeProfileRef.current = activeProfile; }, [activeProfile]);
  useEffect(() => { playerUsesTorrentRef.current = playerUsesTorrent; }, [playerUsesTorrent]);
  useEffect(() => { playerLoadingOverlayRef.current = playerLoadingOverlay; }, [playerLoadingOverlay]);

  const setLoadingStatus = useCallback((status: string) => {
    setPlayerLoadingOverlay((prev) => (prev ? { ...prev, status } : prev));
  }, []);

  const isPlaybackCancelled = useCallback((generation: number) => playGenerationRef.current !== generation, []);

  const playInEmbeddedMpv = usePlayerMpvLifecycle({
    stateRef,
    mpvInitializedRef,
    playerUsesTorrentRef,
    inNativePlayerRef,
    artworkPrefetchRef,
    pendingArtworkRef,
    isCancelled: isPlaybackCancelled,
    stopTorrent: () => stopTorrentStream().catch(() => false),
    debugLog,
    setPlayerTitle,
    setPlayerUrl,
    setPlayerUsesTorrent,
    setPlayerSubtitleUrl,
    setPlayerSubtitleWarning,
  });

  const failPlayerLoading = useCallback(async (message: string) => {
    ++playGenerationRef.current;
    const shouldStopTorrent = playerUsesTorrentRef.current;
    setPlayerUrl(null);
    setPlayerSubtitleUrl(undefined);
    setPlayerStreamHeaders(undefined);
    setPlayerUsesTorrent(false);
    setPlayerPlaybackError(null);
    inNativePlayerRef.current = false;
    setPlayerLoadingOverlay((prev) => {
      if (prev) return { ...prev, error: message };
      const stream = playingStreamRef.current ?? undefined;
      const title = playerDisplayTitle(playingMetaRef.current ?? undefined, playingEpisodeRef.current, stream);
      const artwork = pendingArtworkRef.current ?? playerArtwork(playingMetaRef.current ?? undefined, playingEpisodeRef.current);
      return {
        background: artwork.background,
        logo: artwork.logo,
        title: title.contentTitle,
        episodeLine: title.episodeLine,
        error: message,
        source: stream ? {
          title: stream.name ?? stream.title ?? stream.description,
          addon: stream.addonName,
          filename: stream.behaviorHints?.filename,
          fileIdx: stream.fileIdx,
          infoHash: stream.infoHash,
          sources: stream.sources,
        } : undefined,
      };
    });
    await embeddedMpvHide().catch(() => undefined);
    await embeddedMpvStop().catch(() => undefined);
    if (shouldStopTorrent) await stopTorrentStream().catch(() => false);
  }, []);

  const nextRetrySource = usePlayerRetry({ stateRef, sourceCandidatesRef: playingSourceCandidatesRef, attemptedSourceKeysRef });

  const showPlayerLoading = useCallback((
    generation: number,
    title: PlayerDisplayTitle,
    artwork: PlayerArtwork,
    stream: Stream,
  ): Promise<unknown> => {
    const isCancelled = () => playGenerationRef.current !== generation;
    setPlayerTitle(title.contentTitle);
    setPlayerEpisodeTitle(title.episodeLine ?? undefined);
    pendingArtworkRef.current = artwork;
    setPlayerLoadingOverlay({
      background: artwork.background,
      logo: artwork.logo,
      title: title.contentTitle,
      episodeLine: title.episodeLine,
      status: t('player.status_preparing'),
      source: {
        title: stream.name ?? stream.title ?? stream.description,
        addon: stream.addonName,
        filename: stream.behaviorHints?.filename,
        fileIdx: stream.fileIdx,
        infoHash: stream.infoHash,
        sources: stream.sources,
      },
    });

    if (!inNativePlayerRef.current) {
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
    return ready;
  }, []);

  const dispatchScrobbleLifecycle = usePlayerScrobbling({
    playerUrl,
    activeProfileRef,
    playingMetaRef,
    playingEpisodeRef,
    closingPlayerRef,
    inNativePlayerRef,
    lastPlaybackStatusRef,
    scrobbleStartedRef,
    scrobbleStoppedRef,
    scrobbleWasPausedRef,
    onProfileUpdated,
  });
  const closePlayer = useCallback(async () => {
    if (closingPlayerRef.current) return;
    closingPlayerRef.current = true;
    ++playGenerationRef.current;
    const closeGeneration = playGenerationRef.current;
    const captureMeta = playingMetaRef.current;
    const captureEpisode = playingEpisodeRef.current;
    const captureStream = playingStreamRef.current;
    const shouldStopTorrent = playerUsesTorrentRef.current;
    setPlayerUrl(null);
    setPlayerTitle(undefined);
    setPlayerEpisode(null);
    setPlayerPosterUrl(undefined);
    setPlayerLogoUrl(undefined);
    setPlayerMetaId(undefined);
    setPlayerSubtitleUrl(undefined);
    setPlayerStreamHeaders(undefined);
    setPlayerUsesTorrent(false);
    setPlayerLoadingOverlay(null);
    setPlayerPlaybackError(null);
    setPlayerSubtitleWarning(null);
    inNativePlayerRef.current = false;
    await playerClearSkipInfo();
    void playerClearChapters();
    void playerClearEpisodes();
    try {
      const status = await withCloseTimeout(embeddedMpvStatus(), 700).catch(() => null) ?? lastPlaybackStatusRef.current;
      if (!status && captureMeta) {
        debugLog('closePlayer: embeddedMpvStatus timed out and no cached playback status is available');
      }
      if (captureMeta && captureStream) {
        await persistLastPlaybackSource(captureMeta, captureStream).catch(() => undefined);
      }
      await withCloseTimeout(embeddedMpvHide(), 400).catch(() => undefined);
      await withCloseTimeout(embeddedMpvStop(), 900).catch(() => undefined);
      await withCloseTimeout(destroyEmbeddedMpv(), 900).catch(() => undefined);
      closingPlayerRef.current = false;
      if (status && captureMeta) {
        const timePos = parseFloat(status.timePos ?? '0');
        const duration = parseFloat(status.duration ?? '0');
        const closePrefs = appPrefs(stateRef.current);
        const closePlan = await coreInvoke<{
          shouldScrobble: boolean;
          progressAction: Record<string, unknown>;
          markWatchedAction: Record<string, unknown> | null;
          upNextAction: Record<string, unknown> | null;
          reloadHome: boolean;
        }>('playbackClosePlan', JSON.stringify({
          meta: captureMeta,
          episode: captureEpisode,
          stream: captureStream,
          nextEpisode: playingNextEpisodeRef.current,
          timePos,
          duration,
          streamIndex: stateRef.current.player.currentStreamIndex ?? null,
          prefs: closePrefs,
        }));
        if (scrobbleStartedRef.current) await dispatchScrobbleLifecycle('stop', status);
        await applyPlayerCloseActions([closePlan?.progressAction, closePlan?.markWatchedAction, closePlan?.upNextAction], updateState);
        if (closePlan?.reloadHome) {
          void dispatchAction(JSON.stringify({ type: 'homeLoadRequested', language: getLanguage(), force: true })).then((result) => {
            if (!result) return;
            updateState(result.state);
            if (result.effects.length > 0) void pumpEffects(result.effects, updateState);
          }).catch(() => undefined);
        }
      }
    } finally {
      const stillCurrent = playGenerationRef.current === closeGeneration;
      if (shouldStopTorrent && stillCurrent) {
        await stopTorrentStream().catch(() => false);
      }
      closingPlayerRef.current = false;
    }
    if (playGenerationRef.current === closeGeneration) {
      playingMetaRef.current = null;
      playingEpisodeRef.current = null;
      playingStreamRef.current = null;
      playingSourceCandidatesRef.current = [];
      attemptedSourceKeysRef.current = new Set();
      lastResumeAtSecondsRef.current = undefined;
      lastTotalDurationSecondsRef.current = undefined;
      lastPlaybackStatusRef.current = null;
    }
  }, [stateRef, updateState, dispatchScrobbleLifecycle]);

  const saveProgressTick = usePlayerProgressPersistence({
    playerUrl,
    stateRef,
    closingPlayerRef,
    inNativePlayerRef,
    playingMetaRef,
    playingEpisodeRef,
    playingStreamRef,
    lastPlaybackStatusRef,
    updateState,
  });

  const handlePlay = usePlayerPlaybackStart({
    stateRef, onEpisodePlaybackFailed, playGenerationRef, scrobbleStartedRef, scrobbleStoppedRef, scrobbleWasPausedRef, setPlayerPlaybackError, setPlayerSubtitleWarning, openSourcePickerOnFailureRef, setPlayerUrl, playingSourceCandidatesRef, attemptedSourceKeysRef, setPlayerUsesTorrent, prefetchedNextEpRef, playingMetaRef, playingEpisodeRef, playingNextEpisodeRef, playingStreamRef, lastResumeAtSecondsRef, lastTotalDurationSecondsRef, setPlayerEpisode, playerDisplayTitle, playerArtwork, setPlayerPosterUrl, setPlayerLogoUrl, setPlayerMetaId, setPlayerStreamHeaders, artworkPrefetchRef, prefetchPlayerArtwork, showPlayerLoading, pendingArtworkRef, inNativePlayerRef, setPlayerLoadingOverlay, setLoadingStatus, playerLoadingOverlayRef, playInEmbeddedMpv, nextRetrySource, failPlayerLoading, debugLog, playbackErrorMessage,
  });
  const handleNativePlayerError = useCallback(async (message: string) => {
    const nextSource = await nextRetrySource(playingStreamRef.current);
    if (nextSource && playingMetaRef.current) {
      const status = await embeddedMpvStatus().catch(() => null);
      const timePos = Number.parseFloat(status?.timePos ?? '');
      await handlePlay(
        nextSource,
        playingMetaRef.current,
        playingEpisodeRef.current,
        Number.isFinite(timePos) && timePos > 0 ? Math.floor(timePos) : lastResumeAtSecondsRef.current,
        lastTotalDurationSecondsRef.current,
      );
      return;
    }
    if (openSourcePickerOnFailureRef.current && playingMetaRef.current && playingEpisodeRef.current && onEpisodePlaybackFailed) {
      await onEpisodePlaybackFailed(playingMetaRef.current, playingEpisodeRef.current, message);
      return;
    }
    if (!playerLoadingOverlayRef.current?.error) await failPlayerLoading(message);
  }, [failPlayerLoading, handlePlay, nextRetrySource, onEpisodePlaybackFailed]);

  const showEpisodeTransitionLoading = useCallback((meta: Meta, episode: Video, stream: Stream) => {
    const title = playerDisplayTitle(meta, episode, stream);
    const artwork = playerArtwork(meta, episode);
    setPlayerTitle(title.contentTitle);
    setPlayerEpisodeTitle(title.episodeLine ?? undefined);
    setPlayerPosterUrl(artwork.background ?? meta.poster);
    setPlayerLogoUrl(artwork.logo ?? undefined);
    setPlayerMetaId(meta.id);
    setPlayerPlaybackError(null);
    setPlayerSubtitleWarning(null);
    setPlayerLoadingOverlay({
      background: artwork.background,
      logo: artwork.logo,
      title: title.contentTitle,
      episodeLine: title.episodeLine,
      status: t('player.status_preparing'),
      source: {
        title: stream.name ?? stream.title ?? stream.description,
        addon: stream.addonName,
        filename: stream.behaviorHints?.filename,
        fileIdx: stream.fileIdx,
        infoHash: stream.infoHash,
        sources: stream.sources,
      },
    });
  }, []);

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
    onPlayerError: handleNativePlayerError,
    onEpisodePlaybackFailed,
    showEpisodeTransitionLoading,
  });

  const notifyFirstFrame = useCallback(() => {
    if (firstFrameHandoffPendingRef.current) return;
    firstFrameHandoffPendingRef.current = true;
    requestAnimationFrame(() => {
      requestAnimationFrame(() => {
        firstFrameHandoffPendingRef.current = false;
        setPlayerLoadingOverlay((prev) => (prev?.error ? prev : null));
      });
    });
  }, []);

  const dismissSubtitleWarning = useCallback(() => {
    setPlayerSubtitleWarning(null);
  }, []);

  return { playerLoadingOverlay, playerUrl, playerPlaybackError, playerSubtitleWarning, dismissSubtitleWarning, playerTitle, playerEpisodeTitle, playerEpisode, playerUsesTorrent, playerPosterUrl, playerLogoUrl, playerMetaId, playerSubtitleUrl, playerStreamHeaders, playingStreamRef, playingMetaRef, handlePlay, closePlayer, notifyFirstFrame, flushProgressOnQuit: saveProgressTick };
}
