import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import {
  Captions,
  ChevronLeft,
  ExternalLink,
  Maximize,
  Pause,
  PictureInPicture2,
  Play,
  RotateCcw,
  RotateCw,
  Volume2,
  VolumeX,
} from 'lucide-react';
import { t } from '../i18n';
import { transcodeUrl, type PlaybackUrlChoice } from '../platform/web/stream';
import { attachMseRemuxSource } from '../platform/web/mseMkvSource';
import { PlayerOverlayStyles } from './player/PlayerOverlayStyles';
import type { PlayerSubtitleSource } from '../core/playerUtils';
import type { PlaybackSnapshot, ScrobbleEvent } from '../core/playbackSession';
import type { IntroSegmentResult } from '../core/effectRunner';
import { skipLabelForType } from './player/PlayerOverlayPrimitives';
import { appPrefs, prefBool } from '../core/appPrefs';
import {
  WatchTogetherClient,
  type WatchTogetherConnection,
  type WatchTogetherContent,
  type WatchTogetherState,
} from '../core/watchTogether';
import { useLibassSubtitles } from '../hooks/useLibassSubtitles';
import { applyWebOSMediaOption, hdrKindFrom, IS_WEBOS } from '../platform/webos';
import { isTextEntryTarget, tvActionFor } from '../platform/webos/keys';
import { useIsTouch } from '../platform/viewport';
import { enterFullscreen, exitFullscreen } from '../platform/fullscreenOrientation';
import { useWakeLock } from '../hooks/useWakeLock';
import { externalLinkTargets } from '../platform/externalPlayerLinks';
import { OFFICIAL_WATCH_TOGETHER_URL } from '../appConstants';
import { attachWebAudioProcessing } from '../platform/web/audioProcessing';

function isSupabaseInstance(instanceUrl: string): boolean {
  try {
    const absolute = /^https?:\/\//.test(instanceUrl) ? instanceUrl : `https://${instanceUrl}`;
    return new URL(absolute).hostname.endsWith('.supabase.co');
  } catch {
    return false;
  }
}

interface Props {
  url: string;
  mode?: PlaybackUrlChoice['mode'] | null;
  title?: string;
  subtitles: PlayerSubtitleSource[];
  onClose: () => Promise<void>;
  onFirstFrame: () => void;
  onPlaybackError?: (message: string) => void;
  contentId?: string;
  contentType?: string;
  videoId?: string;
  codecs?: { videoCodec: string | null; audioCodec: string | null } | null;
  audioProcessingMode?: string;
  resumeAt?: number;
  snapshotRef?: React.RefObject<PlaybackSnapshot | null>;
  onPlaybackEvent?: (event: ScrobbleEvent) => void;
  skipSegments?: IntroSegmentResult[];
  nextEpisode?: { title?: string; season?: number; episode?: number; number?: number } | null;
  onPlayNextEpisode?: () => void;
  autoSkip?: boolean;
  autoPlayNext?: boolean;
  onHandoff?: (target: string, positionSeconds: number) => void;
}

function formatTime(value: number) {
  if (!Number.isFinite(value) || value < 0) return '0:00';
  const seconds = Math.floor(value);
  const hours = Math.floor(seconds / 3600);
  const minutes = Math.floor((seconds % 3600) / 60);
  const remainder = seconds % 60;
  return hours > 0
    ? `${hours}:${String(minutes).padStart(2, '0')}:${String(remainder).padStart(2, '0')}`
    : `${minutes}:${String(remainder).padStart(2, '0')}`;
}

const S = {
  skipButton: {
    padding: '0.6rem 1rem',
    borderRadius: '0.45rem',
    border: '1px solid rgba(255,255,255,0.2)',
    background: 'rgba(20,25,34,0.92)',
    color: '#fff',
    fontSize: '0.85rem',
    fontWeight: 700,
    cursor: 'pointer',
    pointerEvents: 'auto',
  },
} as const;

const iconButton = {
  width: '2.75rem',
  height: '2.75rem',
  border: 'none',
  borderRadius: '0.5rem',
  background: 'none',
  color: '#fff',
  display: 'inline-flex',
  alignItems: 'center',
  justifyContent: 'center',
  cursor: 'pointer',
} as const;

const DOUBLE_TAP_MS = 320;

const pipSupported = typeof document !== 'undefined' && document.pictureInPictureEnabled === true;

export function WebPlayerOverlay({
  url,
  mode,
  title,
  subtitles,
  onClose,
  onFirstFrame,
  onPlaybackError,
  contentId,
  contentType = 'movie',
  videoId,
  codecs,
  audioProcessingMode = 'reference',
  resumeAt,
  snapshotRef,
  onPlaybackEvent,
  skipSegments = [],
  nextEpisode,
  onPlayNextEpisode,
  autoSkip = false,
  autoPlayNext = false,
  onHandoff,
}: Props) {
  const videoRef = useRef<HTMLVideoElement>(null);
  const subtitleCanvasRef = useRef<HTMLCanvasElement>(null);
  const isTouch = useIsTouch();
  const lastTapRef = useRef<{ at: number; side: 'left' | 'right' } | null>(null);
  const [seekFlash, setSeekFlash] = useState<{ side: 'left' | 'right'; at: number } | null>(null);
  const [handoffOpen, setHandoffOpen] = useState(false);
  const [currentTime, setCurrentTime] = useState(0);
  const externalTargets = useMemo(
    () =>
      onHandoff
        ? externalLinkTargets(url, `${window.location.origin}${window.location.pathname}?fluxa_external=done`, {
            errorUrl: `${window.location.origin}${window.location.pathname}?fluxa_external=error`,
            position: currentTime || resumeAt,
            filename: title,
            subtitleUrl: subtitles.find((subtitle) => subtitle.url.trim())?.url,
          })
        : [],
    [currentTime, onHandoff, resumeAt, subtitles, title, url],
  );
  const fallbackUsedRef = useRef(false);
  const hideTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const [paused, setPaused] = useState(false);
  const [muted, setMuted] = useState(false);
  const [volume, setVolume] = useState(1);
  const [duration, setDuration] = useState(0);
  const [controlsVisible, setControlsVisible] = useState(true);
  const [selectedSubtitle, setSelectedSubtitle] = useState<number>(subtitles.length > 0 ? 0 : -1);
  const [subtitleMenuOpen, setSubtitleMenuOpen] = useState(false);
  const [nextCountdown, setNextCountdown] = useState<number | null>(null);
  const autoSkippedRef = useRef<Set<string>>(new Set());
  const [watchTogetherState, setWatchTogetherState] = useState<WatchTogetherState>({
    connectionState: 'disconnected',
    roomCode: null,
    isHost: false,
    members: [],
    content: null,
    errorMessage: null,
  });
  const watchTogetherRef = useRef<WatchTogetherClient | null>(null);
  const watchTogetherContent: WatchTogetherContent | null = contentId ? { id: contentId, contentType, videoId, title: title ?? '' } : null;

  if (!watchTogetherRef.current) watchTogetherRef.current = new WatchTogetherClient({ onState: setWatchTogetherState });

  useEffect(() => () => watchTogetherRef.current?.leave(), []);

  useEffect(() => {
    const video = videoRef.current;
    if (!video || !watchTogetherContent) return;
    watchTogetherRef.current?.attach(video, watchTogetherContent);
    return () => watchTogetherRef.current?.detach();
  }, [watchTogetherContent?.id, watchTogetherContent?.contentType, watchTogetherContent?.videoId, watchTogetherContent?.title]);

  useEffect(() => {
    if (watchTogetherContent) watchTogetherRef.current?.updateContent(watchTogetherContent);
  }, [watchTogetherContent?.id, watchTogetherContent?.contentType, watchTogetherContent?.videoId, watchTogetherContent?.title]);

  const openWatchTogether = async () => {
    const client = watchTogetherRef.current;
    if (!client) return;
    if (watchTogetherState.roomCode) {
      client.leave();
      return;
    }
    const stored = window.localStorage.getItem('fluxa.watchTogether.instanceUrl') ?? OFFICIAL_WATCH_TOGETHER_URL;
    const instanceUrl = (stored || window.prompt(t('player.watch_party_instance_prompt')) || '').trim();
    if (!instanceUrl) return;
    window.localStorage.setItem('fluxa.watchTogether.instanceUrl', instanceUrl);
    const displayName = window.prompt(t('player.watch_party_name_prompt'), t('player.watch_party_guest')) ?? t('player.watch_party_guest');
    const roomCode = window.prompt(t('player.watch_party_room_prompt'));
    try {
      let connection: WatchTogetherConnection;
      if (isSupabaseInstance(instanceUrl)) {
        const anonKey =
          window.localStorage.getItem('fluxa.watchTogether.supabaseAnonKey') ?? window.prompt(t('player.watch_party_supabase_key_prompt'));
        if (!anonKey) return;
        window.localStorage.setItem('fluxa.watchTogether.supabaseAnonKey', anonKey);
        connection = { mode: 'supabase', projectUrl: instanceUrl, anonKey };
      } else {
        const secret =
          window.prompt(t('player.watch_party_secret_prompt'), window.localStorage.getItem('fluxa.watchTogether.secret') ?? '') ?? '';
        window.localStorage.setItem('fluxa.watchTogether.secret', secret);
        connection = { mode: 'websocket', serverUrl: instanceUrl, secret };
      }
      const secret = connection.mode === 'websocket' ? connection.secret : undefined;
      if (roomCode?.trim()) await client.join(connection, roomCode.trim(), displayName, secret);
      else await client.create(connection, displayName, secret);
    } catch (error) {
      setWatchTogetherState((state) => ({
        ...state,
        connectionState: 'error',
        errorMessage: error instanceof Error ? error.message : String(error),
      }));
    }
  };

  useEffect(() => {
    setSelectedSubtitle(subtitles.length > 0 ? 0 : -1);
  }, [subtitles]);

  useLibassSubtitles(videoRef, subtitleCanvasRef, subtitles, selectedSubtitle);

  const activeSegment = skipSegments.find((segment) => currentTime >= segment.startTime && currentTime < segment.endTime) ?? null;

  const skipActiveSegment = useCallback(() => {
    const video = videoRef.current;
    if (!video || !activeSegment) return;
    video.currentTime = activeSegment.endTime;
  }, [activeSegment]);

  useEffect(() => {
    if (!autoSkip || !activeSegment) return;
    const key = `${activeSegment.type}:${activeSegment.startTime}`;
    if (autoSkippedRef.current.has(key)) return;
    autoSkippedRef.current.add(key);
    skipActiveSegment();
  }, [autoSkip, activeSegment, skipActiveSegment]);

  useEffect(() => {
    autoSkippedRef.current = new Set();
  }, [url]);

  useEffect(() => {
    if (!autoPlayNext || !nextEpisode || !onPlayNextEpisode) return undefined;
    const remaining = duration - currentTime;
    if (!Number.isFinite(duration) || duration <= 0 || remaining > 15 || remaining < 0) {
      setNextCountdown(null);
      return undefined;
    }
    setNextCountdown(Math.max(0, Math.ceil(remaining)));
    if (remaining <= 1) onPlayNextEpisode();
    return undefined;
  }, [autoPlayNext, nextEpisode, onPlayNextEpisode, duration, currentTime]);

  useWakeLock(!paused);

  useEffect(() => {
    const session = typeof navigator !== 'undefined' ? navigator.mediaSession : undefined;
    if (!session || typeof MediaMetadata === 'undefined') return undefined;
    session.metadata = new MediaMetadata({ title: title ?? 'Fluxa', artist: 'Fluxa' });
    const nudge = (delta: number) => () => {
      const video = videoRef.current;
      if (!video) return;
      video.currentTime = Math.max(0, Math.min(video.duration || Infinity, video.currentTime + delta));
    };
    session.setActionHandler('play', () => {
      void videoRef.current?.play();
    });
    session.setActionHandler('pause', () => videoRef.current?.pause());
    session.setActionHandler('seekbackward', nudge(-10));
    session.setActionHandler('seekforward', nudge(10));
    session.setActionHandler('seekto', (details) => {
      const video = videoRef.current;
      if (video && details.seekTime != null) video.currentTime = details.seekTime;
    });
    session.setActionHandler('nexttrack', nextEpisode && onPlayNextEpisode ? () => onPlayNextEpisode() : null);
    return () => {
      for (const action of ['play', 'pause', 'seekbackward', 'seekforward', 'seekto', 'nexttrack'] as const) {
        session.setActionHandler(action, null);
      }
      session.metadata = null;
    };
  }, [title, nextEpisode, onPlayNextEpisode]);

  useEffect(() => {
    if (typeof navigator === 'undefined' || !navigator.mediaSession) return;
    navigator.mediaSession.playbackState = paused ? 'paused' : 'playing';
  }, [paused]);

  const recordSnapshot = useCallback(
    (video: HTMLVideoElement) => {
      if (!snapshotRef) return;
      snapshotRef.current = { timePos: video.currentTime, duration: video.duration };
    },
    [snapshotRef],
  );

  const resetActivity = useCallback(() => {
    setControlsVisible(true);
    if (hideTimerRef.current) clearTimeout(hideTimerRef.current);
    hideTimerRef.current = setTimeout(() => {
      if (!videoRef.current?.paused) setControlsVisible(false);
    }, 2500);
  }, []);

  useEffect(() => {
    const video = videoRef.current;
    if (!video) return;
    fallbackUsedRef.current = false;
    let cancelled = false;
    let releaseMseSource: (() => void) | null = null;
    let releaseAudioProcessing: (() => void) | null = null;
    const start = async () => {
      if (IS_WEBOS) {
        await applyWebOSMediaOption(video, url, {
          hdr: hdrKindFrom(codecs?.videoCodec),
          multiChannelAudio: true,
          audioCodec: codecs?.audioCodec,
          audioProcessingMode: audioProcessingMode === 'balanced' || audioProcessingMode === 'night' ? audioProcessingMode : 'reference',
        });
        if (cancelled) return;
      }
      if (mode === 'mse-remux') {
        try {
          releaseMseSource = await attachMseRemuxSource(video, url, codecs?.videoCodec ?? null, codecs?.audioCodec ?? null);
        } catch {
          // In-browser remux failed (most commonly: the source doesn't send
          // CORS headers, so we can't read its bytes via fetch()). Fall back
          // to the same companion-server transcode path the plain <video>
          // onError handler below uses.
          if (cancelled) return;
          fallbackUsedRef.current = true;
          video.src = transcodeUrl(url);
          video.load();
        }
        if (cancelled) return;
      } else {
        video.src = url;
        video.load();
      }
      // Attach only after the source exists so the helper can make a safe
      // same-origin/CORS decision. Remote media stays on native PCM if CORS
      // does not permit Web Audio routing.
      releaseAudioProcessing = await attachWebAudioProcessing(video, audioProcessingMode);
      if (cancelled) return;
      void video.play().catch(() => setPaused(true));
    };
    void start();
    return () => {
      cancelled = true;
      if (hideTimerRef.current) clearTimeout(hideTimerRef.current);
      video.pause();
      video.removeAttribute('src');
      releaseMseSource?.();
      releaseAudioProcessing?.();
    };
  }, [audioProcessingMode, url, mode, codecs?.videoCodec, codecs?.audioCodec]);

  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if (isTextEntryTarget(event.target)) return;
      const video = videoRef.current;
      const nudge = (delta: number) => {
        if (!video) return;
        video.currentTime = Math.max(0, Math.min(video.duration || Infinity, video.currentTime + delta));
        watchTogetherRef.current?.notifyLocalPlayback();
      };
      const action = tvActionFor(event);
      if (action === 'back' || action === 'stop') {
        event.preventDefault();
        void onClose();
        return;
      }
      if (action === 'play') {
        event.preventDefault();
        void video?.play();
      } else if (action === 'pause') {
        event.preventDefault();
        video?.pause();
      } else if (action === 'playPause' || action === 'enter' || event.key === ' ') {
        event.preventDefault();
        if (video?.paused) void video.play();
        else video?.pause();
      } else if (action === 'rewind') {
        event.preventDefault();
        nudge(-30);
      } else if (action === 'fastForward') {
        event.preventDefault();
        nudge(30);
      } else if (action === 'left') {
        event.preventDefault();
        nudge(-10);
      } else if (action === 'right') {
        event.preventDefault();
        nudge(10);
      } else if (action === 'blue') {
        event.preventDefault();
        setSubtitleMenuOpen((value) => !value);
      }
      resetActivity();
    };
    window.addEventListener('keydown', onKeyDown);
    return () => window.removeEventListener('keydown', onKeyDown);
  }, [onClose, resetActivity]);

  const togglePause = () => {
    const video = videoRef.current;
    if (!video) return;
    if (video.paused) void video.play();
    else video.pause();
    resetActivity();
  };

  const seek = (seconds: number) => {
    const video = videoRef.current;
    if (!video) return;
    video.currentTime = Math.max(0, Math.min(video.duration || Infinity, video.currentTime + seconds));
    resetActivity();
  };

  const toggleMute = () => {
    const video = videoRef.current;
    if (!video) return;
    video.muted = !video.muted;
    setMuted(video.muted);
    resetActivity();
  };

  const setVideoVolume = (value: number) => {
    const video = videoRef.current;
    if (!video) return;
    video.volume = value;
    video.muted = value === 0;
    setVolume(value);
    setMuted(video.muted);
  };

  const handleSurfaceClick = (event: React.MouseEvent) => {
    if (!isTouch) {
      resetActivity();
      return;
    }
    if ((event.target as HTMLElement).closest('button, input')) {
      resetActivity();
      return;
    }

    const now = Date.now();
    const side = event.clientX < window.innerWidth / 2 ? 'left' : 'right';
    const last = lastTapRef.current;
    lastTapRef.current = { at: now, side };
    if (last && now - last.at < DOUBLE_TAP_MS && last.side === side) {
      lastTapRef.current = null;
      const delta = side === 'left' ? -10 : 10;
      seek(delta);
      setSeekFlash({ side, at: now });
      window.setTimeout(() => setSeekFlash((current) => (current?.at === now ? null : current)), 550);
      return;
    }

    if (controlsVisible) {
      if (hideTimerRef.current) clearTimeout(hideTimerRef.current);
      setControlsVisible(false);
    } else {
      resetActivity();
    }
  };

  const toggleFullscreen = () => {
    const root = videoRef.current?.parentElement;
    if (!root) return;
    if (document.fullscreenElement) void exitFullscreen();
    else void enterFullscreen(root, isTouch);
    resetActivity();
  };

  const togglePictureInPicture = () => {
    const video = videoRef.current;
    if (!video) return;
    if (document.pictureInPictureElement) void document.exitPictureInPicture().catch(() => undefined);
    else void video.requestPictureInPicture().catch(() => undefined);
    resetActivity();
  };

  return (
    <div
      onMouseMove={resetActivity}
      onClick={handleSurfaceClick}
      style={{ position: 'fixed', inset: 0, zIndex: 9998, background: '#000', display: 'flex', flexDirection: 'column' }}
    >
      <PlayerOverlayStyles />
      <video
        ref={videoRef}
        title={title}
        controls={false}
        playsInline
        onLoadedMetadata={(event) => {
          const video = event.currentTarget;
          setDuration(video.duration);
          recordSnapshot(video);
          if (resumeAt && resumeAt > 0 && Number.isFinite(video.duration) && resumeAt < video.duration - 5) {
            video.currentTime = resumeAt;
          }
        }}
        onTimeUpdate={(event) => {
          setCurrentTime(event.currentTarget.currentTime);
          recordSnapshot(event.currentTarget);
        }}
        onPlay={() => {
          setPaused(false);
          resetActivity();
          watchTogetherRef.current?.notifyLocalPlayback();
          onPlaybackEvent?.('start');
        }}
        onPause={() => {
          setPaused(true);
          setControlsVisible(true);
          watchTogetherRef.current?.notifyLocalPlayback();
          onPlaybackEvent?.('pause');
        }}
        onEnded={() => onPlaybackEvent?.('stop')}
        onVolumeChange={(event) => {
          setMuted(event.currentTarget.muted);
          setVolume(event.currentTarget.volume);
        }}
        onPlaying={onFirstFrame}
        onError={() => {
          const video = videoRef.current;
          if (!video) return;
          if (fallbackUsedRef.current || url.includes('/transcode?')) {
            onPlaybackError?.(t('player.playback_error_detail'));
            return;
          }
          fallbackUsedRef.current = true;
          video.src = transcodeUrl(url);
          video.load();
          void video.play().catch(() => setPaused(true));
        }}
        style={{ width: '100%', height: '100%', flex: 1, objectFit: 'contain', minHeight: 0 }}
      />
      <canvas ref={subtitleCanvasRef} style={{ position: 'absolute', inset: 0, width: '100%', height: '100%', pointerEvents: 'none' }} />
      {seekFlash && (
        <div
          style={{
            position: 'absolute',
            top: '50%',
            [seekFlash.side]: '15%',
            transform: 'translateY(-50%)',
            display: 'flex',
            alignItems: 'center',
            gap: '0.375rem',
            padding: '0.75rem 1rem',
            borderRadius: '62.4375rem',
            background: 'rgba(0,0,0,0.55)',
            color: '#fff',
            fontSize: '0.875rem',
            fontWeight: 800,
            pointerEvents: 'none',
            animation: 'fadeIn 0.12s ease-out',
          }}
        >
          {seekFlash.side === 'left' ? <RotateCcw size={18} /> : <RotateCw size={18} />}
          10s
        </div>
      )}
      {(activeSegment || (nextCountdown !== null && nextEpisode)) && (
        <div
          style={{
            position: 'absolute',
            right: '1.25rem',
            bottom: '5.5rem',
            display: 'flex',
            flexDirection: 'column',
            alignItems: 'flex-end',
            gap: '0.5rem',
            zIndex: 2,
          }}
        >
          {activeSegment && (
            <button type="button" onClick={skipActiveSegment} style={S.skipButton}>
              {skipLabelForType(activeSegment.type)}
            </button>
          )}
          {nextCountdown !== null && nextEpisode && onPlayNextEpisode && (
            <button type="button" onClick={onPlayNextEpisode} style={S.skipButton}>
              {t('player.playing_in_seconds', String(nextCountdown))}
            </button>
          )}
        </div>
      )}
      <div
        style={{
          position: 'absolute',
          inset: 0,
          display: 'flex',
          flexDirection: 'column',
          pointerEvents: 'none',
          opacity: controlsVisible ? 1 : 0,
          transition: 'opacity 0.25s ease',
        }}
      >
        <div
          className="web-player-topbar"
          style={{
            display: 'flex',
            alignItems: 'center',
            padding: '0.875rem 0.75rem',
            background: 'linear-gradient(rgba(0,0,0,0.7), transparent)',
            pointerEvents: 'auto',
          }}
        >
          <button
            type="button"
            onClick={() => {
              void onClose();
            }}
            style={{ ...iconButton, background: 'rgba(255,255,255,0.1)' }}
            title={t('player.back')}
          >
            <ChevronLeft size={22} />
          </button>
          <div
            style={{
              color: '#fff',
              fontSize: '0.9375rem',
              fontWeight: 700,
              marginLeft: '0.75rem',
              overflow: 'hidden',
              textOverflow: 'ellipsis',
              whiteSpace: 'nowrap',
            }}
          >
            {title ?? ''}
          </div>
          <div style={{ flex: 1 }} />
          {onHandoff && externalTargets.length > 0 && (
            <div style={{ position: 'relative' }}>
              <button
                type="button"
                onClick={() => setHandoffOpen((value) => !value)}
                style={{
                  ...iconButton,
                  width: 'auto',
                  padding: '0 0.75rem',
                  background: 'rgba(255,255,255,0.1)',
                  fontSize: '0.75rem',
                  fontWeight: 700,
                }}
                title={t('external.open_in_app')}
              >
                <ExternalLink size={16} />
              </button>
              {handoffOpen && (
                <div
                  style={{
                    position: 'absolute',
                    right: 0,
                    top: '3rem',
                    minWidth: '10rem',
                    padding: '0.35rem',
                    borderRadius: '0.55rem',
                    background: 'rgba(20,20,20,0.98)',
                    boxShadow: '0 0.5rem 2rem rgba(0,0,0,0.45)',
                  }}
                >
                  {externalTargets.map((target) => (
                    <button
                      key={target.id}
                      type="button"
                      onClick={() => {
                        setHandoffOpen(false);
                        videoRef.current?.pause();
                        onHandoff(target.id, Math.floor(videoRef.current?.currentTime ?? 0));
                        window.location.href = target.href;
                      }}
                      style={{
                        display: 'block',
                        width: '100%',
                        padding: '0.55rem 0.7rem',
                        border: 0,
                        borderRadius: '0.35rem',
                        textAlign: 'left',
                        color: '#fff',
                        background: 'transparent',
                        cursor: 'pointer',
                      }}
                    >
                      {target.label}
                    </button>
                  ))}
                </div>
              )}
            </div>
          )}
          {contentId && (
            <button
              type="button"
              onClick={() => {
                void openWatchTogether();
              }}
              style={{
                ...iconButton,
                width: 'auto',
                padding: '0 0.75rem',
                background: watchTogetherState.roomCode ? 'var(--primary-accent-color)' : 'rgba(255,255,255,0.1)',
                fontSize: '0.75rem',
                fontWeight: 700,
              }}
              title={t('player.watch_party')}
            >
              {watchTogetherState.roomCode ? `${t('player.watch_party_room')} ${watchTogetherState.roomCode}` : t('player.watch_party')}
            </button>
          )}
        </div>
        <div style={{ flex: 1 }} />
        <div
          className="web-player-controls"
          style={{ pointerEvents: 'auto', padding: '0 0.75rem 0.875rem', background: 'linear-gradient(transparent, rgba(0,0,0,0.8))' }}
        >
          <input
            className="web-player-seek"
            aria-label={t('player.seek')}
            type="range"
            min={0}
            max={duration || 0}
            step={0.1}
            value={Math.min(currentTime, duration || 0)}
            onChange={(event) => {
              if (videoRef.current) {
                videoRef.current.currentTime = Number(event.target.value);
                watchTogetherRef.current?.notifyLocalPlayback();
              }
            }}
            style={{ width: '100%', accentColor: 'var(--primary-accent-color)' }}
          />
          <div style={{ display: 'flex', alignItems: 'center', gap: '0.125rem' }}>
            <button type="button" onClick={togglePause} style={iconButton} title={paused ? t('player.play') : t('player.pause')}>
              {paused ? <Play size={25} fill="currentColor" strokeWidth={0} /> : <Pause size={25} fill="currentColor" strokeWidth={0} />}
            </button>
            <button type="button" onClick={() => seek(-10)} style={iconButton} title={t('player.seek_back')}>
              <RotateCcw size={21} />
            </button>
            <button type="button" onClick={() => seek(10)} style={iconButton} title={t('player.seek_forward')}>
              <RotateCw size={21} />
            </button>
            <button
              type="button"
              className="web-player-volume"
              onClick={toggleMute}
              style={iconButton}
              title={muted ? t('player.unmute') : t('player.mute')}
            >
              {muted ? <VolumeX size={21} /> : <Volume2 size={21} />}
            </button>
            <input
              className="web-player-volume"
              aria-label={t('player.volume')}
              type="range"
              min={0}
              max={1}
              step={0.01}
              value={muted ? 0 : volume}
              onChange={(event) => setVideoVolume(Number(event.target.value))}
              style={{ width: '5rem', accentColor: 'var(--primary-accent-color)' }}
            />
            <span style={{ color: 'rgba(255,255,255,0.85)', fontSize: '0.8rem', fontVariantNumeric: 'tabular-nums', marginLeft: '0.4rem' }}>
              {formatTime(currentTime)} / {formatTime(duration)}
            </span>
            <div style={{ flex: 1 }} />
            {subtitles.length > 0 && (
              <div style={{ position: 'relative' }}>
                <button
                  type="button"
                  onClick={() => setSubtitleMenuOpen((value) => !value)}
                  style={{ ...iconButton, color: selectedSubtitle >= 0 ? 'var(--primary-accent-color)' : '#fff' }}
                  title={t('player.subtitles')}
                >
                  <Captions size={21} />
                </button>
                {subtitleMenuOpen && (
                  <div
                    style={{
                      position: 'absolute',
                      right: 0,
                      bottom: '3rem',
                      minWidth: '11rem',
                      padding: '0.35rem',
                      borderRadius: '0.55rem',
                      background: 'rgba(20,20,20,0.98)',
                      boxShadow: '0 0.5rem 2rem rgba(0,0,0,0.45)',
                    }}
                  >
                    <button
                      type="button"
                      onClick={() => {
                        setSelectedSubtitle(-1);
                        setSubtitleMenuOpen(false);
                      }}
                      style={{
                        display: 'block',
                        width: '100%',
                        padding: '0.55rem 0.7rem',
                        border: 0,
                        borderRadius: '0.35rem',
                        textAlign: 'left',
                        color: '#fff',
                        background: selectedSubtitle < 0 ? 'rgba(255,255,255,0.14)' : 'transparent',
                        cursor: 'pointer',
                      }}
                    >
                      {t('player.subtitles_off')}
                    </button>
                    {subtitles.map((subtitle, index) => (
                      <button
                        key={`${subtitle.url}-menu-${index}`}
                        type="button"
                        onClick={() => {
                          setSelectedSubtitle(index);
                          setSubtitleMenuOpen(false);
                        }}
                        style={{
                          display: 'block',
                          width: '100%',
                          padding: '0.55rem 0.7rem',
                          border: 0,
                          borderRadius: '0.35rem',
                          textAlign: 'left',
                          color: '#fff',
                          background: selectedSubtitle === index ? 'rgba(255,255,255,0.14)' : 'transparent',
                          cursor: 'pointer',
                        }}
                      >
                        {subtitle.label ?? subtitle.lang ?? t('player.subtitles')}
                      </button>
                    ))}
                  </div>
                )}
              </div>
            )}
            {nextEpisode && onPlayNextEpisode && (
              <button
                type="button"
                onClick={onPlayNextEpisode}
                style={{ ...iconButton, width: 'auto', padding: '0 0.7rem', fontSize: '0.75rem', fontWeight: 700 }}
                title={t('auto.next_episode')}
              >
                {t('auto.next_episode')}
              </button>
            )}
            {pipSupported && (
              <button type="button" onClick={togglePictureInPicture} style={iconButton} title={t('player.picture_in_picture')}>
                <PictureInPicture2 size={21} />
              </button>
            )}
            <button type="button" onClick={toggleFullscreen} style={iconButton} title={t('player.fullscreen')}>
              <Maximize size={21} />
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
