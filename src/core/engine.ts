import { invoke } from '@tauri-apps/api/core';
import type { DispatchResult, EffectResult } from './types';

let engineHandle: number | null = null;

export async function initEngine(initialJson: string = '{}'): Promise<void> {
  if (engineHandle !== null) return;
  engineHandle = await invoke<number>('engine_init', { initialJson });
}

export async function dispatchAction(actionJson: string): Promise<DispatchResult | null> {
  const raw = await invoke<string | null>('engine_dispatch', { actionJson });
  if (!raw) return null;
  return JSON.parse(raw) as DispatchResult;
}

export async function completeEffect(result: EffectResult): Promise<DispatchResult | null> {
  const raw = await invoke<string | null>('engine_complete_effect', {
    resultJson: JSON.stringify(result),
  });
  if (!raw) return null;
  return JSON.parse(raw) as DispatchResult;
}

export async function getSnapshot(): Promise<unknown | null> {
  const raw = await invoke<string | null>('engine_snapshot');
  if (!raw) return null;
  return JSON.parse(raw);
}

export async function httpFetchText(url: string): Promise<{ statusCode: number; body: string }> {
  const response = await invoke<{ status_code: number; body: string }>('http_fetch_text', { url });
  return { statusCode: response.status_code, body: response.body };
}

export async function corePlaybackPreparePlan(request: unknown): Promise<Record<string, unknown> | null> {
  const raw = await invoke<string | null>('core_playback_prepare_plan', {
    requestJson: JSON.stringify(request),
  });
  return raw ? JSON.parse(raw) : null;
}

export async function coreLibraryLocalStatePlan(request: unknown): Promise<Record<string, unknown> | null> {
  const raw = await invoke<string | null>('core_library_local_state_plan', {
    requestJson: JSON.stringify(request),
  });
  return raw ? JSON.parse(raw) : null;
}

export async function corePreferencesSchema(): Promise<Record<string, unknown> | null> {
  const raw = await invoke<string>('core_preferences_schema');
  return JSON.parse(raw) as Record<string, unknown>;
}

export async function coreApplyPreferenceUpdate(request: unknown): Promise<Record<string, unknown> | null> {
  const raw = await invoke<string | null>('core_apply_preference_update', {
    requestJson: JSON.stringify(request),
  });
  return raw ? JSON.parse(raw) : null;
}

export async function coreDetailEpisodePlan(request: unknown): Promise<Record<string, unknown> | null> {
  const raw = await invoke<string | null>('core_detail_episode_plan', {
    requestJson: JSON.stringify(request),
  });
  return raw ? JSON.parse(raw) : null;
}

export async function coreNormalizeAddonSubtitles(subtitles: unknown[], resourceUrl: string): Promise<unknown[]> {
  const raw = await invoke<string>('core_normalize_addon_subtitles', {
    subtitlesJson: JSON.stringify(subtitles),
    resourceUrl,
  });
  return JSON.parse(raw) as unknown[];
}

export async function streamPlaybackInfo(streamJson: string): Promise<unknown | null> {
  const raw = await invoke<string | null>('core_stream_playback_info', { streamJson });
  return raw ? JSON.parse(raw) : null;
}

export async function coreSearchResultGrouping(request: unknown): Promise<unknown | null> {
  const raw = await invoke<string | null>('core_search_result_grouping', {
    requestJson: JSON.stringify(request),
  });
  return raw ? JSON.parse(raw) : null;
}

export async function coreBuildMetadataFeedOptions(addons: unknown[]): Promise<unknown[] | null> {
  const raw = await invoke<string | null>('core_build_metadata_feed_options', {
    addonsJson: JSON.stringify(addons),
  });
  return raw ? JSON.parse(raw) : null;
}

export async function coreDiscoverCatalogOptions(addons: unknown[], selectedType: string): Promise<unknown[] | null> {
  const raw = await invoke<string | null>('core_discover_catalog_options', {
    addonsJson: JSON.stringify(addons),
    selectedType,
  });
  return raw ? JSON.parse(raw) : null;
}

export async function coreDiscoverSortPlan(request: unknown): Promise<unknown | null> {
  const raw = await invoke<string | null>('core_discover_sort_plan', {
    requestJson: JSON.stringify(request),
  });
  return raw ? JSON.parse(raw) : null;
}

export async function coreLibrarySortPlan(request: unknown): Promise<unknown | null> {
  const raw = await invoke<string | null>('core_library_sort_plan', {
    requestJson: JSON.stringify(request),
  });
  return raw ? JSON.parse(raw) : null;
}

export async function coreWatchlistTogglePlan(request: unknown): Promise<unknown | null> {
  const raw = await invoke<string | null>('core_watchlist_toggle_plan', {
    requestJson: JSON.stringify(request),
  });
  return raw ? JSON.parse(raw) : null;
}

export async function corePlaybackProgressMergePlan(request: unknown): Promise<unknown | null> {
  const raw = await invoke<string | null>('core_playback_progress_merge_plan', {
    requestJson: JSON.stringify(request),
  });
  return raw ? JSON.parse(raw) : null;
}

export async function coreLibraryContinueWatchingItems(items: unknown[]): Promise<unknown[] | null> {
  const raw = await invoke<string | null>('core_library_continue_watching_items', {
    itemsJson: JSON.stringify(items),
  });
  return raw ? JSON.parse(raw) : null;
}

export async function coreDetailSeriesLookupId(rawId: string): Promise<string> {
  return invoke<string>('core_detail_series_lookup_id', { rawId });
}

export async function coreDetailSeasonLoadPlan(request: unknown): Promise<unknown | null> {
  const raw = await invoke<string | null>('core_detail_season_load_plan', {
    requestJson: JSON.stringify(request),
  });
  return raw ? JSON.parse(raw) : null;
}

export async function corePlayerBackendSelection(request: unknown): Promise<unknown | null> {
  const raw = await invoke<string | null>('core_player_backend_selection', {
    requestJson: JSON.stringify(request),
  });
  return raw ? JSON.parse(raw) : null;
}

export async function corePlayerBufferTargets(request: unknown): Promise<unknown | null> {
  const raw = await invoke<string | null>('core_player_buffer_targets', {
    requestJson: JSON.stringify(request),
  });
  return raw ? JSON.parse(raw) : null;
}

export async function coreOfflineDownloadPlan(request: unknown): Promise<unknown | null> {
  const raw = await invoke<string | null>('core_offline_download_plan', {
    requestJson: JSON.stringify(request),
  });
  return raw ? JSON.parse(raw) : null;
}

export async function enqueueOfflineDownload(request: unknown): Promise<unknown | null> {
  const raw = await invoke<string | null>('enqueue_offline_download', {
    requestJson: JSON.stringify(request),
  });
  return raw ? JSON.parse(raw) : null;
}

export async function corePlaybackIntroLookupContentId(id: string): Promise<string> {
  return invoke<string>('core_playback_intro_lookup_content_id', { id });
}

export async function corePlayerSourceSidebarPlan(request: unknown): Promise<unknown | null> {
  const raw = await invoke<string | null>('core_player_source_sidebar_plan', {
    requestJson: JSON.stringify(request),
  });
  return raw ? JSON.parse(raw) : null;
}

export async function corePlayerRetryPolicy(request: unknown): Promise<unknown | null> {
  const raw = await invoke<string | null>('core_player_retry_policy', {
    requestJson: JSON.stringify(request),
  });
  return raw ? JSON.parse(raw) : null;
}

export async function coreEffectiveMetadataFeedSelection(
  selectedKeys: string[],
  availableKeys: string[],
): Promise<string[] | null> {
  const raw = await invoke<string | null>('core_effective_metadata_feed_selection', {
    selectedKeysJson: JSON.stringify(selectedKeys),
    availableKeysJson: JSON.stringify(availableKeys),
  });
  return raw ? JSON.parse(raw) : null;
}

export async function coreToggleMetadataFeedLimited(
  selectedKeys: string[],
  availableKeys: string[],
  key: string,
  maxEnabled: number,
): Promise<string[] | null> {
  const raw = await invoke<string | null>('core_toggle_metadata_feed_limited', {
    selectedKeysJson: JSON.stringify(selectedKeys),
    availableKeysJson: JSON.stringify(availableKeys),
    key,
    maxEnabled,
  });
  return raw ? JSON.parse(raw) : null;
}

export async function coreFindPreferredSubtitleIndex(
  tracks: unknown[],
  lastSubtitleLanguage?: string | null,
  preferredSubtitleLanguage?: string | null,
  secondarySubtitleLanguage?: string | null,
): Promise<number> {
  return invoke<number>('core_find_preferred_subtitle_index', {
    tracksJson: JSON.stringify(tracks),
    lastSubtitleLanguage: lastSubtitleLanguage ?? null,
    preferredSubtitleLanguage: preferredSubtitleLanguage ?? null,
    secondarySubtitleLanguage: secondarySubtitleLanguage ?? null,
  });
}

export async function storageRead<T>(key: string): Promise<T | null> {
  const raw = await invoke<string | null>('storage_read', { key });
  if (!raw) return null;
  try {
    return JSON.parse(raw) as T;
  } catch {
    return null;
  }
}

export async function storageWrite(key: string, value: unknown): Promise<boolean> {
  return invoke<boolean>('storage_write', { key, value: JSON.stringify(value) });
}

export async function storageDelete(key: string): Promise<boolean> {
  return invoke<boolean>('storage_delete', { key });
}

export async function coreParseVideoId(id: string): Promise<{
  imdb?: string; tmdb?: string; season?: number; episode?: number; isEpisode: boolean;
}> {
  const raw = await invoke<string>('core_parse_video_id', { id });
  return JSON.parse(raw);
}

export async function coreBuildTraktIds(videoId: string): Promise<Record<string, unknown> | null> {
  const raw = await invoke<string | null>('core_build_trakt_ids', { videoId });
  return raw ? JSON.parse(raw) : null;
}

export async function coreCalendarItemsFromMeta(metaJson: string, monthPrefix: string): Promise<unknown[]> {
  const raw = await invoke<string | null>('core_calendar_items_from_meta', { metaJson, monthPrefix });
  return raw ? JSON.parse(raw) : [];
}

export async function coreCalendarItemMatchesMonth(itemJson: string, monthPrefix: string): Promise<boolean> {
  return invoke<boolean>('core_calendar_item_matches_month', { itemJson, monthPrefix });
}

export async function coreTraktPlaybackItemsToLibrary(itemsJson: string): Promise<unknown[] | null> {
  const raw = await invoke<string | null>('core_trakt_playback_items_to_library', { itemsJson });
  return raw ? JSON.parse(raw) : null;
}

export async function coreTraktWatchlistToItems(moviesJson: string, showsJson: string): Promise<unknown[] | null> {
  const raw = await invoke<string | null>('core_trakt_watchlist_to_items', { moviesJson, showsJson });
  return raw ? JSON.parse(raw) : null;
}

export async function coreTraktWatchedToIds(moviesJson: string, showsJson: string): Promise<unknown[] | null> {
  const raw = await invoke<string | null>('core_trakt_watched_to_ids', { moviesJson, showsJson });
  return raw ? JSON.parse(raw) : null;
}

export async function coreMergeExternalWatchlist(localJson: string, externalJson: string): Promise<Record<string, unknown>[]> {
  const raw = await invoke<string>('core_merge_external_watchlist', { localJson, externalJson });
  return JSON.parse(raw);
}

export async function coreMergeExternalWatched(localJson: string, externalJson: string): Promise<Record<string, boolean>> {
  const raw = await invoke<string>('core_merge_external_watched', { localJson, externalJson });
  return JSON.parse(raw);
}

export async function coreMergeContinueWatchingLists(
  localJson: string,
  externalJson: string,
  progressJson: string,
): Promise<unknown[] | null> {
  const raw = await invoke<string | null>('core_merge_continue_watching_lists', {
    localJson, externalJson, progressJson,
  });
  return raw ? JSON.parse(raw) : null;
}

export async function coreSimklWatchingToItems(showsJson: string, moviesJson: string): Promise<unknown[] | null> {
  const raw = await invoke<string | null>('core_simkl_watching_to_items', { showsJson, moviesJson });
  return raw ? JSON.parse(raw) : null;
}

export async function coreSimklWatchlistToItems(showsJson: string, moviesJson: string): Promise<unknown[] | null> {
  const raw = await invoke<string | null>('core_simkl_watchlist_to_items', { showsJson, moviesJson });
  return raw ? JSON.parse(raw) : null;
}

export async function coreSimklWatchedToIds(showsJson: string, moviesJson: string): Promise<Record<string, boolean> | null> {
  const raw = await invoke<string | null>('core_simkl_watched_to_ids', { showsJson, moviesJson });
  return raw ? JSON.parse(raw) : null;
}

export async function coreNormalizeLibraryDocument(json: string): Promise<Record<string, unknown>> {
  const raw = await invoke<string>('core_normalize_library_document', { json });
  return JSON.parse(raw);
}

export async function coreIsUpNextContinueWatchingItem(itemJson: string): Promise<boolean> {
  return invoke<boolean>('core_is_up_next_continue_watching_item', { itemJson });
}

export async function coreRememberLastWatchedEpisodes(
  libJson: string,
  watchedIdsJson: string,
): Promise<Record<string, unknown>> {
  const raw = await invoke<string>('core_remember_last_watched_episodes', { libJson, watchedIdsJson });
  return JSON.parse(raw);
}

export async function coreBuildContinueWatchingFromProgress(progressJson: string): Promise<unknown[] | null> {
  const raw = await invoke<string | null>('core_build_continue_watching_from_progress', { progressJson });
  return raw ? JSON.parse(raw) : null;
}

export async function coreComputeContinueWatchingBadges(
  candidatesJson: string,
  videosBySeriesJson: string,
  lastWatchedJson: string,
  nowMs: number,
): Promise<unknown[] | null> {
  const raw = await invoke<string | null>('core_compute_continue_watching_badges', {
    candidatesJson, videosBySeriesJson, lastWatchedJson, nowMs,
  });
  return raw ? JSON.parse(raw) : null;
}

export async function coreTmdbContentType(contentType: string): Promise<string> {
  return invoke<string>('core_tmdb_content_type', { contentType });
}

export async function coreTmdbLanguage(language: string): Promise<string> {
  return invoke<string>('core_tmdb_language', { language });
}

export async function coreTmdbImageUrl(path: string | null, size: string): Promise<string | null> {
  return invoke<string | null>('core_tmdb_image_url', { path, size });
}

export async function coreTmdbMetaToMeta(
  itemJson: string, requestedType: string, language: string,
): Promise<unknown | null> {
  const raw = await invoke<string | null>('core_tmdb_meta_to_meta', { itemJson, requestedType, language });
  return raw ? JSON.parse(raw) : null;
}

export async function coreTmdbVideoToTrailer(videoJson: string): Promise<unknown | null> {
  const raw = await invoke<string | null>('core_tmdb_video_to_trailer', { videoJson });
  return raw ? JSON.parse(raw) : null;
}

export async function coreTmdbBulkMetas(
  itemsJson: string, requestedType: string, language: string,
): Promise<unknown[] | null> {
  const raw = await invoke<string | null>('core_tmdb_bulk_metas', { itemsJson, requestedType, language });
  return raw ? JSON.parse(raw) : null;
}

export async function coreTmdbBulkVideosToTrailers(itemsJson: string): Promise<unknown[] | null> {
  const raw = await invoke<string | null>('core_tmdb_bulk_videos_to_trailers', { itemsJson });
  return raw ? JSON.parse(raw) : null;
}

export async function coreTmdbResolveIdHint(contentId: string): Promise<[string, boolean]> {
  return invoke<[string, boolean]>('core_tmdb_resolve_id_hint', { contentId });
}

export async function coreParseIntroDbSegments(dataJson: string): Promise<unknown[] | null> {
  const raw = await invoke<string | null>('core_parse_intro_db_segments', { dataJson });
  return raw ? JSON.parse(raw) : null;
}

export async function coreParseAniskipResults(resultsJson: string): Promise<unknown[] | null> {
  const raw = await invoke<string | null>('core_parse_aniskip_results', { resultsJson });
  return raw ? JSON.parse(raw) : null;
}

export async function coreUniqueIntroSegments(
  segmentsAJson: string, segmentsBJson: string,
): Promise<unknown[] | null> {
  const raw = await invoke<string | null>('core_unique_intro_segments', { segmentsAJson, segmentsBJson });
  return raw ? JSON.parse(raw) : null;
}

export async function coreMergeIntroSegments(sourcesJson: string): Promise<unknown[] | null> {
  const raw = await invoke<string | null>('core_merge_intro_segments', { sourcesJson });
  return raw ? JSON.parse(raw) : null;
}
