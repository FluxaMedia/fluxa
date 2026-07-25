import {
  coreBuildTraktIds,
  coreInvoke,
  coreMergeExternalWatched,
  coreMergeExternalWatchlist,
  coreTraktMarkWatchedBody,
  coreTraktPlaybackItemsDedup,
  coreTraktPlaybackItemsToLibrary,
  coreTraktWatchlistToItems,
  coreTraktWatchedToIds,
} from './engine';
import { loadLibrary, saveLibrary, persistStatusListMerge, persistWatchedMerge } from './libraryOps';
import { platformFetch } from './httpClient';
import { refreshTraktProfile, traktHeaders } from './traktSync';
import { enrichWithAddonMeta, replaceExternalContinueWatching } from './externalSyncUtils';
import { saveProviderLibrary } from './providerLibraries';

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

export async function syncTraktNow(payload: Record<string, unknown>): Promise<unknown> {
  const profile = payload.profile as import('./types').UserProfile | undefined;
  const refreshedProfile = profile ? await refreshTraktProfile(profile).catch(() => profile) : undefined;
  const token = refreshedProfile?.traktAccessToken ?? (typeof payload.token === 'string' ? payload.token : undefined);
  const clientId = typeof payload.clientId === 'string' ? payload.clientId : '';
  if (!token) return { synced: false, error: 'Trakt is not connected' };

  const headers = traktHeaders(token, clientId);
  const response = await platformFetch('https://api.trakt.tv/sync/playback', { headers });
  if (!response.ok) {
    return { synced: false, error: `Trakt sync failed: HTTP ${response.status}` };
  }
  const playbackItems = await response.json();
  const allItems = Array.isArray(playbackItems)
    ? ((await coreTraktPlaybackItemsToLibrary(JSON.stringify(playbackItems))) ?? []) as Record<string, unknown>[]
    : [];

  const rawItems = ((await coreTraktPlaybackItemsDedup(JSON.stringify(allItems))) ?? []) as Record<string, unknown>[];

  const items = await enrichWithAddonMeta(rawItems);
  await replaceExternalContinueWatching({ items, provider: 'trakt' });
  const { promoteExternalProgress } = await import('./externalSync');
  await promoteExternalProgress(items, 'trakt', refreshedProfile ?? null);

  let watchlistCount = 0;
  try {
    const [watchlistMovies, watchlistShows, watchedMovies, watchedShows] = await Promise.all([
      fetchAllPages('https://api.trakt.tv/users/me/watchlist/movies', headers, 250),
      fetchAllPages('https://api.trakt.tv/users/me/watchlist/shows', headers, 250),
      fetchAllPages('https://api.trakt.tv/users/me/watched/movies', headers, 250),
      fetchAllPages('https://api.trakt.tv/users/me/watched/shows?extended=progress', headers, 100),
    ]);

    const watchlistItems = ((await coreTraktWatchlistToItems(JSON.stringify(watchlistMovies), JSON.stringify(watchlistShows))) ?? []) as Record<string, unknown>[];
    watchlistCount = watchlistItems.length;
    await mergeExternalWatchlist(watchlistItems);

    await saveProviderLibrary('trakt', { watchlist: watchlistItems, watching: items, completed: [], dropped: [] });

    const watchedIds = ((await coreTraktWatchedToIds(JSON.stringify(watchedMovies), JSON.stringify(watchedShows))) ?? {}) as Record<string, boolean>;
    await mergeExternalWatched(watchedIds);
  } catch {}

  return { synced: true, provider: 'trakt', continueWatchingCount: items.length, watchlistCount };
}

export async function fetchTraktCalendarItems(token: string, clientId: string): Promise<Record<string, unknown>[]> {
  const headers = traktHeaders(token, clientId);
  const start = new Date();
  start.setDate(start.getDate() - 14);
  const startIso = start.toISOString().slice(0, 10);
  const days = 90;

  const [shows, movies] = await Promise.all([
    platformFetch(`https://api.trakt.tv/calendars/my/shows/${startIso}/${days}`, { headers })
      .then((res) => (res.ok ? res.json() : [])).catch(() => []),
    platformFetch(`https://api.trakt.tv/calendars/my/movies/${startIso}/${days}`, { headers })
      .then((res) => (res.ok ? res.json() : [])).catch(() => []),
  ]);

  return (await coreInvoke<Record<string, unknown>[]>('providerCalendarItems', JSON.stringify({ provider: 'trakt', shows, movies }))) ?? [];
}

export async function pushMarkWatchedTrakt(
  videoIds: string[],
  watched: boolean,
  token: string,
  clientId: string,
): Promise<void> {
  const body = await coreTraktMarkWatchedBody(JSON.stringify(videoIds));
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
