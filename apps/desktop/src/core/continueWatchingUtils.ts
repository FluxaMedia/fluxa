import type { LibraryItem, Meta } from './types';
import { coreContinueWatchingCardFields, coreInvoke } from './engine';
import { t, getLanguage } from '../i18n';

// Batched for a whole Continue Watching row — one IPC round trip for the whole list
// instead of each card fetching its own artwork + episode line independently.
export async function continueWatchingCardFields(
  items: Meta[],
  artworkPreference: string,
  isHorizontal: boolean,
): Promise<Map<string, { artwork: string | null; episodeLine: string }>> {
  const fields = await coreContinueWatchingCardFields(items, artworkPreference, isHorizontal);
  return new Map((fields ?? []).map((f) => [f.id, { artwork: f.artwork, episodeLine: f.episodeLine }]));
}

export function formatWatched(offset: number): string {
  const mins = Math.max(1, Math.floor(offset / 60));
  if (mins < 60) return t('format.watched_minutes', mins);
  const h = Math.floor(mins / 60);
  const m = mins % 60;
  return m === 0 ? t('format.watched_hours', h) : t('format.watched_hours_minutes', h, m);
}

export function formatRemaining(offset: number, duration: number): string {
  const remaining = Math.max(0, duration - offset);
  const mins = Math.max(1, Math.floor(remaining / 60));
  if (mins < 60) return t('format.remaining_minutes', mins);
  const h = Math.floor(mins / 60);
  const m = mins % 60;
  return m === 0 ? t('format.remaining_hours', h) : t('format.remaining_hours_minutes', h, m);
}

export async function partitionThisWeek(items: Meta[], keepScheduled: boolean): Promise<{ thisWeek: Meta[]; continueWatching: Meta[] }> {
  const result = await coreInvoke<{ thisWeek: Meta[]; continueWatching: Meta[] }>(
    'partitionThisWeek',
    JSON.stringify({
      itemsJson: JSON.stringify(items),
      nowMs: Date.now(),
      keepScheduled,
    }),
  );
  return result ?? { thisWeek: [], continueWatching: items };
}

export function formatAirDay(date?: string): string {
  if (!date) return '';
  const locale = getLanguage() === 'tr' ? 'tr-TR' : 'en-US';
  return new Intl.DateTimeFormat(locale, { weekday: 'short' }).format(new Date(date));
}

export function formatReleaseCountdown(date?: string): string {
  if (!date) return '';
  const target = new Date(date).getTime();
  const now = Date.now();
  const diff = target - now;
  if (diff <= 0) return t('format.available_now');
  const mins = Math.floor(diff / 60000);
  const hours = Math.floor(mins / 60);
  const days = Math.floor(hours / 24);
  if (days > 0) return hours % 24 > 0 ? t('format.countdown_days_hours', days, hours % 24) : t('format.countdown_days', days);
  if (hours > 0) return mins % 60 > 0 ? t('format.countdown_hours_minutes', hours, mins % 60) : t('format.countdown_hours', hours);
  return t('format.countdown_minutes', mins);
}

export async function markContinueWatchingItemWatched(meta: Meta, onDispatch: (actionJson: string) => void | Promise<void>): Promise<void> {
  const item = meta as unknown as LibraryItem & {
    lastVideoId?: string;
    lastEpisodeName?: string;
    lastEpisodeSeason?: number;
    lastEpisodeNumber?: number;
    lastEpisodeThumbnail?: string;
  };
  const videoId = meta.type === 'series' ? item.lastVideoId : meta.id;
  await Promise.resolve(
    onDispatch(
      JSON.stringify({
        type: 'markWatchedRequested',
        seriesId: meta.id,
        videoIds: videoId ? [videoId] : [meta.id],
        watched: true,
        meta,
        episodes:
          meta.type === 'series' && videoId
            ? [
                {
                  id: videoId,
                  name: item.lastEpisodeName ?? undefined,
                  season: item.lastEpisodeSeason ?? undefined,
                  number: item.lastEpisodeNumber ?? undefined,
                  thumbnail: item.lastEpisodeThumbnail ?? meta.background ?? meta.poster,
                },
              ]
            : [],
      }),
    ),
  );
  if (meta.type === 'series') {
    await Promise.resolve(
      onDispatch(
        JSON.stringify({
          type: 'clearPlaybackProgressRequested',
          meta: { ...meta, _preserveLastWatched: true },
        }),
      ),
    );
    void onDispatch(JSON.stringify({ type: 'refreshContinueWatchingRequested', language: getLanguage() }));
  } else {
    await dropContinueWatchingItem(meta, onDispatch);
  }
}

export async function dropContinueWatchingItem(meta: Meta, onDispatch: (actionJson: string) => void | Promise<void>): Promise<void> {
  await Promise.resolve(
    onDispatch(JSON.stringify({ type: 'clearPlaybackProgressRequested', meta: { ...meta, _dropContinueWatching: true } })),
  );
}
