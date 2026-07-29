import {
  coreMergeExternalWatched,
  coreMergeExternalWatchlist,
  coreInvoke,
  coreSimklMergeDelta,
  coreSimklResourceSyncPlan,
  coreSimklWatchedToIds,
  coreSimklWatchingToItems,
  coreSimklWatchlistToItems,
  storageRead,
  storageWrite,
} from './engine';
import { loadLibrary, saveLibrary, persistStatusListMerge, persistWatchedMerge, profileStorageKey } from './libraryOps';
import { _appVersion, platformFetch } from './httpClient';
import { enrichWithAddonMeta, replaceExternalContinueWatching } from './externalSyncUtils';
import { saveProviderLibrary } from './providerLibraries';
import type { ImportCategory } from './importCategories';

type SimklDeltaCache = {
  activities?: Record<string, unknown>;
  resources: Record<string, unknown>;
};

async function mergeExternalWatchlist(externalItems: Record<string, unknown>[], profileKey?: string): Promise<void> {
  const lib = await loadLibrary(profileKey);
  const local = (lib.watchlist as Record<string, unknown>[] | undefined) ?? [];
  const mergedJson = await coreMergeExternalWatchlist(JSON.stringify(local), JSON.stringify(externalItems));
  const mergedList = mergedJson as Record<string, unknown>[];
  if (mergedList.length > local.length) {
    lib.watchlist = mergedList;
    await persistStatusListMerge(local, mergedList, 'watchlist', profileKey);
    await saveLibrary(lib, profileKey);
  }
}

async function mergeExternalWatched(externalWatched: Record<string, boolean>, profileKey?: string): Promise<void> {
  const lib = await loadLibrary(profileKey);
  const local = (lib.watched as Record<string, boolean> | undefined) ?? {};
  const merged = await coreMergeExternalWatched(JSON.stringify(local), JSON.stringify(externalWatched));
  lib.watched = merged;
  await persistWatchedMerge(local, merged, profileKey);
  await saveLibrary(lib, profileKey);
}

export async function syncSimklNow(payload: Record<string, unknown>): Promise<unknown> {
  const token = typeof payload.token === 'string' ? payload.token : undefined;
  const clientId = typeof payload.clientId === 'string' && payload.clientId ? payload.clientId : '';
  if (!token) return { synced: false, error: 'Simkl is not connected' };
  const categories = payload.categories as ImportCategory[] | undefined;
  const wants = (category: ImportCategory) => !categories || categories.includes(category);

  const headers: HeadersInit = {
    'Authorization': `Bearer ${token}`,
    'Content-Type': 'application/json',
    'User-Agent': `Fluxa Desktop/${_appVersion}`,
  };
  const query = `client_id=${encodeURIComponent(clientId)}&app-name=fluxa&app-version=${encodeURIComponent(_appVersion)}`;

  const profile = payload.profile as import('./types').UserProfile | undefined;
  const profileKey = profile ? profileStorageKey(profile) : undefined;
  const profileId = typeof profile?.id === 'string' ? profile.id : 'default';
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
  const plan = await coreSimklResourceSyncPlan({
    previous: cache.activities ?? null,
    current: activities,
    resources: resources.map(([key, type, status]) => ({ key, type, status, hasCached: Boolean(cache.resources[key]) })),
  });
  const planByKey = new Map(plan.map((entry) => [entry.key, entry]));
  let nextResources: unknown[];
  try {
    nextResources = await Promise.all(resources.map(async ([key, , status, path]) => {
      const entry = planByKey.get(key);
      if (!entry || entry.action === 'unchanged') return cache.resources[key];
      const dateFrom = entry.action === 'delta' && entry.dateFrom ? `&date_from=${encodeURIComponent(entry.dateFrom)}` : '';
      const response = await platformFetch(`https://api.simkl.com/sync/all-items/${path}/${status}?extended=full&episode_watched_at=yes&${query}${dateFrom}`, { headers, signal: AbortSignal.timeout(60_000) });
      if (!response.ok) throw new Error(`Simkl sync failed: HTTP ${response.status}`);
      const changes = await response.json();
      return entry.action === 'full' ? changes : coreSimklMergeDelta(JSON.stringify(cache.resources[key] ?? null), JSON.stringify(changes));
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
  if (wants('continueWatching')) {
    await replaceExternalContinueWatching({ items, provider: 'simkl', profileKey });
    const { promoteExternalProgress } = await import('./externalSync');
    await promoteExternalProgress(items, 'simkl', profile ?? null);
  }

  const wlShowsData = JSON.stringify(wlShows);
  const wlMoviesData = JSON.stringify(wlMovies);
  const watchlistItems = ((await coreSimklWatchlistToItems(wlShowsData, wlMoviesData)) ?? []) as Record<string, unknown>[];
  if (wants('watchlist')) await mergeExternalWatchlist(watchlistItems, profileKey);
  await saveProviderLibrary('simkl', { watchlist: watchlistItems, watching: items, completed: [], dropped: [] }, profileKey);

  const doneShowsData = JSON.stringify(doneShows);
  const doneMoviesData = JSON.stringify(doneMovies);
  const watchedMap = ((await coreSimklWatchedToIds(doneShowsData, doneMoviesData)) ?? {}) as Record<string, boolean>;
  if (wants('watched')) await mergeExternalWatched(watchedMap, profileKey);

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
  watchedAtMs?: number,
): Promise<void> {
  const simklHeaders: HeadersInit = {
    'Authorization': `Bearer ${token}`,
    'Content-Type': 'application/json',
    'User-Agent': `Fluxa Desktop/${_appVersion}`,
  };
  const endpoint = watched ? '/sync/history' : '/sync/history/remove';
  const body = await coreInvoke<Record<string, unknown>>('simklMarkWatchedBody', JSON.stringify({ videoIds, meta, watchedAtMs }));
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
