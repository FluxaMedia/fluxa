import {
  coreBuildTraktIds,
  coreInvoke,
  coreTraktActivityDiff,
  coreTraktMarkWatchedBody,
  coreTraktPlaybackItemsDedup,
  coreTraktPlaybackItemsToLibrary,
  coreTraktUpNextToItems,
  coreTraktWatchlistToItems,
  coreTraktWatchedToIds,
  httpExecuteText,
  storageRead,
  storageWrite,
} from './engine';
import { loadPrefs, profileStorageKey } from './libraryOps';
import { saveProviderLibrary } from './providerLibraries';
import { platformFetch } from './httpClient';
import { refreshTraktProfile, traktHeaders } from './traktSync';
import { replaceExternalContinueWatching } from './externalSyncUtils';
import type { ImportCategory } from './importCategories';
import { platformInvoke as invoke } from '../platform/invoke';

function debugLog(msg: string) {
  void invoke('debug_log', { msg }).catch(() => {});
}

const TRAKT_CACHE_VERSION = 3;

type TraktDeltaCache = {
  version?: number;
  activities?: Record<string, unknown>;
  playbackItems?: Record<string, unknown>[];
  watchlistMovies?: Record<string, unknown>[];
  watchlistShows?: Record<string, unknown>[];
  watchedMovies?: Record<string, unknown>[];
  watchedShows?: Record<string, unknown>[];
};

const TRAKT_SHOW_ARTWORK_CACHE_KEY = 'trakt_show_artwork_cache';
const TRAKT_SHOW_ARTWORK_TTL_MS = 24 * 60 * 60 * 1000;
const TRAKT_SHOW_ARTWORK_MAX_AGE_MS = 30 * 24 * 60 * 60 * 1000;

type TraktShowArtwork = { poster: string | null; background: string | null; logo: string | null };
type TraktShowArtworkCacheEntry = { fetchedAt: number; artwork: TraktShowArtwork };
type TraktShowArtworkCache = Record<string, TraktShowArtworkCacheEntry>;

async function resolveTraktShowArtwork(items: Record<string, unknown>[], headers: HeadersInit): Promise<Record<string, unknown>[]> {
  const needsArtwork = items.filter((item) => item.type === 'series' && typeof item.id === 'string' && !item.poster);
  if (needsArtwork.length === 0) return items;

  const diskCache = (await storageRead<TraktShowArtworkCache>(TRAKT_SHOW_ARTWORK_CACHE_KEY)) ?? {};
  let diskCacheDirty = false;
  const now = Date.now();
  const inFlight = new Map<string, Promise<TraktShowArtwork>>();
  const fetchArtwork = (id: string) => {
    let pending = inFlight.get(id);
    if (pending) return pending;
    const cached = diskCache[id];
    if (cached && now - cached.fetchedAt < TRAKT_SHOW_ARTWORK_TTL_MS) {
      pending = Promise.resolve(cached.artwork);
    } else {
      pending = platformFetch(`https://api.trakt.tv/shows/${encodeURIComponent(id)}?extended=full,images`, { headers })
        .then((res) => (res.ok ? res.json() : null))
        .then((show: Record<string, unknown> | null) => {
          const images = show?.images as Record<string, unknown> | undefined;
          const firstOf = (kind: string) => {
            const value = images?.[kind];
            return Array.isArray(value) && typeof value[0] === 'string' ? `https://${value[0]}` : null;
          };
          const artwork: TraktShowArtwork = { poster: firstOf('poster'), background: firstOf('fanart'), logo: firstOf('logo') };
          diskCache[id] = { fetchedAt: now, artwork };
          diskCacheDirty = true;
          return artwork;
        })
        .catch(() => ({ poster: null, background: null, logo: null }));
    }
    inFlight.set(id, pending);
    return pending;
  };

  const CONCURRENCY = 4;
  let cursor = 0;
  async function worker() {
    while (cursor < needsArtwork.length) {
      const item = needsArtwork[cursor++];
      const artwork = await fetchArtwork(item.id as string);
      if (artwork.poster) item.poster = artwork.poster;
      if (artwork.background) item.background = artwork.background;
      if (artwork.logo) item.logo = artwork.logo;
    }
  }
  await Promise.all(Array.from({ length: Math.min(CONCURRENCY, needsArtwork.length) }, worker));

  if (diskCacheDirty) {
    for (const [key, entry] of Object.entries(diskCache)) {
      if (now - entry.fetchedAt > TRAKT_SHOW_ARTWORK_MAX_AGE_MS) delete diskCache[key];
    }
    await storageWrite(TRAKT_SHOW_ARTWORK_CACHE_KEY, diskCache);
  }
  return items;
}

async function fetchAllPages(url: string, headers: HeadersInit, limit: number): Promise<Record<string, unknown>[]> {
  type PaginationPlan = { items: Record<string, unknown>[]; done: boolean; page: number; requestUrl?: string | null };
  let plan = await coreInvoke<PaginationPlan>('providerPaginationPlan', JSON.stringify({ baseUrl: url, limit }));
  while (plan && !plan.done && plan.requestUrl) {
    const res = await platformFetch(plan.requestUrl, { headers });
    const data = res.ok ? await res.json().catch(() => []) : [];
    const pageItems = Array.isArray(data) ? data : [];
    const pageCount = Number(res.headers.get('x-pagination-page-count'));
    plan = await coreInvoke<PaginationPlan>(
      'providerPaginationPlan',
      JSON.stringify({
        baseUrl: url,
        limit,
        page: plan.page,
        items: plan.items,
        pageItems,
        pageCount: Number.isFinite(pageCount) ? pageCount : null,
        responseOk: res.ok,
      }),
    );
  }
  return plan?.items ?? [];
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
  const readOnly = payload.readOnly === true;
  const force = payload.force === true;

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
  const cachedRaw = (await storageRead<TraktDeltaCache>(cacheKey)) ?? {};
  const cacheStale = cachedRaw.version !== TRAKT_CACHE_VERSION;
  const cache = cacheStale ? {} : cachedRaw;
  debugLog(`syncTraktNow: cachedVersion=${cachedRaw.version} expected=${TRAKT_CACHE_VERSION} cacheStale=${cacheStale}`);

  let activities: Record<string, unknown> | undefined;
  try {
    const response = await fetchWithRefresh((h) => platformFetch('https://api.trakt.tv/sync/last_activities', { headers: h }));
    if (response.ok) activities = (await response.json().catch(() => undefined)) as Record<string, unknown> | undefined;
  } catch {}

  const { playbackChanged, watchlistMoviesChanged, watchlistShowsChanged, watchedMoviesChanged, watchedShowsChanged } =
    await coreTraktActivityDiff({
      previous: cache.activities ?? null,
      current: activities ?? null,
      hasPlayback: !force && Boolean(cache.playbackItems),
      hasWatchlistMovies: !force && Boolean(cache.watchlistMovies),
      hasWatchlistShows: !force && Boolean(cache.watchlistShows),
      hasWatchedMovies: !force && Boolean(cache.watchedMovies),
      hasWatchedShows: !force && Boolean(cache.watchedShows),
    });

  let playbackItems: Record<string, unknown>[];
  if (playbackChanged) {
    const responses = await Promise.all([
      fetchWithRefresh((h) => platformFetch('https://api.trakt.tv/sync/playback/movies?extended=full,images', { headers: h })),
      fetchWithRefresh((h) => platformFetch('https://api.trakt.tv/sync/playback/episodes?extended=full,images', { headers: h })),
    ]);
    const failedResponse = responses.find((response) => !response.ok);
    if (failedResponse) {
      return { synced: false, error: `Trakt sync failed: HTTP ${failedResponse.status}` };
    }
    const playbackPages = await Promise.all(responses.map((response) => response.json().catch(() => [])));
    playbackItems = playbackPages.flatMap((page) => (Array.isArray(page) ? page : []));
  } else {
    playbackItems = cache.playbackItems ?? [];
  }
  const allItems = ((await coreTraktPlaybackItemsToLibrary(JSON.stringify(playbackItems))) ?? []) as Record<string, unknown>[];
  let items = allItems;

  const prefs = await loadPrefs();
  const librarySource = String((prefs as Record<string, unknown>).integrationLibrarySource ?? 'local');
  const isSelectedLibrarySource = librarySource === 'trakt';

  let watchlistCount = 0;
  let watchedCount = 0;
  let providerSnapshot = {
    watchlist: [] as Record<string, unknown>[],
    watching: items,
    completed: [] as Record<string, unknown>[],
    dropped: [] as Record<string, unknown>[],
    favorites: [] as Record<string, unknown>[],
  };
  try {
    const [watchlistMovies, watchlistShows, watchedMovies, watchedShows] = await Promise.all([
      watchlistMoviesChanged
        ? fetchAllPages('https://api.trakt.tv/users/me/watchlist/movies/rank?extended=full,images', headers, 250)
        : Promise.resolve(cache.watchlistMovies ?? []),
      watchlistShowsChanged
        ? fetchAllPages('https://api.trakt.tv/users/me/watchlist/shows/rank?extended=full,images', headers, 250)
        : Promise.resolve(cache.watchlistShows ?? []),
      watchedMoviesChanged
        ? fetchAllPages('https://api.trakt.tv/users/me/watched/movies?extended=full,images', headers, 250)
        : Promise.resolve(cache.watchedMovies ?? []),
      watchedShowsChanged
        ? fetchAllPages('https://api.trakt.tv/users/me/watched/shows?extended=full,images', headers, 100)
        : Promise.resolve(cache.watchedShows ?? []),
    ]);

    const upNext = await fetchAllPages('https://api.trakt.tv/sync/progress/up_next?extended=full,images', headers, 100);
    const upNextItems = ((await coreTraktUpNextToItems(JSON.stringify(upNext))) ?? []) as Record<string, unknown>[];
    const rawItems = ((await coreTraktPlaybackItemsDedup(JSON.stringify([...allItems, ...upNextItems]))) ?? []) as Record<
      string,
      unknown
    >[];
    items = await resolveTraktShowArtwork(rawItems, headers);

    const watchlistItemsRaw = ((await coreTraktWatchlistToItems(JSON.stringify(watchlistMovies), JSON.stringify(watchlistShows))) ??
      []) as Record<string, unknown>[];
    const watchedIds = ((await coreTraktWatchedToIds(JSON.stringify(watchedMovies), JSON.stringify(watchedShows))) ?? {}) as Record<
      string,
      boolean
    >;
    const completedItemsRaw = ((await coreTraktWatchlistToItems(JSON.stringify(watchedMovies), JSON.stringify(watchedShows))) ??
      []) as Record<string, unknown>[];
    const favoriteItemsRaw = isSelectedLibrarySource ? await fetchTraktFavorites(token, clientId).catch(() => []) : [];
    const [watchlistItems, completedItems, favoriteItems] = await Promise.all([
      resolveTraktShowArtwork(watchlistItemsRaw, headers),
      resolveTraktShowArtwork(completedItemsRaw, headers),
      resolveTraktShowArtwork(favoriteItemsRaw, headers),
    ]);
    providerSnapshot = { watchlist: watchlistItems, watching: items, completed: completedItems, dropped: [], favorites: favoriteItems };
    debugLog(
      `syncTraktNow: completedItems=${JSON.stringify(completedItems.map((item) => ({ id: item.id, name: item.name, poster: item.poster, background: item.background })))}`,
    );
    watchlistCount = watchlistItems.length;
    watchedCount = Object.values(watchedIds).filter(Boolean).length;

    if (!dryRun && !readOnly) {
      await saveProviderLibrary(
        'trakt',
        { watchlist: watchlistItems, watching: items, completed: completedItems, dropped: [], favorites: favoriteItems },
        profileKey,
      );
    }

    if (activities) {
      await storageWrite(cacheKey, {
        version: TRAKT_CACHE_VERSION,
        activities,
        playbackItems,
        watchlistMovies,
        watchlistShows,
        watchedMovies,
        watchedShows,
      } satisfies TraktDeltaCache);
    }
  } catch (error) {
    debugLog(
      `syncTraktNow: watchlist/completed block threw ${error instanceof Error ? `${error.message}\n${error.stack}` : String(error)}`,
    );
  }

  if (wants('continueWatching') && !dryRun && !readOnly) {
    await replaceExternalContinueWatching({ items, provider: 'trakt', profileKey });
    const { promoteExternalProgress } = await import('./externalSync');
    await promoteExternalProgress(items, 'trakt', refreshedProfile ?? null);
  }

  return {
    synced: true,
    provider: 'trakt',
    continueWatchingCount: items.length,
    watchlistCount,
    watchedCount,
    snapshot: providerSnapshot,
  };
}

export async function fetchTraktCalendarItems(
  token: string,
  clientId: string,
  calendarMonth?: { year: number; month: number },
): Promise<Record<string, unknown>[]> {
  const headers = traktHeaders(token, clientId) as Record<string, string>;
  const start = calendarMonth ? new Date(calendarMonth.year, calendarMonth.month - 1, 1) : new Date();
  if (!calendarMonth) start.setDate(start.getDate() - 14);
  const startIso = start.toISOString().slice(0, 10);
  const days = calendarMonth ? new Date(calendarMonth.year, calendarMonth.month, 0).getDate() : 90;

  const readCalendar = async (kind: 'shows' | 'movies'): Promise<unknown[]> => {
    const response = await httpExecuteText(`https://api.trakt.tv/calendars/my/${kind}/${startIso}/${days}?extended=full`, 'GET', headers);
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

export async function pushFavoriteTrakt(
  id: string,
  contentType: string,
  command: 'add' | 'remove',
  token: string,
  clientId: string,
): Promise<void> {
  const headers = traktHeaders(token, clientId);
  const ids = await coreBuildTraktIds(id);
  if (!ids) return;
  const endpoint = command === 'add' ? '/sync/favorites' : '/sync/favorites/remove';
  const body = contentType === 'series' ? { shows: [{ ids }] } : { movies: [{ ids }] };
  await platformFetch(`https://api.trakt.tv${endpoint}`, { method: 'POST', headers, body: JSON.stringify(body) });
}

export async function fetchTraktFavorites(token: string, clientId: string): Promise<Record<string, unknown>[]> {
  const headers = traktHeaders(token, clientId);
  const [movies, shows] = await Promise.all([
    fetchAllPages('https://api.trakt.tv/sync/favorites/movies/added/desc?extended=full,images', headers, 250),
    fetchAllPages('https://api.trakt.tv/sync/favorites/shows/added/desc?extended=full,images', headers, 250),
  ]);
  return ((await coreTraktWatchlistToItems(JSON.stringify(movies), JSON.stringify(shows))) ?? []) as Record<string, unknown>[];
}
