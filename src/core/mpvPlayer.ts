import { invoke } from '@tauri-apps/api/core';

export async function startTorrentStream(
  streamJson: string,
  title?: string,
  preferences?: Record<string, unknown>,
): Promise<string> {
  return invoke<string>('start_torrent_stream', { streamJson, title: title ?? null, preferences: preferences ?? null });
}

export async function stopTorrentStream(): Promise<boolean> {
  return invoke<boolean>('stop_torrent_stream');
}

export type EmbeddedMpvFrame = {
  width: number;
  height: number;
  pixelsBase64: string;
};

export type EmbeddedMpvStatus = {
  loaded: boolean;
  path?: string | null;
  mediaTitle?: string | null;
  timePos?: string | null;
  duration?: string | null;
  percentPos?: string | null;
  pause?: string | null;
  coreIdle?: string | null;
  eofReached?: string | null;
  voConfigured?: string | null;
  videoFormat?: string | null;
  width?: string | null;
  height?: string | null;
  cacheSpeed?: string | null;
  demuxerCacheDuration?: string | null;
};

export async function initEmbeddedMpv(): Promise<void> {
  await invoke('player_init');
}

export async function embeddedMpvLoad(url: string, startAt?: number, totalDuration?: number): Promise<void> {
  await invoke('player_load', {
    url,
    startAt: startAt && startAt > 5 ? Math.floor(startAt) : null,
    totalDuration: totalDuration && totalDuration > 0 ? Math.floor(totalDuration) : null,
  });
}

export async function embeddedMpvApplyPreferences(preferences: Record<string, unknown>): Promise<void> {
  await invoke('player_apply_preferences', { preferences });
}

export async function embeddedMpvSetTitle(title?: string, episodeTitle?: string): Promise<void> {
  if (!title) return;
  await invoke('player_set_title', { title, episodeTitle });
}

export async function prefetchPlayerArtwork(backgroundUrl?: string | null, logoUrl?: string | null): Promise<void> {
  await invoke('player_prefetch_artwork', {
    backgroundUrl: backgroundUrl ?? null,
    logoUrl: logoUrl ?? null,
  });
}

export async function embeddedMpvSetLoadingArtwork(
  title: string,
  episodeTitle?: string | null,
  backgroundUrl?: string | null,
  logoUrl?: string | null,
): Promise<void> {
  await invoke('player_set_loading_artwork', {
    title,
    episodeTitle,
    backgroundUrl: normalizeAssetUrl(backgroundUrl),
    logoUrl: normalizeAssetUrl(logoUrl),
  });
}

function normalizeAssetUrl(url?: string | null): string | null {
  const trimmed = url?.trim();
  if (!trimmed) return null;
  if (trimmed.startsWith('//')) return `https:${trimmed}`;
  return trimmed;
}

export async function embeddedMpvAddSubtitle(
  url: string,
  title?: string,
  language?: string,
): Promise<void> {
  await invoke('player_add_subtitle', {
    url,
    title: title ?? null,
    language: language ?? null,
  });
}

export async function embeddedMpvStop(): Promise<void> {
  await invoke('player_command', { command: 'stop' });
}

export async function embeddedMpvSeekTo(seconds: number): Promise<void> {
  if (!Number.isFinite(seconds) || seconds <= 0) return;
  await invoke('player_command', { command: `set time-pos ${Math.floor(seconds)}` });
}

export async function embeddedMpvHide(): Promise<void> {
  await invoke('player_hide');
}

export async function embeddedMpvShowLoading(title?: string, episodeTitle?: string | null): Promise<void> {
  await invoke('player_show_loading', {
    title: title ?? 'Fluxa',
    episodeTitle: episodeTitle ?? null,
  });
}

export async function embeddedMpvRenderFrame(width: number, height: number): Promise<EmbeddedMpvFrame> {
  return invoke<EmbeddedMpvFrame>('player_render_frame', {
    width: Math.max(2, Math.floor(width)),
    height: Math.max(2, Math.floor(height)),
  });
}

export async function embeddedMpvStatus(): Promise<EmbeddedMpvStatus | null> {
  return invoke<EmbeddedMpvStatus>('player_status');
}

export async function destroyEmbeddedMpv(): Promise<void> {
  await invoke('player_destroy');
}

export async function playerSetChapters(chaptersJson: string): Promise<void> {
  await invoke('player_set_chapters', { chaptersJson });
}

export async function playerClearChapters(): Promise<void> {
  await invoke('player_clear_chapters');
}

export async function playerSetSkipInfo(
  segmentsJson: string,
  nextEpSubtitle?: string,
  nextEpThresholdPercent?: number,
  autoPlayNextEpisode?: boolean,
): Promise<void> {
  await invoke('player_set_skip_info', {
    segmentsJson,
    nextEpSubtitle: nextEpSubtitle ?? null,
    nextEpThresholdPercent: nextEpThresholdPercent ?? null,
    autoPlayNextEpisode: autoPlayNextEpisode ?? null,
  });
}

export async function playerClearSkipInfo(): Promise<void> {
  await invoke('player_clear_skip_info');
}

export async function playerSetEpisodes(episodesJson: string): Promise<void> {
  await invoke('player_set_episodes', { episodesJson });
}

export async function playerClearEpisodes(): Promise<void> {
  await invoke('player_clear_episodes');
}

export type PlayerTrackOption = { id: string; label: string; selected: boolean };

export type PlayerPlaybackInfo = {
  skipSegmentsJson: string | null;
  chaptersJson: string | null;
  episodesJson: string | null;
  nextEpSubtitle: string;
  nextEpThresholdPercent: number;
  autoPlayNextEpisode: boolean;
};

export async function playerGetPlaybackInfo(): Promise<PlayerPlaybackInfo> {
  return invoke('player_get_playback_info');
}

export async function playerGetTrackOptions(trackType: 'audio' | 'sub'): Promise<PlayerTrackOption[]> {
  return invoke('player_track_options', { trackType });
}
