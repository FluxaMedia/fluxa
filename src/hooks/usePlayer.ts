import { useCallback, useEffect, useRef, useState, type RefObject } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { open as shellOpen } from '@tauri-apps/plugin-shell';
import { dispatchAction, coreDetectAnimePlayback, coreInvoke, corePlaybackIntroLookupContentId, corePlaybackPreparePlan, coreResolveNextEpisode, coreCanPrefetchNextEpisode, coreSelectNextEpisodeStream, coreStreamShellPlan, coreTorrentStatusInfo, coreTorrentReadyBudget } from '../core/engine';
import { subscribePlayerStatus } from '../core/playerStatusStore';

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
import { useExternalPlayerTracking, type ExternalPlayerSession, type ExternalPlayerStatus } from './useExternalPlayerTracking';
import { AsyncScope } from '../core/asyncScope';
import { useWebPlayer, type WebPlayerResult } from './useWebPlayer';

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
  playerTorrentTelemetryContext: import('../core/mpvPlayer').TorrentTelemetryContext | null;
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
  handlePlay: (stream: Stream, meta?: Meta, episode?: Video | null, resumeAtSeconds?: number, totalDurationSeconds?: number, sourceCandidates?: Stream[], openSourcePickerOnFailure?: boolean, resumePercent?: number) => Promise<void>;
  closePlayer: () => Promise<void>;
  notifyFirstFrame: () => void;
  flushProgressOnQuit: () => Promise<void>;
  skipSegmentCoverage: Record<string, string[]>;
}

function useDesktopPlayer({ stateRef, activeProfile, updateState, onProfileUpdated, onEpisodePlaybackFailed }: UsePlayerOptions): UsePlayerResult {
  const [playerUrl, setPlayerUrl] = useState<string | null>(null);
  const [playerTorrentTelemetryContext, setPlayerTorrentTelemetryContext] = useState<import('../core/mpvPlayer').TorrentTelemetryContext | null>(null);
  const [playerTitle, setPlayerTitle] = useState<string | undefined>();
  const [playerEpisodeTitle, setPlayerEpisodeTitle] = useState<string | undefined>();
  const [playerEpisode, setPlayerEpisode] = useState<Video | null>(null);
  const [playerPosterUrl, setPlayerPosterUrl] = useState<string | undefined>();
  const [playerLogoUrl, setPlayerLogoUrl] = useState<string | undefined>();
  const [playerMetaId, setPlayerMetaId] = useState<string | undefined>();
  const [playerSubtitleUrl, setPlayerSubtitleUrl] = useState<string | undefined>();
  const [playerStreamHeaders, setPlayerStreamHeaders] = useState<Record<string, string> | undefined>();
  const [playerUsesTorrent, setPlayerUsesTorrent] = useState(false);
  const [skipSegmentCoverage, setSkipSegmentCoverage] = useState<Record<string, string[]>>({});
  const [playerLoadingOverlay, setPlayerLoadingOverlay] = useState<PlayerLoadingOverlayState | null>(null);
  const [playerPlaybackError, setPlayerPlaybackError] = useState<string | null>(null);
  const [playerSubtitleWarning, setPlayerSubtitleWarning] = useState<string[] | null>(null);
  const [externalPlayerSession, setExternalPlayerSession] = useState<ExternalPlayerSession | null>(null);

  const activeProfileRef = useRef<UserProfile | null>(null);
  const mpvInitializedRef = useRef(false);
  const closingPlayerRef = useRef(false);
  const playbackScopeRef = useRef(new AsyncScope());
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
  const pendingResumePercentRef = useRef<number | null>(null);
  const playingNextEpisodeRef = useRef<Video | null>(null);
  const prefetchedNextEpRef = useRef<{ episodeId: string; stream: Stream } | null>(null);
  const playerUsesTorrentRef = useRef(false);
  const lastPlaybackStatusRef = useRef<EmbeddedMpvStatus | null>(null);
  const openSourcePickerOnFailureRef = useRef(false);
  const firstFrameHandoffPendingRef = useRef(false);
  const scrobbleStartedRef = useRef(false);
  const scrobbleStoppedRef = useRef(false);
  const scrobbleWasPausedRef = useRef(false);
  const externalPlaybackFinalizedRef = useRef(false);
  const lastExternalProgressWriteRef = useRef(0);

  const playerLoadingOverlayRef = useRef<PlayerLoadingOverlayState | null>(null);

  useEffect(() => { activeProfileRef.current = activeProfile; }, [activeProfile]);
  useEffect(() => { playerUsesTorrentRef.current = playerUsesTorrent; }, [playerUsesTorrent]);
  useEffect(() => { playerLoadingOverlayRef.current = playerLoadingOverlay; }, [playerLoadingOverlay]);

  useEffect(() => {
    if (!playerUrl) return;
    const unsubscribe = subscribePlayerStatus((status) => {
      const percent = pendingResumePercentRef.current;
      if (percent === null) return;
      const duration = Number.parseFloat(status.duration ?? '');
      if (!Number.isFinite(duration) || duration <= 0) return;
      pendingResumePercentRef.current = null;
      const seconds = (percent / 100) * duration;
      void invoke('player_command', { command: `set time-pos ${seconds.toFixed(3)}` }).catch(() => undefined);
    });
    return unsubscribe;
  }, [playerUrl]);

  const setLoadingStatus = useCallback((status: string) => {
    setPlayerLoadingOverlay((prev) => (prev ? { ...prev, status } : prev));
  }, []);

  const isPlaybackCancelled = useCallback((generation: number) => !playbackScopeRef.current.isCurrent(generation), []);

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
    playbackScopeRef.current.invalidate();
    const shouldStopTorrent = playerUsesTorrentRef.current;
    setPlayerUrl(null);
    setPlayerTorrentTelemetryContext(null);
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
    const isCancelled = () => !playbackScopeRef.current.isCurrent(generation);
    setPlayerTitle(title.contentTitle);
    setPlayerEpisodeTitle(title.episodeLine ?? undefined);
    pendingArtworkRef.current = artwork;
    setPlayerLoadingOverlay((prev) => ({
      background: artwork.background,
      logo: artwork.logo ?? (prev?.title === title.contentTitle ? prev.logo : undefined),
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
    }));

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
  const finalizeExternalPlayback = useCallback(async (status: EmbeddedMpvStatus | null) => {
    if (externalPlaybackFinalizedRef.current) {
      debugLog('finalizeExternalPlayback: already finalized, skipping');
      return;
    }
    externalPlaybackFinalizedRef.current = true;
    debugLog(`finalizeExternalPlayback: status=${status ? JSON.stringify(status) : 'null'} scrobbleStarted=${scrobbleStartedRef.current}`);
    const meta = playingMetaRef.current;
    const stream = playingStreamRef.current;
    if (meta && stream && status) {
      const timePos = Number.parseFloat(status.timePos ?? '0');
      const duration = Number.parseFloat(status.duration ?? '0');
      if (Number.isFinite(timePos) && Number.isFinite(duration) && duration > 0) {
        const closePlan = await coreInvoke<{
          progressAction: Record<string, unknown>;
          markWatchedAction: Record<string, unknown> | null;
          upNextAction: Record<string, unknown> | null;
          reloadHome: boolean;
        }>('playbackClosePlan', JSON.stringify({
          meta,
          episode: playingEpisodeRef.current,
          stream,
          nextEpisode: playingNextEpisodeRef.current,
          timePos,
          duration,
          streamIndex: stateRef.current.player.currentStreamIndex ?? null,
          prefs: appPrefs(stateRef.current),
        }));
        debugLog(`finalizeExternalPlayback: closePlan=${JSON.stringify({ progressAction: closePlan?.progressAction, markWatchedAction: closePlan?.markWatchedAction, upNextAction: closePlan?.upNextAction, reloadHome: closePlan?.reloadHome })}`);
        if (scrobbleStartedRef.current) {
          debugLog(`finalizeExternalPlayback: dispatching scrobble stop timePos=${timePos} duration=${duration}`);
          await dispatchScrobbleLifecycle('stop', status);
        } else {
          debugLog('finalizeExternalPlayback: scrobble was never started, skipping stop scrobble');
        }
        await applyPlayerCloseActions([closePlan?.progressAction, closePlan?.markWatchedAction, closePlan?.upNextAction], updateState);
        debugLog('finalizeExternalPlayback: close actions applied');
        if (closePlan?.reloadHome) {
          debugLog('finalizeExternalPlayback: refreshing continue watching');
          void dispatchAction(JSON.stringify({ type: 'refreshContinueWatchingRequested', language: getLanguage() })).then((result) => {
            if (!result) return;
            updateState(result.state);
            if (result.effects.length > 0) void pumpEffects(result.effects, updateState);
          }).catch(() => undefined);
        }
      } else {
        debugLog(`finalizeExternalPlayback: skipping close plan, invalid timePos/duration timePos=${timePos} duration=${duration}`);
      }
    } else {
      debugLog(`finalizeExternalPlayback: skipping close plan, missing meta/stream/status meta=${!!meta} stream=${!!stream} status=${!!status}`);
    }
    if (playerUsesTorrentRef.current) {
      await stopTorrentStream().catch(() => false);
      setPlayerUsesTorrent(false);
    }
    setExternalPlayerSession(null);
  }, [dispatchScrobbleLifecycle, stateRef, updateState]);
  const handleExternalPlayerStatus = useCallback((status: ExternalPlayerStatus) => {
    const previousStatus = lastPlaybackStatusRef.current;
    const previousDuration = Number.parseFloat(previousStatus?.duration ?? '0');
    const previousTimePos = Number.parseFloat(previousStatus?.timePos ?? '0');
    const duration = status.duration ?? (previousDuration > 0 ? previousDuration : lastTotalDurationSecondsRef.current ?? 0);
    const timePos = status.timePos ?? (previousTimePos > 0 ? previousTimePos : 0);
    const playbackStatus = {
      pause: status.paused ? 'yes' : 'no',
      timePos: String(timePos),
      duration: String(duration),
    } as EmbeddedMpvStatus;
    lastPlaybackStatusRef.current = playbackStatus;
    if (!status.active) {
      debugLog(`handleExternalPlayerStatus: player reported inactive, finalizing timePos=${timePos} duration=${duration}`);
      void finalizeExternalPlayback(duration > 0 ? playbackStatus : null);
      return;
    }
    if (duration <= 0 || timePos < 0) {
      debugLog(`handleExternalPlayerStatus: skipping tick, invalid timePos/duration timePos=${timePos} duration=${duration}`);
      return;
    }
    if (status.paused) void dispatchScrobbleLifecycle('pause', playbackStatus);
    else void dispatchScrobbleLifecycle('start', playbackStatus);
    if (Date.now() - lastExternalProgressWriteRef.current < 30_000) return;
    debugLog(`handleExternalPlayerStatus: writing periodic progress timePos=${timePos} duration=${duration}`);
    lastExternalProgressWriteRef.current = Date.now();
    void coreInvoke<{ shouldScrobble: boolean; progressAction: Record<string, unknown> }>('playbackClosePlan', JSON.stringify({
      meta: playingMetaRef.current,
      episode: playingEpisodeRef.current,
      stream: playingStreamRef.current,
      nextEpisode: null,
      timePos,
      duration: Math.floor(duration),
      streamIndex: stateRef.current.player.currentStreamIndex ?? null,
      prefs: appPrefs(stateRef.current),
      scrobbleTraktPause: false,
    })).then(async (plan) => {
      if (!plan?.shouldScrobble || !plan.progressAction) return;
      await applyPlayerCloseActions([plan.progressAction], updateState);
    }).catch(() => undefined);
  }, [dispatchScrobbleLifecycle, finalizeExternalPlayback, stateRef, updateState]);
  const handleExternalPlayerCallback = useCallback((position: number | null, failed: boolean) => {
    debugLog(`handleExternalPlayerCallback: position=${position} failed=${failed}`);
    if (failed) {
      setPlayerPlaybackError(t('player.external_player_failed'));
      void finalizeExternalPlayback(null);
      return;
    }
    const duration = lastTotalDurationSecondsRef.current ?? 0;
    const timePos = position ?? 0;
    const status = { pause: 'no', timePos: String(timePos), duration: String(duration) } as EmbeddedMpvStatus;
    lastPlaybackStatusRef.current = status;
    void finalizeExternalPlayback(duration > 0 ? status : null);
  }, [finalizeExternalPlayback]);
  useExternalPlayerTracking({ session: externalPlayerSession, onStatus: handleExternalPlayerStatus, onCallback: handleExternalPlayerCallback });
  const closePlayer = useCallback(async () => {
    if (closingPlayerRef.current) return;
    closingPlayerRef.current = true;
    const closeGeneration = playbackScopeRef.current.advance();
    const captureMeta = playingMetaRef.current;
    const captureEpisode = playingEpisodeRef.current;
    const captureStream = playingStreamRef.current;
    const shouldStopTorrent = playerUsesTorrentRef.current;
    setPlayerUrl(null);
    setPlayerTorrentTelemetryContext(null);
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
      mpvInitializedRef.current = false;
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
          playbackStarted: status.firstFramePresented,
          streamIndex: stateRef.current.player.currentStreamIndex ?? null,
          prefs: closePrefs,
        }));
        if (scrobbleStartedRef.current) await dispatchScrobbleLifecycle('stop', status);
        await applyPlayerCloseActions([closePlan?.progressAction, closePlan?.markWatchedAction, closePlan?.upNextAction], updateState);
        if (closePlan?.reloadHome) {
          void dispatchAction(JSON.stringify({ type: 'refreshContinueWatchingRequested', language: getLanguage() })).then((result) => {
            if (!result) return;
            updateState(result.state);
            if (result.effects.length > 0) void pumpEffects(result.effects, updateState);
          }).catch(() => undefined);
        }
      }
    } finally {
      const stillCurrent = playbackScopeRef.current.isCurrent(closeGeneration);
      if (shouldStopTorrent && stillCurrent) {
        await stopTorrentStream().catch(() => false);
      }
      closingPlayerRef.current = false;
    }
    if (playbackScopeRef.current.isCurrent(closeGeneration)) {
      playingMetaRef.current = null;
      playingEpisodeRef.current = null;
      playingStreamRef.current = null;
      playingSourceCandidatesRef.current = [];
      attemptedSourceKeysRef.current = new Set();
      lastResumeAtSecondsRef.current = undefined;
      lastTotalDurationSecondsRef.current = undefined;
      lastPlaybackStatusRef.current = null;
      pendingResumePercentRef.current = null;
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
  const flushOnQuit = useCallback(async () => {
    await saveProgressTick();
    if (externalPlayerSession) {
      debugLog('flushOnQuit: external session still active, finalizing before quit');
      await finalizeExternalPlayback(lastPlaybackStatusRef.current);
    }
  }, [saveProgressTick, externalPlayerSession, finalizeExternalPlayback]);

  const handlePlay = usePlayerPlaybackStart({
    stateRef, onEpisodePlaybackFailed, playbackScope: playbackScopeRef.current, scrobbleStartedRef, scrobbleStoppedRef, scrobbleWasPausedRef, setPlayerPlaybackError, setPlayerSubtitleWarning, openSourcePickerOnFailureRef, setPlayerUrl, setPlayerTorrentTelemetryContext, playingSourceCandidatesRef, attemptedSourceKeysRef, setPlayerUsesTorrent, prefetchedNextEpRef, playingMetaRef, playingEpisodeRef, playingNextEpisodeRef, playingStreamRef, lastResumeAtSecondsRef, lastTotalDurationSecondsRef, pendingResumePercentRef, setPlayerEpisode, playerDisplayTitle, playerArtwork, setPlayerPosterUrl, setPlayerLogoUrl, setPlayerMetaId, setPlayerStreamHeaders, artworkPrefetchRef, prefetchPlayerArtwork, showPlayerLoading, pendingArtworkRef, inNativePlayerRef, setPlayerLoadingOverlay, setLoadingStatus, playerLoadingOverlayRef, playInEmbeddedMpv, nextRetrySource, failPlayerLoading, debugLog, playbackErrorMessage, setSkipSegmentCoverage, onExternalPlayerLaunched: (session: ExternalPlayerSession) => {
      if (externalPlayerSession && externalPlayerSession.sessionId !== session.sessionId) {
        debugLog(`onExternalPlayerLaunched: stopping orphaned previous session=${externalPlayerSession.sessionId}`);
        void invoke('external_player_stop', { sessionId: externalPlayerSession.sessionId }).catch(() => undefined);
      }
      externalPlaybackFinalizedRef.current = false;
      lastExternalProgressWriteRef.current = 0;
      setExternalPlayerSession(session);
    },
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
    const logo = artwork.logo ?? (playingMetaRef.current?.id === meta.id ? pendingArtworkRef.current?.logo : undefined);
    setPlayerTitle(title.contentTitle);
    setPlayerEpisodeTitle(title.episodeLine ?? undefined);
    setPlayerPosterUrl(artwork.background ?? meta.poster);
    if (logo) setPlayerLogoUrl(logo);
    setPlayerMetaId(meta.id);
    setPlayerPlaybackError(null);
    setPlayerSubtitleWarning(null);
    setPlayerLoadingOverlay({
      background: artwork.background,
      logo,
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
    scrobbleStartedRef,
    dispatchScrobbleLifecycle,
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

  return { playerLoadingOverlay, playerUrl, playerTorrentTelemetryContext, playerPlaybackError, playerSubtitleWarning, dismissSubtitleWarning, playerTitle, playerEpisodeTitle, playerEpisode, playerUsesTorrent, playerPosterUrl, playerLogoUrl, playerMetaId, playerSubtitleUrl, playerStreamHeaders, playingStreamRef, playingMetaRef, handlePlay, closePlayer, notifyFirstFrame, flushProgressOnQuit: flushOnQuit, skipSegmentCoverage };
}

export function usePlayer(options: UsePlayerOptions): UsePlayerResult | WebPlayerResult {
  if (import.meta.env.VITE_FLUXA_TARGET === 'web' || import.meta.env.VITE_FLUXA_TARGET === 'webos') {
    return useWebPlayer(options);
  }
  return useDesktopPlayer(options);
}
