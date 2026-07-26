import { invoke } from '@tauri-apps/api/core';
import { Volume1, Volume2, VolumeOff } from 'lucide-react';
import { t } from '../../i18n';
import type { EpisodeInfo } from './EpisodePanel';

export type Chapter = { title: string; startMs: number };
export type SkipSegment = { type: string; startTime: number; endTime: number };
export type ActiveSkip = { label: string; startMs: number; endMs: number; type: string };
export type FeedbackFlash = { icon: 'play' | 'pause' | 'seekBack' | 'seekFwd' | 'speed' | 'abLoop' | 'screenshot' | 'subDelay' | 'volume' | 'anime4k'; label: string };

export const ANIME4K_MODES = ['a', 'b', 'c', 'aa', 'bb', 'ca'] as const;

export function fmtTime(s: number): string {
  if (!isFinite(s) || s < 0) return '0:00';
  const h = Math.floor(s / 3600);
  const m = Math.floor((s % 3600) / 60);
  const sec = Math.floor(s % 60);
  if (h > 0) return `${h}:${String(m).padStart(2, '0')}:${String(sec).padStart(2, '0')}`;
  return `${m}:${String(sec).padStart(2, '0')}`;
}

export function sendCmd(command: string) {
  invoke('player_command', { command }).catch(() => undefined);
}

export function parseChapters(json: string | null | undefined): Chapter[] {
  if (!json) return [];
  try {
    const arr = JSON.parse(json) as Array<{ title?: string; startTime?: number }>;
    return arr.map((chapter) => ({ title: chapter.title ?? '', startMs: chapter.startTime ?? 0 }));
  } catch {
    return [];
  }
}

export function parseSegments(json: string | null | undefined): SkipSegment[] {
  if (!json) return [];
  try {
    return JSON.parse(json) as SkipSegment[];
  } catch {
    return [];
  }
}

export function parseEpisodes(json: string | null | undefined): EpisodeInfo[] {
  if (!json) return [];
  try {
    return JSON.parse(json) as EpisodeInfo[];
  } catch {
    return [];
  }
}

export function skipLabelForType(type: string): string {
  switch (type) {
    case 'intro': return t('player.skip_intro');
    case 'outro': return t('player.skip_outro');
    case 'recap': return t('player.skip_recap');
    case 'preview': return t('player.skip_preview');
    default: return t('player.skip');
  }
}

export function IconVolume({ muted, level }: { muted: boolean; level: number }) {
  if (muted || level === 0) return <VolumeOff size={24} />;
  if (level < 50) return <Volume1 size={24} />;
  return <Volume2 size={24} />;
}

const sparklineMaxSamples = 60;

export function addSparklineSample(samples: number[], value: number): number[] {
  return [...samples, value].slice(-sparklineMaxSamples);
}

export function Sparkline({ data, w = 64, h = 16, gradId }: { data: number[]; w?: number; h?: number; gradId: string }) {
  if (data.length < 2) return <span style={{ display: 'inline-block', width: w, height: h, verticalAlign: 'middle' }} />;
  const max = Math.max(...data, 0.001);
  const pad = 1;
  const points = data.map((value, index) => [
    pad + (index / (data.length - 1)) * (w - pad * 2),
    h - pad - (value / max) * (h - pad * 2),
  ]);
  const line = points.map(([x, y], index) => `${index === 0 ? 'M' : 'L'}${x.toFixed(1)},${y.toFixed(1)}`).join(' ');
  const area = `${line} L${(w - pad).toFixed(1)},${h} L${pad},${h} Z`;
  return (
    <svg width={w} height={h} style={{ display: 'inline-block', verticalAlign: 'middle', overflow: 'visible', flexShrink: 0 }}>
      <defs>
        <linearGradient id={gradId} x1="0" y1="0" x2="0" y2="1">
          <stop offset="0%" stopColor="rgba(255,255,255,0.1)" />
          <stop offset="100%" stopColor="rgba(255,255,255,0)" />
        </linearGradient>
      </defs>
      <path d={area} fill={`url(#${gradId})`} />
      <path d={line} fill="none" stroke="rgba(255,255,255,0.6)" strokeWidth="1.5" strokeLinejoin="round" strokeLinecap="round" />
    </svg>
  );
}
