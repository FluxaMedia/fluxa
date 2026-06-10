import { useCallback, useEffect, useRef, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen, emit } from '@tauri-apps/api/event';
import type { EmbeddedMpvStatus } from '../core/mpvPlayer';
import { playerGetPlaybackInfo, playerGetTrackOptions } from '../core/mpvPlayer';
import type { PlayerTrackOption } from '../core/mpvPlayer';

// ─── Types ────────────────────────────────────────────────────────────────────

type Chapter = { title: string; startMs: number };
type SkipSegment = { type: string; startTime: number; endTime: number };
type EpisodeInfo = {
  id: string;
  name?: string;
  title?: string;
  season?: number;
  episode?: number;
  number?: number;
  thumbnail?: string;
};
type ActiveSkip = { label: string; endMs: number };
type FeedbackFlash = { icon: 'play' | 'pause' | 'seekBack' | 'seekFwd' | 'speed'; label: string };

// ─── Helpers ──────────────────────────────────────────────────────────────────

function fmtTime(s: number): string {
  if (!isFinite(s) || s < 0) return '0:00';
  const h = Math.floor(s / 3600);
  const m = Math.floor((s % 3600) / 60);
  const sec = Math.floor(s % 60);
  if (h > 0) return `${h}:${String(m).padStart(2, '0')}:${String(sec).padStart(2, '0')}`;
  return `${m}:${String(sec).padStart(2, '0')}`;
}

function sendCmd(command: string) {
  invoke('player_command', { command }).catch(() => undefined);
}

function parseChapters(json: string | null | undefined): Chapter[] {
  if (!json) return [];
  try {
    const arr = JSON.parse(json) as Array<{ title?: string; startTime?: number }>;
    return arr.map((c) => ({ title: c.title ?? '', startMs: c.startTime ?? 0 }));
  } catch { return []; }
}

function parseSegments(json: string | null | undefined): SkipSegment[] {
  if (!json) return [];
  try { return JSON.parse(json) as SkipSegment[]; } catch { return []; }
}

function parseEpisodes(json: string | null | undefined): EpisodeInfo[] {
  if (!json) return [];
  try { return JSON.parse(json) as EpisodeInfo[]; } catch { return []; }
}

function skipLabelForType(type: string): string {
  switch (type) {
    case 'intro': return 'Skip Intro';
    case 'outro': return 'Skip Outro';
    case 'recap': return 'Skip Recap';
    case 'preview': return 'Skip Preview';
    default: return 'Skip';
  }
}

function epLabel(ep: EpisodeInfo): string {
  const s = ep.season != null ? `S${ep.season}` : '';
  const e = (ep.episode ?? ep.number) != null ? `E${ep.episode ?? ep.number}` : '';
  const se = s || e ? `${s}${e} — ` : '';
  return se + (ep.name ?? ep.title ?? '');
}

// ─── Icons (inline SVG) ───────────────────────────────────────────────────────

function IconBack() {
  return (
    <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round">
      <polyline points="15 18 9 12 15 6" />
    </svg>
  );
}
function IconPlay() {
  return (
    <svg width="22" height="22" viewBox="0 0 24 24" fill="currentColor">
      <polygon points="5,3 19,12 5,21" />
    </svg>
  );
}
function IconPause() {
  return (
    <svg width="22" height="22" viewBox="0 0 24 24" fill="currentColor">
      <rect x="5" y="3" width="4" height="18" rx="1" />
      <rect x="15" y="3" width="4" height="18" rx="1" />
    </svg>
  );
}
function IconSeekBack() {
  return (
    <svg width="22" height="22" viewBox="0 0 24 24" fill="currentColor">
      <path d="M12.5 8c-2.65 0-5.05 1.01-6.85 2.65L3 8v7h7l-2.77-2.77c1.3-1.22 3.05-1.98 4.99-1.98 3.36 0 6.18 2.13 7.21 5.11l1.9-.63C19.94 11.33 16.52 8 12.5 8z" />
      <text x="12" y="20" textAnchor="middle" fontSize="5.5" fill="currentColor" fontWeight="bold">10</text>
    </svg>
  );
}
function IconSeekFwd() {
  return (
    <svg width="22" height="22" viewBox="0 0 24 24" fill="currentColor">
      <path d="M11.5 8c2.65 0 5.05 1.01 6.85 2.65L21 8v7h-7l2.77-2.77C15.47 11.01 13.72 10.25 11.78 10.25c-3.36 0-6.18 2.13-7.21 5.11L2.67 14.73C4.06 11.33 7.48 8 11.5 8z" />
      <text x="12" y="20" textAnchor="middle" fontSize="5.5" fill="currentColor" fontWeight="bold">10</text>
    </svg>
  );
}
function IconVolume({ muted, level }: { muted: boolean; level: number }) {
  if (muted || level === 0) {
    return (
      <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
        <path d="M16.5 12c0-1.77-1.02-3.29-2.5-4.03v2.21l2.45 2.45c.03-.2.05-.41.05-.63zm2.5 0c0 .94-.2 1.82-.54 2.64l1.51 1.51C20.63 14.91 21 13.5 21 12c0-4.28-2.99-7.86-7-8.77v2.06c2.89.86 5 3.54 5 6.71zM4.27 3L3 4.27 7.73 9H3v6h4l5 5v-6.73l4.25 4.25c-.67.52-1.42.93-2.25 1.18v2.06c1.38-.31 2.63-.95 3.69-1.81L19.73 21 21 19.73l-9-9L4.27 3zM12 4L9.91 6.09 12 8.18V4z" />
      </svg>
    );
  }
  if (level < 50) {
    return (
      <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
        <path d="M18.5 12c0-1.77-1.02-3.29-2.5-4.03v8.05c1.48-.73 2.5-2.25 2.5-4.02zM5 9v6h4l5 5V4L9 9H5z" />
      </svg>
    );
  }
  return (
    <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
      <path d="M3 9v6h4l5 5V4L7 9H3zm13.5 3c0-1.77-1.02-3.29-2.5-4.03v8.05c1.48-.73 2.5-2.25 2.5-4.02zM14 3.23v2.06c2.89.86 5 3.54 5 6.71s-2.11 5.85-5 6.71v2.06c4.01-.91 7-4.49 7-8.77s-2.99-7.86-7-8.77z" />
    </svg>
  );
}
function IconFullscreen() {
  return (
    <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
      <path d="M7 14H5v5h5v-2H7v-3zm-2-4h2V7h3V5H5v5zm12 7h-3v2h5v-5h-2v3zM14 5v2h3v3h2V5h-5z" />
    </svg>
  );
}
function IconEpisodes() {
  return (
    <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
      <path d="M3 5h2V3c-1.1 0-2 .9-2 2zm0 8h2v-2H3v2zm4 8h2v-2H7v2zM3 9h2V7H3v2zm10-6h-2v2h2V3zm6 0v2h2c0-1.1-.9-2-2-2zM5 21v-2H3c0 1.1.9 2 2 2zm-2-4h2v-2H3v2zM9 3H7v2h2V3zm2 18h2v-2h-2v2zm8-8h2v-2h-2v2zm0 8c1.1 0 2-.9 2-2h-2v2zm0-12h2V7h-2v2zm0 8h2v-2h-2v2zm-4 4h2v-2h-2v2zm0-16h2V3h-2v2z" />
      <rect x="7" y="7" width="10" height="10" rx="1" fillOpacity="0.25" />
      <polygon points="10,9.5 10,14.5 15,12" />
    </svg>
  );
}
function IconSubtitles() {
  return (
    <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor">
      <path d="M20 4H4c-1.1 0-2 .9-2 2v12c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V6c0-1.1-.9-2-2-2zm-8 11H4v-2h8v2zm8 0h-6v-2h6v2zm0-4H4V9h16v2z" />
    </svg>
  );
}
function IconAudio() {
  return (
    <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor">
      <path d="M12 3v9.28c-.47-.17-.97-.28-1.5-.28C8.01 12 6 14.01 6 16.5S8.01 21 10.5 21c2.31 0 4.2-1.75 4.45-4H15V6h4V3h-7z" />
    </svg>
  );
}
function IconCheck() {
  return (
    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round">
      <polyline points="20 6 9 17 4 12" />
    </svg>
  );
}

// ─── Main component ───────────────────────────────────────────────────────────

interface Props {
  closePlayer: () => Promise<void>;
}

export function ReactPlayerOverlay({ closePlayer }: Props) {
  // Structural state (causes re-render)
  const [paused, setPaused] = useState(false);
  const [muted, setMuted] = useState(false);
  const [volumeLevel, setVolumeLevel] = useState(100);
  const [controlsVisible, setControlsVisible] = useState(true);
  const [title, setTitle] = useState('');
  const [episodeTitle, setEpisodeTitle] = useState('');
  const [chapters, setChapters] = useState<Chapter[]>([]);
  const [skipSegments, setSkipSegments] = useState<SkipSegment[]>([]);
  const [nextEpSubtitle, setNextEpSubtitle] = useState('');
  const [nextEpThreshold, setNextEpThreshold] = useState(85);
  const [episodes, setEpisodes] = useState<EpisodeInfo[]>([]);
  const [showEpisodePanel, setShowEpisodePanel] = useState(false);
  const [activeSkip, setActiveSkip] = useState<ActiveSkip | null>(null);
  const [showNextEpCard, setShowNextEpCard] = useState(false);
  const [trackPopover, setTrackPopover] = useState<'audio' | 'sub' | null>(null);
  const [audioTracks, setAudioTracks] = useState<PlayerTrackOption[]>([]);
  const [subTracks, setSubTracks] = useState<PlayerTrackOption[]>([]);
  const [feedback, setFeedback] = useState<FeedbackFlash | null>(null);
  const [seekPreview, setSeekPreview] = useState<{ x: number; time: number } | null>(null);
  const [isDragging, setIsDragging] = useState(false);

  // Refs for direct DOM mutation (high-frequency: seekbar, time labels)
  const seekFillRef = useRef<HTMLDivElement>(null);
  const seekBufferRef = useRef<HTMLDivElement>(null);
  const seekDotRef = useRef<HTMLDivElement>(null);
  const currentTimeRef = useRef<HTMLSpanElement>(null);
  const durationRef = useRef<HTMLSpanElement>(null);
  const seekbarRef = useRef<HTMLDivElement>(null);
  const overlayRef = useRef<HTMLDivElement>(null);

  // Internal refs (no re-render needed)
  const posRef = useRef(0);
  const durRef = useRef(0);
  const pausedRef = useRef(false);
  const lastActivityRef = useRef(Date.now());
  const isDraggingRef = useRef(false);
  const dragPosRef = useRef(0);
  const feedbackTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const holdTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const holdActiveRef = useRef(false);
  const preSpeedRef = useRef(1.0);
  const controlsVisibleRef = useRef(true);
  const episodePanelOpenRef = useRef(false);

  // ─── Activity tracking ───────────────────────────────────────────────────────

  const resetActivity = useCallback(() => {
    lastActivityRef.current = Date.now();
    if (!controlsVisibleRef.current) {
      controlsVisibleRef.current = true;
      setControlsVisible(true);
      if (overlayRef.current) overlayRef.current.style.cursor = '';
    }
  }, []);

  // ─── Feedback flash ──────────────────────────────────────────────────────────

  const flashFeedback = useCallback((icon: FeedbackFlash['icon'], label: string) => {
    setFeedback({ icon, label });
    if (feedbackTimerRef.current) clearTimeout(feedbackTimerRef.current);
    feedbackTimerRef.current = setTimeout(() => setFeedback(null), 700);
  }, []);

  // ─── Seek helpers ────────────────────────────────────────────────────────────

  const seekToFraction = useCallback((fraction: number) => {
    const t = fraction * durRef.current;
    sendCmd(`set time-pos ${Math.floor(t)}`);
  }, []);

  const fractionFromSeekbarEvent = useCallback((clientX: number): number => {
    const bar = seekbarRef.current;
    if (!bar) return 0;
    const rect = bar.getBoundingClientRect();
    return Math.max(0, Math.min(1, (clientX - rect.left) / rect.width));
  }, []);

  // ─── Polling: player status (every 250 ms) ───────────────────────────────────

  useEffect(() => {
    const interval = setInterval(async () => {
      let status: EmbeddedMpvStatus | null = null;
      try { status = await invoke<EmbeddedMpvStatus>('player_status'); } catch { return; }
      if (!status) return;

      const pos = parseFloat(status.timePos ?? '0');
      const dur = parseFloat(status.duration ?? '0');
      const isPaused = status.pause === 'yes';
      const isMuted = (status as Record<string, unknown>).mute === 'yes';
      const vol = parseFloat((status as Record<string, unknown>).volume as string ?? '100');
      const buffered = parseFloat(status.demuxerCacheDuration ?? '0');

      posRef.current = pos;
      durRef.current = dur;
      pausedRef.current = isPaused;

      // Direct DOM mutation for time/seekbar — no re-render
      if (currentTimeRef.current) currentTimeRef.current.textContent = fmtTime(pos);
      if (durationRef.current) durationRef.current.textContent = fmtTime(dur);

      const fraction = dur > 0 ? pos / dur : 0;
      const bufFraction = dur > 0 ? Math.min(1, (pos + buffered) / dur) : 0;
      const pct = `${(fraction * 100).toFixed(3)}%`;
      const bufPct = `${(bufFraction * 100).toFixed(3)}%`;

      if (!isDraggingRef.current) {
        if (seekFillRef.current) seekFillRef.current.style.width = pct;
        if (seekBufferRef.current) seekBufferRef.current.style.width = bufPct;
        if (seekDotRef.current) seekDotRef.current.style.left = pct;
      }

      // Update paused state if changed
      setPaused((prev) => (prev !== isPaused ? isPaused : prev));
      setMuted((prev) => (prev !== isMuted ? isMuted : prev));
      setVolumeLevel((prev) => {
        const rounded = Math.round(vol);
        return prev !== rounded ? rounded : prev;
      });

      // Auto-hide controls check
      const idle = Date.now() - lastActivityRef.current;
      if (idle > 3000 && !isPaused && !episodePanelOpenRef.current && trackPopover === null) {
        if (controlsVisibleRef.current) {
          controlsVisibleRef.current = false;
          setControlsVisible(false);
          if (overlayRef.current) overlayRef.current.style.cursor = 'none';
        }
      }

      // Skip segment check
      const posMs = pos * 1000;
      const seg = skipSegments.find((s) => posMs >= s.startTime && posMs < s.endTime);
      if (seg) {
        setActiveSkip({ label: skipLabelForType(seg.type), endMs: seg.endTime });
      } else {
        setActiveSkip(null);
      }

      // Next episode card
      if (dur > 0 && nextEpSubtitle) {
        const progress = (pos / dur) * 100;
        setShowNextEpCard(progress >= nextEpThreshold);
      } else {
        setShowNextEpCard(false);
      }
    }, 250);

    return () => clearInterval(interval);
  }, [skipSegments, nextEpSubtitle, nextEpThreshold, trackPopover]);

  // ─── Polling: playback info (every 2 s, cheap) ───────────────────────────────

  useEffect(() => {
    const poll = async () => {
      try {
        const info = await playerGetPlaybackInfo();
        setChapters(parseChapters(info.chaptersJson));
        setSkipSegments(parseSegments(info.skipSegmentsJson));
        setNextEpSubtitle(info.nextEpSubtitle ?? '');
        setNextEpThreshold(info.nextEpThresholdPercent ?? 85);
        setEpisodes(parseEpisodes(info.episodesJson));
      } catch { /* renderer may not be ready yet */ }
    };
    poll();
    const interval = setInterval(poll, 2000);
    return () => clearInterval(interval);
  }, []);

  // ─── Listen for title updates ────────────────────────────────────────────────

  useEffect(() => {
    let cancelled = false;
    listen<{ title?: string; episodeTitle?: string }>('native-player-title', (ev) => {
      if (cancelled) return;
      setTitle(ev.payload.title ?? '');
      setEpisodeTitle(ev.payload.episodeTitle ?? '');
    }).catch(() => undefined);
    return () => { cancelled = true; };
  }, []);

  // ─── Keyboard shortcuts ──────────────────────────────────────────────────────

  useEffect(() => {
    const onKeyDown = (e: KeyboardEvent) => {
      resetActivity();
      switch (e.code) {
        case 'Space':
          e.preventDefault();
          if (holdTimerRef.current) return;
          holdActiveRef.current = false;
          {
            const spd = parseFloat(String((window as unknown as Record<string, unknown>).__fluxaSpeed ?? '1')) || 1;
            preSpeedRef.current = spd;
          }
          holdTimerRef.current = setTimeout(() => {
            holdActiveRef.current = true;
            sendCmd('set speed 2.00');
            flashFeedback('speed', '2×');
          }, 300);
          break;
        case 'ArrowLeft':
          e.preventDefault();
          flashFeedback('seekBack', '-10s');
          sendCmd('seek -10 relative');
          break;
        case 'ArrowRight':
          e.preventDefault();
          flashFeedback('seekFwd', '+10s');
          sendCmd('seek 10 relative');
          break;
        case 'ArrowUp':
          e.preventDefault();
          sendCmd('add volume 5');
          break;
        case 'ArrowDown':
          e.preventDefault();
          sendCmd('add volume -5');
          break;
        case 'KeyM':
          e.preventDefault();
          sendCmd('cycle mute');
          break;
        case 'KeyF':
          e.preventDefault();
          sendCmd('cycle fullscreen');
          break;
        case 'Escape':
          e.preventDefault();
          if (showEpisodePanel) { setShowEpisodePanel(false); episodePanelOpenRef.current = false; return; }
          if (trackPopover) { setTrackPopover(null); return; }
          void closePlayer();
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
  }, [closePlayer, flashFeedback, resetActivity, showEpisodePanel, trackPopover]);

  // ─── Mouse activity tracking ─────────────────────────────────────────────────

  useEffect(() => {
    const onMove = () => resetActivity();
    window.addEventListener('mousemove', onMove);
    return () => window.removeEventListener('mousemove', onMove);
  }, [resetActivity]);

  // ─── Seekbar interaction ──────────────────────────────────────────────────────

  const onSeekMouseDown = useCallback((e: React.MouseEvent) => {
    if (e.button !== 0) return;
    e.preventDefault();
    resetActivity();
    isDraggingRef.current = true;
    setIsDragging(true);
    const frac = fractionFromSeekbarEvent(e.clientX);
    dragPosRef.current = frac;
    if (seekFillRef.current) seekFillRef.current.style.width = `${frac * 100}%`;
    if (seekDotRef.current) seekDotRef.current.style.left = `${frac * 100}%`;
  }, [fractionFromSeekbarEvent, resetActivity]);

  const onSeekMouseMove = useCallback((e: React.MouseEvent) => {
    const bar = seekbarRef.current;
    if (!bar) return;
    const rect = bar.getBoundingClientRect();
    const frac = Math.max(0, Math.min(1, (e.clientX - rect.left) / rect.width));
    const previewTime = frac * durRef.current;
    setSeekPreview({ x: e.clientX - rect.left, time: previewTime });

    if (isDraggingRef.current) {
      dragPosRef.current = frac;
      if (seekFillRef.current) seekFillRef.current.style.width = `${frac * 100}%`;
      if (seekDotRef.current) seekDotRef.current.style.left = `${frac * 100}%`;
    }
  }, []);

  const onSeekMouseLeave = useCallback(() => {
    setSeekPreview(null);
  }, []);

  useEffect(() => {
    const onMouseUp = () => {
      if (!isDraggingRef.current) return;
      isDraggingRef.current = false;
      setIsDragging(false);
      seekToFraction(dragPosRef.current);
    };
    const onMouseMove = (e: MouseEvent) => {
      if (!isDraggingRef.current) return;
      const bar = seekbarRef.current;
      if (!bar) return;
      const rect = bar.getBoundingClientRect();
      const frac = Math.max(0, Math.min(1, (e.clientX - rect.left) / rect.width));
      dragPosRef.current = frac;
      if (seekFillRef.current) seekFillRef.current.style.width = `${frac * 100}%`;
      if (seekDotRef.current) seekDotRef.current.style.left = `${frac * 100}%`;
    };
    window.addEventListener('mouseup', onMouseUp);
    window.addEventListener('mousemove', onMouseMove);
    return () => {
      window.removeEventListener('mouseup', onMouseUp);
      window.removeEventListener('mousemove', onMouseMove);
    };
  }, [seekToFraction]);

  // Click on seekbar without drag
  const onSeekClick = useCallback((e: React.MouseEvent) => {
    if (isDragging) return;
    resetActivity();
    seekToFraction(fractionFromSeekbarEvent(e.clientX));
  }, [isDragging, seekToFraction, fractionFromSeekbarEvent, resetActivity]);

  // ─── Track popover ────────────────────────────────────────────────────────────

  const openTrackPopover = useCallback(async (type: 'audio' | 'sub') => {
    resetActivity();
    if (trackPopover === type) { setTrackPopover(null); return; }
    try {
      if (type === 'audio') setAudioTracks(await playerGetTrackOptions('audio'));
      else setSubTracks(await playerGetTrackOptions('sub'));
    } catch { /* ignore */ }
    setTrackPopover(type);
  }, [trackPopover, resetActivity]);

  const selectTrack = useCallback((type: 'audio' | 'sub', id: string) => {
    sendCmd(type === 'audio' ? `set aid ${id}` : `set sid ${id}`);
    setTrackPopover(null);
    if (type === 'audio') setAudioTracks((prev) => prev.map((t) => ({ ...t, selected: t.id === id })));
    else setSubTracks((prev) => prev.map((t) => ({ ...t, selected: t.id === id })));
  }, []);

  // ─── Center click (pause/play) ────────────────────────────────────────────────

  const onCenterClick = useCallback(() => {
    resetActivity();
    if (showEpisodePanel) { setShowEpisodePanel(false); episodePanelOpenRef.current = false; return; }
    if (trackPopover) { setTrackPopover(null); return; }
    const icon = pausedRef.current ? 'play' : 'pause';
    flashFeedback(icon, '');
    sendCmd('cycle pause');
  }, [flashFeedback, resetActivity, showEpisodePanel, trackPopover]);

  // ─── Episode panel ────────────────────────────────────────────────────────────

  const seasonGroups = episodes.reduce<Record<number, EpisodeInfo[]>>((acc, ep) => {
    const s = ep.season ?? 1;
    if (!acc[s]) acc[s] = [];
    acc[s].push(ep);
    return acc;
  }, {});
  const seasons = Object.keys(seasonGroups).map(Number).sort((a, b) => a - b);
  const [activeSeason, setActiveSeason] = useState(1);

  // ─── Render ───────────────────────────────────────────────────────────────────

  const opacityStyle: React.CSSProperties = {
    opacity: controlsVisible ? 1 : 0,
    transition: 'opacity 0.4s ease',
    pointerEvents: controlsVisible ? 'auto' : 'none',
  };

  // Chapter-aware seekbar segments
  const dur = durRef.current;
  const chapterSegments = chapters.length >= 2 && dur > 0
    ? chapters.map((ch, i) => {
        const start = ch.startMs / 1000 / dur;
        const end = i + 1 < chapters.length ? chapters[i + 1].startMs / 1000 / dur : 1;
        return { start, end };
      })
    : null;

  return (
    <div
      ref={overlayRef}
      style={{
        position: 'fixed',
        inset: 0,
        zIndex: 9998,
        display: 'flex',
        flexDirection: 'column',
        background: 'transparent',
      }}
    >
      {/* ── Top bar ── */}
      <div
        style={{
          ...opacityStyle,
          position: 'absolute',
          top: 0,
          left: 0,
          right: 0,
          padding: '20px 20px 40px',
          background: 'linear-gradient(to bottom, rgba(0,0,0,0.75) 0%, transparent 100%)',
          display: 'flex',
          alignItems: 'center',
          gap: 12,
          zIndex: 2,
        }}
      >
        <button
          onClick={(e) => { e.stopPropagation(); resetActivity(); void closePlayer(); }}
          style={styles.iconBtn}
          title="Back"
        >
          <IconBack />
        </button>
        <div style={{ flex: 1, minWidth: 0 }}>
          {title && (
            <div style={{ color: '#fff', fontSize: 15, fontWeight: 700, letterSpacing: 0.1, whiteSpace: 'nowrap', overflow: 'hidden', textOverflow: 'ellipsis', textShadow: '0 1px 8px rgba(0,0,0,0.8)' }}>
              {title}
            </div>
          )}
          {episodeTitle && (
            <div style={{ color: 'rgba(255,255,255,0.65)', fontSize: 13, marginTop: 1, whiteSpace: 'nowrap', overflow: 'hidden', textOverflow: 'ellipsis' }}>
              {episodeTitle}
            </div>
          )}
        </div>
        <button
          onClick={(e) => { e.stopPropagation(); void openTrackPopover('sub'); }}
          style={{ ...styles.iconBtn, opacity: trackPopover === 'sub' ? 1 : 0.75 }}
          title="Subtitles"
        >
          <IconSubtitles />
        </button>
        <button
          onClick={(e) => { e.stopPropagation(); void openTrackPopover('audio'); }}
          style={{ ...styles.iconBtn, opacity: trackPopover === 'audio' ? 1 : 0.75 }}
          title="Audio"
        >
          <IconAudio />
        </button>
      </div>

      {/* ── Track popover ── */}
      {trackPopover && (
        <div
          style={{
            position: 'absolute',
            top: 68,
            right: 16,
            background: 'rgba(18,22,30,0.97)',
            backdropFilter: 'blur(16px)',
            border: '1px solid rgba(255,255,255,0.12)',
            borderRadius: 10,
            padding: '6px 0',
            minWidth: 200,
            maxHeight: 300,
            overflowY: 'auto',
            zIndex: 10,
            boxShadow: '0 8px 32px rgba(0,0,0,0.6)',
          }}
          onClick={(e) => e.stopPropagation()}
        >
          <div style={{ color: 'rgba(255,255,255,0.45)', fontSize: 11, fontWeight: 700, letterSpacing: 0.8, padding: '4px 14px 8px', textTransform: 'uppercase' }}>
            {trackPopover === 'audio' ? 'Audio' : 'Subtitles'}
          </div>
          {(trackPopover === 'audio' ? audioTracks : subTracks).map((t) => (
            <button
              key={t.id}
              onClick={() => selectTrack(trackPopover, t.id)}
              style={{
                display: 'flex',
                alignItems: 'center',
                gap: 10,
                width: '100%',
                background: 'none',
                border: 'none',
                color: t.selected ? '#fff' : 'rgba(255,255,255,0.7)',
                fontSize: 13,
                fontWeight: t.selected ? 600 : 400,
                padding: '8px 14px',
                cursor: 'pointer',
                textAlign: 'left',
              }}
            >
              <span style={{ width: 14, color: 'var(--primary-accent-color)' }}>
                {t.selected && <IconCheck />}
              </span>
              {t.label}
            </button>
          ))}
          {(trackPopover === 'audio' ? audioTracks : subTracks).length === 0 && (
            <div style={{ color: 'rgba(255,255,255,0.35)', fontSize: 13, padding: '8px 14px' }}>None available</div>
          )}
        </div>
      )}

      {/* ── Center clickable area ── */}
      <div
        style={{ flex: 1, cursor: 'default' }}
        onClick={onCenterClick}
      />

      {/* ── Feedback flash ── */}
      {feedback && (
        <div
          style={{
            position: 'absolute',
            top: '50%',
            left: '50%',
            transform: 'translate(-50%, -50%)',
            background: 'rgba(0,0,0,0.6)',
            backdropFilter: 'blur(8px)',
            borderRadius: 14,
            padding: '12px 22px',
            display: 'flex',
            alignItems: 'center',
            gap: 8,
            color: '#fff',
            fontSize: 18,
            fontWeight: 700,
            pointerEvents: 'none',
            zIndex: 5,
          }}
        >
          {feedback.icon === 'play' && <IconPlay />}
          {feedback.icon === 'pause' && <IconPause />}
          {feedback.icon === 'seekBack' && <IconSeekBack />}
          {feedback.icon === 'seekFwd' && <IconSeekFwd />}
          {feedback.icon === 'speed' && <span style={{ fontSize: 20 }}>⚡</span>}
          {feedback.label && <span>{feedback.label}</span>}
        </div>
      )}

      {/* ── Skip button ── */}
      {activeSkip && (
        <div
          style={{
            ...opacityStyle,
            position: 'absolute',
            bottom: 110,
            right: 22,
            zIndex: 4,
          }}
        >
          <button
            onClick={(e) => {
              e.stopPropagation();
              resetActivity();
              sendCmd(`set time-pos ${Math.floor(activeSkip.endMs / 1000)}`);
            }}
            style={styles.skipBtn}
          >
            {activeSkip.label}
          </button>
        </div>
      )}

      {/* ── Next episode card ── */}
      {showNextEpCard && nextEpSubtitle && !showEpisodePanel && (
        <div
          style={{
            ...opacityStyle,
            position: 'absolute',
            bottom: activeSkip ? 160 : 110,
            right: 22,
            zIndex: 4,
          }}
        >
          <button
            onClick={(e) => {
              e.stopPropagation();
              resetActivity();
              void emit('native-player-next-episode', null);
            }}
            style={styles.nextEpBtn}
          >
            <span style={{ fontSize: 11, color: 'rgba(255,255,255,0.6)', display: 'block', marginBottom: 2 }}>Up next</span>
            <span style={{ fontSize: 13, fontWeight: 700 }}>{nextEpSubtitle}</span>
          </button>
        </div>
      )}

      {/* ── Episode side panel ── */}
      {showEpisodePanel && (
        <div
          style={{
            position: 'absolute',
            top: 0,
            right: 0,
            bottom: 0,
            width: 320,
            background: 'rgba(10,12,18,0.97)',
            backdropFilter: 'blur(20px)',
            borderLeft: '1px solid rgba(255,255,255,0.08)',
            zIndex: 6,
            display: 'flex',
            flexDirection: 'column',
            overflow: 'hidden',
          }}
          onClick={(e) => e.stopPropagation()}
        >
          <div style={{ padding: '18px 16px 10px', display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
            <span style={{ color: '#fff', fontWeight: 700, fontSize: 15 }}>Episodes</span>
            <button onClick={() => { setShowEpisodePanel(false); episodePanelOpenRef.current = false; }} style={{ ...styles.iconBtn, width: 32, height: 32 }}>
              ✕
            </button>
          </div>
          {seasons.length > 1 && (
            <div style={{ display: 'flex', gap: 4, padding: '0 12px 8px', overflowX: 'auto' }}>
              {seasons.map((s) => (
                <button
                  key={s}
                  onClick={() => setActiveSeason(s)}
                  style={{
                    background: activeSeason === s ? 'var(--primary-accent-color)' : 'rgba(255,255,255,0.1)',
                    border: 'none',
                    borderRadius: 6,
                    color: '#fff',
                    fontSize: 12,
                    fontWeight: 600,
                    padding: '4px 10px',
                    cursor: 'pointer',
                    whiteSpace: 'nowrap',
                  }}
                >
                  S{s}
                </button>
              ))}
            </div>
          )}
          <div style={{ flex: 1, overflowY: 'auto', padding: '0 0 8px' }}>
            {(seasonGroups[activeSeason] ?? []).map((ep) => (
              <button
                key={ep.id}
                onClick={() => {
                  void emit('native-player-play-episode', ep.id);
                  setShowEpisodePanel(false);
                  episodePanelOpenRef.current = false;
                }}
                style={{
                  display: 'flex',
                  alignItems: 'flex-start',
                  gap: 10,
                  width: '100%',
                  background: 'none',
                  border: 'none',
                  borderBottom: '1px solid rgba(255,255,255,0.05)',
                  color: '#fff',
                  padding: '10px 16px',
                  cursor: 'pointer',
                  textAlign: 'left',
                }}
              >
                {ep.thumbnail && (
                  <img
                    src={ep.thumbnail}
                    alt=""
                    style={{ width: 80, height: 45, objectFit: 'cover', borderRadius: 4, flexShrink: 0, background: '#1a1a2e' }}
                  />
                )}
                <div style={{ minWidth: 0 }}>
                  <div style={{ fontSize: 13, fontWeight: 600, whiteSpace: 'nowrap', overflow: 'hidden', textOverflow: 'ellipsis' }}>
                    {epLabel(ep)}
                  </div>
                </div>
              </button>
            ))}
          </div>
        </div>
      )}

      {/* ── Bottom controls ── */}
      <div
        style={{
          ...opacityStyle,
          position: 'absolute',
          bottom: 0,
          left: 0,
          right: showEpisodePanel ? 320 : 0,
          padding: '50px 12px 14px',
          background: 'linear-gradient(to top, rgba(0,0,0,0.85) 0%, transparent 100%)',
          display: 'flex',
          flexDirection: 'column',
          gap: 4,
          zIndex: 2,
        }}
      >
        {/* Seekbar row */}
        <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
          <span ref={currentTimeRef} style={styles.timeLabel}>0:00</span>

          {/* Seekbar */}
          <div
            ref={seekbarRef}
            style={{ flex: 1, height: 20, display: 'flex', alignItems: 'center', cursor: 'pointer', position: 'relative' }}
            onMouseDown={onSeekMouseDown}
            onMouseMove={onSeekMouseMove}
            onMouseLeave={onSeekMouseLeave}
            onClick={onSeekClick}
          >
            {/* Track background */}
            <div style={{ position: 'absolute', left: 0, right: 0, height: 4, borderRadius: 2, background: 'rgba(255,255,255,0.2)' }} />

            {chapterSegments ? (
              // Chapter-segmented seekbar
              chapterSegments.map((seg, i) => {
                const segW = seg.end - seg.start;
                const left = `${seg.start * 100}%`;
                const width = `calc(${segW * 100}% - 2px)`;
                return (
                  <div
                    key={i}
                    style={{ position: 'absolute', left, width, height: 4, borderRadius: 2, overflow: 'hidden', background: 'rgba(255,255,255,0.2)' }}
                  >
                    {/* Buffer */}
                    <div ref={i === 0 ? seekBufferRef : undefined} style={{ position: 'absolute', left: 0, width: '0%', height: '100%', background: 'rgba(255,255,255,0.35)' }} />
                    {/* Fill */}
                    <div ref={i === 0 ? seekFillRef : undefined} style={{ position: 'absolute', left: 0, width: '0%', height: '100%', background: 'var(--primary-accent-color, #E85D3F)' }} />
                  </div>
                );
              })
            ) : (
              <>
                {/* Buffer */}
                <div ref={seekBufferRef} style={{ position: 'absolute', left: 0, width: '0%', height: 4, borderRadius: 2, background: 'rgba(255,255,255,0.35)' }} />
                {/* Fill */}
                <div ref={seekFillRef} style={{ position: 'absolute', left: 0, width: '0%', height: 4, borderRadius: 2, background: 'var(--primary-accent-color, #E85D3F)' }} />
              </>
            )}

            {/* Scrubber dot */}
            <div
              ref={seekDotRef}
              style={{
                position: 'absolute',
                left: '0%',
                width: 14,
                height: 14,
                borderRadius: '50%',
                background: '#fff',
                transform: 'translate(-50%, 0)',
                boxShadow: '0 1px 6px rgba(0,0,0,0.5)',
                pointerEvents: 'none',
                transition: isDragging ? 'none' : undefined,
              }}
            />

            {/* Seek time preview tooltip */}
            {seekPreview && (
              <div
                style={{
                  position: 'absolute',
                  bottom: 22,
                  left: seekPreview.x,
                  transform: 'translateX(-50%)',
                  background: 'rgba(0,0,0,0.85)',
                  borderRadius: 6,
                  padding: '3px 8px',
                  fontSize: 12,
                  fontWeight: 600,
                  color: '#fff',
                  pointerEvents: 'none',
                  whiteSpace: 'nowrap',
                }}
              >
                {fmtTime(seekPreview.time)}
              </div>
            )}
          </div>

          <span ref={durationRef} style={styles.timeLabel}>0:00</span>
        </div>

        {/* Buttons row */}
        <div style={{ display: 'flex', alignItems: 'center', gap: 2 }}>
          {/* Play/Pause */}
          <button
            onClick={(e) => {
              e.stopPropagation();
              resetActivity();
              flashFeedback(paused ? 'play' : 'pause', '');
              sendCmd('cycle pause');
            }}
            style={{ ...styles.iconBtn, width: 44, height: 44 }}
            title={paused ? 'Play' : 'Pause'}
          >
            {paused ? <IconPlay /> : <IconPause />}
          </button>

          {/* Seek back */}
          <button
            onClick={(e) => { e.stopPropagation(); resetActivity(); flashFeedback('seekBack', '-10s'); sendCmd('seek -10 relative'); }}
            style={styles.iconBtn}
            title="-10s"
          >
            <IconSeekBack />
          </button>

          {/* Seek forward */}
          <button
            onClick={(e) => { e.stopPropagation(); resetActivity(); flashFeedback('seekFwd', '+10s'); sendCmd('seek 10 relative'); }}
            style={styles.iconBtn}
            title="+10s"
          >
            <IconSeekFwd />
          </button>

          {/* Volume */}
          <button
            onClick={(e) => { e.stopPropagation(); resetActivity(); sendCmd('cycle mute'); }}
            style={{ ...styles.iconBtn, marginLeft: 4 }}
            title="Mute"
          >
            <IconVolume muted={muted} level={volumeLevel} />
          </button>
          <input
            type="range"
            min={0}
            max={130}
            value={muted ? 0 : volumeLevel}
            onChange={(e) => {
              resetActivity();
              const v = Number(e.target.value);
              sendCmd(`set volume ${v}`);
              if (muted && v > 0) sendCmd('set mute no');
            }}
            style={styles.volumeSlider}
            title={`Volume: ${muted ? 0 : volumeLevel}%`}
          />

          <div style={{ flex: 1 }} />

          {/* Episode list button */}
          {episodes.length > 0 && (
            <button
              onClick={(e) => {
                e.stopPropagation();
                resetActivity();
                const next = !showEpisodePanel;
                setShowEpisodePanel(next);
                episodePanelOpenRef.current = next;
                if (next && seasons.length > 0) setActiveSeason(seasons[0]);
              }}
              style={{ ...styles.iconBtn, opacity: showEpisodePanel ? 1 : 0.75 }}
              title="Episodes"
            >
              <IconEpisodes />
            </button>
          )}

          {/* Fullscreen */}
          <button
            onClick={(e) => { e.stopPropagation(); resetActivity(); sendCmd('cycle fullscreen'); }}
            style={styles.iconBtn}
            title="Fullscreen"
          >
            <IconFullscreen />
          </button>
        </div>
      </div>
    </div>
  );
}

// ─── Styles ───────────────────────────────────────────────────────────────────

const styles = {
  iconBtn: {
    background: 'none',
    border: 'none',
    color: '#fff',
    cursor: 'pointer',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    width: 38,
    height: 38,
    borderRadius: 8,
    padding: 0,
    flexShrink: 0,
  } as React.CSSProperties,

  timeLabel: {
    color: 'rgba(255,255,255,0.8)',
    fontSize: 12,
    fontWeight: 600,
    fontVariantNumeric: 'tabular-nums',
    minWidth: 36,
    textAlign: 'center',
    flexShrink: 0,
  } as React.CSSProperties,

  volumeSlider: {
    width: 80,
    height: 4,
    accentColor: 'var(--primary-accent-color, #E85D3F)',
    cursor: 'pointer',
    flexShrink: 0,
  } as React.CSSProperties,

  skipBtn: {
    background: 'rgba(255,255,255,0.15)',
    backdropFilter: 'blur(8px)',
    border: '1px solid rgba(255,255,255,0.3)',
    borderRadius: 8,
    color: '#fff',
    fontSize: 13,
    fontWeight: 700,
    padding: '8px 18px',
    cursor: 'pointer',
    letterSpacing: 0.3,
  } as React.CSSProperties,

  nextEpBtn: {
    background: 'rgba(18,22,30,0.92)',
    backdropFilter: 'blur(10px)',
    border: '1px solid rgba(255,255,255,0.15)',
    borderRadius: 10,
    color: '#fff',
    padding: '10px 16px',
    cursor: 'pointer',
    textAlign: 'left',
    minWidth: 180,
    maxWidth: 260,
  } as React.CSSProperties,
} satisfies Record<string, React.CSSProperties>;
