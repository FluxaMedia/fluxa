import {
  coreInvoke,
  coreSimklMergeDelta,
  coreSimklMergePlaybackProgress,
  coreSimklResourceSyncPlan,
  coreSimklWatchedToIds,
  coreSimklWatchingToItems,
  coreSimklWatchlistToItems,
  storageRead,
  storageWrite,
} from './engine';
import { loadActiveProfile, profileStorageKey } from './libraryOps';
import { _appVersion, platformFetch } from './httpClient';
import { replaceExternalContinueWatching } from './externalSyncUtils';
import { saveProviderLibrary } from './providerLibraries';
import { getOAuthClientId } from './traktSync';
import type { ImportCategory } from './importCategories';
import { invoke } from '@tauri-apps/api/core';

function debugLog(msg: string) {
  void invoke('debug_log', { msg }).catch(() => {});
}

type SimklDeltaCache = {
  activities?: Record<string, unknown>;
  resources: Record<string, unknown>;
};

const SIMKL_EPISODE_LIST_CACHE_KEY = 'simkl_episode_list_cache';
const SIMKL_EPISODE_LIST_TTL_MS = 24 * 60 * 60 * 1000;
const SIMKL_EPISODE_LIST_MAX_AGE_MS = 30 * 24 * 60 * 60 * 1000;

type SimklEpisodeListCacheEntry = { fetchedAt: number; episodes: Record<string, unknown>[] };
type SimklEpisodeListCache = Record<string, SimklEpisodeListCacheEntry>;

async function resolveSimklEpisodeDetails(
  items: Record<string, unknown>[],
  headers: HeadersInit,
  query: string,
): Promise<Record<string, unknown>[]> {
  const needsDetails = items.filter((item) =>
    item.type === 'series' &&
    typeof item.simklId === 'number' &&
    typeof item.lastEpisodeNumber === 'number' &&
    (!item.lastEpisodeThumbnail || !item.lastEpisodeName),
  );
  if (needsDetails.length === 0) return items;

  const diskCache = (await storageRead<SimklEpisodeListCache>(SIMKL_EPISODE_LIST_CACHE_KEY)) ?? {};
  let diskCacheDirty = false;
  const now = Date.now();
  const episodeListCache = new Map<string, Promise<Record<string, unknown>[]>>();
  const fetchEpisodeList = (simklId: number, isAnime: boolean) => {
    const cacheKey = `${isAnime ? 'anime' : 'tv'}:${simklId}`;
    let pending = episodeListCache.get(cacheKey);
    if (pending) return pending;
    const cached = diskCache[cacheKey];
    if (cached && now - cached.fetchedAt < SIMKL_EPISODE_LIST_TTL_MS) {
      pending = Promise.resolve(cached.episodes);
    } else {
      const path = isAnime ? 'anime' : 'tv';
      pending = platformFetch(`https://api.simkl.com/${path}/episodes/${simklId}?${query}`, { headers, signal: AbortSignal.timeout(10_000) })
        .then((res) => (res.ok ? res.json() : []))
        .then((list) => (Array.isArray(list) ? list as Record<string, unknown>[] : []))
        .then((episodes) => {
          diskCache[cacheKey] = { fetchedAt: now, episodes };
          diskCacheDirty = true;
          return episodes;
        })
        .catch(() => []);
    }
    episodeListCache.set(cacheKey, pending);
    return pending;
  };

  const CONCURRENCY = 4;
  let cursor = 0;
  async function worker() {
    while (cursor < needsDetails.length) {
      const item = needsDetails[cursor++];
      const simklId = item.simklId as number;
      const isAnime = item.isAnime === true;
      const season = typeof item.lastEpisodeSeason === 'number' ? item.lastEpisodeSeason : undefined;
      const number = item.lastEpisodeNumber as number;
      const episodes = await fetchEpisodeList(simklId, isAnime);
      const match = isAnime
        ? episodes.find((ep) => {
            const tvdb = ep.tvdb as Record<string, unknown> | undefined;
            if (season != null && tvdb && tvdb.season === season && tvdb.episode === number) return true;
            return ep.episode === number;
          })
        : episodes.find((ep) => ep.season === season && ep.episode === number);
      const img = match && typeof match.img === 'string' ? match.img : undefined;
      if (img) item.lastEpisodeThumbnail = img.startsWith('http') ? img : `https://simkl.in/episodes/${img}_w.webp`;
      const title = match && typeof match.title === 'string' ? match.title : undefined;
      if (title && !item.lastEpisodeName) item.lastEpisodeName = title;
    }
  }
  await Promise.all(Array.from({ length: Math.min(CONCURRENCY, needsDetails.length) }, worker));

  if (diskCacheDirty) {
    for (const [key, entry] of Object.entries(diskCache)) {
      if (now - entry.fetchedAt > SIMKL_EPISODE_LIST_MAX_AGE_MS) delete diskCache[key];
    }
    await storageWrite(SIMKL_EPISODE_LIST_CACHE_KEY, diskCache);
  }
  return items;
}

export async function syncSimklNow(payload: Record<string, unknown>): Promise<unknown> {
  const token = typeof payload.token === 'string' ? payload.token : undefined;
  const clientId = typeof payload.clientId === 'string' && payload.clientId ? payload.clientId : '';
  if (!token) return { synced: false, error: 'Simkl is not connected' };
  const categories = payload.categories as ImportCategory[] | undefined;
  const force = payload.force === true;
  const wants = (category: ImportCategory) => !categories || categories.includes(category);
  const dryRun = payload.dryRun === true;

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
    ['episodesPlayback', 'tv_shows', 'playback', 'episodes'],
    ['moviesPlayback', 'movies', 'playback', 'movies'],
  ] as const;
  const plan = await coreSimklResourceSyncPlan({
    previous: cache.activities ?? null,
    current: activities,
    resources: resources.map(([key, type, status]) => ({ key, type, status, hasCached: Boolean(cache.resources[key]) })),
  });
  const planByKey = new Map(plan.map((entry) => [entry.key, entry]));
  for (const [key] of resources) {
    if (key === 'episodesPlayback' || key === 'moviesPlayback') {
      debugLog(`syncSimklNow: resource=${key} action=${planByKey.get(key)?.action} force=${force} hasCached=${Boolean(cache.resources[key])}`);
    }
  }
  let nextResources: unknown[];
  try {
    nextResources = await Promise.all(resources.map(async ([key, , status, path]) => {
      const entry = planByKey.get(key);
      if (!force && (!entry || entry.action === 'unchanged')) return cache.resources[key];
      if (status === 'playback') {
        const response = await platformFetch(`https://api.simkl.com/sync/playback/${path}?${query}`, { headers, signal: AbortSignal.timeout(60_000) });
        if (!response.ok) throw new Error(`Simkl sync failed: HTTP ${response.status}`);
        const json = await response.json();
        debugLog(`syncSimklNow: fetched playback key=${key} count=${Array.isArray(json) ? json.length : 'n/a'} raw=${JSON.stringify(json).slice(0, 2000)}`);
        return json;
      }
      if (force) {
        const response = await platformFetch(`https://api.simkl.com/sync/all-items/${path}/${status}?extended=full&episode_watched_at=yes&${query}`, { headers, signal: AbortSignal.timeout(60_000) });
        if (!response.ok) throw new Error(`Simkl sync failed: HTTP ${response.status}`);
        return response.json();
      }
      const resolvedEntry = entry!;
      const dateFrom = resolvedEntry.action === 'delta' && resolvedEntry.dateFrom ? `&date_from=${encodeURIComponent(resolvedEntry.dateFrom)}` : '';
      const response = await platformFetch(`https://api.simkl.com/sync/all-items/${path}/${status}?extended=full&episode_watched_at=yes&${query}${dateFrom}`, { headers, signal: AbortSignal.timeout(60_000) });
      if (!response.ok) throw new Error(`Simkl sync failed: HTTP ${response.status}`);
      const changes = await response.json();
      return resolvedEntry.action === 'full' ? changes : coreSimklMergeDelta(JSON.stringify(cache.resources[key] ?? null), JSON.stringify(changes));
    }));
  } catch (error) {
    return { synced: false, error: error instanceof Error ? error.message : String(error) };
  }
  for (const [index, [key]] of resources.entries()) cache.resources[key] = nextResources[index];
  cache.activities = activities;
  await storageWrite(cacheKey, cache);
  const [shows, movies, wlShows, wlMovies, doneShows, doneMovies, episodesPlayback, moviesPlayback] = nextResources;
  const showsData = JSON.stringify(shows);
  const moviesData = JSON.stringify(movies);
  const rawItems = ((await coreSimklWatchingToItems(showsData, moviesData)) ?? []) as Record<string, unknown>[];
  debugLog(`syncSimklNow: rawItems=${JSON.stringify(rawItems.map((item) => ({ id: item.id, lastEpisodeSeason: item.lastEpisodeSeason, lastEpisodeNumber: item.lastEpisodeNumber, badge: item.continueWatchingBadge, poster: item.poster, background: item.background, simklId: item.simklId })))}`);
  const playbackData = JSON.stringify([
    ...(Array.isArray(episodesPlayback) ? episodesPlayback : []),
    ...(Array.isArray(moviesPlayback) ? moviesPlayback : []),
  ]);
  const mergedItems = ((await coreSimklMergePlaybackProgress(JSON.stringify(rawItems), playbackData)) ?? rawItems) as Record<string, unknown>[];
  const items = wants('continueWatching')
    ? await resolveSimklEpisodeDetails(mergedItems, headers, query).catch(() => mergedItems)
    : mergedItems;
  debugLog(`syncSimklNow: progressItems=${JSON.stringify(items.map((item) => ({ id: item.id, resumeProgressPercent: item.resumeProgressPercent, badge: item.continueWatchingBadge, poster: item.poster, background: item.background, lastEpisodeThumbnail: item.lastEpisodeThumbnail })))}`);
  if (wants('continueWatching') && !dryRun) {
    await replaceExternalContinueWatching({ items, provider: 'simkl', profileKey });
    const { promoteExternalProgress } = await import('./externalSync');
    await promoteExternalProgress(items, 'simkl', profile ?? null);
  }

  const wlShowsData = JSON.stringify(wlShows);
  const wlMoviesData = JSON.stringify(wlMovies);
  const watchlistItems = ((await coreSimklWatchlistToItems(wlShowsData, wlMoviesData)) ?? []) as Record<string, unknown>[];

  const doneShowsData = JSON.stringify(doneShows);
  const doneMoviesData = JSON.stringify(doneMovies);
  const completedItems = ((await coreSimklWatchlistToItems(doneShowsData, doneMoviesData)) ?? []) as Record<string, unknown>[];
  if (!dryRun) await saveProviderLibrary('simkl', { watchlist: watchlistItems, watching: items, completed: completedItems, dropped: [], favorites: [] }, profileKey);

  const watchedMap = ((await coreSimklWatchedToIds(doneShowsData, doneMoviesData)) ?? {}) as Record<string, boolean>;

  const watchlistCount = watchlistItems.length;
  const watchedCount = Object.values(watchedMap).filter(Boolean).length;

  return { synced: true, provider: 'simkl', continueWatchingCount: items.length, watchlistCount, watchedCount };
}

export async function dropSimklPlaybackProgress(showId: string): Promise<void> {
  const profile = await loadActiveProfile();
  const token = profile?.simklAccessToken;
  if (!token) return;

  const clientId = await getOAuthClientId('simkl');
  const query = `client_id=${encodeURIComponent(clientId)}&app-name=fluxa&app-version=${encodeURIComponent(_appVersion)}`;
  const headers: HeadersInit = {
    'Authorization': `Bearer ${token}`,
    'Content-Type': 'application/json',
    'User-Agent': `Fluxa Desktop/${_appVersion}`,
  };

  try {
    const response = await platformFetch(`https://api.simkl.com/sync/playback?${query}`, { headers });
    if (!response.ok) return;
    const playbackItems = await response.json().catch(() => []);

    const deleteIds = await coreInvoke<number[]>('simklPlaybackDeleteIds', JSON.stringify({ contentId: showId, items: Array.isArray(playbackItems) ? playbackItems : [] })) ?? [];
    await Promise.all(deleteIds.map((id) =>
      platformFetch(`https://api.simkl.com/sync/playback/${id}?${query}`, {
        method: 'DELETE',
        headers,
      }).then(() => undefined).catch(() => undefined),
    ));
  } catch {
  }
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
