import {
  coreLibraryContinueWatchingItems,
  coreInvoke,
  coreMergeExternalWatched,
  coreMergeExternalWatchlist,
  coreStremioWatchedToIds,
  coreStremioWatchlistToItems,
} from './engine';
import { loadLibrary, saveAddons, saveLibrary, persistStatusListMerge, persistWatchedMerge, profileStorageKey } from './libraryOps';
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

async function mergeExternalWatchlist(externalItems: Record<string, unknown>[], profileKey?: string): Promise<number> {
  const lib = await loadLibrary(profileKey);
  const local = (lib.watchlist as Record<string, unknown>[] | undefined) ?? [];
  const merged = await coreMergeExternalWatchlist(JSON.stringify(local), JSON.stringify(externalItems));
  if (merged.length > local.length) {
    lib.watchlist = merged;
    await persistStatusListMerge(local, merged, 'watchlist', profileKey);
    await saveLibrary(lib, profileKey);
  }
  return externalItems.length;
}

async function mergeExternalWatched(externalWatched: Record<string, boolean>, profileKey?: string): Promise<void> {
  const lib = await loadLibrary(profileKey);
  const local = (lib.watched as Record<string, boolean> | undefined) ?? {};
  const merged = await coreMergeExternalWatched(JSON.stringify(local), JSON.stringify(externalWatched));
  lib.watched = merged;
  await persistWatchedMerge(local, merged, profileKey);
  await saveLibrary(lib, profileKey);
}

export async function syncStremioNow(payload: Record<string, unknown>): Promise<unknown> {
  const authKey = typeof payload.token === 'string' ? payload.token : undefined;
  if (!authKey) return { synced: false, error: 'Stremio is not connected' };
  const profile = payload.profile as UserProfile | undefined;
  const profileKey = profile ? profileStorageKey(profile) : undefined;

  let libraryItems: Record<string, unknown>[];
  try {
    libraryItems = await stremioPullLibrary(authKey);
  } catch (err) {
    return { synced: false, error: err instanceof Error ? err.message : String(err) };
  }

  const rawItems = ((await coreLibraryContinueWatchingItems(libraryItems)) ?? []) as Record<string, unknown>[];
  const items = await enrichWithAddonMeta(rawItems);
  await replaceExternalContinueWatching({ items, provider: 'stremio', profileKey });

  let watchlistCount = 0;
  try {
    const watchlistItems = ((await coreStremioWatchlistToItems(libraryItems)) ?? []) as Record<string, unknown>[];
    watchlistCount = await mergeExternalWatchlist(watchlistItems, profileKey);
    await saveProviderLibrary('stremio', { watchlist: watchlistItems, watching: items, completed: [], dropped: [] }, profileKey);
  } catch {}

  try {
    const watchedIds = ((await coreStremioWatchedToIds(libraryItems)) ?? {}) as Record<string, boolean>;
    await mergeExternalWatched(watchedIds, profileKey);
  } catch {}

  let addonCount = 0;
  try {
    const addons = await stremioPullAddons(authKey);
    await saveAddons(await Promise.all(addons.map(normalizeAddonDescriptor)));
    addonCount = addons.length;
  } catch {}

  return { synced: true, provider: 'stremio', continueWatchingCount: items.length, watchlistCount, addonCount };
}
