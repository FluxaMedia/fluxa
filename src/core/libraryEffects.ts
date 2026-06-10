import {
  coreCalendarItemMatchesMonth,
  coreCalendarItemsFromMeta,
  coreLibraryLocalStatePlan,
  corePlaybackProgressMergePlan,
  coreRememberLastWatchedEpisodes,
  coreWatchlistTogglePlan,
  storageRead,
  storageWrite,
} from './engine';
import { buildContinueWatching, loadActiveProfile, loadLibrary, saveLibrary } from './libraryOps';
import { pushMarkWatchedExternal, pushWatchlistExternal } from './externalSync';
import type { LibraryItem } from './types';

export async function applyLibraryCommand(payload: Record<string, unknown>): Promise<unknown> {
  const lib = await loadLibrary();
  const command = payload.command as { type: string; item?: unknown; watched?: boolean; videoIds?: string[] } | undefined;
  if (!command) return lib;

  if (command.type === 'toggleWatchlist' && command.item) {
    const item = command.item as { id: string };
    const watchlist = (lib.watchlist as LibraryItem[] | undefined) ?? [];
    const idx = watchlist.findIndex((i) => i.id === item.id);
    const plan = await coreWatchlistTogglePlan({
      item: command.item,
      isCurrentlyInWatchlist: idx >= 0,
      profileId: payload.profileId,
    }) as { command?: 'add' | 'remove' } | null;
    const nextCommand = plan?.command ?? (idx >= 0 ? 'remove' : 'add');
    if (nextCommand === 'remove') {
      lib.watchlist = watchlist.filter((_, i) => i !== idx);
    } else {
      lib.watchlist = [command.item as LibraryItem, ...watchlist];
    }
    // Fire-and-forget push to external services
    void loadActiveProfile().then((profile) =>
      pushWatchlistExternal(command.item as Record<string, unknown>, nextCommand, profile)
    );
  }

  if (command.type === 'markWatched' && command.videoIds) {
    const watched = (lib.watched as Record<string, boolean> | undefined) ?? {};
    for (const vid of command.videoIds) {
      watched[vid] = command.watched !== false;
    }
    lib.watched = watched;

    // When marking as watched, clean up Continue Watching so the finished
    // episode doesn't linger. Remove from both local CW and the Trakt cache.
    if (command.watched !== false) {
      const watchedSet = new Set(command.videoIds);
      const updatedLib = await coreRememberLastWatchedEpisodes(JSON.stringify(lib), JSON.stringify([...watchedSet]));
      Object.assign(lib, updatedLib);
      const stripWatched = (items: unknown[]) =>
        (items as Record<string, unknown>[]).filter(
          (item) => !item.lastVideoId || !watchedSet.has(String(item.lastVideoId)),
        );
      lib.externalContinueWatching = stripWatched(
        (lib.externalContinueWatching as unknown[] | undefined) ?? [],
      );
      // Clean progress entries whose last episode was just watched, then
      // rebuild continueWatching from the cleaned map.
      const progressMap = (lib.progress as Record<string, unknown> | undefined) ?? {};
      for (const key of Object.keys(progressMap)) {
        const entry = progressMap[key] as Record<string, unknown> | undefined;
        if (entry?.lastVideoId && watchedSet.has(String(entry.lastVideoId))) {
          delete progressMap[key];
        }
      }
      lib.progress = progressMap;
      lib.continueWatching = await buildContinueWatching(progressMap);
    }
    // Fire-and-forget push to external services
    void loadActiveProfile().then((profile) =>
      pushMarkWatchedExternal(
        command.videoIds as string[],
        command.watched !== false,
        command.item as Record<string, unknown> | undefined,
        profile,
      )
    );
  }

  await saveLibrary(lib);
  return lib;
}

export async function writePlaybackProgress(payload: Record<string, unknown>): Promise<unknown> {
  const lib = await loadLibrary();
  const progress = payload.progress as Record<string, unknown> | undefined;
  const meta = progress?.meta as { id?: string; name?: string; type?: string; poster?: string } | undefined;
  if (meta?.id) {
    const progressMap = (lib.progress as Record<string, unknown> | undefined) ?? {};
    const existing = (progressMap[meta.id] as Record<string, unknown> | undefined) ?? {};
    const mergePlan = await corePlaybackProgressMergePlan({
      existing,
      incoming: progress,
    }) as Record<string, unknown> | null;
    const existingMeta = (existing.meta as Record<string, unknown> | undefined) ?? {};
    const mergedMeta = {
      ...meta,
      poster: (meta as Record<string, unknown>).poster ?? existingMeta.poster,
      background: (meta as Record<string, unknown>).background ?? existingMeta.background,
      logo: (meta as Record<string, unknown>).logo ?? existingMeta.logo,
    };
    progressMap[meta.id] = {
      ...existing,
      ...progress,
      ...(mergePlan ?? {}),
      meta: mergedMeta,
      savedAt: new Date().toISOString(),
    };
    lib.progress = progressMap;
    lib.continueWatching = await buildContinueWatching(progressMap);
    await saveLibrary(lib);
  }
  return {};
}

export async function writeSettings(payload: Record<string, unknown>): Promise<unknown> {
  const existing = (await storageRead<Record<string, unknown>>('settings')) ?? {};
  const updated = { ...existing, ...payload };
  await storageWrite('settings', updated);
  return updated;
}

export async function readLibraryState(): Promise<unknown> {
  return loadLibrary();
}

export async function readPlaybackProgress(payload: Record<string, unknown>): Promise<unknown> {
  const lib = await loadLibrary();
  const id = payload.id as string | undefined;
  if (!id) return null;
  const progressMap = (lib.progress as Record<string, unknown> | undefined) ?? {};
  return progressMap[id] ?? null;
}

export async function readDetailLocalState(payload: Record<string, unknown>): Promise<unknown> {
  const lib = await loadLibrary();
  return (await coreLibraryLocalStatePlan({ library: lib, ...payload })) ?? {
    progress: null,
    isInWatchlist: false,
    watchedVideoIds: [],
  };
}

export async function readCalendarMonth(payload: Record<string, unknown>): Promise<unknown> {
  const year = Number(payload.year);
  const month = Number(payload.month);
  const monthPrefix = Number.isFinite(year) && Number.isFinite(month)
    ? `${Math.trunc(year)}-${String(Math.trunc(month)).padStart(2, '0')}`
    : '';
  const plannedItems = Array.isArray(payload.plannedItems) ? payload.plannedItems : [];
  const lib = await loadLibrary();
  const libraryItems = [
    ...(((lib.watchlist as unknown[] | undefined) ?? [])),
    ...(((lib.continueWatching as unknown[] | undefined) ?? [])),
  ];
  const seen = new Set<string>();
  const localItemsNested = await Promise.all(libraryItems.map((raw) => coreCalendarItemsFromMeta(JSON.stringify(raw), monthPrefix)));
  const localItems = localItemsNested.flat().filter((item) => {
    const id = String((item as Record<string, unknown>).id ?? '');
    if (seen.has(id)) return false;
    seen.add(id);
    return true;
  });
  const externalItems = await Promise.all(
    ((lib.externalCalendarItems as unknown[] | undefined) ?? [])
      .map(async (item) => {
        const match = await coreCalendarItemMatchesMonth(JSON.stringify(item), monthPrefix);
        return match ? item : null;
      })
  ).then((items) => items.filter((i) => i !== null));
  return {
    items: plannedItems,
    localItems,
    externalItems,
  };
}
