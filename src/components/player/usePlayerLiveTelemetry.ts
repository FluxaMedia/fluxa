import { getCurrentWindow } from '@tauri-apps/api/window';
import { useEffect, useLayoutEffect, useRef, type Dispatch, type MutableRefObject, type RefObject, type SetStateAction } from 'react';
import { imdbButtonFor, updateDiscordPresence } from '../../core/discordPresence';
import { embeddedMpvSetCursorVisible, playerTorrentStats, playerTorrentTelemetry, type EmbeddedMpvStatus, type TorrentStats, type TorrentTelemetryContext } from '../../core/mpvPlayer';
import { setPlayerStatusPositionInterval, subscribePlayerStatus } from '../../core/playerStatusStore';
import { t } from '../../i18n';
import { addSparklineSample, fmtTime, sendCmd, skipLabelForType, type ActiveSkip, type FeedbackFlash, type SkipSegment } from './PlayerOverlayPrimitives';
import type { PlayerTelemetryControls } from './usePlayerTelemetryState';

type Setter<T> = Dispatch<SetStateAction<T>>;
type Timer = ReturnType<typeof setTimeout> | null;

type PlaybackTelemetrySession = {
  id: string;
  startedAt: number;
  context: TorrentTelemetryContext | null;
};

type Bindings = {
  skipSegments: SkipSegment[];
  nextEpSubtitle: string;
  nextEpThreshold: number;
  nextEpDismissed: boolean;
  trackPopover: 'audio' | 'sub' | 'speed' | null;
  title: string;
  episodeTitle: string;
  initialPosterUrl?: string;
  metaId?: string;
  autoSkipSegments: boolean;
  isTorrentStream: boolean;
  playbackUrl?: string | null;
  torrentTelemetryContext?: TorrentTelemetryContext | null;
  showStats: boolean;
  showTorrentPopover: boolean;
  onFirstFrame?: () => void;
  applyFills: (fraction: number, bufferFraction?: number) => void;
  flashFeedback: (icon: FeedbackFlash['icon'], label: string) => void;
  telemetry: PlayerTelemetryControls;
  setShowSeekOverlay: Setter<boolean>;
  setControlsVisible: Setter<boolean>;
  setActiveSkip: Setter<ActiveSkip | null>;
  setShowNextEpCard: Setter<boolean>;
  liveStatusRef: MutableRefObject<EmbeddedMpvStatus | null>;
  torrentStatsRef: MutableRefObject<TorrentStats | null>;
  prevPausedForCacheRef: MutableRefObject<boolean>;
  stallCountRef: MutableRefObject<number>;
  bufferHistoryRef: MutableRefObject<number[]>;
  netSpeedHistoryRef: MutableRefObject<number[]>;
  posRef: MutableRefObject<number>;
  durRef: MutableRefObject<number>;
  pausedRef: MutableRefObject<boolean>;
  firstFrameFiredRef: MutableRefObject<boolean>;
  hasAppliedInitialFillRef: MutableRefObject<boolean>;
  currentTimeRef: RefObject<HTMLSpanElement | null>;
  durationRef: RefObject<HTMLSpanElement | null>;
  lastSeekAtRef: MutableRefObject<number>;
  isDraggingRef: MutableRefObject<boolean>;
  seekOverlayTimerRef: MutableRefObject<Timer>;
  lastActivityRef: MutableRefObject<number>;
  controlsVisibleRef: MutableRefObject<boolean>;
  controlsVisible: boolean;
  overlayRef: RefObject<HTMLDivElement | null>;
  episodePanelOpenRef: MutableRefObject<boolean>;
  isOverControlsRef: MutableRefObject<boolean>;
  miniProgressRef: RefObject<HTMLDivElement | null>;
  activeSkipKeyRef: MutableRefObject<string | null>;
  autoSkippedKeysRef: MutableRefObject<Set<string>>;
  skipFillRef: RefObject<HTMLDivElement | null>;
  discordPresenceKeyRef: MutableRefObject<string | null>;
  discordPresenceSentAtRef: MutableRefObject<number>;
};

export function usePlayerLiveTelemetry(options: Bindings) {
  const optionsRef = useRef(options);
  optionsRef.current = options;
  const stallStartedAtRef = useRef<number | null>(null);
  const telemetrySessionRef = useRef<PlaybackTelemetrySession>({
    id: '',
    startedAt: Date.now(),
    context: null,
  });

  useLayoutEffect(() => {
    const sessionId = globalThis.crypto?.randomUUID?.() ?? `fx-${Date.now()}-${Math.random().toString(36).slice(2)}`;
    telemetrySessionRef.current = {
      id: sessionId,
      startedAt: Date.now(),
      context: options.torrentTelemetryContext ?? null,
    };
    stallStartedAtRef.current = null;
    options.firstFrameFiredRef.current = false;
    options.prevPausedForCacheRef.current = false;
    options.stallCountRef.current = 0;
  }, [options.playbackUrl, options.torrentTelemetryContext?.generation]);

  useEffect(() => {
    let lastBufferUpdate = 0;
    let lastSegmentUpdate = 0;
    let lastHdrSignature: string | null = null;
    const handleStatus = (status: EmbeddedMpvStatus) => {
      const {
        skipSegments, nextEpSubtitle, nextEpThreshold, nextEpDismissed, trackPopover, title, episodeTitle, initialPosterUrl, metaId, autoSkipSegments, isTorrentStream, showStats: _, showTorrentPopover: __, onFirstFrame, applyFills, flashFeedback, telemetry, setShowSeekOverlay, setControlsVisible, setActiveSkip, setShowNextEpCard, liveStatusRef, torrentStatsRef, prevPausedForCacheRef, stallCountRef, bufferHistoryRef, netSpeedHistoryRef, posRef, durRef, pausedRef, firstFrameFiredRef, hasAppliedInitialFillRef, currentTimeRef, durationRef, lastSeekAtRef, isDraggingRef, seekOverlayTimerRef, lastActivityRef, controlsVisibleRef, overlayRef, episodePanelOpenRef, isOverControlsRef, miniProgressRef, activeSkipKeyRef, autoSkippedKeysRef, skipFillRef, discordPresenceKeyRef, discordPresenceSentAtRef,
      } = optionsRef.current;
      liveStatusRef.current = status;
      const now = Date.now();
      const telemetrySession = telemetrySessionRef.current;
      const bufferUpdateDue = now - lastBufferUpdate >= 500;
      const pausedForCache = status.pausedForCache === 'yes';
      if (!isTorrentStream && bufferUpdateDue) {
        const cacheProgress = parseFloat(status.cacheBufferingState ?? '');
        telemetry.setBuffering(pausedForCache, pausedForCache ? (Number.isFinite(cacheProgress) ? Math.max(0, Math.min(100, cacheProgress)) : 0) : undefined);
        lastBufferUpdate = now;
      }
      if (pausedForCache && !prevPausedForCacheRef.current) {
        stallCountRef.current++;
        stallStartedAtRef.current = now;
        if (isTorrentStream && telemetrySession.context) void playerTorrentTelemetry('stallStarted', undefined, telemetrySession.id, telemetrySession.context);
      } else if (!pausedForCache && prevPausedForCacheRef.current && stallStartedAtRef.current != null) {
        if (isTorrentStream && telemetrySession.context) void playerTorrentTelemetry('stallEnded', now - stallStartedAtRef.current, telemetrySession.id, telemetrySession.context);
        stallStartedAtRef.current = null;
      }
      prevPausedForCacheRef.current = pausedForCache;

      const gamma = (status.colorGamma ?? '').toLowerCase();
      const sigPeak = parseFloat(status.sigPeak ?? '');
      const contentIsHdr = gamma.includes('hlg') || gamma.includes('pq') || gamma.includes('2084') || (Number.isFinite(sigPeak) && sigPeak > 1);
      const hdrSignature = `${status.hdrActive}|${gamma}|${status.sigPeak ?? ''}`;
      if (hdrSignature !== lastHdrSignature) {
        lastHdrSignature = hdrSignature;
        telemetry.setHdrLabel(status.hdrActive && contentIsHdr ? (gamma.includes('hlg') ? 'HLG' : 'HDR10') : null);
      }

      const pos = parseFloat(status.timePos ?? '0');
      const dur = parseFloat(status.duration ?? '0');
      const isPaused = status.pause === 'yes';
      const isMuted = (status as Record<string, unknown>).mute === 'yes';
      const vol = parseFloat((status as Record<string, unknown>).volume as string ?? '100');
      const buffered = parseFloat(status.demuxerCacheDuration ?? '0');
      posRef.current = pos;
      durRef.current = dur;
      pausedRef.current = isPaused;
      if (bufferUpdateDue) {
        bufferHistoryRef.current = addSparklineSample(bufferHistoryRef.current, buffered);
        netSpeedHistoryRef.current = addSparklineSample(netSpeedHistoryRef.current, parseInt(status.cacheSpeed ?? '0') || 0);
      }

      const presenceKey = `${title}|${episodeTitle}|${isPaused}`;
      const presenceDue = now - discordPresenceSentAtRef.current > 25000;
      if (title && (presenceKey !== discordPresenceKeyRef.current || presenceDue)) {
        discordPresenceKeyRef.current = presenceKey;
        discordPresenceSentAtRef.current = now;
        updateDiscordPresence({ title, detail: episodeTitle || undefined, paused: isPaused, startUnixSecs: isPaused ? undefined : Math.floor(Date.now() / 1000 - pos), endUnixSecs: !isPaused && dur > 0 ? Math.floor(Date.now() / 1000 + (dur - pos)) : undefined, posterUrl: initialPosterUrl, ...imdbButtonFor(metaId) });
      }

      if (!firstFrameFiredRef.current && onFirstFrame) {
        const hasVideoDimensions = (parseFloat(status.width ?? '0') || 0) > 0 && (parseFloat(status.height ?? '0') || 0) > 0;
        const renderedVideo = status.loaded && status.hasVideoTrack && (status.firstFramePresented || (hasVideoDimensions && status.voConfigured === 'yes' && status.framesRendered >= 2 && status.pausedForCache !== 'yes' && pos > 0.15));
        const activeAudioOnlyPlayback = status.loaded && status.trackListReady && !status.hasVideoTrack && status.pausedForCache !== 'yes' && pos > 0.05;
        if (renderedVideo || activeAudioOnlyPlayback) {
          firstFrameFiredRef.current = true;
          if (isTorrentStream && telemetrySession.context) void playerTorrentTelemetry('firstFrame', now - telemetrySession.startedAt, telemetrySession.id, telemetrySession.context);
          onFirstFrame();
        }
      }

      if (currentTimeRef.current) currentTimeRef.current.textContent = fmtTime(pos);
      if (durationRef.current) durationRef.current.textContent = fmtTime(dur);
      const fraction = dur > 0 ? pos / dur : 0;
      const torrentProgress = isTorrentStream ? torrentStatsRef.current?.progress : undefined;
      const bufferFraction = torrentProgress != null ? Math.max(0, Math.min(1, torrentProgress / 100)) : (dur > 0 ? Math.min(1, (pos + buffered) / dur) : 0);
      const seekSuppressed = now - lastSeekAtRef.current < 800 || status.seeking === 'yes';
      if (!isDraggingRef.current && (!seekSuppressed || !hasAppliedInitialFillRef.current) && dur > 0) {
        applyFills(fraction, bufferFraction);
        hasAppliedInitialFillRef.current = true;
      }
      if (status.seeking !== 'yes') {
        setShowSeekOverlay((previous) => {
          if (!previous) return previous;
          if (seekOverlayTimerRef.current) { clearTimeout(seekOverlayTimerRef.current); seekOverlayTimerRef.current = null; }
          return false;
        });
      }
      telemetry.setPlayback(isPaused, isMuted, Math.round(vol));

      const idle = now - lastActivityRef.current;
      if (idle > 3000 && !isPaused && !episodePanelOpenRef.current && trackPopover === null && !isOverControlsRef.current && controlsVisibleRef.current) {
        controlsVisibleRef.current = false;
        setControlsVisible(false);
        overlayRef.current?.classList.add('fluxa-cursor-hidden');
        getCurrentWindow().setCursorVisible(false).catch(() => {});
        embeddedMpvSetCursorVisible(false).catch(() => {});
      }
      if (miniProgressRef.current) miniProgressRef.current.style.width = `${(fraction * 100).toFixed(3)}%`;

      if (now - lastSegmentUpdate < 1000) return;
      lastSegmentUpdate = now;
      const posMs = pos * 1000;
      const segment = skipSegments.find((item) => posMs >= item.startTime && posMs < item.endTime);
      const newSkipKey = segment ? `${segment.type}:${segment.endTime}` : null;
      if (newSkipKey !== activeSkipKeyRef.current) {
        activeSkipKeyRef.current = newSkipKey;
        if (segment && newSkipKey && autoSkipSegments && !autoSkippedKeysRef.current.has(newSkipKey)) {
          autoSkippedKeysRef.current.add(newSkipKey);
          lastSeekAtRef.current = now;
          sendCmd(`set time-pos ${Math.floor(segment.endTime / 1000)}`);
          flashFeedback('seekFwd', t('player.skipped'));
          setActiveSkip(null);
        } else {
          const outroEndsPlayback = segment?.type === 'outro' && dur > 0 && segment.endTime >= dur * 1000 - 500;
          setActiveSkip(segment && !outroEndsPlayback ? { label: skipLabelForType(segment.type), startMs: segment.startTime, endMs: segment.endTime, type: segment.type } : null);
        }
      }
      if (segment && skipFillRef.current) {
        const span = segment.endTime - segment.startTime;
        const skipFraction = span > 0 ? Math.min(1, Math.max(0, (posMs - segment.startTime) / span)) : 0;
        skipFillRef.current.style.width = `${(skipFraction * 100).toFixed(2)}%`;
      }
      const outroEndsPlayback = segment?.type === 'outro' && dur > 0 && segment.endTime >= dur * 1000 - 500;
      setShowNextEpCard((previous) => {
        const next = dur > 0 && !!nextEpSubtitle && !nextEpDismissed && (outroEndsPlayback || (pos / dur) * 100 >= nextEpThreshold);
        return previous === next ? previous : next;
      });
    };
    return subscribePlayerStatus(handleStatus);
  }, []);

  useEffect(() => {
    setPlayerStatusPositionInterval(options.controlsVisible);
  }, [options.controlsVisible]);

  useEffect(() => {
    const tick = async () => {
      const { showStats, showTorrentPopover, isTorrentStream, liveStatusRef, telemetry, torrentStatsRef } = optionsRef.current;
      if (showStats && liveStatusRef.current) telemetry.setStatsSnap({ ...liveStatusRef.current });
      if (!isTorrentStream && !showTorrentPopover) {
        torrentStatsRef.current = null;
        telemetry.setTorrentStatsSnap(null);
        return;
      }
      const raw = await playerTorrentStats().catch(() => null);
      const stats = raw && typeof raw.stat === 'number' ? raw : null;
      torrentStatsRef.current = stats;
      telemetry.setTorrentStatsSnap(stats);
      if (isTorrentStream) {
        if (stats && stats.preload < 100) telemetry.setBuffering(true, Math.max(0, Math.min(100, stats.preload)));
        else telemetry.setBuffering(false);
      }
      if (stats) telemetry.recordTorrentSpeed(stats.download_speed);
    };
    void tick();
    const interval = setInterval(() => { void tick(); }, (options.showStats || options.showTorrentPopover || options.isTorrentStream) ? 500 : 1500);
    return () => clearInterval(interval);
  }, [options.showStats, options.showTorrentPopover, options.isTorrentStream]);
}
