import { useCallback, useEffect, useRef, useState, type RefObject } from 'react';
import { t } from '../i18n';
import { invoke } from '@tauri-apps/api/core';
import { listen, emit } from '@tauri-apps/api/event';
import { currentMonitor, getCurrentWindow } from '@tauri-apps/api/window';
import { PhysicalPosition, PhysicalSize } from '@tauri-apps/api/dpi';
import {
  AudioLines,
  Camera,
  Captions,
  Cast,
  ChevronLeft,
  Download,
  Fullscreen,
  GalleryVerticalEnd,
  Gauge,
  Link2,
  Magnet,
  Minimize2,
  Pause,
  PictureInPicture2,
  Play,
  Repeat,
  RotateCcw,
  RotateCw,
  ListPlus,
  Settings,
  Share2,
  SkipForward,
  Sparkles,
  Volume1,
  Volume2,
  VolumeOff,
} from 'lucide-react';
import { setSuppressWindowGeometrySave } from '../core/windowGeometry';
import type { EmbeddedMpvStatus, TorrentStats } from '../core/mpvPlayer';
import { embeddedMpvRenderFrame, embeddedMpvSetCursorVisible, playerGetPlaybackInfo, playerGetTrackOptions, playerTorrentStats } from '../core/mpvPlayer';
import { subscribePlayerStatus } from '../core/playerStatusStore';
import type { PlayerTrackOption } from '../core/mpvPlayer';
import { VolumeBar } from './player/VolumeBar';
import { NextEpCard } from './player/NextEpCard';
import { EpisodePanel, epLabel } from './player/EpisodePanel';
import type { EpisodeInfo } from './player/EpisodePanel';
import type { Meta, Stream, Video } from '../core/types';
import { streamMagnetLink, enqueueOfflineDownload } from '../core/engine';
import { buildOfflineDownloadRequest, streamShellPlan } from '../core/streamLinks';
import { ContextMenu } from './ui/ContextMenu';
import { TrackPopover, type SubtitleCaptureCue } from './player/TrackPopover';
import { CastPopover } from './player/CastPopover';
import { TorrentStatsPopover } from './player/TorrentStatsPopover';
import { PlayerSettingsPopover } from './player/PlayerSettingsPopover';
import { SegmentMarkerPanel } from './player/SegmentMarkerPanel';
import { PlayerSeekBar } from './player/PlayerSeekBar';
import { PlayerBufferingOverlay } from './player/PlayerBufferingOverlay';
import { PlayerStatusToasts } from './player/PlayerStatusToasts';
import { PlayerShortcutsDialog } from './player/PlayerShortcutsDialog';
import { PlayerContextMenu } from './player/PlayerContextMenu';
import { PlayerHeader } from './player/PlayerHeader';
import { corePlaybackIntroLookupContentId, coreResolveNextEpisode } from '../core/engine';
import { imdbButtonFor, updateDiscordPresence } from '../core/discordPresence';
import { castDisconnect, castPlay, castPause, castSeek, castSetVolume, discoverCastDevices, proxyMediaUrl, resolveCastMediaUrl, startCasting } from '../core/cast';
import type { CastDevice } from '../core/cast';
import { comboFromEvent, findActionForCombo, loadShortcutOverrides, onShortcutsChanged, type ShortcutOverrides } from '../core/shortcuts';

import { ANIME4K_MODES, addSparklineSample, fmtTime, IconVolume, parseChapters, parseEpisodes, parseSegments, sendCmd, skipLabelForType, Sparkline, type ActiveSkip, type Chapter, type FeedbackFlash, type SkipSegment } from './player/PlayerOverlayPrimitives';

interface Props {
  closePlayer: () => Promise<void>;
  onFirstFrame?: () => void;
  initialTitle?: string;
  initialEpisodeTitle?: string;
  currentEpisode?: Video | null;
  isTorrentStream?: boolean;
  initialPosterUrl?: string;
  initialLogoUrl?: string;
  metaId?: string;
  initialSubtitleUrl?: string;
  initialStreamHeaders?: Record<string, string>;
  streamRef?: RefObject<Stream | null>;
  metaRef?: RefObject<Meta | null>;
  playbackUrl?: string | null;
  playbackError?: string | null;
  subtitleWarning?: string[] | null;
  onDismissSubtitleWarning?: () => void;
  softwareVideoActive?: boolean;
  bannerOffset?: number;
  prefs?: Record<string, unknown>;
  onDispatch?: (actionJson: string) => Promise<void> | void;
}

export function ReactPlayerOverlay({ closePlayer, onFirstFrame, initialTitle, initialEpisodeTitle, currentEpisode, isTorrentStream = false, initialPosterUrl, initialLogoUrl, metaId, initialSubtitleUrl, initialStreamHeaders, streamRef, metaRef, playbackUrl, playbackError, subtitleWarning, onDismissSubtitleWarning, softwareVideoActive = false, bannerOffset = 0, prefs, onDispatch }: Props) {
  const [paused, setPaused] = useState(false);
  const [muted, setMuted] = useState(false);
  const [volumeLevel, setVolumeLevel] = useState(100);
  const [controlsVisible, setControlsVisible] = useState(true);
  const [title, setTitle] = useState(initialTitle ?? '');
  const [episodeTitle, setEpisodeTitle] = useState(initialEpisodeTitle ?? '');
  const [chapters, setChapters] = useState<Chapter[]>([]);
  const [skipSegments, setSkipSegments] = useState<SkipSegment[]>([]);
  const [nextEpSubtitle, setNextEpSubtitle] = useState('');
  const [nextEpThreshold, setNextEpThreshold] = useState(85);
  const [autoPlayNextEpisode, setAutoPlayNextEpisode] = useState(false);
  const [autoPlayCountdownSecs, setAutoPlayCountdownSecs] = useState(7);
  const [countdown, setCountdown] = useState<number | null>(null);
  const [nextEpDismissed, setNextEpDismissed] = useState(false);
  const [episodes, setEpisodes] = useState<EpisodeInfo[]>([]);
  const [nextEpThumbnail, setNextEpThumbnail] = useState<string | null>(null);
  useEffect(() => {
    let active = true;
    if (!currentEpisode) {
      setNextEpThumbnail(null);
      return () => { active = false; };
    }
    void coreResolveNextEpisode(
      JSON.stringify(episodes),
      currentEpisode.season ?? 1,
      currentEpisode.episode ?? currentEpisode.number ?? 0,
      Date.now(),
      false,
    ).then((next) => {
      if (active) setNextEpThumbnail((next as EpisodeInfo | null)?.thumbnail ?? null);
    });
    return () => { active = false; };
  }, [episodes, currentEpisode]);
  const [showEpisodePanel, setShowEpisodePanel] = useState(false);
  const [activeSkip, setActiveSkip] = useState<ActiveSkip | null>(null);
  const [autoSkipSegments, setAutoSkipSegments] = useState(false);
  const autoSkippedKeysRef = useRef<Set<string>>(new Set());
  const [showNextEpCard, setShowNextEpCard] = useState(false);
  const [trackPopover, setTrackPopover] = useState<'audio' | 'sub' | 'speed' | null>(null);
  const [streamLinksMenuPoint, setStreamLinksMenuPoint] = useState<{ x: number; y: number } | null>(null);
  const [streamLinksPlan, setStreamLinksPlan] = useState<{ isTorrent: boolean; sourceLink?: string; downloadLink?: string } | null>(null);
  const streamLinksBtnRef = useRef<HTMLButtonElement | null>(null);
  const [miniPlayerActive, setMiniPlayerActive] = useState(false);
  const miniPlayerActiveRef = useRef(false);
  const preMiniPlayerSizeRef = useRef<PhysicalSize | null>(null);
  const preMiniPlayerPosRef = useRef<PhysicalPosition | null>(null);
  const [castPopoverOpen, setCastPopoverOpen] = useState(false);
  const [showPlayerSettings, setShowPlayerSettings] = useState(false);
  const [showSegmentMarker, setShowSegmentMarker] = useState(false);
  const [introDbImdbId, setIntroDbImdbId] = useState<string | null>(null);
  const introDbSubmitEnabled = prefs?.introDbSubmitEnabled === true;
  const introDbApiKey = typeof prefs?.introDbApiKey === 'string' ? prefs.introDbApiKey : '';
  const [anime4kEnabled, setAnime4kEnabledState] = useState(false);
  const [anime4kMode, setAnime4kMode] = useState(() => String(prefs?.animeUpscalingModePreset ?? 'a'));
  useEffect(() => {
    let unlisten: (() => void) | undefined;
    void invoke<boolean>('player_get_anime4k_enabled').then(setAnime4kEnabledState).catch(() => undefined);
    void listen<{ enabled?: boolean }>('player-anime4k-state', (event) => {
      setAnime4kEnabledState(event.payload.enabled === true);
    }).then((stop) => {
      unlisten = stop;
    });
    return () => unlisten?.();
  }, []);
  const [showTorrentPopover, setShowTorrentPopover] = useState(false);
  const [castDevices, setCastDevices] = useState<CastDevice[]>([]);
  const [castDiscovering, setCastDiscovering] = useState(false);
  const [activeCastDeviceId, setActiveCastDeviceId] = useState<string | null>(null);
  const activeCastDeviceIdRef = useRef<string | null>(null);
  const [activeCastDeviceName, setActiveCastDeviceName] = useState('');
  const [castPaused, setCastPaused] = useState(false);
  const [abLoopStage, setAbLoopStage] = useState<'none' | 'a' | 'ab'>('none');
  const [playbackSpeed, setPlaybackSpeed] = useState(1.0);
  const [audioTracks, setAudioTracks] = useState<PlayerTrackOption[]>([]);
  const [subTracks, setSubTracks] = useState<PlayerTrackOption[]>([]);
  const [feedback, setFeedback] = useState<FeedbackFlash | null>(null);
  const [showVolumeSlider, setShowVolumeSlider] = useState(false);
  const volumeHideTimer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const [volumeScrolling, setVolumeScrolling] = useState(false);
  const volumeScrollTimer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const [showSeekOverlay, setShowSeekOverlay] = useState(false);
  const seekOverlayTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const [contextMenu, setContextMenu] = useState<{ x: number; y: number } | null>(null);
  const [showShortcutsHelp, setShowShortcutsHelp] = useState(false);
  const [shortcutOverrides, setShortcutOverrides] = useState<ShortcutOverrides>({});
  useEffect(() => {
    loadShortcutOverrides().then(setShortcutOverrides);
    return onShortcutsChanged(setShortcutOverrides);
  }, []);
  const cycleAbLoopRef = useRef<() => void>(() => {});
  const openCastPopoverRef = useRef<() => Promise<void>>(async () => {});
  const takeScreenshotRef = useRef<() => Promise<void>>(async () => {});
  const cycleAnime4kModeRef = useRef<(direction: 1 | -1) => void>(() => {});
  const [showStats, setShowStats] = useState(false);
  const [statsSnap, setStatsSnap] = useState<EmbeddedMpvStatus | null>(null);
  const [torrentStatsSnap, setTorrentStatsSnap] = useState<TorrentStats | null>(null);
  const [isBuffering, setIsBuffering] = useState(false);
  const [hdrLabel, setHdrLabel] = useState<string | null>(null);
  const [bufferingProgress, setBufferingProgress] = useState(0);
  const bufferHistoryRef = useRef<number[]>([]);
  const netSpeedHistoryRef = useRef<number[]>([]);
  const [torrentSpeedHistory, setTorrentSpeedHistory] = useState<number[]>([]);
  const liveStatusRef = useRef<EmbeddedMpvStatus | null>(null);
  const torrentStatsRef = useRef<TorrentStats | null>(null);
  const stallCountRef = useRef(0);
  const prevPausedForCacheRef = useRef(false);

  const seekFillRef = useRef<HTMLDivElement>(null);
  const seekBufferRef = useRef<HTMLDivElement>(null);
  const seekDotRef = useRef<HTMLDivElement>(null);
  const currentTimeRef = useRef<HTMLSpanElement>(null);
  const durationRef = useRef<HTMLSpanElement>(null);
  const seekbarRef = useRef<HTMLDivElement>(null);
  const softwareCanvasRef = useRef<HTMLCanvasElement>(null);
  const overlayRef = useRef<HTMLDivElement>(null);
  const subTrackBtnRef = useRef<HTMLButtonElement>(null);
  const audioTrackBtnRef = useRef<HTMLButtonElement>(null);
  const speedBtnRef = useRef<HTMLButtonElement>(null);
  const castBtnRef = useRef<HTMLButtonElement>(null);
  const playerSettingsBtnRef = useRef<HTMLButtonElement>(null);
  const torrentBtnRef = useRef<HTMLButtonElement>(null);
  const segFillRefs = useRef<(HTMLDivElement | null)[]>([]);
  const segBufRefs = useRef<(HTMLDivElement | null)[]>([]);
  const skipFillRef = useRef<HTMLDivElement>(null);
  const chapterSegmentsRef = useRef<Array<{ start: number; end: number }> | null>(null);
  const chaptersRef = useRef<Chapter[]>([]);

  const posRef = useRef(0);
  const durRef = useRef(0);
  const pausedRef = useRef(false);
  const lastActivityRef = useRef(Date.now());
  const isOverControlsRef = useRef(false);
  const miniProgressRef = useRef<HTMLDivElement>(null);
  const isDraggingRef = useRef(false);
  const dragPosRef = useRef(0);
  const lastSeekAtRef = useRef(0);
  const feedbackTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const holdTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const holdActiveRef = useRef(false);
  const preSpeedRef = useRef(1.0);
  const controlsVisibleRef = useRef(true);
  const episodePanelOpenRef = useRef(false);
  const firstFrameFiredRef = useRef(false);
  useEffect(() => {
    firstFrameFiredRef.current = false;
  }, [currentEpisode?.id]);
  const isFullscreenRef = useRef(false);
  const activeSkipKeyRef = useRef<string | null>(null);
  const discordPresenceKeyRef = useRef<string | null>(null);
  const discordPresenceSentAtRef = useRef(0);

  const resetActivity = useCallback(() => {
    lastActivityRef.current = Date.now();
    if (!controlsVisibleRef.current) {
      controlsVisibleRef.current = true;
      setControlsVisible(true);
      if (overlayRef.current) overlayRef.current.classList.remove('fluxa-cursor-hidden');
      getCurrentWindow().setCursorVisible(true).catch(() => {});
      embeddedMpvSetCursorVisible(true).catch(() => {});
    }
  }, []);

  useEffect(() => {
    const win = getCurrentWindow();
    win.isFullscreen().then((fs) => { isFullscreenRef.current = fs; }).catch(() => {});
    let unlisten: (() => void) | null = null;
    win.listen('tauri://resize', () => {
      win.isFullscreen().then((fs) => { isFullscreenRef.current = fs; }).catch(() => {});
      resetActivity();
    }).then((u) => { unlisten = u; }).catch(() => {});
    return () => { unlisten?.(); };
  }, [resetActivity]);

  useEffect(() => {
    if (!introDbSubmitEnabled || !metaId) { setIntroDbImdbId(null); return; }
    let cancelled = false;
    corePlaybackIntroLookupContentId(metaId).then((id) => { if (!cancelled) setIntroDbImdbId(id || null); }).catch(() => {});
    return () => { cancelled = true; };
  }, [introDbSubmitEnabled, metaId]);

  useEffect(() => {
    if (!softwareVideoActive) return;
    let cancelled = false;
    let timer: ReturnType<typeof setTimeout> | null = null;

    const draw = async () => {
      if (cancelled) return;
      const canvas = softwareCanvasRef.current;
      if (!canvas) {
        timer = setTimeout(draw, 100);
        return;
      }
      const dpr = Math.min(window.devicePixelRatio || 1, 1.5);
      const viewportW = Math.max(320, window.innerWidth);
      const viewportH = Math.max(180, window.innerHeight);
      const scale = Math.min(dpr, 960 / viewportW, 540 / viewportH);
      const targetW = Math.max(320, Math.floor(viewportW * scale));
      const targetH = Math.max(180, Math.floor(viewportH * scale));
      try {
        const frame = await embeddedMpvRenderFrame(targetW, targetH);
        if (cancelled) return;
        if (canvas.width !== frame.width || canvas.height !== frame.height) {
          canvas.width = frame.width;
          canvas.height = frame.height;
        }
        const ctx = canvas.getContext('2d');
        if (ctx) {
          const binary = atob(frame.pixelsBase64);
          const pixels = new Uint8ClampedArray(binary.length);
          for (let i = 0; i < binary.length; i++) pixels[i] = binary.charCodeAt(i);
          ctx.putImageData(new ImageData(pixels, frame.width, frame.height), 0, 0);
          const status = liveStatusRef.current;
          const position = parseFloat(status?.timePos ?? '0');
          const hasRenderedVideo =
            status?.loaded &&
            status.hasVideoTrack &&
            status.voConfigured === 'yes' &&
            status.framesRendered >= 2 &&
            status.pausedForCache !== 'yes' &&
            position > 0.15;
          if (!firstFrameFiredRef.current && onFirstFrame && hasRenderedVideo) {
            firstFrameFiredRef.current = true;
            onFirstFrame();
          }
        }
        timer = setTimeout(draw, 42);
      } catch {
        timer = setTimeout(draw, 120);
      }
    };

    void draw();
    return () => {
      cancelled = true;
      if (timer) clearTimeout(timer);
    };
  }, [softwareVideoActive, onFirstFrame]);

  const setPlayerFullscreen = useCallback(async (next: boolean) => {
    isFullscreenRef.current = next;
    const win = getCurrentWindow();
    await win.setFullscreen(next);
  }, []);

  const toggleFullscreen = useCallback(async () => {
    await setPlayerFullscreen(!isFullscreenRef.current);
  }, [setPlayerFullscreen]);

  const toggleMiniPlayer = useCallback(async () => {
    const win = getCurrentWindow();
    if (!miniPlayerActive) {
      if (isFullscreenRef.current) {
        await setPlayerFullscreen(false);
      }
      try {
        preMiniPlayerSizeRef.current = await win.outerSize();
        preMiniPlayerPosRef.current = await win.outerPosition();
      } catch {}
      setSuppressWindowGeometrySave(true);
      const width = 420;
      const height = 236;
      try {
        const monitor = await currentMonitor();
        if (monitor) {
          const margin = 24;
          await win.setPosition(new PhysicalPosition(
            monitor.position.x + monitor.size.width - width - margin,
            monitor.position.y + monitor.size.height - height - margin,
          ));
        }
      } catch {}
      await win.setSize(new PhysicalSize(width, height));
      await win.setAlwaysOnTop(true);
      setMiniPlayerActive(true);
    } else {
      await win.setAlwaysOnTop(false);
      try {
        if (preMiniPlayerSizeRef.current) await win.setSize(preMiniPlayerSizeRef.current);
        if (preMiniPlayerPosRef.current) await win.setPosition(preMiniPlayerPosRef.current);
      } catch {}
      setSuppressWindowGeometrySave(false);
      setMiniPlayerActive(false);
    }
  }, [miniPlayerActive, setPlayerFullscreen]);

  const flashFeedback = useCallback((icon: FeedbackFlash['icon'], label: string) => {
    setFeedback({ icon, label });
    if (feedbackTimerRef.current) clearTimeout(feedbackTimerRef.current);
    feedbackTimerRef.current = setTimeout(() => setFeedback(null), 700);
  }, []);

  const startSeekOverlay = useCallback(() => {
    setShowSeekOverlay(true);
    if (seekOverlayTimerRef.current) clearTimeout(seekOverlayTimerRef.current);
    seekOverlayTimerRef.current = setTimeout(() => setShowSeekOverlay(false), 30000);
  }, []);

  useEffect(() => {
    const mediaSession = navigator.mediaSession;
    if (!mediaSession) return;
    mediaSession.metadata = new MediaMetadata({
      title: episodeTitle || title || 'Fluxa',
      artist: episodeTitle ? title : undefined,
      artwork: initialPosterUrl ? [{ src: initialPosterUrl }] : undefined,
    });
    mediaSession.setActionHandler('play', () => {
      setPaused(false);
      sendCmd('set pause no');
    });
    mediaSession.setActionHandler('pause', () => {
      setPaused(true);
      sendCmd('set pause yes');
    });
    mediaSession.setActionHandler('seekbackward', () => {
      startSeekOverlay();
      flashFeedback('seekBack', '-10s');
      sendCmd('seek -10 relative');
    });
    mediaSession.setActionHandler('seekforward', () => {
      startSeekOverlay();
      flashFeedback('seekFwd', '+10s');
      sendCmd('seek 10 relative');
    });
    mediaSession.setActionHandler('nexttrack', () => {
      void emit('native-player-next-episode', null);
    });
    return () => {
      mediaSession.setActionHandler('play', null);
      mediaSession.setActionHandler('pause', null);
      mediaSession.setActionHandler('seekbackward', null);
      mediaSession.setActionHandler('seekforward', null);
      mediaSession.setActionHandler('nexttrack', null);
      mediaSession.metadata = null;
    };
  }, [title, episodeTitle, initialPosterUrl, startSeekOverlay, flashFeedback]);

  const seekToFraction = useCallback((fraction: number) => {
    const tt = fraction * durRef.current;
    lastSeekAtRef.current = Date.now();
    startSeekOverlay();
    if (activeCastDeviceIdRef.current) castSeek(tt);
    else sendCmd(`set time-pos ${Math.floor(tt)}`);
  }, [startSeekOverlay]);

  const fractionFromSeekbarEvent = useCallback((clientX: number): number => {
    const bar = seekbarRef.current;
    if (!bar) return 0;
    const rect = bar.getBoundingClientRect();
    return Math.max(0, Math.min(1, (clientX - rect.left) / rect.width));
  }, []);

  const applyFills = useCallback((frac: number, bufFrac?: number) => {
    const segs = chapterSegmentsRef.current;
    if (segs && segs.length > 0) {
      for (let i = 0; i < segs.length; i++) {
        const seg = segs[i];
        const segLen = seg.end - seg.start;
        const fillEl = segFillRefs.current[i];
        const bufEl = segBufRefs.current[i];
        if (fillEl) {
          if (frac >= seg.end) fillEl.style.width = '100%';
          else if (frac <= seg.start) fillEl.style.width = '0%';
          else fillEl.style.width = `${((frac - seg.start) / segLen * 100).toFixed(3)}%`;
        }
        if (bufEl && bufFrac !== undefined) {
          if (bufFrac >= seg.end) bufEl.style.width = '100%';
          else if (bufFrac <= seg.start) bufEl.style.width = '0%';
          else bufEl.style.width = `${((bufFrac - seg.start) / segLen * 100).toFixed(3)}%`;
        }
      }
    } else {
      if (seekFillRef.current) seekFillRef.current.style.width = `${(frac * 100).toFixed(3)}%`;
      if (seekBufferRef.current && bufFrac !== undefined) seekBufferRef.current.style.width = `${(bufFrac * 100).toFixed(3)}%`;
    }
    if (seekDotRef.current) seekDotRef.current.style.left = `${(frac * 100).toFixed(3)}%`;
  }, []);

  useEffect(() => {
    const handleStatus = (status: EmbeddedMpvStatus) => {
      liveStatusRef.current = status;

      const pausedForCache = status.pausedForCache === 'yes';
      const cacheProgress = parseFloat(status.cacheBufferingState ?? '');
      setIsBuffering(pausedForCache);
      if (pausedForCache) setBufferingProgress(Number.isFinite(cacheProgress) ? Math.max(0, Math.min(100, cacheProgress)) : 0);
      if (pausedForCache && !prevPausedForCacheRef.current) stallCountRef.current++;
      prevPausedForCacheRef.current = pausedForCache;

      const gamma = (status.colorGamma ?? '').toLowerCase();
      const sigPeak = parseFloat(status.sigPeak ?? '');
      const contentIsHdr = gamma.includes('hlg') || gamma.includes('pq') || gamma.includes('2084') || (Number.isFinite(sigPeak) && sigPeak > 1);
      setHdrLabel(status.hdrActive && contentIsHdr ? (gamma.includes('hlg') ? 'HLG' : 'HDR10') : null);

      const pos = parseFloat(status.timePos ?? '0');
      const dur = parseFloat(status.duration ?? '0');
      const isPaused = status.pause === 'yes';
      const isMuted = (status as Record<string, unknown>).mute === 'yes';
      const vol = parseFloat((status as Record<string, unknown>).volume as string ?? '100');
      const buffered = parseFloat(status.demuxerCacheDuration ?? '0');

      posRef.current = pos;
      durRef.current = dur;
      pausedRef.current = isPaused;

      const bh = bufferHistoryRef.current;
      bufferHistoryRef.current = addSparklineSample(bh, buffered);
      const nh = netSpeedHistoryRef.current;
      netSpeedHistoryRef.current = addSparklineSample(nh, parseInt(status.cacheSpeed ?? '0') || 0);

      const presenceKey = `${title}|${episodeTitle}|${isPaused}`;
      const presenceDue = Date.now() - discordPresenceSentAtRef.current > 25000;
      if (title && (presenceKey !== discordPresenceKeyRef.current || presenceDue)) {
        discordPresenceKeyRef.current = presenceKey;
        discordPresenceSentAtRef.current = Date.now();
        updateDiscordPresence({
          title,
          detail: episodeTitle || undefined,
          paused: isPaused,
          startUnixSecs: isPaused ? undefined : Math.floor(Date.now() / 1000 - pos),
          endUnixSecs: !isPaused && dur > 0 ? Math.floor(Date.now() / 1000 + (dur - pos)) : undefined,
          posterUrl: initialPosterUrl,
          ...imdbButtonFor(metaId),
        });
      }

      if (!firstFrameFiredRef.current && onFirstFrame) {
        const hasVideoDimensions =
          (parseFloat(status.width ?? '0') || 0) > 0 &&
          (parseFloat(status.height ?? '0') || 0) > 0;
        const renderedVideo =
          status.loaded &&
          status.hasVideoTrack &&
          (
            status.firstFramePresented ||
            (
              hasVideoDimensions &&
              status.voConfigured === 'yes' &&
              status.framesRendered >= 2 &&
              status.pausedForCache !== 'yes' &&
              pos > 0.15
            )
          );
        const activeAudioOnlyPlayback =
          status.loaded &&
          status.trackListReady &&
          !status.hasVideoTrack &&
          status.pausedForCache !== 'yes' &&
          pos > 0.05;
        if (renderedVideo || activeAudioOnlyPlayback) {
          firstFrameFiredRef.current = true;
          onFirstFrame();
        }
      }

      if (currentTimeRef.current) currentTimeRef.current.textContent = fmtTime(pos);
      if (durationRef.current) durationRef.current.textContent = fmtTime(dur);

      const fraction = dur > 0 ? pos / dur : 0;
      const torrentProgress = isTorrentStream ? torrentStatsRef.current?.progress : undefined;
      const bufFraction = torrentProgress != null
        ? Math.max(0, Math.min(1, torrentProgress / 100))
        : (dur > 0 ? Math.min(1, (pos + buffered) / dur) : 0);

      const seekSuppressed = Date.now() - lastSeekAtRef.current < 800 || status.seeking === 'yes';
      if (!isDraggingRef.current && !seekSuppressed) {
        applyFills(fraction, bufFraction);
      }

      if (status.seeking !== 'yes') {
        setShowSeekOverlay((prev) => {
          if (!prev) return prev;
          if (seekOverlayTimerRef.current) { clearTimeout(seekOverlayTimerRef.current); seekOverlayTimerRef.current = null; }
          return false;
        });
      }

      setPaused((prev) => (prev !== isPaused ? isPaused : prev));
      setMuted((prev) => (prev !== isMuted ? isMuted : prev));
      setVolumeLevel((prev) => {
        const rounded = Math.round(vol);
        return prev !== rounded ? rounded : prev;
      });

      const idle = Date.now() - lastActivityRef.current;
      if (idle > 3000 && !isPaused && !episodePanelOpenRef.current && trackPopover === null && !isOverControlsRef.current) {
        if (controlsVisibleRef.current) {
          controlsVisibleRef.current = false;
          setControlsVisible(false);
          if (overlayRef.current) overlayRef.current.classList.add('fluxa-cursor-hidden');
          getCurrentWindow().setCursorVisible(false).catch(() => {});
          embeddedMpvSetCursorVisible(false).catch(() => {});
        }
      }

      if (miniProgressRef.current) miniProgressRef.current.style.width = `${(fraction * 100).toFixed(3)}%`;

      const posMs = pos * 1000;
      const seg = skipSegments.find((s) => posMs >= s.startTime && posMs < s.endTime);
      const newSkipKey = seg ? `${seg.type}:${seg.endTime}` : null;
      if (newSkipKey !== activeSkipKeyRef.current) {
        activeSkipKeyRef.current = newSkipKey;
        if (seg && newSkipKey && autoSkipSegments && !autoSkippedKeysRef.current.has(newSkipKey)) {
          autoSkippedKeysRef.current.add(newSkipKey);
          lastSeekAtRef.current = Date.now();
          sendCmd(`set time-pos ${Math.floor(seg.endTime / 1000)}`);
          flashFeedback('seekFwd', t('player.skipped'));
          setActiveSkip(null);
        } else {
          const outroEndsPlayback = seg?.type === 'outro' && dur > 0 && seg.endTime >= dur * 1000 - 500;
          setActiveSkip(seg && !outroEndsPlayback ? { label: skipLabelForType(seg.type), startMs: seg.startTime, endMs: seg.endTime, type: seg.type } : null);
        }
      }
      if (seg && skipFillRef.current) {
        const span = seg.endTime - seg.startTime;
        const skipFrac = span > 0 ? Math.min(1, Math.max(0, (posMs - seg.startTime) / span)) : 0;
        skipFillRef.current.style.width = `${(skipFrac * 100).toFixed(2)}%`;
      }

      const outroEndsPlayback = seg?.type === 'outro' && dur > 0 && seg.endTime >= dur * 1000 - 500;
      setShowNextEpCard((prev) => {
        const next = dur > 0 && !!nextEpSubtitle && !nextEpDismissed && (outroEndsPlayback || (pos / dur) * 100 >= nextEpThreshold);
        return prev === next ? prev : next;
      });
    };

    return subscribePlayerStatus(handleStatus);
  }, [skipSegments, nextEpSubtitle, nextEpThreshold, nextEpDismissed, trackPopover, onFirstFrame, applyFills, title, episodeTitle, initialPosterUrl, metaId, autoSkipSegments, flashFeedback, isTorrentStream, playbackUrl]);

  useEffect(() => {
    const tick = async () => {
      if (showStats && liveStatusRef.current) setStatsSnap({ ...liveStatusRef.current });
      const raw = await playerTorrentStats().catch(() => null);
      const ts = raw && typeof raw.stat === 'number' ? raw : null;
      torrentStatsRef.current = ts;
      setTorrentStatsSnap(ts);
      if (liveStatusRef.current?.pausedForCache === 'yes' && ts) setBufferingProgress(Math.max(0, Math.min(100, ts.preload)));
      if (ts) setTorrentSpeedHistory((history) => addSparklineSample(history, ts.download_speed));
    };
    void tick();
    const id = setInterval(() => { void tick(); }, (showStats || showTorrentPopover) ? 500 : 1500);
    return () => clearInterval(id);
  }, [showStats, showTorrentPopover]);

  useEffect(() => {
    miniPlayerActiveRef.current = miniPlayerActive;
  }, [miniPlayerActive]);

  useEffect(() => {
    return () => {
      if (!miniPlayerActiveRef.current) return;
      const win = getCurrentWindow();
      setSuppressWindowGeometrySave(false);
      void win.setAlwaysOnTop(false).catch(() => undefined);
      if (preMiniPlayerSizeRef.current) void win.setSize(preMiniPlayerSizeRef.current).catch(() => undefined);
      if (preMiniPlayerPosRef.current) void win.setPosition(preMiniPlayerPosRef.current).catch(() => undefined);
    };
  }, []);

  useEffect(() => {
    activeCastDeviceIdRef.current = activeCastDeviceId;
    return () => { if (activeCastDeviceId) castDisconnect(); };
  }, [activeCastDeviceId]);

  useEffect(() => {
    const poll = async () => {
      try {
        const info = await playerGetPlaybackInfo();
        const parsedChapters = parseChapters(info.chaptersJson);
        chaptersRef.current = parsedChapters;
        setChapters(parsedChapters);
        setSkipSegments(parseSegments(info.skipSegmentsJson));
        setNextEpSubtitle((prev) => {
          const next = info.nextEpSubtitle ?? '';
          if (next !== prev) setNextEpDismissed(false);
          return next;
        });
        setNextEpThreshold(info.nextEpThresholdPercent ?? 85);
        setAutoPlayNextEpisode(info.autoPlayNextEpisode ?? false);
        setAutoPlayCountdownSecs(info.autoPlayCountdownSecs ?? 7);
        setAutoSkipSegments(info.autoSkipSegments ?? false);
        setEpisodes(parseEpisodes(info.episodesJson));
      } catch {}
    };
    poll();
    const interval = setInterval(poll, 2000);
    let unlisten: (() => void) | undefined;
    void listen('player-skip-info-updated', () => { void poll(); }).then((stop) => { unlisten = stop; });
    return () => { clearInterval(interval); unlisten?.(); };
  }, []);

  useEffect(() => {
    if (!showNextEpCard) {
      setCountdown(null);
      return;
    }
    if (!autoPlayNextEpisode || nextEpDismissed) {
      setCountdown(null);
      return;
    }
    setCountdown(autoPlayCountdownSecs);
  }, [showNextEpCard, autoPlayNextEpisode, nextEpDismissed, autoPlayCountdownSecs]);

  useEffect(() => {
    if (showNextEpCard && activeSkip?.type === 'outro') setActiveSkip(null);
  }, [showNextEpCard, activeSkip]);

  useEffect(() => {
    if (countdown === null) return;
    const id = setInterval(() => {
      if (pausedRef.current) return;
      setCountdown((c) => (c !== null && c > 0 ? c - 1 : c));
    }, 1000);
    return () => clearInterval(id);
  }, [countdown === null]);

  useEffect(() => {
    if (countdown === 0) {
      resetActivity();
      void emit('native-player-next-episode', null);
    }
  }, [countdown]);

  useEffect(() => {
    let cancelled = false;
    listen<{ title?: string; episodeTitle?: string }>('native-player-title', (ev) => {
      if (cancelled) return;
      setTitle(ev.payload.title ?? '');
      setEpisodeTitle(ev.payload.episodeTitle ?? '');
      setAbLoopStage('none');
      setNextEpDismissed(false);
      autoSkippedKeysRef.current.clear();
      stallCountRef.current = 0;
      prevPausedForCacheRef.current = false;
      bufferHistoryRef.current = [];
      netSpeedHistoryRef.current = [];
      setTorrentSpeedHistory([]);
    }).catch(() => undefined);
    return () => { cancelled = true; };
  }, []);

  const triggerActiveSkip = useCallback(() => {
    if (!activeSkip) return false;
    sendCmd(`set time-pos ${Math.floor(activeSkip.endMs / 1000)}`);
    flashFeedback('seekFwd', activeSkip.label);
    return true;
  }, [activeSkip, flashFeedback]);

  useEffect(() => {
    const onKeyDown = (e: KeyboardEvent) => {
      if (e.code === 'Space') {
        e.preventDefault();
        if (holdTimerRef.current) return;
        holdActiveRef.current = false;
        preSpeedRef.current = playbackSpeed;
        holdTimerRef.current = setTimeout(() => {
          holdActiveRef.current = true;
          sendCmd('set speed 2.00');
          flashFeedback('speed', '2×');
        }, 300);
        return;
      }
      if (e.shiftKey && /^Digit[1-9]$/.test(e.code)) {
        const index = parseInt(e.code.replace('Digit', ''), 10) - 1;
        if (index < chaptersRef.current.length) {
          e.preventDefault();
          sendCmd(`set chapter ${index}`);
          flashFeedback('seekFwd', chaptersRef.current[index].title || `${t('player.chapter')} ${index + 1}`);
        }
        return;
      }
      if (/^Digit[1-9]$/.test(e.code)) {
        e.preventDefault();
        const pct = parseInt(e.code.replace('Digit', ''), 10) * 10;
        startSeekOverlay();
        flashFeedback(pct < (posRef.current / Math.max(1, durRef.current) * 100) ? 'seekBack' : 'seekFwd', `${pct}%`);
        sendCmd(`seek ${pct} absolute-percent`);
        return;
      }
      if (e.code === 'Digit0') {
        e.preventDefault();
        startSeekOverlay();
        flashFeedback('seekBack', '0%');
        sendCmd('seek 0 absolute');
        return;
      }
      if (e.code === 'Enter') {
        if (triggerActiveSkip()) e.preventDefault();
        return;
      }
      if (e.code === 'F11') {
        e.preventDefault();
        void toggleFullscreen();
        return;
      }
      if (e.code === 'Escape') {
        e.preventDefault();
        if (showShortcutsHelp) { setShowShortcutsHelp(false); return; }
        if (contextMenu) { setContextMenu(null); return; }
        if (showEpisodePanel) { setShowEpisodePanel(false); episodePanelOpenRef.current = false; return; }
        if (trackPopover) { setTrackPopover(null); return; }
        if (isFullscreenRef.current) { void setPlayerFullscreen(false); }
        return;
      }
      if (e.code === 'Backspace') {
        if (contextMenu || showEpisodePanel || trackPopover || showShortcutsHelp) return;
        e.preventDefault();
        void closePlayer();
        return;
      }

      const action = findActionForCombo(comboFromEvent(e), 'player', shortcutOverrides);
      switch (action) {
        case 'player_seek_back':
          e.preventDefault();
          startSeekOverlay();
          flashFeedback('seekBack', '-10s');
          sendCmd('seek -10 relative');
          break;
        case 'player_seek_forward':
          e.preventDefault();
          startSeekOverlay();
          flashFeedback('seekFwd', '+10s');
          sendCmd('seek 10 relative');
          break;
        case 'player_volume_up':
          e.preventDefault();
          flashFeedback('volume', '');
          sendCmd('add volume 5');
          break;
        case 'player_volume_down':
          e.preventDefault();
          flashFeedback('volume', '');
          sendCmd('add volume -5');
          break;
        case 'player_seek_big_back':
          e.preventDefault();
          startSeekOverlay();
          flashFeedback('seekBack', t('player.seek_big_back'));
          sendCmd('seek -60 relative');
          break;
        case 'player_seek_big_forward':
          e.preventDefault();
          startSeekOverlay();
          flashFeedback('seekFwd', t('player.seek_big_forward'));
          sendCmd('seek 60 relative');
          break;
        case 'player_play_pause':
          e.preventDefault();
          {
            const icon = pausedRef.current ? 'play' : 'pause';
            flashFeedback(icon, '');
            setPaused((prev) => !prev);
            sendCmd('cycle pause');
          }
          break;
        case 'player_speed_decrease': {
          e.preventDefault();
          const next = Math.max(0.25, parseFloat((playbackSpeed - 0.25).toFixed(2)));
          sendCmd(`set speed ${next}`);
          setPlaybackSpeed(next);
          flashFeedback('speed', t('player.speed_decrease'));
          break;
        }
        case 'player_speed_increase': {
          e.preventDefault();
          const next = Math.min(4, parseFloat((playbackSpeed + 0.25).toFixed(2)));
          sendCmd(`set speed ${next}`);
          setPlaybackSpeed(next);
          flashFeedback('speed', t('player.speed_increase'));
          break;
        }
        case 'player_cycle_subtitle':
          e.preventDefault();
          sendCmd('cycle sub');
          break;
        case 'player_cycle_audio':
          e.preventDefault();
          sendCmd('cycle audio');
          break;
        case 'player_toggle_stats':
          e.preventDefault();
          setShowStats((s) => !s);
          break;
        case 'player_frame_step_forward':
          e.preventDefault();
          sendCmd('frame-step');
          flashFeedback('seekFwd', t('player.frame_step'));
          break;
        case 'player_frame_step_back':
          e.preventDefault();
          sendCmd('frame-back-step');
          flashFeedback('seekBack', t('player.frame_back_step'));
          break;
        case 'player_skip_active':
          if (triggerActiveSkip()) e.preventDefault();
          break;
        case 'player_next_episode':
          if (nextEpSubtitle) {
            e.preventDefault();
            void emit('native-player-next-episode', null);
          }
          break;
        case 'player_mute':
          e.preventDefault();
          sendCmd('cycle mute');
          break;
        case 'player_sub_delay_earlier':
          e.preventDefault();
          flashFeedback('subDelay', t('player.subtitle_delay_earlier'));
          sendCmd('add sub-delay -0.100');
          break;
        case 'player_sub_delay_later':
          e.preventDefault();
          flashFeedback('subDelay', t('player.subtitle_delay_later'));
          sendCmd('add sub-delay 0.100');
          break;
        case 'player_fullscreen':
          e.preventDefault();
          void toggleFullscreen();
          break;
        case 'player_toggle_shortcuts_help':
          e.preventDefault();
          setShowShortcutsHelp((s) => !s);
          break;
        case 'player_toggle_pip':
          e.preventDefault();
          void toggleMiniPlayer();
          break;
        case 'player_open_cast':
          e.preventDefault();
          void openCastPopoverRef.current();
          break;
        case 'player_ab_loop':
          e.preventDefault();
          cycleAbLoopRef.current();
          break;
        case 'player_screenshot':
          e.preventDefault();
          void takeScreenshotRef.current();
          break;
        case 'player_seek_start':
          e.preventDefault();
          startSeekOverlay();
          flashFeedback('seekBack', '0%');
          sendCmd('seek 0 absolute');
          break;
        case 'player_seek_end':
          e.preventDefault();
          startSeekOverlay();
          flashFeedback('seekFwd', '100%');
          sendCmd('seek 100 absolute-percent');
          break;
        case 'player_anime4k_mode_next':
          e.preventDefault();
          cycleAnime4kModeRef.current(1);
          break;
        case 'player_anime4k_mode_prev':
          e.preventDefault();
          cycleAnime4kModeRef.current(-1);
          break;
        default:
          break;
      }
    };

    const onKeyUp = (e: KeyboardEvent) => {
      if (e.code === 'Space') {
        if (holdTimerRef.current) { clearTimeout(holdTimerRef.current); holdTimerRef.current = null; }
        if (holdActiveRef.current) {
          holdActiveRef.current = false;
          sendCmd(`set speed ${preSpeedRef.current.toFixed(2)}`);
        } else {
          const icon = pausedRef.current ? 'play' : 'pause';
          flashFeedback(icon, '');
          setPaused((prev) => !prev);
          sendCmd('cycle pause');
        }
      }
    };

    window.addEventListener('keydown', onKeyDown);
    window.addEventListener('keyup', onKeyUp);
    return () => {
      window.removeEventListener('keydown', onKeyDown);
      window.removeEventListener('keyup', onKeyUp);
    };
  }, [closePlayer, contextMenu, flashFeedback, nextEpSubtitle, playbackSpeed, setPlayerFullscreen, shortcutOverrides, showEpisodePanel, showShortcutsHelp, startSeekOverlay, toggleFullscreen, toggleMiniPlayer, trackPopover, triggerActiveSkip]);

  useEffect(() => {
    return () => {
      getCurrentWindow().setCursorVisible(true).catch(() => {});
      embeddedMpvSetCursorVisible(true).catch(() => {});
    };
  }, []);


  useEffect(() => {
    const onMove = () => resetActivity();
    window.addEventListener('mousemove', onMove);
    return () => window.removeEventListener('mousemove', onMove);
  }, [resetActivity]);

  const onOverlayWheel = useCallback((e: React.WheelEvent) => {
    if (Math.abs(e.deltaY) < 2) return;
    resetActivity();
    if (e.shiftKey) {
      startSeekOverlay();
      const seconds = e.deltaY < 0 ? 5 : -5;
      flashFeedback(seconds > 0 ? 'seekFwd' : 'seekBack', `${seconds > 0 ? '+' : ''}${seconds}s`);
      sendCmd(`seek ${seconds} relative`);
      return;
    }
    flashFeedback('volume', '');
    sendCmd(`add volume ${e.deltaY < 0 ? 5 : -5}`);
  }, [flashFeedback, resetActivity, startSeekOverlay]);

  const onSeekMouseDown = useCallback((e: React.MouseEvent) => {
    if (e.button !== 0) return;
    e.preventDefault();
    resetActivity();
    isDraggingRef.current = true;
    const frac = fractionFromSeekbarEvent(e.clientX);
    dragPosRef.current = frac;
    applyFills(frac);
  }, [fractionFromSeekbarEvent, resetActivity, applyFills]);

  useEffect(() => {
    const onMouseUp = () => {
      if (!isDraggingRef.current) return;
      isDraggingRef.current = false;
      seekToFraction(dragPosRef.current);
    };
    const onMouseMove = (e: MouseEvent) => {
      if (!isDraggingRef.current) return;
      const bar = seekbarRef.current;
      if (!bar) return;
      const rect = bar.getBoundingClientRect();
      const frac = Math.max(0, Math.min(1, (e.clientX - rect.left) / rect.width));
      dragPosRef.current = frac;
      applyFills(frac);
    };
    window.addEventListener('mouseup', onMouseUp);
    window.addEventListener('mousemove', onMouseMove);
    return () => {
      window.removeEventListener('mouseup', onMouseUp);
      window.removeEventListener('mousemove', onMouseMove);
    };
  }, [seekToFraction, applyFills]);

  const openTrackPopover = useCallback(async (type: 'audio' | 'sub' | 'speed') => {
    resetActivity();
    if (trackPopover === type) { setTrackPopover(null); return; }
    if (type === 'audio') { try { setAudioTracks(await playerGetTrackOptions('audio')); } catch {} }
    else if (type === 'sub') { try { setSubTracks(await playerGetTrackOptions('sub')); } catch {} }
    setTrackPopover(type);
  }, [trackPopover, resetActivity]);

  const setSpeed = useCallback((speed: number) => {
    sendCmd(`set speed ${speed.toFixed(2)}`);
    setPlaybackSpeed(speed);
    setTrackPopover(null);
  }, []);

  const selectTrack = useCallback((type: 'audio' | 'sub', id: string) => {
    sendCmd(type === 'audio' ? `set aid ${id}` : `set sid ${id}`);
    if (type === 'audio') sendCmd('seek 0.1 relative exact');
    setTrackPopover(null);
    if (type === 'audio') setAudioTracks((prev) => prev.map((tr) => ({ ...tr, selected: tr.id === id })));
    else setSubTracks((prev) => prev.map((tr) => ({ ...tr, selected: tr.id === id })));
  }, []);

  const disableSubs = useCallback(() => {
    sendCmd('set sid no');
    setSubTracks((prev) => prev.map((tr) => ({ ...tr, selected: false })));
    setTrackPopover(null);
  }, []);

  const setSubtitlePref = useCallback(<K extends string>(key: K, value: string | boolean) => {
    void onDispatch?.(JSON.stringify({ type: 'settingsChanged', key, value }));
  }, [onDispatch]);

  const toggleAnime4k = useCallback((enabled: boolean) => {
    setAnime4kEnabledState(enabled);
    invoke('player_set_anime4k_enabled', {
      enabled,
      quality: String(prefs?.animeUpscalingQuality ?? 'anime4k_m'),
      mode: anime4kMode,
    }).catch(() => setAnime4kEnabledState(!enabled));
  }, [prefs, anime4kMode]);

  const cycleAnime4kMode = useCallback((direction: 1 | -1) => {
    if (!anime4kEnabled) return;
    const index = ANIME4K_MODES.indexOf(anime4kMode as typeof ANIME4K_MODES[number]);
    const nextIndex = (index === -1 ? 0 : index + direction + ANIME4K_MODES.length) % ANIME4K_MODES.length;
    const next = ANIME4K_MODES[nextIndex];
    void invoke('player_set_anime4k_enabled', {
      enabled: true,
      quality: String(prefs?.animeUpscalingQuality ?? 'anime4k_m'),
      mode: next,
    }).then(() => {
      setAnime4kMode(next);
      void setSubtitlePref('animeUpscalingModePreset', next);
      flashFeedback('anime4k', t(`player.anime4k_mode_${next}`));
    }).catch(() => undefined);
  }, [anime4kEnabled, anime4kMode, prefs, setSubtitlePref, flashFeedback]);

  const [subtitleDelay, setSubtitleDelayState] = useState(0);
  const [subtitlePosition, setSubtitlePositionState] = useState(() => Number(prefs?.subtitlePosition ?? 100) || 100);
  const [autoSyncingSubtitles, setAutoSyncingSubtitles] = useState(false);
  const [subtitleCaptureCues, setSubtitleCaptureCues] = useState<SubtitleCaptureCue[]>([]);
  const [subtitleCaptureTime, setSubtitleCaptureTime] = useState<number | null>(null);
  const [subtitleFont, setSubtitleFontState] = useState(() => String(prefs?.subtitleFont ?? 'default'));
  const subtitleSizeRef = useRef(Number(prefs?.subtitleSize ?? 100) || 100);
  const [subtitleSize, setSubtitleSizeState] = useState(() => subtitleSizeRef.current);
  const [subtitleColor, setSubtitleColorState] = useState(() => String(prefs?.subtitleColor ?? '#FFFFFF'));
  const [subtitleTextOpacity, setSubtitleTextOpacity] = useState(() => String(prefs?.subtitleTextOpacity ?? '1.0'));
  const [subtitleBackgroundColor, setSubtitleBackgroundColor] = useState(() => String(prefs?.subtitleBackgroundColor ?? '#000000'));
  const [subtitleBackgroundOpacity, setSubtitleBackgroundOpacity] = useState(() => String(prefs?.subtitleBackgroundOpacity ?? '0.5'));
  const [subtitleOutlineColor, setSubtitleOutlineColor] = useState(() => String(prefs?.subtitleOutlineColor ?? '#000000'));
  const [subtitleOutlineOpacity, setSubtitleOutlineOpacity] = useState(() => String(prefs?.subtitleOutlineOpacity ?? '1.0'));
  const [subtitleForceStyle, setSubtitleForceStyle] = useState(() => prefs?.subtitleForceStyle === true);
  const [subtitleCharacterEdge, setSubtitleCharacterEdge] = useState(() => String(prefs?.subtitleCharacterEdge ?? 'uniform'));
  const [subtitleShadow, setSubtitleShadow] = useState(() => prefs?.subtitleShadow === true);

  const mpvColor = useCallback((color: string, opacity: string) => {
    const alpha = Math.round(Math.max(0, Math.min(1, Number(opacity) || 0)) * 255).toString(16).padStart(2, '0').toUpperCase();
    return `#${alpha}${color.replace(/^#/, '')}`;
  }, []);

  const adjustSubtitleDelay = useCallback((delta: number) => {
    setSubtitleDelayState((prev) => {
      const next = Math.round((prev + delta) * 10) / 10;
      sendCmd(`set sub-delay ${next.toFixed(3)}`);
      return next;
    });
  }, []);

  const resetSubtitleDelay = useCallback(() => {
    sendCmd('set sub-delay 0.000');
    setSubtitleDelayState(0);
  }, []);

  const chooseSubtitlePosition = useCallback((position: number) => {
    sendCmd(`set sub-pos ${position}`);
    setSubtitlePositionState(position);
    setSubtitlePref('subtitlePosition', String(position));
  }, [setSubtitlePref]);

  const autoSyncSubtitles = useCallback(async () => {
    if (autoSyncingSubtitles) return;
    setAutoSyncingSubtitles(true);
    sendCmd('set pause yes');
    try {
      const result = await invoke<{ capturedTime: number; cues: SubtitleCaptureCue[] }>('player_capture_subtitle_cues');
      setSubtitleCaptureTime(result.capturedTime);
      setSubtitleCaptureCues(result.cues);
    } catch {
      flashFeedback('subDelay', t('player.subtitle_auto_sync_failed'));
    } finally {
      setAutoSyncingSubtitles(false);
    }
  }, [autoSyncingSubtitles, flashFeedback]);

  const applySubtitleCapture = useCallback((cueStart: number) => {
    if (subtitleCaptureTime === null) return;
    const delay = Math.round((subtitleCaptureTime - cueStart) * 10) / 10;
    sendCmd(`set sub-delay ${delay.toFixed(3)}`);
    setSubtitleDelayState(delay);
    setSubtitleCaptureCues([]);
    flashFeedback('subDelay', `${delay > 0 ? '+' : ''}${delay.toFixed(1)}s`);
  }, [flashFeedback, subtitleCaptureTime]);

  const chooseSubtitleFont = useCallback((font: string) => {
    sendCmd(`set sub-font "${font === 'default' ? 'sans-serif' : font}"`);
    setSubtitleFontState(font);
    setSubtitlePref('subtitleFont', font);
  }, [setSubtitlePref]);

  const chooseSubtitleSize = useCallback((size: number) => {
    sendCmd(`set sub-scale ${(size / 100).toFixed(2)}`);
    subtitleSizeRef.current = size;
    setSubtitleSizeState(size);
    setSubtitlePref('subtitleSize', String(size));
  }, [setSubtitlePref]);

  const adjustSubtitleSize = useCallback((delta: number) => {
    const size = Math.max(50, Math.min(200, subtitleSizeRef.current + delta));
    subtitleSizeRef.current = size;
    sendCmd(`set sub-scale ${(size / 100).toFixed(2)}`);
    setSubtitleSizeState(size);
    setSubtitlePref('subtitleSize', String(size));
  }, [setSubtitlePref]);

  const chooseSubtitleColor = useCallback((color: string) => {
    sendCmd(`set sub-color "${mpvColor(color, subtitleTextOpacity)}"`);
    setSubtitleColorState(color);
    setSubtitlePref('subtitleColor', color);
  }, [mpvColor, setSubtitlePref, subtitleTextOpacity]);

  const chooseSubtitleStyle = useCallback((key: 'subtitleTextOpacity' | 'subtitleBackgroundColor' | 'subtitleBackgroundOpacity' | 'subtitleOutlineColor' | 'subtitleOutlineOpacity' | 'subtitleForceStyle' | 'subtitleCharacterEdge' | 'subtitleShadow', value: string | boolean) => {
    if (key === 'subtitleTextOpacity') {
      const opacity = String(value);
      setSubtitleTextOpacity(opacity);
      sendCmd(`set sub-color "${mpvColor(subtitleColor, opacity)}"`);
    } else if (key === 'subtitleBackgroundColor') {
      const color = String(value);
      setSubtitleBackgroundColor(color);
      sendCmd(`set sub-back-color "${mpvColor(color, subtitleBackgroundOpacity)}"`);
    } else if (key === 'subtitleBackgroundOpacity') {
      const opacity = String(value);
      setSubtitleBackgroundOpacity(opacity);
      sendCmd(`set sub-back-color "${mpvColor(subtitleBackgroundColor, opacity)}"`);
    } else if (key === 'subtitleOutlineColor') {
      const color = String(value);
      setSubtitleOutlineColor(color);
      sendCmd(`set sub-border-color "${mpvColor(color, subtitleOutlineOpacity)}"`);
    } else if (key === 'subtitleOutlineOpacity') {
      const opacity = String(value);
      setSubtitleOutlineOpacity(opacity);
      sendCmd(`set sub-border-color "${mpvColor(subtitleOutlineColor, opacity)}"`);
    } else if (key === 'subtitleForceStyle') {
      const enabled = value === true;
      setSubtitleForceStyle(enabled);
      sendCmd(`set sub-ass-override ${enabled ? 'force' : 'no'}`);
    } else if (key === 'subtitleCharacterEdge') {
      const edge = String(value);
      setSubtitleCharacterEdge(edge);
      if (edge === 'none') {
        sendCmd('set sub-border-size 0');
        sendCmd('set sub-shadow-offset 0');
      } else if (edge === 'raised') {
        sendCmd('set sub-border-size 1.5');
        sendCmd('set sub-shadow-offset -1');
      } else if (edge === 'depressed') {
        sendCmd('set sub-border-size 1.5');
        sendCmd('set sub-shadow-offset 1');
      } else if (edge === 'drop-shadow') {
        sendCmd('set sub-border-size 0');
        sendCmd('set sub-shadow-offset 3');
        sendCmd('set sub-shadow-color "#80000000"');
      } else {
        sendCmd('set sub-border-size 3');
        sendCmd('set sub-shadow-offset 0');
      }
    } else {
      const enabled = value === true;
      setSubtitleShadow(enabled);
      sendCmd(`set sub-shadow-offset ${enabled ? '3' : '0'}`);
      if (enabled) sendCmd('set sub-shadow-color "#80000000"');
    }
    setSubtitlePref(key, value);
  }, [mpvColor, setSubtitlePref, subtitleBackgroundColor, subtitleBackgroundOpacity, subtitleColor, subtitleOutlineColor, subtitleOutlineOpacity]);

  const openCastPopover = useCallback(async () => {
    resetActivity();
    if (castPopoverOpen) { setCastPopoverOpen(false); return; }
    setCastPopoverOpen(true);
    setCastDiscovering(true);
    setCastDevices(await discoverCastDevices());
    setCastDiscovering(false);
  }, [castPopoverOpen, resetActivity]);
  useEffect(() => { openCastPopoverRef.current = openCastPopover; }, [openCastPopover]);

  const selectCastDevice = useCallback(async (device: CastDevice) => {
    let status: EmbeddedMpvStatus | null = null;
    try { status = await invoke<EmbeddedMpvStatus>('player_status'); } catch {}
    const streamUrl = status?.path;
    if (!streamUrl) { setCastPopoverOpen(false); return; }
    const mediaUrl = initialStreamHeaders && Object.keys(initialStreamHeaders).length > 0
      ? await proxyMediaUrl(streamUrl, initialStreamHeaders)
      : await resolveCastMediaUrl(streamUrl);
    try {
      await startCasting(device, mediaUrl, title || episodeTitle || 'Fluxa', initialSubtitleUrl);
      setActiveCastDeviceId(device.id);
      setActiveCastDeviceName(device.name);
      setCastPaused(false);
      sendCmd('set pause yes');
    } catch {}
    setCastPopoverOpen(false);
  }, [title, episodeTitle, initialSubtitleUrl, initialStreamHeaders]);

  const disconnectCast = useCallback(() => {
    castDisconnect();
    setActiveCastDeviceId(null);
    setActiveCastDeviceName('');
    setCastPopoverOpen(false);
  }, []);

  const toggleCastPause = useCallback(() => {
    if (castPaused) { castPlay(); setCastPaused(false); }
    else { castPause(); setCastPaused(true); }
  }, [castPaused]);

  const cycleAbLoop = useCallback(() => {
    resetActivity();
    if (abLoopStage === 'none') {
      sendCmd(`set ab-loop-a ${posRef.current.toFixed(3)}`);
      setAbLoopStage('a');
      flashFeedback('abLoop', t('player.ab_loop_a_set'));
    } else if (abLoopStage === 'a') {
      sendCmd(`set ab-loop-b ${posRef.current.toFixed(3)}`);
      setAbLoopStage('ab');
      flashFeedback('abLoop', t('player.ab_loop_active'));
    } else {
      sendCmd('set ab-loop-a no');
      sendCmd('set ab-loop-b no');
      setAbLoopStage('none');
      flashFeedback('abLoop', t('player.ab_loop_cleared'));
    }
  }, [abLoopStage, resetActivity, flashFeedback]);
  useEffect(() => { cycleAbLoopRef.current = cycleAbLoop; }, [cycleAbLoop]);

  const takeScreenshot = useCallback(async () => {
    resetActivity();
    try {
      await invoke<string>('player_screenshot', { suggestedName: title || 'fluxa' });
      flashFeedback('screenshot', t('player.screenshot_saved'));
    } catch {
      flashFeedback('screenshot', t('player.screenshot_failed'));
    }
  }, [resetActivity, flashFeedback, title]);
  useEffect(() => { takeScreenshotRef.current = takeScreenshot; }, [takeScreenshot]);
  useEffect(() => { cycleAnime4kModeRef.current = cycleAnime4kMode; }, [cycleAnime4kMode]);

  const copyTimestamp = useCallback(async () => {
    const text = fmtTime(posRef.current);
    try { await navigator.clipboard.writeText(text); } catch {}
    flashFeedback('subDelay', text);
  }, [flashFeedback]);

  const centerClickTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const centerHoldTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const centerHoldActiveRef = useRef(false);
  const centerHoldJustEndedRef = useRef(false);

  const onCenterMouseDown = useCallback((e: React.MouseEvent) => {
    if (e.button !== 0) return;
    resetActivity();
    preSpeedRef.current = playbackSpeed;
    centerHoldTimerRef.current = setTimeout(() => {
      centerHoldActiveRef.current = true;
      sendCmd('set speed 2.00');
      flashFeedback('speed', '2×');
    }, 300);
  }, [flashFeedback, playbackSpeed, resetActivity]);

  const releaseCenterHold = useCallback(() => {
    if (centerHoldTimerRef.current) { clearTimeout(centerHoldTimerRef.current); centerHoldTimerRef.current = null; }
    if (centerHoldActiveRef.current) {
      centerHoldActiveRef.current = false;
      centerHoldJustEndedRef.current = true;
      sendCmd(`set speed ${preSpeedRef.current.toFixed(2)}`);
    }
  }, []);

  const onCenterClick = useCallback(() => {
    if (centerHoldJustEndedRef.current) { centerHoldJustEndedRef.current = false; return; }
    resetActivity();
    if (showEpisodePanel) { setShowEpisodePanel(false); episodePanelOpenRef.current = false; return; }
    if (trackPopover) { setTrackPopover(null); return; }
    if (centerClickTimerRef.current) {
      clearTimeout(centerClickTimerRef.current);
      centerClickTimerRef.current = null;
      void toggleFullscreen();
      return;
    }
    centerClickTimerRef.current = setTimeout(() => {
      centerClickTimerRef.current = null;
      const icon = pausedRef.current ? 'play' : 'pause';
      flashFeedback(icon, '');
      setPaused((prev) => !prev);
      sendCmd('cycle pause');
    }, 250);
  }, [flashFeedback, resetActivity, showEpisodePanel, toggleFullscreen, trackPopover]);

  const opacityStyle: React.CSSProperties = {
    opacity: controlsVisible ? 1 : 0,
    transition: 'opacity 0.4s ease',
    pointerEvents: controlsVisible ? 'auto' : 'none',
  };

  const dur = durRef.current;
  const chapterSegments = chapters.length >= 2 && dur > 0
    ? chapters.map((ch, i) => {
        const start = ch.startMs / 1000 / dur;
        const end = i + 1 < chapters.length ? chapters[i + 1].startMs / 1000 / dur : 1;
        return { start, end };
      })
    : null;
  chapterSegmentsRef.current = chapterSegments;
  const skipMarkers = dur > 0
    ? skipSegments
        .map((seg) => ({
          start: Math.max(0, Math.min(1, (seg.startTime / 1000) / dur)),
          end: Math.max(0, Math.min(1, (seg.endTime / 1000) / dur)),
        }))
        .filter((seg) => seg.end > seg.start)
    : [];

  if (miniPlayerActive) {
    return (
      <div
        ref={overlayRef}
        onMouseMove={resetActivity}
        style={{ position: 'fixed', inset: 0, zIndex: 9998, background: 'transparent' }}
      >
        <div style={{ position: 'absolute', bottom: 0, left: 0, right: 0, height: '0.125rem', background: 'rgba(255,255,255,0.15)' }}>
          <div ref={miniProgressRef} style={{ height: '100%', width: '0%', background: 'var(--primary-accent-color)' }} />
        </div>
        <div
          style={{ ...opacityStyle, position: 'absolute', bottom: '0.125rem', left: 0, right: 0, display: 'flex', alignItems: 'center', justifyContent: 'center', gap: '0.5rem', padding: '0.5rem 0.625rem', background: 'linear-gradient(to top, rgba(0,0,0,0.85), transparent)' }}
        >
          <button
            onClick={(e) => { e.stopPropagation(); resetActivity(); flashFeedback(paused ? 'play' : 'pause', ''); setPaused((prev) => !prev); sendCmd('cycle pause'); }}
            className="fluxa-ibtn"
            style={styles.iconBtn}
            title={paused ? t('player.play') : t('player.pause')}
          >
            {paused ? <Play size={16} fill="currentColor" strokeWidth={0} /> : <Pause size={16} fill="currentColor" strokeWidth={0} />}
          </button>
          <button
            onClick={(e) => { e.stopPropagation(); resetActivity(); void toggleMiniPlayer(); }}
            className="fluxa-ibtn"
            style={styles.iconBtn}
            title={t('player.restore_window')}
          >
            <Minimize2 size={16} />
          </button>
          <button
            onClick={(e) => { e.stopPropagation(); void closePlayer(); }}
            className="fluxa-ibtn"
            style={styles.iconBtn}
            title={t('player.back')}
          >
            <ChevronLeft size={16} />
          </button>
        </div>
      </div>
    );
  }

  return (
    <div
      ref={overlayRef}
      style={{ position: 'fixed', inset: 0, zIndex: 9998, display: 'flex', flexDirection: 'column', background: softwareVideoActive ? '#000' : 'transparent' }}
      onWheel={onOverlayWheel}
      onContextMenu={(e) => { e.preventDefault(); resetActivity(); setContextMenu({ x: e.clientX, y: e.clientY }); }}
    >
      {isBuffering && <PlayerBufferingOverlay logoUrl={initialLogoUrl} progress={bufferingProgress} />}
      <style>{`
        @keyframes fluxa-seek-spin { to { transform: rotate(360deg); } }
        @keyframes fluxa-skip-in { from { opacity: 0; transform: translateX(0.75rem); } to { opacity: 1; transform: translateX(0); } }
        @keyframes fluxa-nextep-in { from { opacity: 0; transform: translateX(0.75rem); } to { opacity: 1; transform: translateX(0); } }
        .fluxa-ibtn { opacity: 0.8; transition: opacity 0.15s, background 0.12s; }
        .fluxa-ibtn:hover { opacity: 1; background: rgba(255,255,255,0.09) !important; }
        .fluxa-skip-btn { animation: fluxa-skip-in 0.2s cubic-bezier(0.16, 1, 0.3, 1); transition: filter 0.12s, transform 0.12s; }
        .fluxa-skip-btn:hover { filter: brightness(1.06); transform: translateY(-0.0625rem); }
        .fluxa-skip-btn:active { transform: translateY(0); }
        .fluxa-skip-btn:focus-visible { outline: 0.125rem solid rgba(255,255,255,0.85); outline-offset: 0.125rem; }
        .fluxa-cursor-hidden, .fluxa-cursor-hidden * { cursor: none !important; }
        .fluxa-seek-track { transition: height 0.15s ease; }
        .fluxa-seek-dot { transition: width 0.15s, height 0.15s; }
      `}</style>

      {softwareVideoActive && (
        <canvas
          ref={softwareCanvasRef}
          style={{ position: 'absolute', inset: 0, width: '100%', height: '100%', objectFit: 'contain', background: '#000', zIndex: 0, pointerEvents: 'none' }}
        />
      )}

      <div style={{ position: 'absolute', left: 0, right: 0, top: 0, height: '8.75rem', background: 'linear-gradient(to bottom, rgba(0,0,0,0.7) 0%, transparent 100%)', zIndex: 1, opacity: controlsVisible ? 1 : 0, transition: 'opacity 0.4s ease', pointerEvents: 'none' }} />
      <div style={{ position: 'absolute', left: 0, right: 0, bottom: 0, height: '14.375rem', background: 'linear-gradient(to top, rgba(0,0,0,0.88) 0%, rgba(0,0,0,0.5) 45%, transparent 100%)', zIndex: 1, opacity: controlsVisible ? 1 : 0, transition: 'opacity 0.4s ease', pointerEvents: 'none' }} />

      <PlayerStatusToasts bannerOffset={bannerOffset} playbackError={playbackError} subtitleWarning={subtitleWarning} onClosePlayer={() => { void closePlayer(); }} onDismissSubtitleWarning={onDismissSubtitleWarning} />

      <PlayerHeader
        style={opacityStyle}
        bannerOffset={bannerOffset}
        title={title}
        episodeTitle={episodeTitle}
        hdrLabel={hdrLabel}
        activeCastDeviceId={activeCastDeviceId}
        activeCastDeviceName={activeCastDeviceName}
        castPaused={castPaused}
        castButtonRef={castBtnRef}
        streamLinksButtonRef={streamLinksBtnRef}
        settingsButtonRef={playerSettingsBtnRef}
        showSegmentMarker={showSegmentMarker}
        canMarkSegments={introDbSubmitEnabled && Boolean(introDbApiKey)}
        onClose={() => { void closePlayer(); }}
        onResetActivity={resetActivity}
        onToggleCastPause={toggleCastPause}
        onOpenCast={() => { void openCastPopover(); }}
        onToggleMiniPlayer={() => { void toggleMiniPlayer(); }}
        onOpenStreamLinks={() => {
          const rect = streamLinksBtnRef.current?.getBoundingClientRect();
          const stream = streamRef?.current;
          if (stream) void streamShellPlan(stream).then(setStreamLinksPlan);
          setStreamLinksMenuPoint(rect ? { x: Math.max(0, rect.right - 216), y: rect.bottom + 8 } : null);
        }}
        onToggleSettings={() => setShowPlayerSettings((value) => !value)}
        onToggleSegmentMarker={() => setShowSegmentMarker((value) => !value)}
      />
      <div style={{ flex: 1, cursor: 'default' }} onMouseDown={onCenterMouseDown} onMouseUp={releaseCenterHold} onMouseLeave={releaseCenterHold} onClick={onCenterClick} />

      {feedback && (
        <div style={{ position: 'absolute', top: '50%', left: '50%', transform: 'translate(-50%, -50%)', background: 'rgba(0,0,0,0.6)', backdropFilter: 'blur(0.5rem)', borderRadius: '0.875rem', padding: '0.75rem 1.375rem', display: 'flex', alignItems: 'center', gap: '0.5rem', color: '#fff', fontSize: '1.125rem', fontWeight: 700, pointerEvents: 'none', zIndex: 5 }}>
          {feedback.icon === 'play' && <Play size={20} fill="currentColor" strokeWidth={0} />}
          {feedback.icon === 'pause' && <Pause size={20} fill="currentColor" strokeWidth={0} />}
          {feedback.icon === 'seekBack' && <RotateCcw size={20} />}
          {feedback.icon === 'seekFwd' && <RotateCw size={20} />}
          {feedback.icon === 'speed' && <Gauge size={20} />}
          {feedback.icon === 'abLoop' && <Repeat size={20} />}
          {feedback.icon === 'screenshot' && <Camera size={20} />}
          {feedback.icon === 'subDelay' && <Captions size={20} />}
          {feedback.icon === 'anime4k' && <Sparkles size={20} />}
          {feedback.icon === 'volume' && (
            muted ? <VolumeOff size={20} /> : volumeLevel < 50 ? <Volume1 size={20} /> : <Volume2 size={20} />
          )}
          {feedback.icon === 'volume' ? (
            <span>{muted ? t('player.muted') : `${Math.round(volumeLevel)}%`}</span>
          ) : (
            feedback.label && <span>{feedback.label}</span>
          )}
        </div>
      )}

      {showSeekOverlay && (
        <div style={{ position: 'absolute', inset: 0, zIndex: 4, pointerEvents: 'none', display: 'flex', alignItems: 'center', justifyContent: 'center' }}>
          <div style={{ width: '2.25rem', height: '2.25rem', borderRadius: '50%', border: '0.1875rem solid rgba(255,255,255,0.15)', borderTopColor: 'rgba(255,255,255,0.75)', animation: 'fluxa-seek-spin 0.75s linear infinite' }} />
        </div>
      )}

      {activeSkip && !showNextEpCard && (
        <div style={{ position: 'absolute', bottom: '6.625rem', right: 0, zIndex: 4, display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
          {activeSkip.type === 'outro' && (
            <button
              onClick={(e) => { e.stopPropagation(); resetActivity(); setActiveSkip(null); }}
              style={styles.watchCreditsBtn}
            >
              {t('player.watch_credits')}
            </button>
          )}
          <button
            onClick={(e) => { e.stopPropagation(); resetActivity(); sendCmd(`set time-pos ${Math.floor(activeSkip.endMs / 1000)}`); }}
            className="fluxa-skip-btn"
            style={styles.skipBtn}
          >
            <div ref={skipFillRef} aria-hidden="true" style={{ position: 'absolute', inset: 0, width: '0%', background: '#fff', transition: 'width 0.15s linear' }} />
            <SkipForward size={18} style={{ position: 'relative', zIndex: 1 }} />
            <span style={{ position: 'relative', zIndex: 1 }}>{activeSkip.label}</span>
          </button>
        </div>
      )}

      {showNextEpCard && nextEpSubtitle && !showEpisodePanel && (
        <NextEpCard
          subtitle={nextEpSubtitle}
          thumbnail={nextEpThumbnail}
          countdown={countdown}
          countdownTotal={autoPlayCountdownSecs}
          bottom={activeSkip && !showNextEpCard ? 168 : 106}
          onPlay={() => { resetActivity(); void emit('native-player-next-episode', null); }}
          onDismiss={() => setNextEpDismissed(true)}
        />
      )}

      {showEpisodePanel && (
        <EpisodePanel
          episodes={episodes}
          currentEpisode={currentEpisode ?? null}
          onClose={() => { setShowEpisodePanel(false); episodePanelOpenRef.current = false; }}
        />
      )}

      {trackPopover && (
        <TrackPopover
          type={trackPopover}
          audioTracks={audioTracks}
          subTracks={subTracks}
          playbackSpeed={playbackSpeed}
          anchorRef={trackPopover === 'audio' ? audioTrackBtnRef : trackPopover === 'sub' ? subTrackBtnRef : speedBtnRef}
          onClose={() => setTrackPopover(null)}
          onSetSpeed={setSpeed}
          onSelectTrack={selectTrack}
          onDisableSubs={disableSubs}
          subtitleDelay={subtitleDelay}
          subtitlePosition={subtitlePosition}
          subtitleFont={subtitleFont}
          subtitleSize={subtitleSize}
          subtitleColor={subtitleColor}
          subtitleTextOpacity={subtitleTextOpacity}
          subtitleBackgroundColor={subtitleBackgroundColor}
          subtitleBackgroundOpacity={subtitleBackgroundOpacity}
          subtitleOutlineColor={subtitleOutlineColor}
          subtitleOutlineOpacity={subtitleOutlineOpacity}
          subtitleForceStyle={subtitleForceStyle}
          subtitleCharacterEdge={subtitleCharacterEdge}
          subtitleShadow={subtitleShadow}
          onAdjustSubtitleDelay={adjustSubtitleDelay}
          onChooseSubtitlePosition={chooseSubtitlePosition}
          onResetSubtitleDelay={resetSubtitleDelay}
          autoSyncing={autoSyncingSubtitles}
          onAutoSyncSubtitles={() => { void autoSyncSubtitles(); }}
          subtitleCaptureCues={subtitleCaptureCues}
          onApplySubtitleCapture={applySubtitleCapture}
          onChooseSubtitleFont={chooseSubtitleFont}
          onChooseSubtitleSize={chooseSubtitleSize}
          onAdjustSubtitleSize={adjustSubtitleSize}
          onChooseSubtitleColor={chooseSubtitleColor}
          onChooseSubtitleStyle={chooseSubtitleStyle}
        />
      )}

      <ContextMenu
        point={streamLinksMenuPoint}
        onClose={() => setStreamLinksMenuPoint(null)}
        items={(() => {
          const stream = streamRef?.current;
          const meta = metaRef?.current;
          if (!stream) return [];
          const sourceLink = streamLinksPlan?.sourceLink;
          const downloadLink = streamLinksPlan?.downloadLink;
          return [
            ...(sourceLink ? [{ icon: <Link2 size={15} />, label: t('player.copy_stream_link'), onSelect: () => { void navigator.clipboard.writeText(sourceLink); } }] : []),
            ...(streamLinksPlan?.isTorrent ? [{ icon: <Magnet size={15} />, label: t('player.copy_magnet_link'), onSelect: () => { void streamMagnetLink(stream).then((link) => { if (link) void navigator.clipboard.writeText(link); }); } }] : []),
            ...(meta && downloadLink ? [{ icon: <Download size={15} />, label: t('player.download_this_video'), onSelect: () => { void enqueueOfflineDownload(buildOfflineDownloadRequest(meta, stream, currentEpisode)); } }] : []),
          ];
        })()}
      />

      {castPopoverOpen && (
        <CastPopover
          devices={castDevices}
          discovering={castDiscovering}
          activeDeviceId={activeCastDeviceId}
          anchorRef={castBtnRef}
          onClose={() => setCastPopoverOpen(false)}
          onSelectDevice={(device) => void selectCastDevice(device)}
          onDisconnect={disconnectCast}
        />
      )}

      {showTorrentPopover && (
        <TorrentStatsPopover stats={torrentStatsSnap} anchorRef={torrentBtnRef} onClose={() => setShowTorrentPopover(false)} />
      )}

      {showSegmentMarker && introDbSubmitEnabled && introDbApiKey && (
        <SegmentMarkerPanel
          onClose={() => setShowSegmentMarker(false)}
          getPosMs={() => posRef.current * 1000}
          imdbId={introDbImdbId}
          season={currentEpisode?.season ?? null}
          episode={currentEpisode?.episode ?? currentEpisode?.number ?? null}
          apiKey={introDbApiKey}
        />
      )}

      {showPlayerSettings && (
        <PlayerSettingsPopover
          anchorRef={playerSettingsBtnRef}
          onClose={() => setShowPlayerSettings(false)}
          anime4kEnabled={anime4kEnabled}
          onToggleAnime4k={toggleAnime4k}
        />
      )}

      {showShortcutsHelp && <PlayerShortcutsDialog overrides={shortcutOverrides} onClose={() => setShowShortcutsHelp(false)} />}

      {showStats && (
        <div
          style={{
            position: 'fixed',
            top: '3.75rem',
            left: '1.25rem',
            background: 'rgba(0,0,0,0.84)',
            border: '1px solid rgba(255,255,255,0.1)',
            borderRadius: '0.5rem',
            padding: '0.625rem 0.875rem',
            fontFamily: 'monospace',
            fontSize: '0.6875rem',
            color: 'rgba(255,255,255,0.82)',
            lineHeight: 1.8,
            zIndex: 25,
            minWidth: '16.25rem',
            userSelect: 'none',
          }}
        >
          {(statsSnap?.width && statsSnap?.height) && (
            <div>{statsSnap.width}×{statsSnap.height}{statsSnap.videoFormat ? `  ${statsSnap.videoFormat}` : ''}{statsSnap.fps ? `  ${parseFloat(statsSnap.fps).toFixed(3)} ${t('player.stats_fps')}` : ''}{statsSnap.containerFps && statsSnap.fps && Math.abs(parseFloat(statsSnap.containerFps) - parseFloat(statsSnap.fps)) > 0.1 ? ` (${t('player.stats_container')} ${parseFloat(statsSnap.containerFps).toFixed(3)})` : ''}</div>
          )}
          {statsSnap?.displayFps && (
            <div>{t('player.stats_display_fps')}: {parseFloat(statsSnap.displayFps).toFixed(3)} {t('player.stats_fps')}</div>
          )}
          {statsSnap?.hwdecCurrent && statsSnap.hwdecCurrent !== 'no' && statsSnap.hwdecCurrent !== '' && (
            <div>{t('player.stats_hwdec')}: {statsSnap.hwdecCurrent}</div>
          )}
          {(statsSnap?.colorMatrix || statsSnap?.colorGamma || statsSnap?.colorPrimaries) && (() => {
            const inVals = [statsSnap.colorMatrix, statsSnap.colorGamma, statsSnap.colorPrimaries];
            const outVals = [statsSnap.videoOutMatrix, statsSnap.videoOutGamma, statsSnap.videoOutPrimaries];
            const inStr = inVals.filter(Boolean).join(' / ');
            const outStr = outVals.filter(Boolean).join(' / ');
            const isHdr = statsSnap.sigPeak != null && parseFloat(statsSnap.sigPeak) > 1;
            const colorsDiffer = inStr !== outStr && outStr.length > 0;
            if (colorsDiffer || isHdr) {
              return (
                <>
                  <div>{t('player.stats_color_in')}: {inStr}{isHdr ? `  ${t('player.stats_peak')} ${parseFloat(statsSnap.sigPeak!).toFixed(0)}` : ''}</div>
                  {outStr && <div>{t('player.stats_color_out')}: {outStr}</div>}
                </>
              );
            }
            return <div>{t('player.stats_color')}: {inStr}</div>;
          })()}
          {(statsSnap?.frameDropCount != null || statsSnap?.decoderFrameDropCount != null || statsSnap?.mistimedFrameCount != null || statsSnap?.voDelayedFrameCount != null) && (() => {
            const vo = parseInt(statsSnap?.frameDropCount ?? '0');
            const dec = parseInt(statsSnap?.decoderFrameDropCount ?? '0');
            const dropStr = (vo > 0 || dec > 0) ? `${vo} (vo) ${dec} (dec)` : '0';
            return <div>{t('player.stats_dropped')}: {dropStr}  {t('player.stats_mistimed')}: {statsSnap?.mistimedFrameCount ?? '0'}  {t('player.stats_vo_delayed')}: {statsSnap?.voDelayedFrameCount ?? '0'}</div>;
          })()}
          {(statsSnap?.videoBitrate || statsSnap?.audioBitrate) && (
            <div>
              {statsSnap.videoBitrate ? `${t('player.stats_video_bitrate')}: ${(parseInt(statsSnap.videoBitrate) / 1000).toFixed(0)} kbps` : ''}
              {statsSnap.audioBitrate ? `  ${t('player.stats_audio_bitrate')}: ${(parseInt(statsSnap.audioBitrate) / 1000).toFixed(0)} kbps` : ''}
            </div>
          )}
          {(statsSnap?.audioCodec || statsSnap?.audioSamplerate || statsSnap?.audioChannels) && (
            <div>{[statsSnap.audioCodec, statsSnap.audioSamplerate ? `${statsSnap.audioSamplerate} Hz` : null, statsSnap.audioChannels].filter(Boolean).join('  ')}</div>
          )}
          {(statsSnap?.demuxerCacheDuration != null) && (
            <div style={{ display: 'flex', alignItems: 'center', gap: '0.375rem', flexWrap: 'wrap', rowGap: '0.125rem' }}>
              <span>{t('player.stats_buffer')}:</span>
              <Sparkline data={bufferHistoryRef.current} gradId="sg-buf" />
              <span>{parseFloat(statsSnap.demuxerCacheDuration ?? '0').toFixed(1)}s</span>
              {stallCountRef.current > 0 && <span style={{ color: 'rgba(255,255,255,0.45)' }}>{stallCountRef.current} {stallCountRef.current === 1 ? t('player.stats_stalls') : t('player.stats_stalls_plural')}</span>}
              {statsSnap.cacheBufferingState && statsSnap.pausedForCache === 'yes' && <span style={{ color: 'rgba(255,255,255,0.45)' }}>{statsSnap.cacheBufferingState}%</span>}
            </div>
          )}
          {(statsSnap?.cacheSpeed != null) && (() => {
            const bytes = parseInt(statsSnap.cacheSpeed ?? '0');
            const speedStr = bytes >= 1024 * 1024
              ? `${(bytes / (1024 * 1024)).toFixed(1)} MB/s`
              : `${(bytes / 1024).toFixed(0)} KB/s`;
            return (
              <div style={{ display: 'flex', alignItems: 'center', gap: '0.375rem' }}>
                <span>{t('player.stats_net')}:</span>
                <Sparkline data={netSpeedHistoryRef.current} gradId="sg-net" />
                <span>{speedStr}</span>
              </div>
            );
          })()}
          {statsSnap?.avsync != null && (
            <div>{t('player.stats_avsync')}: {parseFloat(statsSnap.avsync).toFixed(3)}s</div>
          )}
          {statsSnap?.fileFormat && (
            <div>
              {t('player.stats_container')}: {statsSnap.fileFormat}
              {(() => {
                try {
                  const host = new URL(statsSnap.path ?? '').hostname;
                  return host && !host.startsWith('127.') ? `  · ${host}` : '';
                } catch { return ''; }
              })()}
            </div>
          )}
          {torrentStatsSnap && torrentStatsSnap.stat >= 2 && (
            <div>
              <div>{t('player.stats_torrent')}: {torrentStatsSnap.active_peers}/{torrentStatsSnap.total_peers} {t('player.stats_peers')}  {t('player.stats_preload')}: {torrentStatsSnap.preload}%</div>
              <div style={{ display: 'flex', alignItems: 'center', gap: '0.375rem' }}>
                <span>↓</span>
                <Sparkline data={torrentSpeedHistory} gradId="sg-tor" />
                <span>{(torrentStatsSnap.download_speed / (1024 * 1024)).toFixed(2)} MB/s</span>
              </div>
            </div>
          )}
          <div style={{ marginTop: '0.25rem', borderTop: '1px solid rgba(255,255,255,0.08)', paddingTop: '0.25rem' }}>
            <button
              style={{ background: 'none', border: 'none', color: 'rgba(255,255,255,0.5)', fontSize: '0.6875rem', fontFamily: 'monospace', cursor: 'pointer', padding: 0 }}
              onClick={() => {
                const lines: string[] = [];
                if (statsSnap?.width && statsSnap?.height) lines.push(`${statsSnap.width}×${statsSnap.height}  ${statsSnap.videoFormat ?? ''}  ${statsSnap.fps ? parseFloat(statsSnap.fps).toFixed(3) + ' fps' : ''}`);
                if (statsSnap?.hwdecCurrent && statsSnap.hwdecCurrent !== 'no') lines.push(`HW: ${statsSnap.hwdecCurrent}`);
                if (statsSnap?.colorMatrix) {
                  const inStr = [statsSnap.colorMatrix, statsSnap.colorGamma, statsSnap.colorPrimaries].filter(Boolean).join(' / ');
                  const outStr = [statsSnap.videoOutMatrix, statsSnap.videoOutGamma, statsSnap.videoOutPrimaries].filter(Boolean).join(' / ');
                  const isHdr = statsSnap.sigPeak != null && parseFloat(statsSnap.sigPeak) > 1;
                  if (isHdr || inStr !== outStr) {
                    lines.push(`In: ${inStr}${isHdr ? ` peak ${parseFloat(statsSnap.sigPeak!).toFixed(0)}` : ''}`);
                    if (outStr) lines.push(`Out: ${outStr}`);
                  } else {
                    lines.push(`Color: ${inStr}`);
                  }
                }
                const voDrop = parseInt(statsSnap?.frameDropCount ?? '0');
                const decDrop = parseInt(statsSnap?.decoderFrameDropCount ?? '0');
                const dropStr = (voDrop > 0 || decDrop > 0) ? `${voDrop} (vo) ${decDrop} (dec)` : '0';
                lines.push(`Dropped: ${dropStr}  Mistimed: ${statsSnap?.mistimedFrameCount ?? 0}  VO-delay: ${statsSnap?.voDelayedFrameCount ?? 0}`);
                if (statsSnap?.videoBitrate) lines.push(`Video: ${(parseInt(statsSnap.videoBitrate) / 1000).toFixed(0)} kbps  Audio: ${statsSnap.audioBitrate ? (parseInt(statsSnap.audioBitrate) / 1000).toFixed(0) + ' kbps' : 'n/a'}`);
                if (statsSnap?.audioCodec) lines.push([statsSnap.audioCodec, statsSnap.audioSamplerate ? `${statsSnap.audioSamplerate} Hz` : null, statsSnap.audioChannels].filter(Boolean).join('  '));
                if (statsSnap?.cacheSpeed != null) {
                  const bytes = parseInt(statsSnap.cacheSpeed ?? '0');
                  const speedStr = bytes >= 1024 * 1024 ? `${(bytes / (1024 * 1024)).toFixed(1)} MB/s` : `${(bytes / 1024).toFixed(0)} KB/s`;
                  lines.push(`Net: ${speedStr}`);
                }
                lines.push(`Buffer: ${parseFloat(statsSnap?.demuxerCacheDuration ?? '0').toFixed(1)}s  stalls: ${stallCountRef.current}`);
                if (statsSnap?.avsync) lines.push(`A/V: ${parseFloat(statsSnap.avsync).toFixed(3)}s`);
                if (statsSnap?.fileFormat) lines.push(`Container: ${statsSnap.fileFormat}`);
                if (torrentStatsSnap) lines.push(`Torrent: ${torrentStatsSnap.active_peers}/${torrentStatsSnap.total_peers} peers  ${(torrentStatsSnap.download_speed / (1024 * 1024)).toFixed(2)} MB/s  preload: ${torrentStatsSnap.preload}%`);
                navigator.clipboard.writeText(lines.join('\n')).catch(() => undefined);
              }}
            >{t('player.stats_copy')}</button>
          </div>
        </div>
      )}

      {contextMenu && <PlayerContextMenu point={contextMenu} abLoopStage={abLoopStage} showStats={showStats} onClose={() => setContextMenu(null)} onCycleAbLoop={cycleAbLoop} onCopyTimestamp={() => { void copyTimestamp(); }} onToggleStats={() => setShowStats((value) => !value)} onToggleShortcuts={() => setShowShortcutsHelp((value) => !value)} onOpenAudioTracks={() => { void openTrackPopover('audio'); }} onScreenshot={() => { void takeScreenshot(); }} />}

      <div
        style={{
          ...opacityStyle,
          position: 'relative',
          paddingRight: showEpisodePanel ? 380 : 0,
          background: 'transparent',
          zIndex: 2,
          overflow: 'visible',
        }}
        onMouseEnter={() => { isOverControlsRef.current = true; }}
        onMouseLeave={() => { isOverControlsRef.current = false; }}
      >
        <PlayerSeekBar
          barRef={seekbarRef}
          fillRef={seekFillRef}
          bufferRef={seekBufferRef}
          dotRef={seekDotRef}
          segmentFillRefs={segFillRefs}
          segmentBufferRefs={segBufRefs}
          durationRef={durRef}
          chaptersRef={chaptersRef}
          chapterSegments={chapterSegments}
          skipMarkers={skipMarkers}
          onSeekStart={onSeekMouseDown}
        />

        <div style={{ display: 'flex', alignItems: 'center', padding: '0 0.5rem 0.875rem', gap: 0 }}>
          <button
            onClick={(e) => { e.stopPropagation(); resetActivity(); flashFeedback(paused ? 'play' : 'pause', ''); setPaused((prev) => !prev); sendCmd('cycle pause'); }}
            className="fluxa-ibtn"
            style={{ ...styles.iconBtn, width: '3rem', height: '3rem' }}
            title={paused ? t('player.play') : t('player.pause')}
          >
            {paused ? <Play size={26} fill="currentColor" strokeWidth={0} /> : <Pause size={26} fill="currentColor" strokeWidth={0} />}
          </button>
          <button onClick={(e) => { e.stopPropagation(); resetActivity(); startSeekOverlay(); flashFeedback('seekBack', '-10s'); sendCmd('seek -10 relative'); }} className="fluxa-ibtn" style={styles.iconBtn} title={t('player.seek_back')}>
            <RotateCcw size={22} />
          </button>
          <button onClick={(e) => { e.stopPropagation(); resetActivity(); startSeekOverlay(); flashFeedback('seekFwd', '+10s'); sendCmd('seek 10 relative'); }} className="fluxa-ibtn" style={styles.iconBtn} title={t('player.seek_forward')}>
            <RotateCw size={22} />
          </button>
          <div
            style={{ display: 'flex', alignItems: 'center', position: 'relative', flexShrink: 0 }}
            onMouseEnter={() => { if (volumeHideTimer.current) clearTimeout(volumeHideTimer.current); setShowVolumeSlider(true); }}
            onMouseLeave={() => { volumeHideTimer.current = setTimeout(() => setShowVolumeSlider(false), 200); }}
            onWheel={(e) => {
              e.stopPropagation();
              resetActivity();
              sendCmd(`add volume ${e.deltaY < 0 ? 5 : -5}`);
              flashFeedback('volume', '');
              setShowVolumeSlider(true);
              setVolumeScrolling(true);
              if (volumeScrollTimer.current) clearTimeout(volumeScrollTimer.current);
              volumeScrollTimer.current = setTimeout(() => setVolumeScrolling(false), 700);
            }}
          >
            <button onClick={(e) => { e.stopPropagation(); resetActivity(); setMuted((prev) => !prev); sendCmd('cycle mute'); }} className="fluxa-ibtn" style={styles.iconBtn} title={muted ? t('player.unmute') : t('player.mute')}>
              <IconVolume muted={muted} level={volumeLevel} />
            </button>
            <div style={{ width: showVolumeSlider ? '5.75rem' : 0, flexShrink: 0, opacity: showVolumeSlider ? 1 : 0, pointerEvents: showVolumeSlider ? 'auto' : 'none', transition: 'width 0.18s ease, opacity 0.18s ease', overflow: 'hidden', display: 'flex', alignItems: 'center', paddingLeft: showVolumeSlider ? '0.25rem' : 0 }}>
              <VolumeBar
                value={muted ? 0 : volumeLevel}
                max={130}
                forceTooltip={volumeScrolling}
                onChange={(v) => {
                  resetActivity();
                  flashFeedback('volume', '');
                  if (activeCastDeviceIdRef.current) { castSetVolume(v / 100); return; }
                  sendCmd(`set volume ${v}`);
                  if (muted && v > 0) sendCmd('set mute no');
                }}
              />
            </div>
          </div>
          <div style={{ display: 'flex', alignItems: 'baseline', gap: '0.1875rem', paddingLeft: '0.625rem', pointerEvents: 'none', flexShrink: 0 }}>
            <span ref={currentTimeRef} style={{ fontSize: '0.8125rem', fontWeight: 700, fontVariantNumeric: 'tabular-nums', color: 'rgba(255,255,255,0.9)', letterSpacing: '0.0125rem' }}>0:00</span>
            <span style={{ fontSize: '0.75rem', color: 'rgba(255,255,255,0.3)' }}>/</span>
            <span ref={durationRef} style={{ fontSize: '0.75rem', fontVariantNumeric: 'tabular-nums', color: 'rgba(255,255,255,0.4)', letterSpacing: '0.0125rem' }}>0:00</span>
          </div>

          <div style={{ flex: 1 }} />

          {isTorrentStream && (
            <button
              ref={torrentBtnRef}
              onClick={(e) => { e.stopPropagation(); resetActivity(); setShowTorrentPopover((prev) => !prev); }}
              className="fluxa-ibtn"
              style={{ ...styles.iconBtn, color: showTorrentPopover ? 'var(--primary-accent-color)' : '#fff' }}
              title={t('player.torrent_stats_title')}
            >
              <Share2 size={20} />
            </button>
          )}
          {nextEpSubtitle && (
            <button onClick={(e) => { e.stopPropagation(); resetActivity(); void emit('native-player-next-episode', null); }} className="fluxa-ibtn" style={styles.iconBtn} title={t('player.next_label', nextEpSubtitle)}>
              <SkipForward size={22} />
            </button>
          )}
          {episodes.length > 0 && (
            <button
              onClick={(e) => {
                e.stopPropagation();
                resetActivity();
                const next = !showEpisodePanel;
                setShowEpisodePanel(next);
                episodePanelOpenRef.current = next;
              }}
              className="fluxa-ibtn"
              style={styles.iconBtn}
              title={t('player.episodes')}
            >
              <GalleryVerticalEnd size={22} />
            </button>
          )}
          <button ref={subTrackBtnRef} onClick={(e) => { e.stopPropagation(); void openTrackPopover('sub'); }} className="fluxa-ibtn" style={styles.iconBtn} title={t('player.subtitles')}>
            <Captions size={22} />
          </button>
          <button ref={audioTrackBtnRef} onClick={(e) => { e.stopPropagation(); void openTrackPopover('audio'); }} className="fluxa-ibtn" style={styles.iconBtn} title={t('player.audio')}>
            <AudioLines size={22} />
          </button>
          <button ref={speedBtnRef} onClick={(e) => { e.stopPropagation(); void openTrackPopover('speed'); }} className="fluxa-ibtn" style={styles.iconBtn} title={t('player.speed_label', playbackSpeed === 1 ? t('player.normal') : `${playbackSpeed}×`)}>
            <Gauge size={22} />
          </button>
          <button onClick={(e) => { e.stopPropagation(); resetActivity(); void toggleFullscreen(); }} className="fluxa-ibtn" style={styles.iconBtn} title={t('player.fullscreen')}>
            <Fullscreen size={22} />
          </button>
        </div>
      </div>
    </div>
  );
}


const styles = {
  iconBtn: {
    background: 'none',
    border: 'none',
    color: '#fff',
    cursor: 'pointer',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    width: '2.75rem',
    height: '2.75rem',
    borderRadius: '0.5rem',
    padding: 0,
    flexShrink: 0,
  } as React.CSSProperties,

  skipBtn: {
    appearance: 'none',
    background: 'rgba(218,218,218,0.88)',
    backdropFilter: 'blur(0.5rem)',
    boxShadow: 'none',
    outline: 'none',
    border: 'none',
    borderRadius: '0.1875rem 0 0 0.1875rem',
    color: '#090909',
    fontSize: '0.9375rem',
    fontWeight: 600,
    padding: '0.75rem 1.4375rem',
    cursor: 'pointer',
    letterSpacing: '0.0125rem',
    display: 'flex',
    alignItems: 'center',
    gap: '0.5rem',
    position: 'relative',
    overflow: 'hidden',
  } as React.CSSProperties,

  watchCreditsBtn: {
    appearance: 'none',
    background: 'rgba(218,218,218,0.88)',
    backdropFilter: 'blur(0.5rem)',
    border: 'none',
    borderRadius: '0.1875rem',
    color: '#090909',
    fontSize: '0.9375rem',
    fontWeight: 600,
    padding: '0.75rem 1.125rem',
    cursor: 'pointer',
    letterSpacing: '0.0125rem',
    transition: 'filter 0.12s, transform 0.12s',
  } as React.CSSProperties,
} satisfies Record<string, React.CSSProperties>;
