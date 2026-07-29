import {
  coreBuildTraktIds,
  coreInvoke,
  coreMergeExternalWatched,
  coreMergeExternalWatchlist,
  coreTraktActivityDiff,
  coreTraktMarkWatchedBody,
  coreTraktPlaybackItemsDedup,
  coreTraktPlaybackItemsToLibrary,
  coreTraktWatchedShowsToItems,
  coreTraktWatchlistToItems,
  coreTraktWatchedToIds,
  httpExecuteText,
  storageRead,
  storageWrite,
} from './engine';
import { loadLibrary, saveLibrary, persistStatusListMerge, persistWatchedMerge, profileStorageKey } from './libraryOps';
import { platformFetch } from './httpClient';
import { refreshTraktProfile, traktHeaders } from './traktSync';
import { enrichWithAddonMeta, replaceExternalContinueWatching } from './externalSyncUtils';
import { saveProviderLibrary } from './providerLibraries';
import type { ImportCategory } from './importCategories';

type TraktDeltaCache = {
  activities?: Record<string, unknown>;
  playbackItems?: Record<string, unknown>[];
  watchlistMovies?: Record<string, unknown>[];
  watchlistShows?: Record<string, unknown>[];
  watchedMovies?: Record<string, unknown>[];
  watchedShows?: Record<string, unknown>[];
};

async function fetchAllPages(url: string, headers: HeadersInit, limit: number): Promise<Record<string, unknown>[]> {
  type PaginationPlan = { items: Record<string, unknown>[]; done: boolean; page: number; requestUrl?: string | null };
  let plan = await coreInvoke<PaginationPlan>('providerPaginationPlan', JSON.stringify({ baseUrl: url, limit }));
  while (plan && !plan.done && plan.requestUrl) {
    const res = await platformFetch(plan.requestUrl, { headers });
    const data = res.ok ? await res.json().catch(() => []) : [];
    const pageItems = Array.isArray(data) ? data : [];
    const pageCount = Number(res.headers.get('x-pagination-page-count'));
    plan = await coreInvoke<PaginationPlan>('providerPaginationPlan', JSON.stringify({
      baseUrl: url,
      limit,
      page: plan.page,
      items: plan.items,
      pageItems,
      pageCount: Number.isFinite(pageCount) ? pageCount : null,
      responseOk: res.ok,
    }));
  }
  return plan?.items ?? [];
}

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

export async function syncTraktNow(payload: Record<string, unknown>): Promise<unknown> {
  const profile = payload.profile as import('./types').UserProfile | undefined;
  const profileKey = profile ? profileStorageKey(profile) : undefined;
  let refreshedProfile = profile ? await refreshTraktProfile(profile).catch(() => profile) : undefined;
  let token = refreshedProfile?.traktAccessToken ?? (typeof payload.token === 'string' ? payload.token : undefined);
  const clientId = typeof payload.clientId === 'string' ? payload.clientId : '';
  if (!token) return { synced: false, error: 'Trakt is not connected' };
  const categories = payload.categories as ImportCategory[] | undefined;
  const wants = (category: ImportCategory) => !categories || categories.includes(category);
  const dryRun = payload.dryRun === true;

  let headers = traktHeaders(token, clientId);
  const activeToken = token;
  const fetchWithRefresh = async (request: (h: HeadersInit) => Promise<Response>): Promise<Response> => {
    let response = await request(headers);
    if ((response.status === 401 || response.status === 403) && profile) {
      refreshedProfile = await refreshTraktProfile(profile, true).catch(() => profile);
      headers = traktHeaders(refreshedProfile.traktAccessToken ?? activeToken, clientId);
      response = await request(headers);
    }
    return response;
  };

  const profileId = typeof profile?.id === 'string' ? profile.id : 'default';
  const cacheKey = `trakt_delta_cache_${profileId}`;
  const cache = (await storageRead<TraktDeltaCache>(cacheKey)) ?? {};

  let activities: Record<string, unknown> | undefined;
  try {
    const response = await fetchWithRefresh((h) => platformFetch('https://api.trakt.tv/sync/last_activities', { headers: h }));
    if (response.ok) activities = (await response.json().catch(() => undefined)) as Record<string, unknown> | undefined;
  } catch {}

  const {
    playbackChanged,
    watchlistMoviesChanged,
    watchlistShowsChanged,
    watchedMoviesChanged,
    watchedShowsChanged,
  } = await coreTraktActivityDiff({
    previous: cache.activities ?? null,
    current: activities ?? null,
    hasPlayback: Boolean(cache.playbackItems),
    hasWatchlistMovies: Boolean(cache.watchlistMovies),
    hasWatchlistShows: Boolean(cache.watchlistShows),
    hasWatchedMovies: Boolean(cache.watchedMovies),
    hasWatchedShows: Boolean(cache.watchedShows),
  });

  let playbackItems: Record<string, unknown>[];
  if (playbackChanged) {
    const responses = await Promise.all([
      fetchWithRefresh((h) => platformFetch('https://api.trakt.tv/sync/playback/movies', { headers: h })),
      fetchWithRefresh((h) => platformFetch('https://api.trakt.tv/sync/playback/episodes', { headers: h })),
    ]);
    const failedResponse = responses.find((response) => !response.ok);
    if (failedResponse) {
      return { synced: false, error: `Trakt sync failed: HTTP ${failedResponse.status}` };
    }
    const playbackPages = await Promise.all(responses.map((response) => response.json().catch(() => [])));
    playbackItems = playbackPages.flatMap((page) => Array.isArray(page) ? page : []);
  } else {
    playbackItems = cache.playbackItems ?? [];
  }
  const allItems = ((await coreTraktPlaybackItemsToLibrary(JSON.stringify(playbackItems))) ?? []) as Record<string, unknown>[];
  let items = await enrichWithAddonMeta(allItems);

  let watchlistCount = 0;
  let watchedCount = 0;
  try {
    const [watchlistMovies, watchlistShows, watchedMovies, watchedShows] = await Promise.all([
      watchlistMoviesChanged ? fetchAllPages('https://api.trakt.tv/users/me/watchlist/movies/rank', headers, 250) : Promise.resolve(cache.watchlistMovies ?? []),
      watchlistShowsChanged ? fetchAllPages('https://api.trakt.tv/users/me/watchlist/shows/rank', headers, 250) : Promise.resolve(cache.watchlistShows ?? []),
      watchedMoviesChanged ? fetchAllPages('https://api.trakt.tv/users/me/watched/movies', headers, 250) : Promise.resolve(cache.watchedMovies ?? []),
      watchedShowsChanged ? fetchAllPages('https://api.trakt.tv/users/me/watched/shows?extended=full', headers, 100) : Promise.resolve(cache.watchedShows ?? []),
    ]);

    const watchedShowItems = ((await coreTraktWatchedShowsToItems(JSON.stringify(watchedShows))) ?? []) as Record<string, unknown>[];
    const rawItems = ((await coreTraktPlaybackItemsDedup(JSON.stringify([...allItems, ...watchedShowItems]))) ?? []) as Record<string, unknown>[];
    items = await enrichWithAddonMeta(rawItems);

    const watchlistItems = ((await coreTraktWatchlistToItems(JSON.stringify(watchlistMovies), JSON.stringify(watchlistShows))) ?? []) as Record<string, unknown>[];
    watchlistCount = watchlistItems.length;
    if (wants('watchlist') && !dryRun) await mergeExternalWatchlist(watchlistItems, profileKey);

    if (!dryRun) await saveProviderLibrary('trakt', { watchlist: watchlistItems, watching: items, completed: [], dropped: [] }, profileKey);

    const watchedIds = ((await coreTraktWatchedToIds(JSON.stringify(watchedMovies), JSON.stringify(watchedShows))) ?? {}) as Record<string, boolean>;
    watchedCount = Object.keys(watchedIds).length;
    if (wants('watched') && !dryRun) await mergeExternalWatched(watchedIds, profileKey);

    if (activities) {
      await storageWrite(cacheKey, { activities, playbackItems, watchlistMovies, watchlistShows, watchedMovies, watchedShows } satisfies TraktDeltaCache);
    }
  } catch {}

  if (wants('continueWatching') && !dryRun) {
    await replaceExternalContinueWatching({ items, provider: 'trakt', profileKey });
    const { promoteExternalProgress } = await import('./externalSync');
    await promoteExternalProgress(items, 'trakt', refreshedProfile ?? null);
  }

  return { synced: true, provider: 'trakt', continueWatchingCount: items.length, watchlistCount, watchedCount };
}

export async function fetchTraktCalendarItems(
  token: string,
  clientId: string,
  calendarMonth?: { year: number; month: number },
): Promise<Record<string, unknown>[]> {
  const headers = traktHeaders(token, clientId) as Record<string, string>;
  const start = calendarMonth
    ? new Date(calendarMonth.year, calendarMonth.month - 1, 1)
    : new Date();
  if (!calendarMonth) start.setDate(start.getDate() - 14);
  const startIso = start.toISOString().slice(0, 10);
  const days = calendarMonth
    ? new Date(calendarMonth.year, calendarMonth.month, 0).getDate()
    : 90;

  const readCalendar = async (kind: 'shows' | 'movies'): Promise<unknown[]> => {
    const response = await httpExecuteText(
      `https://api.trakt.tv/calendars/my/${kind}/${startIso}/${days}?extended=full`,
      'GET',
      headers,
    );
    if (response.statusCode < 200 || response.statusCode >= 300) {
      throw new Error(`Trakt calendar ${kind}: HTTP ${response.statusCode}`);
    }
    const entries: unknown = JSON.parse(response.body);
    if (!Array.isArray(entries)) throw new Error(`Trakt calendar ${kind}: invalid response`);
    return entries;
  };

  const [shows, movies] = await Promise.all([readCalendar('shows'), readCalendar('movies')]);

  return (await coreInvoke<Record<string, unknown>[]>('providerCalendarItems', JSON.stringify({ provider: 'trakt', shows, movies }))) ?? [];
}

export async function pushMarkWatchedTrakt(
  videoIds: string[],
  watched: boolean,
  token: string,
  clientId: string,
  watchedAtMs?: number,
): Promise<void> {
  const body = await coreTraktMarkWatchedBody(JSON.stringify({ videoIds, watchedAtMs }));
  if (!body) return;
  const headers = traktHeaders(token, clientId);
  const endpoint = watched ? '/sync/history' : '/sync/history/remove';
  await platformFetch(`https://api.trakt.tv${endpoint}`, { method: 'POST', headers, body: JSON.stringify(body) });
}

export async function pushWatchlistTrakt(
  id: string,
  contentType: string,
  command: 'add' | 'remove',
  token: string,
  clientId: string,
): Promise<void> {
  const headers = traktHeaders(token, clientId);
  const ids = await coreBuildTraktIds(id);
  if (!ids) return;
  const endpoint = command === 'add' ? '/sync/watchlist' : '/sync/watchlist/remove';
  const body = contentType === 'series' ? { shows: [{ ids }] } : { movies: [{ ids }] };
  await platformFetch(`https://api.trakt.tv${endpoint}`, { method: 'POST', headers, body: JSON.stringify(body) });
}
