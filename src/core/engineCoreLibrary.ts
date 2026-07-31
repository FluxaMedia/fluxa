import { coreInvoke } from "./engineCoreClient";

export async function coreNormalizeLibraryDocument(
  json: string,
): Promise<Record<string, unknown>> {
  return (await coreInvoke<Record<string, unknown>>(
    "normalizeLibraryDocument",
    json,
  )) ?? {};
}

export async function coreIsUpNextContinueWatchingItem(
  itemJson: string,
): Promise<boolean> {
  return (await coreInvoke<boolean>(
    "isUpNextContinueWatchingItem",
    itemJson,
  )) ?? false;
}

export async function coreRememberLastWatchedEpisodes(
  libJson: string,
  watchedIdsJson: string,
): Promise<Record<string, unknown>> {
  return (await coreInvoke<Record<string, unknown>>(
    "rememberLastWatchedEpisodes",
    JSON.stringify({ libJson, watchedIdsJson }),
  )) ?? {};
}

export async function coreBuildContinueWatchingFromProgress(
  progressJson: string,
): Promise<unknown[] | null> {
  return coreInvoke("buildContinueWatchingFromProgress", progressJson);
}

export async function coreComputeContinueWatchingBadges(
  candidatesJson: string,
  videosBySeriesJson: string,
  lastWatchedJson: string,
  nowMs: number,
): Promise<unknown[] | null> {
  return coreInvoke(
    "computeContinueWatchingBadges",
    JSON.stringify({
      candidatesJson,
      videosBySeriesJson,
      lastWatchedJson,
      nowMs,
    }),
  );
}


export async function coreParseIntroDbSegments(
  dataJson: string,
): Promise<unknown[] | null> {
  return coreInvoke("parseIntroDbSegments", dataJson);
}

export async function coreParseSkipdbSegments(
  dataJson: string,
): Promise<unknown[] | null> {
  return coreInvoke("parseSkipdbSegments", dataJson);
}

export async function coreAniListMalId(
  dataJson: string,
): Promise<number | null> {
  return coreInvoke("anilistMalId", dataJson);
}

export async function coreParseAniskipResults(
  resultsJson: string,
): Promise<unknown[] | null> {
  return coreInvoke("parseAniskipResults", resultsJson);
}

export async function coreParseAnimeSkipResults(
  resultsJson: string,
): Promise<unknown[] | null> {
  return coreInvoke("parseAnimeSkipResults", resultsJson);
}

export async function coreUniqueIntroSegments(
  segmentsAJson: string,
  segmentsBJson: string,
): Promise<unknown[] | null> {
  return coreInvoke(
    "uniqueIntroSegments",
    JSON.stringify({ segmentsAJson, segmentsBJson }),
  );
}

export async function coreMergeIntroSegments(
  sourcesJson: string,
): Promise<unknown[] | null> {
  return coreInvoke("mergeIntroSegments", sourcesJson);
}

export async function coreResolveNextEpisode(
  videosJson: string,
  currentSeason: number,
  currentEpisode: number,
  nowMs: number,
  releasedOnly: boolean,
): Promise<unknown | null> {
  return coreInvoke(
    "resolveNextEpisode",
    JSON.stringify({
      videos: JSON.parse(videosJson),
      currentSeason,
      currentEpisode,
      nowMs,
      releasedOnly,
    }),
  );
}

export async function coreStreamShellPlan(stream: unknown): Promise<
  {
    identityKey: string;
    isTorrent: boolean;
    requestHeaders?: Record<string, string>;
    sourceLink?: string;
    downloadLink?: string;
  } | null
> {
  return coreInvoke("streamShellPlan", JSON.stringify(stream));
}

export async function coreFormatEpisodeLine(
  lastEpisodeName?: string | null,
  lastEpisodeSeason?: number | null,
  lastEpisodeNumber?: number | null,
  lastVideoId?: string | null,
): Promise<string> {
  return (await coreInvoke<string>(
    "formatEpisodeLine",
    JSON.stringify({
      lastEpisodeName: lastEpisodeName ?? null,
      lastEpisodeSeason: lastEpisodeSeason ?? null,
      lastEpisodeNumber: lastEpisodeNumber ?? null,
      lastVideoId: lastVideoId ?? null,
    }),
  )) ?? "";
}

export async function coreSelectContinueWatchingArtwork(
  itemJson: string,
  artworkPreference: string,
  isHorizontal: boolean,
): Promise<string | null> {
  return coreInvoke(
    "selectContinueWatchingArtwork",
    JSON.stringify({
      item: JSON.parse(itemJson),
      artworkPreference,
      isHorizontal,
    }),
  );
}

export async function coreContinueWatchingCardFields(
  items: unknown[],
  artworkPreference: string,
  isHorizontal: boolean,
): Promise<
  Array<{ id: string; artwork: string | null; episodeLine: string }> | null
> {
  return coreInvoke(
    "continueWatchingCardFields",
    JSON.stringify({ items, artworkPreference, isHorizontal }),
  );
}

export async function coreBuildHomeCollectionShelves(
  profileJson: string,
  addonsJson: string,
): Promise<
  {
    pinnedShelves: unknown[];
    regularShelves: unknown[];
    hiddenFolderCategories: unknown[];
  } | null
> {
  return coreInvoke(
    "buildHomeCollectionShelves",
    JSON.stringify({ profileJson, addonsJson }),
  );
}

export async function coreReplaceExternalContinueWatching(
  existingJson: string,
  provider: string | null,
  itemsJson: string,
  sourceOfTruth?: string,
  rankingMode?: string,
  continueWatchingDays?: number,
): Promise<unknown[]> {
  return (await coreInvoke<unknown[]>(
    "replaceExternalContinueWatching",
    JSON.stringify({
      existingJson,
      provider,
      itemsJson,
      sourceOfTruth,
      rankingMode,
      continueWatchingDays,
    }),
  )) ?? [];
}

export async function coreResourceKindToResource(
  kind: string,
  requestResource?: string | null,
  itemResource?: string | null,
): Promise<string> {
  return (await coreInvoke<string>(
    "resourceKindToResource",
    JSON.stringify({
      kind,
      requestResource: requestResource ?? null,
      itemResource: itemResource ?? null,
    }),
  )) ?? kind;
}

export async function coreCanPrefetchNextEpisode(
  prefsJson: string,
  streamJson: string,
): Promise<boolean> {
  return (await coreInvoke<boolean>(
    "canPrefetchNextEpisode",
    JSON.stringify({ prefsJson, streamJson }),
  )) ?? false;
}

export async function coreSelectNextEpisodeStream(
  streamsJson: string,
  currentStreamJson: string,
  prefsJson: string,
  nextVideoId: string,
): Promise<unknown | null> {
  return coreInvoke(
    "selectNextEpisodeStream",
    JSON.stringify({ streamsJson, currentStreamJson, prefsJson, nextVideoId }),
  );
}

export async function coreImportCollections(
  rawJson: string,
): Promise<unknown[] | null> {
  return coreInvoke("importCollections", rawJson);
}

export async function coreExportCollections(
  collectionsJson: string,
): Promise<unknown | null> {
  return coreInvoke("exportCollections", collectionsJson);
}

export async function coreResolveTransportUrl(
  sourceJson: string,
  addonsJson: string,
): Promise<string | null> {
  return coreInvoke(
    "resolveTransportUrl",
    JSON.stringify({ sourceJson, addonsJson }),
  );
}

export async function coreResolveFeedOptionGenre(
  feedOptionJson: string,
  addonsJson: string,
): Promise<string | null> {
  return coreInvoke(
    "resolveFeedOptionGenre",
    JSON.stringify({ feedOptionJson, addonsJson }),
  );
}

export async function coreTraktPlaybackItemsDedup(
  itemsJson: string,
): Promise<unknown[] | null> {
  return coreInvoke("traktPlaybackItemsDedup", itemsJson);
}

export async function coreTraktMarkWatchedBody(
  videoIdsJson: string,
): Promise<unknown | null> {
  return coreInvoke("traktMarkWatchedBody", videoIdsJson);
}

export async function coreSimklMatchEpisode(
  episodesJson: string,
  targetJson: string,
): Promise<{ season: number; episode: number } | null> {
  return coreInvoke(
    "simklMatchEpisode",
    JSON.stringify({ episodesJson, targetJson }),
  );
}

export async function coreLibraryApplyMarkWatched(
  libJson: string,
  videoIdsJson: string,
): Promise<Record<string, unknown> | null> {
  return coreInvoke(
    "libraryApplyMarkWatched",
    JSON.stringify({ libJson, videoIdsJson }),
  );
}

export async function coreMergeProgressMeta(
  incomingMetaJson: string,
  existingMetaJson: string,
): Promise<Record<string, unknown>> {
  return (await coreInvoke<Record<string, unknown>>(
    "mergeProgressMeta",
    JSON.stringify({ incomingMetaJson, existingMetaJson }),
  )) ?? JSON.parse(incomingMetaJson);
}

export async function coreNuvioBuildLocalProfiles(
  sessionProfile: unknown,
  nuvioProfiles: unknown[],
  avatarCatalog: unknown[],
  existingProfiles: unknown[],
): Promise<unknown[] | null> {
  return coreInvoke(
    "nuvioBuildLocalProfiles",
    JSON.stringify({
      sessionProfile,
      nuvioProfiles,
      avatarCatalog,
      existingProfiles,
    }),
  );
}

export async function coreNuvioLibraryToWatchlist(
  library: unknown[],
): Promise<unknown[] | null> {
  return coreInvoke("nuvioLibraryToWatchlist", JSON.stringify({ library }));
}

export async function coreNuvioProgressMetaNeeds(
  watchProgress: unknown[],
  library: unknown[],
): Promise<Array<{ contentId: string; contentType: string }> | null> {
  return coreInvoke(
    "nuvioProgressMetaNeeds",
    JSON.stringify({ watchProgress, library }),
  );
}

export async function coreNuvioImportMergePlan(args: {
  progress: Record<string, unknown>;
  watched: Record<string, boolean>;
  library: unknown[];
  addonMetas: Record<string, unknown>;
  watchProgress: unknown[] | null;
  watchHistory: unknown[] | null;
  categories?: string[];
  dryRun?: boolean;
}): Promise<
  {
    progress: Record<string, unknown> | null;
    progressCount: number;
    watched: Record<string, boolean> | null;
    watchedCount: number;
  } | null
> {
  return coreInvoke("nuvioImportMergePlan", JSON.stringify(args));
}

export async function coreNuvioMapCollections(
  collections: unknown[],
): Promise<unknown[] | null> {
  return coreInvoke("nuvioMapCollections", JSON.stringify({ collections }));
}

export async function coreNuvioSortAddonsByPriority<T>(
  addons: T[],
): Promise<T[] | null> {
  return coreInvoke("nuvioSortAddonsByPriority", JSON.stringify({ addons }));
}

export async function coreFilterEnabledAddons<T>(
  addons: T[],
  disabledKeys: string[],
): Promise<T[] | null> {
  return coreInvoke(
    "filterEnabledAddons",
    JSON.stringify({ addons, disabledKeys }),
  );
}

export async function coreAirDateRefreshCandidates(
  items: unknown[],
  nowMs: number,
): Promise<string[]> {
  return (await coreInvoke<string[]>(
    "airDateRefreshCandidates",
    JSON.stringify({ items, nowMs }),
  )) ?? [];
}

export async function coreScrobbleCloseAction(
  timePosSec: number,
  durationSec: number,
): Promise<string> {
  return (await coreInvoke<string>(
    "scrobbleCloseAction",
    JSON.stringify({ timePosSec, durationSec }),
  )) ?? "pause";
}

export async function coreTorrentReadyBudget(): Promise<{
  firstAttemptMs: number;
  retryBudgetMs: number;
  hardLimitMs: number;
  stallExtensionMs: number;
  maxPeerRetriesWithAlternatives: number;
  maxPeerRetriesSingleSource: number;
}> {
  return (await coreInvoke("torrentReadyBudget", "{}")) ?? {
    firstAttemptMs: 15_000,
    retryBudgetMs: 45_000,
    hardLimitMs: 120_000,
    stallExtensionMs: 20_000,
    maxPeerRetriesWithAlternatives: 1,
    maxPeerRetriesSingleSource: 2,
  };
}

