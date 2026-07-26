import {
  coreMergeExternalWatched,
  coreMergeExternalWatchlist,
  coreInvoke,
  coreSimklWatchedToIds,
  coreSimklWatchingToItems,
  coreSimklWatchlistToItems,
  storageRead,
  storageWrite,
} from './engine';
import { loadLibrary, saveLibrary, persistStatusListMerge, persistWatchedMerge } from './libraryOps';
import { _appVersion, platformFetch } from './httpClient';
import { enrichWithAddonMeta, replaceExternalContinueWatching } from './externalSyncUtils';
import { saveProviderLibrary } from './providerLibraries';

type SimklDeltaCache = {
  activities?: Record<string, unknown>;
  resources: Record<string, unknown>;
};

function activityAt(activities: Record<string, unknown> | undefined, type: string, status: string): string | undefined {
  const value = (activities?.[type] as Record<string, unknown> | undefined)?.[status];
  return typeof value === 'string' && value ? value : undefined;
}

function itemKey(value: unknown): string | null {
  if (!value || typeof value !== 'object') return null;
  const item = value as Record<string, unknown>;
  const ids = item.ids as Record<string, unknown> | undefined;
  const id = ids?.simkl ?? ids?.imdb ?? ids?.tmdb ?? item.id;
  return id == null ? null : String(id);
}

function mergeSimklDelta(previous: unknown, changes: unknown): unknown {
  if (!Array.isArray(previous) || !Array.isArray(changes)) return changes;
  const updates = new Map(changes.map((item) => [itemKey(item), item] as const).filter(([key]) => key != null));
  const merged = previous.map((item) => updates.get(itemKey(item)) ?? item);
  const existing = new Set(previous.map(itemKey).filter((key): key is string => key != null));
  for (const item of changes) {
    const key = itemKey(item);
    if (key == null || !existing.has(key)) merged.push(item);
  }
  return merged;
}

function mergeSimklResource(previous: unknown, changes: unknown): unknown {
  if (!previous || typeof previous !== 'object' || !changes || typeof changes !== 'object') return changes;
  if (Array.isArray(previous) || Array.isArray(changes)) return mergeSimklDelta(previous, changes);
  const merged: Record<string, unknown> = { ...(previous as Record<string, unknown>) };
  for (const [key, value] of Object.entries(changes as Record<string, unknown>)) {
    merged[key] = mergeSimklResource(merged[key], value);
  }
  return merged;
}

async function mergeExternalWatchlist(externalItems: Record<string, unknown>[]): Promise<void> {
  const lib = await loadLibrary();
  const local = (lib.watchlist as Record<string, unknown>[] | undefined) ?? [];
  const mergedJson = await coreMergeExternalWatchlist(JSON.stringify(local), JSON.stringify(externalItems));
  const mergedList = mergedJson as Record<string, unknown>[];
  if (mergedList.length > local.length) {
    lib.watchlist = mergedList;
    await persistStatusListMerge(local, mergedList, 'watchlist');
    await saveLibrary(lib);
  }
}

async function mergeExternalWatched(externalWatched: Record<string, boolean>): Promise<void> {
  const lib = await loadLibrary();
  const local = (lib.watched as Record<string, boolean> | undefined) ?? {};
  const merged = await coreMergeExternalWatched(JSON.stringify(local), JSON.stringify(externalWatched));
  lib.watched = merged;
  await persistWatchedMerge(local, merged);
  await saveLibrary(lib);
}

export async function syncSimklNow(payload: Record<string, unknown>): Promise<unknown> {
  const token = typeof payload.token === 'string' ? payload.token : undefined;
  const clientId = typeof payload.clientId === 'string' && payload.clientId ? payload.clientId : '';
  if (!token) return { synced: false, error: 'Simkl is not connected' };

  const headers: HeadersInit = {
    'Authorization': `Bearer ${token}`,
    'Content-Type': 'application/json',
    'User-Agent': `Fluxa Desktop/${_appVersion}`,
  };
  const query = `client_id=${encodeURIComponent(clientId)}&app-name=fluxa&app-version=${encodeURIComponent(_appVersion)}`;

  const profileId = typeof (payload.profile as { id?: unknown } | undefined)?.id === 'string'
    ? (payload.profile as { id: string }).id
    : 'default';
  const cacheKey = `simkl_delta_cache_${profileId}`;
  const cache = (await storageRead<SimklDeltaCache>(cacheKey)) ?? { resources: {} };
  let activities: Record<string, unknown>;
  try {
    const response = await platformFetch(`https://api.simkl.com/sync/activities?${query}`, { headers, signal: AbortSignal.timeout(60_000) });
    if (!response.ok) return { synced: false, error: `Simkl activity check failed: HTTP ${response.status}` };
    activities = await response.json() as Record<string, unknown>;
  } catch (error) {
    return { synced: false, error: error instanceof Error ? error.message : String(error) };
  }

  const resources = [
    ['showsWatching', 'tv_shows', 'watching', 'shows'],
    ['moviesWatching', 'movies', 'watching', 'movies'],
    ['showsPlanToWatch', 'tv_shows', 'plantowatch', 'shows'],
    ['moviesPlanToWatch', 'movies', 'plantowatch', 'movies'],
    ['showsCompleted', 'tv_shows', 'completed', 'shows'],
    ['moviesCompleted', 'movies', 'completed', 'movies'],
  ] as const;
  let nextResources: unknown[];
  try {
    nextResources = await Promise.all(resources.map(async ([key, type, status, path]) => {
      const previousActivity = activityAt(cache.activities, type, status);
      const currentActivity = activityAt(activities, type, status);
      const forceFull = !cache.resources[key]
        || activityAt(cache.activities, type, 'removed_from_list') !== activityAt(activities, type, 'removed_from_list');
      if (!forceFull && previousActivity === currentActivity) return cache.resources[key];
      const dateFrom = !forceFull && previousActivity ? `&date_from=${encodeURIComponent(previousActivity)}` : '';
      const response = await platformFetch(`https://api.simkl.com/sync/all-items/${path}/${status}?extended=full&episode_watched_at=yes&${query}${dateFrom}`, { headers, signal: AbortSignal.timeout(60_000) });
      if (!response.ok) throw new Error(`Simkl sync failed: HTTP ${response.status}`);
      const changes = await response.json();
      return forceFull ? changes : mergeSimklResource(cache.resources[key], changes);
    }));
  } catch (error) {
    return { synced: false, error: error instanceof Error ? error.message : String(error) };
  }
  for (const [index, [key]] of resources.entries()) cache.resources[key] = nextResources[index];
  cache.activities = activities;
  await storageWrite(cacheKey, cache);
  const [shows, movies, wlShows, wlMovies, doneShows, doneMovies] = nextResources;
  const showsData = JSON.stringify(shows);
  const moviesData = JSON.stringify(movies);
  const rawItems = ((await coreSimklWatchingToItems(showsData, moviesData)) ?? []) as Record<string, unknown>[];
  const items = await enrichWithAddonMeta(rawItems);
  await replaceExternalContinueWatching({ items, provider: 'simkl' });
  const { promoteExternalProgress } = await import('./externalSync');
  await promoteExternalProgress(items, 'simkl', payload.profile as import('./types').UserProfile | null);

  const wlShowsData = JSON.stringify(wlShows);
  const wlMoviesData = JSON.stringify(wlMovies);
  const watchlistItems = ((await coreSimklWatchlistToItems(wlShowsData, wlMoviesData)) ?? []) as Record<string, unknown>[];
  await mergeExternalWatchlist(watchlistItems);
  await saveProviderLibrary('simkl', { watchlist: watchlistItems, watching: items, completed: [], dropped: [] });

  const doneShowsData = JSON.stringify(doneShows);
  const doneMoviesData = JSON.stringify(doneMovies);
  const watchedMap = ((await coreSimklWatchedToIds(doneShowsData, doneMoviesData)) ?? {}) as Record<string, boolean>;
  await mergeExternalWatched(watchedMap);

  return { synced: true, provider: 'simkl', continueWatchingCount: items.length, watchlistCount: watchlistItems.length, watchedCount: Object.keys(watchedMap).length };
}

export async function fetchSimklCalendarItems(
  _token: string,
  clientId: string,
  allowedContentIds: string[],
  calendarMonth?: { year: number; month: number },
): Promise<Record<string, unknown>[]> {
  const headers: HeadersInit = {
    'User-Agent': `Fluxa Desktop/${_appVersion}`,
  };
  const query = `client_id=${encodeURIComponent(clientId)}&app-name=fluxa&app-version=${encodeURIComponent(_appVersion)}`;
  const monthPath = calendarMonth
    ? `${Math.trunc(calendarMonth.year)}/${Math.trunc(calendarMonth.month)}/`
    : "";

  const [shows, movies] = await Promise.all([
    platformFetch(`https://data.simkl.in/calendar/v2/${monthPath}tv.json?${query}`, { headers })
      .then((res) => (res.ok ? res.json() : {})).catch(() => ({})),
    platformFetch(`https://data.simkl.in/calendar/v2/${monthPath}movie_release.json?${query}`, { headers })
      .then((res) => (res.ok ? res.json() : {})).catch(() => ({})),
  ]);

  return (await coreInvoke<Record<string, unknown>[]>(
    'providerCalendarItems',
    JSON.stringify({ provider: 'simkl', shows, movies, allowedContentIds }),
  )) ?? [];
}

export async function pushMarkWatchedSimkl(
  videoIds: string[],
  watched: boolean,
  meta: Record<string, unknown> | undefined,
  token: string,
  clientId: string,
): Promise<void> {
  const simklHeaders: HeadersInit = {
    'Authorization': `Bearer ${token}`,
    'Content-Type': 'application/json',
    'User-Agent': `Fluxa Desktop/${_appVersion}`,
  };
  const endpoint = watched ? '/sync/history' : '/sync/history/remove';
  const body = await coreInvoke<Record<string, unknown>>('simklMarkWatchedBody', JSON.stringify({ videoIds, meta }));
  if (body) {
    await platformFetch(`https://api.simkl.com${endpoint}?client_id=${encodeURIComponent(clientId)}&app-name=fluxa&app-version=${encodeURIComponent(_appVersion)}`, {
      method: 'POST', headers: simklHeaders, body: JSON.stringify(body),
    });
  }
}

export async function pushWatchlistSimkl(
  id: string,
  contentType: string,
  command: 'add' | 'remove',
  token: string,
  clientId: string,
): Promise<void> {
  const simklHeaders: HeadersInit = {
    'Authorization': `Bearer ${token}`,
    'Content-Type': 'application/json',
    'User-Agent': `Fluxa Desktop/${_appVersion}`,
  };
  const body = await coreInvoke<Record<string, unknown>>('simklWatchlistBody', JSON.stringify({ id, contentType, command }));
  if (!body) return;
  const endpoint = command === 'add' ? '/sync/add-to-list' : '/sync/history/remove';
  await platformFetch(`https://api.simkl.com${endpoint}?client_id=${encodeURIComponent(clientId)}&app-name=fluxa&app-version=${encodeURIComponent(_appVersion)}`, {
    method: 'POST', headers: simklHeaders, body: JSON.stringify(body),
  });
}
