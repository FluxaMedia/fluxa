import {
  coreLibraryContinueWatchingItems,
  coreInvoke,
  coreMergeExternalWatched,
  coreMergeExternalWatchlist,
  coreStremioWatchedToIds,
  coreStremioWatchlistToItems,
} from './engine';
import { loadLibrary, saveAddons, saveLibrary, persistStatusListMerge, persistWatchedMerge } from './libraryOps';
import { stremioPullAddons, stremioPullLibrary, stremioPushLibrary, stremioReplaceAddons } from './stremioApi';
import { normalizeAddonDescriptor } from './addons';
import { enrichWithAddonMeta, replaceExternalContinueWatching } from './externalSyncUtils';
import { saveProviderLibrary } from './providerLibraries';
import type { AddonDescriptor, UserProfile } from './types';

type WatchedEpisode = {
  contentId: string;
  contentType: string;
  videoId?: string;
  season?: number;
  episode?: number;
  title?: string;
};

type PlaybackProgress = {
  contentId: string;
  contentType: string;
  videoId: string;
  positionSeconds: number;
  durationSeconds: number;
  lastWatched: number;
  season?: number;
  episode?: number;
};

export async function pushStremioWatchlist(
  item: Record<string, unknown>,
  command: 'add' | 'remove',
  profile: UserProfile | null,
): Promise<void> {
  const authKey = profile?.stremioAuthKey;
  if (!authKey) return;
  const changes = await coreInvoke<Record<string, unknown>[]>('stremioLibraryMutationPlan', JSON.stringify({ kind: 'watchlist', item, command }));
  if (changes?.length) await stremioPushLibrary(authKey, changes);
}

export async function pushStremioPlaybackProgress(
  meta: Record<string, unknown>,
  progress: PlaybackProgress,
  profile: UserProfile | null,
): Promise<void> {
  const authKey = profile?.stremioAuthKey;
  if (!authKey || progress.durationSeconds <= 0) return;
  const changes = await coreInvoke<Record<string, unknown>[]>('stremioLibraryMutationPlan', JSON.stringify({ kind: 'progress', meta, progress }));
  if (changes?.length) await stremioPushLibrary(authKey, changes);
}

export async function pushStremioWatched(
  meta: Record<string, unknown> | undefined,
  watched: boolean,
  episodes: WatchedEpisode[],
  profile: UserProfile | null,
): Promise<void> {
  const authKey = profile?.stremioAuthKey;
  if (!authKey) return;
  const changes = await coreInvoke<Record<string, unknown>[]>('stremioLibraryMutationPlan', JSON.stringify({ kind: 'watched', meta, watched, episodes, nowMs: Date.now() }));
  if (changes?.length) await stremioPushLibrary(authKey, changes);
}

export async function syncStremioAddons(profile: UserProfile, addons: AddonDescriptor[]): Promise<void> {
  if (!profile.stremioAuthKey) return;
  await stremioReplaceAddons(profile.stremioAuthKey, addons);
}

async function mergeExternalWatchlist(externalItems: Record<string, unknown>[]): Promise<number> {
  const lib = await loadLibrary();
  const local = (lib.watchlist as Record<string, unknown>[] | undefined) ?? [];
  const merged = await coreMergeExternalWatchlist(JSON.stringify(local), JSON.stringify(externalItems));
  if (merged.length > local.length) {
    lib.watchlist = merged;
    await persistStatusListMerge(local, merged, 'watchlist');
    await saveLibrary(lib);
  }
  return externalItems.length;
}

async function mergeExternalWatched(externalWatched: Record<string, boolean>): Promise<void> {
  const lib = await loadLibrary();
  const local = (lib.watched as Record<string, boolean> | undefined) ?? {};
  const merged = await coreMergeExternalWatched(JSON.stringify(local), JSON.stringify(externalWatched));
  lib.watched = merged;
  await persistWatchedMerge(local, merged);
  await saveLibrary(lib);
}

export async function syncStremioNow(payload: Record<string, unknown>): Promise<unknown> {
  const authKey = typeof payload.token === 'string' ? payload.token : undefined;
  if (!authKey) return { synced: false, error: 'Stremio is not connected' };

  let libraryItems: Record<string, unknown>[];
  try {
    libraryItems = await stremioPullLibrary(authKey);
  } catch (err) {
    return { synced: false, error: err instanceof Error ? err.message : String(err) };
  }

  const rawItems = ((await coreLibraryContinueWatchingItems(libraryItems)) ?? []) as Record<string, unknown>[];
  const items = await enrichWithAddonMeta(rawItems);
  await replaceExternalContinueWatching({ items, provider: 'stremio' });

  let watchlistCount = 0;
  try {
    const watchlistItems = ((await coreStremioWatchlistToItems(libraryItems)) ?? []) as Record<string, unknown>[];
    watchlistCount = await mergeExternalWatchlist(watchlistItems);
    await saveProviderLibrary('stremio', { watchlist: watchlistItems, watching: items, completed: [], dropped: [] });
  } catch {}

  try {
    const watchedIds = ((await coreStremioWatchedToIds(libraryItems)) ?? {}) as Record<string, boolean>;
    await mergeExternalWatched(watchedIds);
  } catch {}

  let addonCount = 0;
  try {
    const addons = await stremioPullAddons(authKey);
    await saveAddons(await Promise.all(addons.map(normalizeAddonDescriptor)));
    addonCount = addons.length;
  } catch {}

  return { synced: true, provider: 'stremio', continueWatchingCount: items.length, watchlistCount, addonCount };
}
