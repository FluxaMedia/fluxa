import { coreInvoke } from './engine';

export async function coreTraktScrobblePlan(
  videoId: string,
  isEpisode: boolean,
  season: number | null,
  epNumber: number | null,
  timePosSec: number,
  durationSec: number,
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
