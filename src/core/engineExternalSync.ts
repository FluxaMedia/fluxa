import { coreInvoke } from './engineCoreClient';

export interface CoreImportApplyPlan {
  watchlist: Record<string, unknown>[] | null;
  watchlistCount: number;
  watched: Record<string, boolean> | null;
  watchedCount: number;
  continueWatchingApply: boolean;
}

export async function coreImportApplyPlan(request: {
  localWatchlist: unknown[];
  externalWatchlist: unknown[];
  localWatched: Record<string, unknown>;
  externalWatched: Record<string, unknown>;
  categories?: string[];
  dryRun?: boolean;
}): Promise<CoreImportApplyPlan> {
  return (await coreInvoke<CoreImportApplyPlan>("importApplyPlan", JSON.stringify(request))) ?? {
    watchlist: null, watchlistCount: 0, watched: null, watchedCount: 0, continueWatchingApply: false,
  };
}

export async function coreTraktScrobblePlan(
  videoId: string,
  isEpisode: boolean,
  season: number | null,
  epNumber: number | null,
  timePosSec: number,
  durationSec: number,
  action?: 'start' | 'pause' | 'stop',
): Promise<{ action: string; body: unknown } | null> {
  return coreInvoke(
    "traktScrobblePlan",
    JSON.stringify({
      videoId,
      isEpisode,
      season,
      epNumber,
      timePosSec,
      durationSec,
      action,
    }),
  );
}

export async function coreSimklScrobbleBody(
  idsJson: string,
  isEpisode: boolean,
  season: number,
  epNumber: number,
  timePosSec: number,
  durationSec: number,
): Promise<unknown | null> {
  return coreInvoke(
    "simklScrobbleBody",
    JSON.stringify({
      idsJson,
      isEpisode,
      season,
      epNumber,
      timePosSec,
      durationSec,
    }),
  );
}

export async function coreSimklLookupIdForType(
  lookupJson: string,
  wantType: string,
): Promise<number | null> {
  return coreInvoke(
    "simklLookupIdForType",
    JSON.stringify({ lookupJson, wantType }),
  );
}

export async function coreTraktPlaybackItemsToLibrary(
  itemsJson: string,
): Promise<unknown[] | null> {
  return coreInvoke("traktPlaybackItemsToLibrary", itemsJson);
}

export async function coreTraktWatchedShowsToItems(
  showsJson: string,
): Promise<unknown[] | null> {
  return coreInvoke("traktWatchedShowsToItems", showsJson);
}

export async function coreTraktWatchlistToItems(
  moviesJson: string,
  showsJson: string,
): Promise<unknown[] | null> {
  return coreInvoke(
    "traktWatchlistToItems",
    JSON.stringify({ moviesJson, showsJson }),
  );
}

export async function coreTraktWatchedToIds(
  moviesJson: string,
  showsJson: string,
): Promise<unknown[] | null> {
  return coreInvoke(
    "traktWatchedToIds",
    JSON.stringify({ moviesJson, showsJson }),
  );
}

export interface CorePushPlan {
  watchlistItems?: { id: string; contentType: string }[];
  watchlistNuvioItems?: { contentId: string; contentType: string; name?: string | null; poster?: string | null; background?: string | null }[];
  watchedVideoIds?: string[];
  watchedStatusItems?: { id: string; status: 'completed' | 'dropped' }[];
  watchedItemIds?: string[];
  watchedNuvioItems?: { contentId: string; contentType: string; title?: string | null; season?: number | null; episode?: number | null; watchedAt: number }[];
  progressItemIds?: string[];
  progressNuvioEntries?: { contentId: string; contentType: string; videoId: string; position: number; duration: number; lastWatched: number; season?: number | null; episode?: number | null }[];
}

export async function corePushPlan(request: {
  destination: string;
  categories: string[];
  watchlist: unknown[];
  completed: unknown[];
  dropped: unknown[];
  continueWatching: unknown[];
  nowSec: number;
}): Promise<CorePushPlan> {
  return (await coreInvoke<CorePushPlan>("pushPlan", JSON.stringify(request))) ?? {};
}

export async function coreMergeExternalWatchlist(
  localJson: string,
  externalJson: string,
): Promise<Record<string, unknown>[]> {
  return (await coreInvoke<Record<string, unknown>[]>(
    "mergeExternalWatchlist",
    JSON.stringify({ localJson, externalJson }),
  )) ?? [];
}

export async function coreMergeExternalWatched(
  localJson: string,
  externalJson: string,
): Promise<Record<string, boolean>> {
  return (await coreInvoke<Record<string, boolean>>(
    "mergeExternalWatched",
    JSON.stringify({ localJson, externalJson }),
  )) ?? {};
}

export async function coreMergeContinueWatchingLists(
  localJson: string,
  externalJson: string,
  progressJson: string,
  sourceOfTruth?: string,
  rankingMode?: string,
): Promise<unknown[] | null> {
  return coreInvoke(
    "mergeContinueWatchingLists",
    JSON.stringify({
      localJson,
      externalJson,
      progressJson,
      sourceOfTruth,
      rankingMode,
    }),
  );
}

export async function coreSimklWatchingToItems(
  showsJson: string,
  moviesJson: string,
): Promise<unknown[] | null> {
  return coreInvoke(
    "simklWatchingToItems",
    JSON.stringify({ showsJson, moviesJson }),
  );
}

export async function coreSimklWatchlistToItems(
  showsJson: string,
  moviesJson: string,
): Promise<unknown[] | null> {
  return coreInvoke(
    "simklWatchlistToItems",
    JSON.stringify({ showsJson, moviesJson }),
  );
}

export async function coreSimklWatchedToIds(
  showsJson: string,
  moviesJson: string,
): Promise<Record<string, boolean> | null> {
  return coreInvoke(
    "simklWatchedToIds",
    JSON.stringify({ showsJson, moviesJson }),
  );
}

export async function coreTraktActivityDiff(request: {
  previous?: unknown;
  current?: unknown;
  hasPlayback: boolean;
  hasWatchlistMovies: boolean;
  hasWatchlistShows: boolean;
  hasWatchedMovies: boolean;
  hasWatchedShows: boolean;
}): Promise<{
  playbackChanged: boolean;
  watchlistMoviesChanged: boolean;
  watchlistShowsChanged: boolean;
  watchedMoviesChanged: boolean;
  watchedShowsChanged: boolean;
}> {
  return (await coreInvoke("traktActivityDiff", JSON.stringify(request))) ?? {
    playbackChanged: true,
    watchlistMoviesChanged: true,
    watchlistShowsChanged: true,
    watchedMoviesChanged: true,
    watchedShowsChanged: true,
  };
}

export async function coreSimklResourceSyncPlan(request: {
  previous?: unknown;
  current?: unknown;
  resources: Array<{ key: string; type: string; status: string; hasCached: boolean }>;
}): Promise<Array<{ key: string; action: 'unchanged' | 'full' | 'delta'; dateFrom: string | null }>> {
  return (await coreInvoke(
    "simklResourceSyncPlan",
    JSON.stringify(request),
  )) ?? request.resources.map((resource) => ({ key: resource.key, action: 'full' as const, dateFrom: null }));
}

export async function coreSimklMergeDelta(previousJson: string, changesJson: string): Promise<unknown> {
  return coreInvoke("simklMergeDelta", JSON.stringify({ previousJson, changesJson }));
}

export async function coreStremioWatchlistToItems(
  items: unknown[],
): Promise<unknown[] | null> {
  return coreInvoke("stremioWatchlistToItems", JSON.stringify(items));
}

export async function coreStremioWatchedToIds(
  items: unknown[],
): Promise<Record<string, boolean> | null> {
  return coreInvoke("stremioWatchedToIds", JSON.stringify(items));
}
