import type { LibraryItem, Meta } from './types';
import { t } from '../i18n';

export function selectContinueWatchingArtwork(meta: Meta, artworkPreference: string, isHorizontal: boolean): string | undefined {
  const item = meta as Meta & {
    continueWatchingPoster?: string;
    continueWatchingBackground?: string;
    lastEpisodeThumbnail?: string;
  };
  const existingBackdrop =
    meta.background &&
    meta.background !== meta.poster &&
    !meta.background.toLowerCase().includes('/poster/')
      ? meta.background
      : undefined;

  if (!isHorizontal) {
    return item.lastEpisodeThumbnail ?? item.continueWatchingPoster ?? meta.poster ?? item.continueWatchingBackground ?? meta.background;
  }

  const isSeries = ['series', 'tv', 'anime'].includes(meta.type);
  const candidates =
    artworkPreference === 'poster'
      ? [meta.poster, item.continueWatchingBackground, existingBackdrop]
      : artworkPreference === 'background'
        ? [existingBackdrop, item.continueWatchingBackground, meta.poster]
        : [
            item.lastEpisodeThumbnail,
            isSeries ? item.continueWatchingBackground : item.continueWatchingBackground,
            existingBackdrop,
            meta.background,
            meta.poster,
          ];

  return candidates.find((v) => typeof v === 'string' && v.trim().length > 0);
}

export function formatEpisodeLine(item: {
  lastEpisodeName?: string;
  lastEpisodeSeason?: number;
  lastEpisodeNumber?: number;
  lastVideoId?: string;
}): string {
  let season: number | undefined = typeof item.lastEpisodeSeason === 'number' ? item.lastEpisodeSeason : undefined;
  let episode: number | undefined = typeof item.lastEpisodeNumber === 'number' ? item.lastEpisodeNumber : undefined;

  // Fallback: parse from lastVideoId (Stremio format: "imdbId:season:episode")
  if ((season == null || episode == null) && item.lastVideoId) {
    const parts = item.lastVideoId.split(':');
    if (parts.length >= 3) {
      const s = parseInt(parts[parts.length - 2], 10);
      const e = parseInt(parts[parts.length - 1], 10);
      if (!isNaN(s) && !isNaN(e) && s > 0 && e > 0) {
        if (season == null) season = s;
        if (episode == null) episode = e;
      }
    }
  }

  const code = season != null && episode != null ? `S${season}:E${episode}` : '';
  const name = item.lastEpisodeName?.trim() ?? '';
  return [code, name].filter(Boolean).join(' ');
}

export function formatRemaining(offset: number, duration: number): string {
  const remaining = Math.max(0, duration - offset);
  const mins = Math.max(1, Math.ceil(remaining / 60));
  if (mins < 60) return t('format.remaining_minutes', mins);
  const h = Math.floor(mins / 60);
  const m = mins % 60;
  return m === 0 ? t('format.remaining_hours', h) : t('format.remaining_hours_minutes', h, m);
}

export function formatReleaseCountdown(date?: string): string {
  if (!date) return '';
  const target = new Date(date).getTime();
  const now = Date.now();
  const diff = target - now;
  if (diff <= 0) return 'Available now';
  const mins = Math.floor(diff / 60000);
  const hours = Math.floor(mins / 60);
  const days = Math.floor(hours / 24);
  if (days > 0) return hours % 24 > 0 ? `${days}d ${hours % 24}h left` : `${days}d left`;
  if (hours > 0) return mins % 60 > 0 ? `${hours}h ${mins % 60}m left` : `${hours}h left`;
  return `${mins}m left`;
}

export async function markContinueWatchingItemWatched(
  meta: Meta,
  onDispatch: (actionJson: string) => void | Promise<void>,
): Promise<void> {
  const item = meta as unknown as LibraryItem & {
    lastVideoId?: string;
    lastEpisodeName?: string;
    lastEpisodeSeason?: number;
    lastEpisodeNumber?: number;
    lastEpisodeThumbnail?: string;
    continueWatchingBadge?: string;
  };
  const videoId = meta.type === 'series' ? item.lastVideoId : meta.id;
  if (meta.type === 'series' && item.continueWatchingBadge === 'newEpisode' && videoId) {
    await Promise.resolve(onDispatch(JSON.stringify({
      type: 'savePlaybackProgressRequested',
      meta,
      timeOffset: 1,
      duration: 99999,
      lastVideoId: videoId,
      lastStreamIndex: null,
      lastEpisodeName: item.lastEpisodeName ?? null,
      lastEpisodeSeason: item.lastEpisodeSeason ?? null,
      lastEpisodeNumber: item.lastEpisodeNumber ?? null,
      lastEpisodeThumbnail: item.lastEpisodeThumbnail ?? null,
      lastStreamUrl: null,
      lastStreamTitle: null,
      lastAudioLanguage: null,
      lastSubtitleLanguage: null,
      scrobbleTraktPause: false,
    })));
  }
  await Promise.resolve(onDispatch(JSON.stringify({
    type: 'markWatchedRequested',
    seriesId: meta.id,
    videoIds: videoId ? [videoId] : [meta.id],
    watched: true,
    meta,
    episodes: meta.type === 'series' && videoId ? [{
      id: videoId,
      name: item.lastEpisodeName ?? undefined,
      season: item.lastEpisodeSeason ?? undefined,
      number: item.lastEpisodeNumber ?? undefined,
      thumbnail: item.lastEpisodeThumbnail ?? meta.background ?? meta.poster,
    }] : [],
  })));
  await dropContinueWatchingItem(meta, onDispatch);
}

export async function dropContinueWatchingItem(
  meta: Meta,
  onDispatch: (actionJson: string) => void | Promise<void>,
): Promise<void> {
  await Promise.resolve(onDispatch(JSON.stringify({ type: 'clearPlaybackProgressRequested', meta })));
}
