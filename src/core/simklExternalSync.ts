import {
  coreMergeExternalWatched,
  coreMergeExternalWatchlist,
  coreInvoke,
  coreSimklWatchedToIds,
  coreSimklWatchingToItems,
  coreSimklWatchlistToItems,
} from './engine';
import { loadLibrary, saveLibrary, persistStatusListMerge, persistWatchedMerge } from './libraryOps';
import { _appVersion, platformFetch } from './httpClient';
import { enrichWithAddonMeta, replaceExternalContinueWatching } from './externalSyncUtils';
import { saveProviderLibrary } from './providerLibraries';

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
    'simkl-api-key': clientId,
    'Content-Type': 'application/json',
  };
  const query = `client_id=${encodeURIComponent(clientId)}&app-name=fluxa&app-version=${encodeURIComponent(_appVersion)}`;

  const syncSignal = () => AbortSignal.timeout(60_000);
  const [showsRes, moviesRes, wlShowsRes, wlMoviesRes, doneShowsRes, doneMoviesRes] = await Promise.all([
    platformFetch(`https://api.simkl.com/sync/all-items/shows/watching?extended=full&${query}`, { headers, signal: syncSignal() }),
    platformFetch(`https://api.simkl.com/sync/all-items/movies/watching?extended=full&${query}`, { headers, signal: syncSignal() }),
    platformFetch(`https://api.simkl.com/sync/all-items/shows/plantowatch?extended=full&${query}`, { headers, signal: syncSignal() }),
    platformFetch(`https://api.simkl.com/sync/all-items/movies/plantowatch?extended=full&${query}`, { headers, signal: syncSignal() }),
    platformFetch(`https://api.simkl.com/sync/all-items/shows/completed?extended=full&${query}`, { headers, signal: syncSignal() }),
    platformFetch(`https://api.simkl.com/sync/all-items/movies/completed?extended=full&${query}`, { headers, signal: syncSignal() }),
  ]);

  const required = [showsRes, moviesRes, wlShowsRes, wlMoviesRes, doneShowsRes, doneMoviesRes];
  const failed = required.find((response) => !response.ok);
  if (failed) return { synced: false, error: `Simkl sync failed: HTTP ${failed.status}` };

  const showsData = JSON.stringify(await showsRes.json());
  const moviesData = JSON.stringify(await moviesRes.json());
  const rawItems = ((await coreSimklWatchingToItems(showsData, moviesData)) ?? []) as Record<string, unknown>[];
  const items = await enrichWithAddonMeta(rawItems);
  await replaceExternalContinueWatching({ items, provider: 'simkl' });
  const { promoteExternalProgress } = await import('./externalSync');
  await promoteExternalProgress(items, 'simkl', payload.profile as import('./types').UserProfile | null);

  const wlShowsData = JSON.stringify(await wlShowsRes.json());
  const wlMoviesData = JSON.stringify(await wlMoviesRes.json());
  const watchlistItems = ((await coreSimklWatchlistToItems(wlShowsData, wlMoviesData)) ?? []) as Record<string, unknown>[];
  await mergeExternalWatchlist(watchlistItems);
  await saveProviderLibrary('simkl', { watchlist: watchlistItems, watching: items, completed: [], dropped: [] });

  const doneShowsData = JSON.stringify(await doneShowsRes.json());
  const doneMoviesData = JSON.stringify(await doneMoviesRes.json());
  const watchedMap = ((await coreSimklWatchedToIds(doneShowsData, doneMoviesData)) ?? {}) as Record<string, boolean>;
  await mergeExternalWatched(watchedMap);

  return { synced: true, provider: 'simkl', continueWatchingCount: items.length, watchlistCount: watchlistItems.length, watchedCount: Object.keys(watchedMap).length };
}

export async function fetchSimklCalendarItems(token: string, clientId: string): Promise<Record<string, unknown>[]> {
  const headers: HeadersInit = {
    'Authorization': `Bearer ${token}`,
    'simkl-api-key': clientId,
    'Content-Type': 'application/json',
  };
  const start = new Date();
  start.setDate(start.getDate() - 14);
  const startIso = start.toISOString().slice(0, 10);
  const days = 90;
  const query = `client_id=${encodeURIComponent(clientId)}&app-name=fluxa&app-version=${encodeURIComponent(_appVersion)}`;

  const [shows, movies] = await Promise.all([
    platformFetch(`https://api.simkl.com/calendar/shows/${startIso}/${days}?extended=full&${query}`, { headers })
      .then((res) => (res.ok ? res.json() : [])).catch(() => []),
    platformFetch(`https://api.simkl.com/calendar/movies/${startIso}/${days}?extended=full&${query}`, { headers })
      .then((res) => (res.ok ? res.json() : [])).catch(() => []),
  ]);

  return (await coreInvoke<Record<string, unknown>[]>('providerCalendarItems', JSON.stringify({ provider: 'simkl', shows, movies }))) ?? [];
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
    'simkl-api-key': clientId,
    'Content-Type': 'application/json',
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
    'simkl-api-key': clientId,
    'Content-Type': 'application/json',
  };
  const body = await coreInvoke<Record<string, unknown>>('simklWatchlistBody', JSON.stringify({ id, contentType, command }));
  if (!body) return;
  const endpoint = command === 'add' ? '/sync/add-to-list' : '/sync/history/remove';
  await platformFetch(`https://api.simkl.com${endpoint}?client_id=${encodeURIComponent(clientId)}&app-name=fluxa&app-version=${encodeURIComponent(_appVersion)}`, {
    method: 'POST', headers: simklHeaders, body: JSON.stringify(body),
  });
}
