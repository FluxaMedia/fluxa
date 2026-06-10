import {
  coreBuildTraktIds,
  coreMergeExternalWatched,
  coreMergeExternalWatchlist,
  coreParseVideoId,
  coreTraktPlaybackItemsToLibrary,
} from './engine';
import { loadLibrary, saveLibrary } from './libraryOps';
import { platformFetch } from './httpClient';
import { traktHeaders } from './traktSync';
import { enrichWithAddonMeta, replaceExternalContinueWatching } from './externalSyncUtils';

async function mergeExternalWatchlist(externalItems: Record<string, unknown>[]): Promise<void> {
  const lib = await loadLibrary();
  const local = (lib.watchlist as Record<string, unknown>[] | undefined) ?? [];
  const mergedJson = await coreMergeExternalWatchlist(JSON.stringify(local), JSON.stringify(externalItems));
  const mergedList = mergedJson as Record<string, unknown>[];
  if (mergedList.length > local.length) {
    lib.watchlist = mergedList;
    await saveLibrary(lib);
  }
}

async function mergeExternalWatched(externalWatched: Record<string, boolean>): Promise<void> {
  const lib = await loadLibrary();
  const local = (lib.watched as Record<string, boolean> | undefined) ?? {};
  const merged = await coreMergeExternalWatched(JSON.stringify(local), JSON.stringify(externalWatched));
  lib.watched = merged;
  await saveLibrary(lib);
}

export async function syncTraktNow(payload: Record<string, unknown>): Promise<unknown> {
  const token = typeof payload.token === 'string' ? payload.token : undefined;
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

  const bestByShowId = new Map<string, Record<string, unknown>>();
  for (const item of allItems) {
    const id = String(item.id ?? '');
    const existing = bestByShowId.get(id);
    if (!existing) {
      bestByShowId.set(id, item);
    } else {
      const curTime = new Date(String(item.savedAt ?? 0)).getTime();
      const bestTime = new Date(String(existing.savedAt ?? 0)).getTime();
      if (curTime > bestTime) bestByShowId.set(id, item);
    }
  }
  const rawItems = [...bestByShowId.values()].sort((a, b) => {
    const aTime = new Date(String(a.savedAt ?? 0)).getTime();
    const bTime = new Date(String(b.savedAt ?? 0)).getTime();
    return bTime - aTime;
  });

  const items = await enrichWithAddonMeta(rawItems);
  await replaceExternalContinueWatching({ items, provider: 'trakt' });

  let watchlistCount = 0;
  try {
    const [watchlistMoviesRes, watchlistShowsRes, watchedMoviesRes, watchedShowsRes] = await Promise.all([
      platformFetch('https://api.trakt.tv/users/me/watchlist/movies?limit=500', { headers }),
      platformFetch('https://api.trakt.tv/users/me/watchlist/shows?limit=500', { headers }),
      platformFetch('https://api.trakt.tv/users/me/watched/movies', { headers }),
      platformFetch('https://api.trakt.tv/users/me/watched/shows?extended=episodes', { headers }),
    ]);

    const wlMovies = watchlistMoviesRes.ok ? (await watchlistMoviesRes.json() as unknown[]) : [];
    const wlShows = watchlistShowsRes.ok ? (await watchlistShowsRes.json() as unknown[]) : [];
    const watchlistItems: Record<string, unknown>[] = [];
    for (const entry of wlMovies) {
      const e = entry as Record<string, unknown>;
      const movie = e.movie as Record<string, unknown> | undefined;
      if (!movie) continue;
      const ids = movie.ids as Record<string, unknown> | undefined;
      const imdb = typeof ids?.imdb === 'string' ? ids.imdb : null;
      const tmdb = ids?.tmdb != null ? `tmdb:${ids.tmdb}` : null;
      const id = imdb ?? tmdb;
      if (!id) continue;
      watchlistItems.push({ id, name: String(movie.title ?? ''), type: 'movie', source: 'trakt' });
    }
    for (const entry of wlShows) {
      const e = entry as Record<string, unknown>;
      const show = e.show as Record<string, unknown> | undefined;
      if (!show) continue;
      const ids = show.ids as Record<string, unknown> | undefined;
      const imdb = typeof ids?.imdb === 'string' ? ids.imdb : null;
      const tmdb = ids?.tmdb != null ? `tmdb:${ids.tmdb}` : null;
      const id = imdb ?? tmdb;
      if (!id) continue;
      watchlistItems.push({ id, name: String(show.title ?? ''), type: 'series', source: 'trakt' });
    }
    watchlistCount = watchlistItems.length;
    await mergeExternalWatchlist(watchlistItems);

    const watchedMovies = watchedMoviesRes.ok ? (await watchedMoviesRes.json() as unknown[]) : [];
    const watchedShows = watchedShowsRes.ok ? (await watchedShowsRes.json() as unknown[]) : [];
    const watchedIds: Record<string, boolean> = {};
    for (const entry of watchedMovies) {
      const e = entry as Record<string, unknown>;
      const movie = e.movie as Record<string, unknown> | undefined;
      const ids = movie?.ids as Record<string, unknown> | undefined;
      const imdb = typeof ids?.imdb === 'string' ? ids.imdb : null;
      if (imdb) watchedIds[imdb] = true;
    }
    for (const entry of watchedShows) {
      const e = entry as Record<string, unknown>;
      const show = e.show as Record<string, unknown> | undefined;
      const ids = show?.ids as Record<string, unknown> | undefined;
      const imdb = typeof ids?.imdb === 'string' ? ids.imdb : null;
      if (!imdb) continue;
      const seasons = Array.isArray(e.seasons) ? e.seasons as Record<string, unknown>[] : [];
      for (const season of seasons) {
        const sNum = Number(season.number ?? 0);
        const episodes = Array.isArray(season.episodes) ? season.episodes as Record<string, unknown>[] : [];
        for (const ep of episodes) {
          const eNum = Number(ep.number ?? 0);
          if (sNum > 0 && eNum > 0) watchedIds[`${imdb}:${sNum}:${eNum}`] = true;
        }
      }
    }
    await mergeExternalWatched(watchedIds);
  } catch {}

  return { synced: true, provider: 'trakt', continueWatchingCount: items.length, watchlistCount };
}

export async function pushMarkWatchedTrakt(
  videoIds: string[],
  watched: boolean,
  token: string,
  clientId: string,
): Promise<void> {
  const headers = traktHeaders(token, clientId);
  const endpoint = watched ? '/sync/history' : '/sync/history/remove';
  const moviePayloads: Record<string, unknown>[] = [];
  const showPayloads: Map<string, Record<string, unknown>> = new Map();

  for (const vid of videoIds) {
    const parsed = await coreParseVideoId(vid);
    const ids = await coreBuildTraktIds(vid);
    if (!ids) continue;
    if (parsed.isEpisode) {
      const showId = String(parsed.imdb ?? parsed.tmdb ?? '');
      if (!showPayloads.has(showId)) showPayloads.set(showId, { ids, seasons: [] });
      const showEntry = showPayloads.get(showId)!;
      const seasons = showEntry.seasons as Record<string, unknown>[];
      let seasonEntry = seasons.find((s) => s.number === parsed.season);
      if (!seasonEntry) { seasonEntry = { number: parsed.season, episodes: [] }; seasons.push(seasonEntry); }
      (seasonEntry.episodes as Record<string, unknown>[]).push({ number: parsed.episode });
    } else {
      moviePayloads.push({ ids });
    }
  }

  const body: Record<string, unknown> = {};
  if (moviePayloads.length > 0) body.movies = moviePayloads;
  if (showPayloads.size > 0) body.shows = [...showPayloads.values()];
  if (Object.keys(body).length > 0) {
    await platformFetch(`https://api.trakt.tv${endpoint}`, { method: 'POST', headers, body: JSON.stringify(body) });
  }
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
