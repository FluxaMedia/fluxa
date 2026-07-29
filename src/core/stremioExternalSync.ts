import {
  coreImportApplyPlan,
  coreLibraryContinueWatchingItems,
  coreInvoke,
  coreStremioWatchedToIds,
  coreStremioWatchlistToItems,
} from './engine';
import { loadLibrary, saveAddons, saveLibrary, persistStatusListMerge, persistWatchedMerge, profileStorageKey } from './libraryOps';
import { stremioPullAddons, stremioPullLibrary, stremioPushLibrary, stremioReplaceAddons } from './stremioApi';
import { normalizeAddonDescriptor } from './addons';
import { enrichWithAddonMeta, replaceExternalContinueWatching } from './externalSyncUtils';
import { saveProviderLibrary } from './providerLibraries';
import type { AddonDescriptor, UserProfile } from './types';
import type { ImportCategory } from './importCategories';

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

async function applyWatchlistMerge(merged: Record<string, unknown>[], before: Record<string, unknown>[], profileKey?: string): Promise<void> {
  if (merged.length <= before.length) return;
  const lib = await loadLibrary(profileKey);
  lib.watchlist = merged;
  await persistStatusListMerge(before, merged, 'watchlist', profileKey);
  await saveLibrary(lib, profileKey);
}

async function applyWatchedMerge(merged: Record<string, boolean>, before: Record<string, boolean>, profileKey?: string): Promise<void> {
  const lib = await loadLibrary(profileKey);
  lib.watched = merged;
  await persistWatchedMerge(before, merged, profileKey);
  await saveLibrary(lib, profileKey);
}

export async function syncStremioNow(payload: Record<string, unknown>): Promise<unknown> {
  const authKey = typeof payload.token === 'string' ? payload.token : undefined;
  if (!authKey) return { synced: false, error: 'Stremio is not connected' };
  const profile = payload.profile as UserProfile | undefined;
  const profileKey = profile ? profileStorageKey(profile) : undefined;
  const categories = payload.categories as ImportCategory[] | undefined;
  const wants = (category: ImportCategory) => !categories || categories.includes(category);
  const dryRun = payload.dryRun === true;

  let libraryItems: Record<string, unknown>[];
  try {
    libraryItems = await stremioPullLibrary(authKey);
  } catch (err) {
    return { synced: false, error: err instanceof Error ? err.message : String(err) };
  }

  const rawItems = ((await coreLibraryContinueWatchingItems(libraryItems)) ?? []) as Record<string, unknown>[];
  const items = await enrichWithAddonMeta(rawItems);
  if (wants('continueWatching') && !dryRun) await replaceExternalContinueWatching({ items, provider: 'stremio', profileKey });

  let watchlistCount = 0;
  let watchedCount = 0;
  try {
    const watchlistItems = ((await coreStremioWatchlistToItems(libraryItems)) ?? []) as Record<string, unknown>[];
    const watchedIds = ((await coreStremioWatchedToIds(libraryItems)) ?? {}) as Record<string, boolean>;

    const localLib = await loadLibrary(profileKey);
    const localWatchlist = (localLib.watchlist as Record<string, unknown>[] | undefined) ?? [];
    const localWatched = (localLib.watched as Record<string, unknown> | undefined) ?? {};
    const applyPlan = await coreImportApplyPlan({
      localWatchlist,
      externalWatchlist: watchlistItems,
      localWatched,
      externalWatched: watchedIds,
      categories,
      dryRun,
    });
    watchlistCount = applyPlan.watchlistCount;
    watchedCount = applyPlan.watchedCount;
    if (applyPlan.watchlist != null) {
      await applyWatchlistMerge(applyPlan.watchlist, localWatchlist, profileKey);
      await saveProviderLibrary('stremio', { watchlist: watchlistItems, watching: items, completed: [], dropped: [] }, profileKey);
    }
    if (applyPlan.watched != null) await applyWatchedMerge(applyPlan.watched, localWatched as Record<string, boolean>, profileKey);
  } catch {}

  let addonCount = 0;
  if (wants('addons')) {
    try {
      const addons = await stremioPullAddons(authKey);
      addonCount = addons.length;
      if (!dryRun) await saveAddons(await Promise.all(addons.map(normalizeAddonDescriptor)));
    } catch {}
  }

  return { synced: true, provider: 'stremio', continueWatchingCount: items.length, watchlistCount, watchedCount, addonCount };
}
