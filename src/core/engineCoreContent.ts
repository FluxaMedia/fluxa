import { coreInvoke } from "./engineCoreClient";

export async function corePlaybackPreparePlan(
  request: unknown,
): Promise<Record<string, unknown> | null> {
  return coreInvoke("playbackPreparePlan", JSON.stringify(request));
}

export async function coreLibraryLocalStatePlan(
  request: unknown,
): Promise<Record<string, unknown> | null> {
  return coreInvoke("libraryLocalStatePlan", JSON.stringify(request));
}

export async function corePreferencesSchema(): Promise<
  Record<string, unknown> | null
> {
  return coreInvoke("preferencesSchema", "{}");
}

export async function coreApplyPreferenceUpdate(
  request: unknown,
): Promise<Record<string, unknown> | null> {
  return coreInvoke("applyPreferenceUpdate", JSON.stringify(request));
}

export async function coreDetailEpisodePlan(
  request: unknown,
): Promise<Record<string, unknown> | null> {
  return coreInvoke("detailEpisodePlan", JSON.stringify(request));
}

export async function coreNormalizeAddonSubtitles(
  subtitles: unknown[],
  resourceUrl: string,
): Promise<unknown[]> {
  return (await coreInvoke<unknown[]>(
    "normalizeAddonSubtitles",
    JSON.stringify({ subtitles: JSON.stringify(subtitles), resourceUrl }),
  )) ?? [];
}

export async function streamPlaybackInfo(
  streamJson: string,
): Promise<unknown | null> {
  return coreInvoke("streamPlaybackInfo", streamJson);
}

export async function coreSearchResultGrouping(
  request: unknown,
): Promise<unknown | null> {
  return coreInvoke("searchResultGrouping", JSON.stringify(request));
}

export async function coreBuildMetadataFeedOptions(
  addons: unknown[],
): Promise<unknown[] | null> {
  return coreInvoke("buildMetadataFeedOptions", JSON.stringify(addons));
}

export async function coreDiscoverCatalogOptions(
  addons: unknown[],
  selectedType: string,
): Promise<unknown[] | null> {
  return coreInvoke(
    "discoverCatalogOptions",
    JSON.stringify({ addons: JSON.stringify(addons), selectedType }),
  );
}

export async function coreLibrarySortPlan(
  request: unknown,
): Promise<unknown | null> {
  return coreInvoke("librarySortPlan", JSON.stringify(request));
}

export async function coreWatchlistTogglePlan(
  request: unknown,
): Promise<unknown | null> {
  return coreInvoke("watchlistTogglePlan", JSON.stringify(request));
}

export async function corePlaybackProgressMergePlan(
  request: unknown,
): Promise<unknown | null> {
  return coreInvoke("playbackProgressMergePlan", JSON.stringify(request));
}

export async function coreLibraryContinueWatchingItems(
  items: unknown[],
): Promise<unknown[] | null> {
  return coreInvoke("libraryContinueWatchingItems", JSON.stringify(items));
}

export async function coreDetailSeriesLookupId(rawId: string): Promise<string> {
  return (await coreInvoke<string>(
    "detailSeriesLookupId",
    JSON.stringify({ id: rawId }),
  )) ?? rawId;
}

export async function coreDetailSeasonLoadPlan(
  request: unknown,
): Promise<unknown | null> {
  return coreInvoke("detailSeasonLoadPlan", JSON.stringify(request));
}

export async function corePlayerBackendSelection(
  request: unknown,
): Promise<unknown | null> {
  return coreInvoke("playerBackendSelection", JSON.stringify(request));
}

export async function corePlayerBufferTargets(
  request: unknown,
): Promise<unknown | null> {
  return coreInvoke("playerBufferTargets", JSON.stringify(request));
}

export type TorrentStatusInfo = {
  bufferProgress: number;
  isPlayableEnough: boolean;
  statusKey: string;
};

export async function coreTorrentStatusInfo(
  status: unknown,
): Promise<TorrentStatusInfo | null> {
  return coreInvoke("torrentStatusInfo", JSON.stringify(status));
}

export async function coreOfflineDownloadPlan(
  request: unknown,
): Promise<unknown | null> {
  return coreInvoke("offlineDownloadPlan", JSON.stringify(request));
}

export async function corePlaybackIntroLookupContentId(
  id: string,
): Promise<string> {
  return (await coreInvoke<string>(
    "playbackIntroLookupContentId",
    JSON.stringify({ id }),
  )) ?? id;
}

export async function corePlayerSourceSidebarPlan(
  request: unknown,
): Promise<unknown | null> {
  return coreInvoke("playerSourceSidebarPlan", JSON.stringify(request));
}

export async function corePlayerRetryPolicy(
  request: unknown,
): Promise<unknown | null> {
  return coreInvoke("playerRetryPolicy", JSON.stringify(request));
}

export async function coreEffectiveMetadataFeedSelection(
  selectedKeys: string[],
  availableKeys: string[],
): Promise<string[] | null> {
  return coreInvoke(
    "effectiveMetadataFeedSelection",
    JSON.stringify({
      selectedKeys: JSON.stringify(selectedKeys),
      availableKeys: JSON.stringify(availableKeys),
    }),
  );
}

export async function coreToggleMetadataFeedLimited(
  selectedKeys: string[],
  availableKeys: string[],
  key: string,
  maxEnabled: number,
): Promise<string[] | null> {
  return coreInvoke(
    "toggleMetadataFeedLimited",
    JSON.stringify({
      selectedKeys: JSON.stringify(selectedKeys),
      availableKeys: JSON.stringify(availableKeys),
      key,
      maxEnabled,
    }),
  );
}

export async function coreFindPreferredSubtitleIndex(
  tracks: unknown[],
  lastSubtitleLanguage?: string | null,
  preferredSubtitleLanguage?: string | null,
  secondarySubtitleLanguage?: string | null,
): Promise<number> {
  return (await coreInvoke<number>(
    "findPreferredSubtitleIndex",
    JSON.stringify({
      tracks: JSON.stringify(tracks),
      lastSubtitleLanguage: lastSubtitleLanguage ?? null,
      preferredSubtitleLanguage: preferredSubtitleLanguage ?? null,
      secondarySubtitleLanguage: secondarySubtitleLanguage ?? null,
    }),
  )) ?? -1;
}

export async function coreSubtitleLanguageDedupKeepIndices(
  languages: (string | null | undefined)[],
  maxPerLanguage = 2,
): Promise<number[]> {
  return (await coreInvoke<number[]>(
    "subtitleLanguageDedupKeepIndices",
    JSON.stringify({ languages: languages.map((lang) => lang ?? null), maxPerLanguage }),
  )) ?? languages.map((_, index) => index);
}

export async function coreParseVideoId(id: string): Promise<{
  imdb?: string;
  tmdb?: string;
  season?: number;
  episode?: number;
  isEpisode: boolean;
}> {
  return (await coreInvoke("parseVideoId", JSON.stringify({ id }))) ??
    { isEpisode: false };
}

export async function coreBuildTraktIds(
  videoId: string,
): Promise<Record<string, unknown> | null> {
  return coreInvoke("buildTraktIds", JSON.stringify({ id: videoId }));
}

export async function coreDetectAnimePlayback(
  meta: unknown,
  episode: unknown,
  stream: unknown,
  addons: unknown[],
): Promise<{ isAnime: boolean; confidence: number; reasons: string[] }> {
  return (await coreInvoke(
    "detectAnimePlayback",
    JSON.stringify({ meta, episode, stream, addons }),
  )) ??
    { isAnime: false, confidence: 0, reasons: [] };
}

export async function coreAnilistEntriesToSync(
  entries: unknown[],
  nowMs: number,
  categories?: string[],
  dryRun?: boolean,
): Promise<
  {
    watchlist: Record<string, unknown>[] | null;
    watchlistCount: number;
    completed: Record<string, unknown>[] | null;
    completedCount: number;
    dropped: Record<string, unknown>[] | null;
    droppedCount: number;
    watching: Record<string, unknown>[] | null;
    watchingCount: number;
    watched: Record<string, boolean> | null;
    watchedUpdatedAtMs: Record<string, unknown> | null;
    progress: Record<string, unknown> | null;
  } | null
> {
  return coreInvoke("anilistEntriesToSync", JSON.stringify({ entries, nowMs, categories, dryRun }));
}

export async function coreMergeLibraryItemsById(
  local: unknown[],
  incoming: unknown[],
): Promise<Record<string, unknown>[]> {
  return (await coreInvoke<Record<string, unknown>[]>(
    "mergeLibraryItemsById",
    JSON.stringify({ local, incoming }),
  )) ?? [];
}

export async function coreShouldAttemptAnimeTracking(
  meta: unknown,
): Promise<boolean> {
  return (await coreInvoke<boolean>(
    "shouldAttemptAnimeTracking",
    JSON.stringify(meta),
  )) ?? false;
}

export async function coreExtractAnilistIdFromLinks(
  meta: unknown,
): Promise<number | null> {
  return coreInvoke("extractAnilistIdFromLinks", JSON.stringify(meta));
}

export async function coreAnilistSearchBestMatch(
  meta: unknown,
  candidates: unknown[],
): Promise<
  {
    anilistId: number;
    confidence: "title-year";
  } | null
> {
  return coreInvoke(
    "anilistSearchBestMatch",
    JSON.stringify({ meta, candidates }),
  );
}

export async function coreAnilistMediaListStatus(
  totalEpisodes: number,
  progressEpisode: number,
): Promise<"COMPLETED" | "CURRENT"> {
  return (await coreInvoke<"COMPLETED" | "CURRENT">(
    "anilistMediaListStatus",
    JSON.stringify({ totalEpisodes, progressEpisode }),
  )) ?? "CURRENT";
}

export async function coreAnilistGraphqlQueries(): Promise<{
  saveMediaListEntry: string;
  mediaListEntryLookup: string;
  deleteMediaListEntry: string;
} | null> {
  return coreInvoke("anilistGraphqlQueries", "{}");
}

export async function coreAnilistSaveMediaListEntryVariables(
  contentId: string,
  status: "COMPLETED" | "CURRENT",
  progress?: number,
): Promise<Record<string, unknown> | null> {
  return coreInvoke(
    "anilistSaveMediaListEntryVariables",
    JSON.stringify({ contentId, status, progress }),
  );
}

export async function coreTmdbPeopleRequestPlan(
  meta: unknown,
  apiKey: string,
  language: string,
): Promise<
  {
    creditsUrl?: string;
    findUrl?: string;
  } | null
> {
  return coreInvoke(
    "tmdbPeopleRequestPlan",
    JSON.stringify({ meta, apiKey, language }),
  );
}

export async function coreTmdbCreditsUrlFromFind(
  find: unknown,
  meta: unknown,
  apiKey: string,
  language: string,
): Promise<string | null> {
  return coreInvoke(
    "tmdbCreditsUrlFromFind",
    JSON.stringify({ find, meta, apiKey, language }),
  );
}

export async function coreTmdbPeopleImagesFromCredits(
  credits: unknown,
  links: unknown[],
): Promise<Record<string, string>> {
  return (await coreInvoke<Record<string, string>>(
    "tmdbPeopleImagesFromCredits",
    JSON.stringify({ credits, links }),
  )) ?? {};
}

export async function coreCalendarItemsFromMeta(
  metaJson: string,
  monthPrefix: string,
): Promise<unknown[]> {
  return (await coreInvoke<unknown[]>(
    "calendarItemsFromMeta",
    JSON.stringify({ metaJson, monthPrefix }),
  )) ?? [];
}

export async function coreCalendarItemMatchesMonth(
  itemJson: string,
  monthPrefix: string,
): Promise<boolean> {
  return (await coreInvoke<boolean>(
    "calendarItemMatchesMonth",
    JSON.stringify({ itemJson, monthPrefix }),
  )) ?? false;
}

export async function coreNextUnairedEpisode(
  videosJson: string,
  nowMs: number,
): Promise<
  {
    released?: string;
    season?: number;
    episode?: number;
    number?: number;
    title?: string;
    name?: string;
    thumbnail?: string;
  } | null
> {
  return coreInvoke(
    "nextUnairedEpisode",
    JSON.stringify({ videosJson, nowMs }),
  );
}

