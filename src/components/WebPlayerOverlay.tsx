import { useCallback, useEffect, useRef, useState } from 'react';
import { ChevronLeft, Maximize, Pause, Play, RotateCcw, RotateCw, Volume2, VolumeX } from 'lucide-react';
import { t } from '../i18n';
import { transcodeUrl } from '../platform/web/stream';
import { PlayerOverlayStyles } from './player/PlayerOverlayStyles';

interface Props {
  url: string;
  title?: string;
  onClose: () => Promise<void>;
  onFirstFrame: () => void;
}

function formatTime(value: number) {
  if (!Number.isFinite(value) || value < 0) return '0:00';
  const seconds = Math.floor(value);
  const hours = Math.floor(seconds / 3600);
  const minutes = Math.floor((seconds % 3600) / 60);
  const remainder = seconds % 60;
  return hours > 0 ? `${hours}:${String(minutes).padStart(2, '0')}:${String(remainder).padStart(2, '0')}` : `${minutes}:${String(remainder).padStart(2, '0')}`;
}

const iconButton = { width: '2.75rem', height: '2.75rem', border: 'none', borderRadius: '0.5rem', background: 'none', color: '#fff', display: 'inline-flex', alignItems: 'center', justifyContent: 'center', cursor: 'pointer' } as const;

export function WebPlayerOverlay({ url, title, onClose, onFirstFrame }: Props) {
  const videoRef = useRef<HTMLVideoElement>(null);
  const fallbackUsedRef = useRef(false);
  const hideTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const [paused, setPaused] = useState(false);
  const [muted, setMuted] = useState(false);
  const [volume, setVolume] = useState(1);
  const [currentTime, setCurrentTime] = useState(0);
  const [duration, setDuration] = useState(0);
  const [controlsVisible, setControlsVisible] = useState(true);

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
    video.src = url;
    video.load();
    void video.play().catch(() => setPaused(true));
    return () => {
      if (hideTimerRef.current) clearTimeout(hideTimerRef.current);
      video.pause();
      video.removeAttribute('src');
    };
  }, [url]);

  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === 'Escape') { void onClose(); return; }
      if (event.key === ' ') { event.preventDefault(); videoRef.current?.paused ? videoRef.current.play() : videoRef.current?.pause(); }
      if (event.key === 'ArrowLeft') { event.preventDefault(); if (videoRef.current) videoRef.current.currentTime = Math.max(0, videoRef.current.currentTime - 10); }
      if (event.key === 'ArrowRight') { event.preventDefault(); if (videoRef.current) videoRef.current.currentTime = Math.min(videoRef.current.duration || Infinity, videoRef.current.currentTime + 10); }
      resetActivity();
    };
    window.addEventListener('keydown', onKeyDown);
    return () => window.removeEventListener('keydown', onKeyDown);
  }, [onClose, resetActivity]);

  const togglePause = () => {
    const video = videoRef.current;
    if (!video) return;
    if (video.paused) void video.play(); else video.pause();
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

  const toggleFullscreen = () => {
    const root = videoRef.current?.parentElement;
    if (!root) return;
    if (document.fullscreenElement) void document.exitFullscreen();
    else void root.requestFullscreen?.();
    resetActivity();
  };

  return (
    <div onMouseMove={resetActivity} onClick={resetActivity} style={{ position: 'fixed', inset: 0, zIndex: 9998, background: '#000', display: 'flex', flexDirection: 'column' }}>
      <PlayerOverlayStyles />
      <video
        ref={videoRef}
        title={title}
        controls={false}
        playsInline
        onLoadedMetadata={(event) => setDuration(event.currentTarget.duration)}
        onTimeUpdate={(event) => setCurrentTime(event.currentTarget.currentTime)}
        onPlay={() => { setPaused(false); resetActivity(); }}
        onPause={() => { setPaused(true); setControlsVisible(true); }}
        onVolumeChange={(event) => { setMuted(event.currentTarget.muted); setVolume(event.currentTarget.volume); }}
        onPlaying={onFirstFrame}
        onError={() => {
          const video = videoRef.current;
          if (!video || fallbackUsedRef.current || url.includes('/transcode?')) return;
          fallbackUsedRef.current = true;
          video.src = transcodeUrl(url);
          video.load();
          void video.play().catch(() => setPaused(true));
        }}
        style={{ width: '100%', height: '100%', flex: 1, objectFit: 'contain', minHeight: 0 }}
      />
      <div style={{ position: 'absolute', inset: 0, display: 'flex', flexDirection: 'column', pointerEvents: 'none', opacity: controlsVisible ? 1 : 0, transition: 'opacity 0.25s ease' }}>
        <div style={{ display: 'flex', alignItems: 'center', padding: '0.875rem 0.75rem', background: 'linear-gradient(rgba(0,0,0,0.7), transparent)', pointerEvents: 'auto' }}>
          <button type="button" onClick={() => { void onClose(); }} style={{ ...iconButton, background: 'rgba(255,255,255,0.1)' }} title={t('player.back')}><ChevronLeft size={22} /></button>
          <div style={{ color: '#fff', fontSize: '0.9375rem', fontWeight: 700, marginLeft: '0.75rem', overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>{title ?? ''}</div>
        </div>
        <div style={{ flex: 1 }} />
        <div style={{ pointerEvents: 'auto', padding: '0 0.75rem 0.875rem', background: 'linear-gradient(transparent, rgba(0,0,0,0.8))' }}>
          <input aria-label={t('player.seek')} type="range" min={0} max={duration || 0} step={0.1} value={Math.min(currentTime, duration || 0)} onChange={(event) => { if (videoRef.current) videoRef.current.currentTime = Number(event.target.value); }} style={{ width: '100%', accentColor: 'var(--primary-accent-color)' }} />
          <div style={{ display: 'flex', alignItems: 'center', gap: '0.125rem' }}>
            <button type="button" onClick={togglePause} style={iconButton} title={paused ? t('player.play') : t('player.pause')}>{paused ? <Play size={25} fill="currentColor" strokeWidth={0} /> : <Pause size={25} fill="currentColor" strokeWidth={0} />}</button>
            <button type="button" onClick={() => seek(-10)} style={iconButton} title={t('player.seek_back')}><RotateCcw size={21} /></button>
            <button type="button" onClick={() => seek(10)} style={iconButton} title={t('player.seek_forward')}><RotateCw size={21} /></button>
            <button type="button" onClick={toggleMute} style={iconButton} title={muted ? t('player.unmute') : t('player.mute')}>{muted ? <VolumeX size={21} /> : <Volume2 size={21} />}</button>
            <input aria-label={t('player.volume')} type="range" min={0} max={1} step={0.01} value={muted ? 0 : volume} onChange={(event) => setVideoVolume(Number(event.target.value))} style={{ width: '5rem', accentColor: 'var(--primary-accent-color)' }} />
            <span style={{ color: 'rgba(255,255,255,0.85)', fontSize: '0.8rem', fontVariantNumeric: 'tabular-nums', marginLeft: '0.4rem' }}>{formatTime(currentTime)} / {formatTime(duration)}</span>
            <div style={{ flex: 1 }} />
            <button type="button" onClick={toggleFullscreen} style={iconButton} title={t('player.fullscreen')}><Maximize size={21} /></button>
          </div>
        </div>
      </div>
    </div>
  );
}
