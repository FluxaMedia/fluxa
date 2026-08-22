import type { Meta, Stream, Video } from './types';

const KEY = 'fluxa.externalSession';

export type ExternalSession = {
  id: string;
  meta: Meta;
  episode: Video | null;
  stream: Stream | null;
  target: string;
  startedAt: number;
  resumeAt: number;
  runtime: number;
};

export function runtimeSeconds(meta: Meta | null, episode: Video | null): number {
  const candidates = [(episode as unknown as { runtime?: unknown })?.runtime, (meta as unknown as { runtime?: unknown })?.runtime];
  for (const candidate of candidates) {
    if (typeof candidate === 'number' && Number.isFinite(candidate) && candidate > 0) {
      return candidate > 600 ? candidate : candidate * 60;
    }
    if (typeof candidate !== 'string') continue;
    const hours = candidate.match(/(\d+)\s*h/i);
    const minutes = candidate.match(/(\d+)\s*(?:m|min)/i);
    if (hours || minutes) {
      return Number(hours?.[1] ?? 0) * 3600 + Number(minutes?.[1] ?? 0) * 60;
    }
    const bare = candidate.match(/(\d+)/);
    if (bare) return Number(bare[1]) * 60;
  }
  return 0;
}

export function estimateSession(session: ExternalSession, now: number): { timePos: number; duration: number; finished: boolean } | null {
  const away = Math.max(0, (now - session.startedAt) / 1000);
  if (away < 30) return null;
  if (session.runtime > 0) {
    const timePos = Math.min(session.runtime, session.resumeAt + away);
    return { timePos, duration: session.runtime, finished: timePos >= session.runtime * 0.9 };
  }
  const duration = session.resumeAt + away;
  return { timePos: duration, duration, finished: true };
}

export function readExternalSession(): ExternalSession | null {
  try {
    const raw = localStorage.getItem(KEY);
    if (!raw) return null;
    const parsed = JSON.parse(raw) as ExternalSession;
    return parsed?.id && parsed.meta ? parsed : null;
  } catch {
    return null;
  }
}

export function writeExternalSession(session: ExternalSession): void {
  try {
    localStorage.setItem(KEY, JSON.stringify(session));
  } catch {}
}

export function clearExternalSession(): void {
  try {
    localStorage.removeItem(KEY);
  } catch {}
}
