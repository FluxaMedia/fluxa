import { invoke } from "@tauri-apps/api/core";
import * as Sentry from "@sentry/react";
import type { DispatchResult, EffectResult } from "./types";
import type { CoreMethod } from "./coreMethods";

let engineHandle: number | null = null;

export async function initEngine(initialJson: string = "{}"): Promise<void> {
  if (engineHandle !== null) return;
  engineHandle = await invoke<number>("engine_init", { initialJson });
}

export async function dispatchAction(
  actionJson: string,
): Promise<DispatchResult | null> {
  let label = "dispatch";
  try {
    label = `dispatch:${
      (JSON.parse(actionJson) as { type?: string }).type ?? "?"
    }`;
  } catch {}
  return Sentry.startSpan({ name: label, op: "fluxa.ipc" }, async () => {
    const raw = await invoke<string | null>("engine_dispatch", { actionJson });
    if (!raw) return null;
    return JSON.parse(raw) as DispatchResult;
  });
}

export async function completeEffect(
  result: EffectResult,
): Promise<DispatchResult | null> {
  return Sentry.startSpan({
    name: `completeEffect:${result.effectId}`,
    op: "fluxa.ipc",
  }, async () => {
    const raw = await invoke<string | null>("engine_complete_effect", {
      resultJson: JSON.stringify(result),
    });
    if (!raw) return null;
    return JSON.parse(raw) as DispatchResult;
  });
}

export async function getSnapshot(): Promise<unknown | null> {
  const raw = await invoke<string | null>("engine_snapshot");
  if (!raw) return null;
  return JSON.parse(raw);
}

export async function httpFetchText(
  url: string,
): Promise<{ statusCode: number; body: string }> {
  const response = await invoke<{ status_code: number; body: string }>(
    "http_fetch_text",
    { url },
  );
  return { statusCode: response.status_code, body: response.body };
}

export async function httpExecuteText(
  url: string,
  method: string,
  headers: Record<string, string>,
  body?: unknown,
): Promise<{ statusCode: number; body: string }> {
  const response = await invoke<{ status_code: number; body: string }>(
    "http_execute_text",
    { url, method, headers, body },
  );
  return { statusCode: response.status_code, body: response.body };
}

export async function registerTrailerProxyUrl(url: string): Promise<string> {
  return invoke<string>("register_trailer_proxy_url", { url });
}

export async function runPluginScraper(
  code: string,
  tmdbId: string,
  mediaType: string,
  season: number | null,
  episode: number | null,
): Promise<string> {
  return invoke<string>("run_plugin_scraper", {
    code,
    tmdbId,
    mediaType,
    season,
    episode,
  });
}

export interface YoutubeTrailerSubtitleTrack {
  languageTag: string;
  label: string;
  url: string;
  mimeType: string;
  isAuto: boolean;
}

export * from "./engineStorage";

export async function coreInvoke<T>(
  method: CoreMethod,
  argsJson: string,
): Promise<T | null> {
  return Sentry.startSpan(
    { name: `coreInvoke:${method}`, op: "fluxa.core" },
    async () => {
      const raw = await invoke<string>("core_invoke", { method, argsJson });
      const envelope = JSON.parse(raw) as {
        ok: boolean;
        value?: T;
        error?: { kind: string; message: string };
      };
      if (!envelope.ok) {
        throw new Error(
          `[core] ${method}: ${envelope.error?.message ?? "unknown error"}`,
        );
      }
      return envelope.value ?? null;
    },
  );
}

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

export * from "./engineExternalSync";

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

export * from "./engineTmdb";
export * from "./engineMdblist";

export async function coreParseIntroDbSegments(
  dataJson: string,
): Promise<unknown[] | null> {
  return coreInvoke("parseIntroDbSegments", dataJson);
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

// Batched form of the two functions above for a whole Continue Watching row — avoids
// one IPC round trip per card on every Home load.
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
