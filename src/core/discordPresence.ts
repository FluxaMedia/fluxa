import { platformInvoke as invoke } from '../platform/invoke';

export function configureDiscordPresence(enabled: boolean): void {
  void invoke('discord_presence_configure', { enabled }).catch(() => undefined);
}

export function updateDiscordPresence(opts: {
  title: string;
  detail?: string;
  paused: boolean;
  startUnixSecs?: number;
  endUnixSecs?: number;
  posterUrl?: string;
  buttonLabel?: string;
  buttonUrl?: string;
}): void {
  void invoke('discord_presence_update', {
    title: opts.title,
    detail: opts.detail,
    paused: opts.paused,
    startUnixSecs: opts.startUnixSecs ?? null,
    endUnixSecs: opts.endUnixSecs ?? null,
    posterUrl: opts.posterUrl ?? null,
    buttonLabel: opts.buttonLabel ?? null,
    buttonUrl: opts.buttonUrl ?? null,
  }).catch(() => undefined);
}

export function setViewingDiscordPresence(opts: {
  title: string;
  posterUrl?: string;
  buttonLabel?: string;
  buttonUrl?: string;
}): void {
  void invoke('discord_presence_set_viewing', {
    title: opts.title,
    posterUrl: opts.posterUrl ?? null,
    buttonLabel: opts.buttonLabel ?? null,
    buttonUrl: opts.buttonUrl ?? null,
  }).catch(() => undefined);
}

export function setBrowsingDiscordPresence(label: string): void {
  void invoke('discord_presence_set_browsing', { label }).catch(() => undefined);
}

export function clearDiscordPresence(): void {
  void invoke('discord_presence_clear').catch(() => undefined);
}

export function imdbButtonFor(id?: string): { buttonLabel?: string; buttonUrl?: string } {
  if (!id || !/^tt\d+$/.test(id)) return {};
  return { buttonLabel: 'View on IMDb', buttonUrl: `https://www.imdb.com/title/${id}/` };
}
