import * as Sentry from '@sentry/react';
import { invoke } from '@tauri-apps/api/core';
import { completeEffect, coreInvoke, dispatchAction, enqueueOfflineDownload, httpExecuteText, libraryContinueWatchingDelete, libraryProgressDelete, registerTrailerProxyUrl } from './engine';
import { startTorrentStream, stopTorrentStream } from './mpvPlayer';
import { effectRunnerLibraryKey, loadActiveProfile, loadEnabledAddons, loadLibrary, loadPrefs, saveLibrary, persistLastWatchedEpisode } from './libraryOps';
import { continueWatchingForSelectedSource, readHomeBootstrap } from './homeEffects';
import { invalidateCalendarCache } from './libraryEffects';
import {
  applyLibraryCommand,
  notifyReleasedEpisodes,
  readCalendarMonth,
  readDetailLocalState,
  readLibraryState,
  readPlaybackProgress,
  writeFeedback,
  writePlaybackProgress,
  writeSettings,
} from './libraryEffects';
import { notify } from './notifications';
import { t } from '../i18n';
import { fetchAddonManifest, fetchAddonResource, refreshInstalledAddons } from './addonEffects';
import { fetchPluginManifestEffect } from './pluginRuntime';
import { fetchCatalogPage, readDiscoverCatalogFilters, runDiscover, runSearch } from './catalogEffects';
import {
  fetchDetailSecondary,
  fetchDetailStreams,
  fetchMdblistRatingsForDetail,
  fetchMetaDetail,
  fetchSeasonEpisodes,
  prefetchDetailStreams,
} from './detailEffects';
import { exchangeAuthCode, refreshAuthToken, runAuthFlow } from './authEffects';
import {
  dropExternalPlaybackProgress,
  enqueueTraktScrobble,
  pushMarkWatchedExternal,
  replaceExternalContinueWatching,
  syncExternalIntegrationNow,
  type WatchProgressInfo,
} from './externalSync';
import { fetchVideosForSeries } from './fetchPlanning';
import { fetchIntroSegments, fetchSubtitles, resolveIntroImdbId, type IntroSegmentResult } from './introEffects';
import type { AppState, Effect, EffectResult } from './types';

export interface YoutubeTrailerSubtitleTrack {
  languageTag: string;
  label: string;
  url: string;
  mimeType: string;
  isAuto: boolean;
}

export interface YoutubeTrailerResolution {
  status: 'ok';
  streamUrl: string;
  audioUrl?: string | null;
  subtitles?: YoutubeTrailerSubtitleTrack[];
}

export { fetchMetaVideos } from './detailEffects';
export { syncExternalIntegrationNow } from './externalSync';
export type { IntroSegmentResult } from './introEffects';

async function startTorrentFromEffect(payload: Record<string, unknown>): Promise<unknown> {
  const stream = payload.stream && typeof payload.stream === 'object'
    ? { ...(payload.stream as Record<string, unknown>) }
    : {};
  if (typeof payload.url === 'string' && !stream.playableUrl && !stream.url) {
    stream.playableUrl = payload.url;
  }
  if (typeof payload.fileIdx === 'number' && stream.fileIdx == null) {
    stream.fileIdx = payload.fileIdx;
  }
  if (Array.isArray(payload.sources) && !Array.isArray(stream.sources)) {
    stream.sources = payload.sources;
  }
  const title = typeof payload.title === 'string' ? payload.title : undefined;
  const prefs = await loadPrefs();
  const started = await startTorrentStream(JSON.stringify(stream), title, prefs);
  return { url: started.url };
}

async function executeYoutubeTrailerRequest(payload: Record<string, unknown>): Promise<unknown> {
  const url = typeof payload.url === 'string' ? payload.url : '';
  const method = typeof payload.method === 'string' ? payload.method : 'GET';
  if (!url) throw new Error('missing trailer request URL');
  const headers = payload.headers && typeof payload.headers === 'object' && !Array.isArray(payload.headers)
    ? Object.fromEntries(Object.entries(payload.headers as Record<string, unknown>).filter((entry): entry is [string, string] => typeof entry[1] === 'string'))
    : {};
  const response = await httpExecuteText(url, method, headers, payload.body);
  return { statusCode: response.statusCode, body: response.body };
}

async function deriveNextProgressFromLastWatched(metaObj: Record<string, unknown>): Promise<WatchProgressInfo | undefined> {
  const id = metaObj.id as string | undefined;
  if (!id || metaObj.type !== 'series') return undefined;
  const currentSeason = metaObj.lastEpisodeSeason as number | undefined;
  const currentEpisode = metaObj.lastEpisodeNumber as number | undefined;
  if (currentSeason == null || currentEpisode == null) return undefined;
  const videos = await fetchVideosForSeries(id, await loadEnabledAddons());
  return (await coreInvoke<WatchProgressInfo>('nextProgressInfoPlan', JSON.stringify({
    contentId: id,
    contentType: 'series',
    videos,
    watchedEpisodes: [{ season: currentSeason, episode: currentEpisode }],
    nowMs: Date.now(),
  }))) ?? undefined;
}

async function runEffect(
  effect: Effect,
  onStateUpdate?: (state: Partial<AppState>) => void,
  signal?: AbortSignal,
): Promise<unknown> {
  const p = effect.payload;
  let value: unknown;

  switch (effect.type) {
    case 'readHomeBootstrap':
      value = await readHomeBootstrap(p);
      break;

    case 'refreshContinueWatching': {
      const lib = await loadLibrary();
      const addons = await loadEnabledAddons();
      const prefs = await loadPrefs();
      if (typeof p.source === 'string') prefs.continueWatchingSource = p.source;
      void invoke('debug_log', { msg: `cw-source: refresh requested source=${String(prefs.continueWatchingSource ?? 'local')}` });
      const continueWatching = await continueWatchingForSelectedSource(
        lib as Record<string, unknown>,
        prefs,
        addons,
      );
      void invoke('debug_log', { msg: `cw-source: refresh resolved source=${String(prefs.continueWatchingSource ?? 'local')} count=${continueWatching.length} first=${String(continueWatching[0]?.name ?? '')}` });
      value = { continueWatching };
      break;
    }

    case 'readLibraryState':
      value = await readLibraryState();
      break;
    case 'readPlaybackProgress':
      value = await readPlaybackProgress(p);
      break;
    case 'readDetailLocalState':
      value = await readDetailLocalState(p);
      break;
    case 'readDiscoverCatalogFilters':
      value = await readDiscoverCatalogFilters(p);
      break;
    case 'readCalendarMonth':
      value = await readCalendarMonth(p);
      break;

    case 'writeLibraryCommand':
      value = await applyLibraryCommand(p);
      break;
    case 'writePlaybackProgress':
      value = await writePlaybackProgress(p);
      break;
    case 'writeFeedback':
      value = await writeFeedback(p);
      break;
    case 'clearPlaybackProgress': {
      const lib = await loadLibrary();
      const metaObj = (p.meta as Record<string, unknown>) ?? {};
      const preserveLastWatched = Boolean(metaObj._preserveLastWatched);
      const dropContinueWatching = Boolean(metaObj._dropContinueWatching);
      const plan = await coreInvoke<{
        library: Record<string, unknown>;
        contentId: string;
        lastWatchedEntry: Record<string, unknown> | null;
        removedExternalContinueWatching: boolean;
        droppedExternalContinueWatching: Record<string, unknown> | null;
      }>('clearPlaybackProgressPlan', JSON.stringify({
        library: lib,
        meta: metaObj,
        preserveLastWatched,
        dropContinueWatching,
        nowIso: new Date().toISOString(),
      }));
      if (plan) {
        await libraryProgressDelete(await effectRunnerLibraryKey(), plan.contentId);
        if (plan.removedExternalContinueWatching) {
          await libraryContinueWatchingDelete(await effectRunnerLibraryKey(), plan.contentId);
        }
        await persistLastWatchedEpisode(plan.contentId, plan.lastWatchedEntry);
        await saveLibrary(plan.library);
        invalidateCalendarCache();
        if (preserveLastWatched && metaObj.lastVideoId != null) {
          const profile = await loadActiveProfile();
          const nextProgress = await deriveNextProgressFromLastWatched(metaObj);
          await pushMarkWatchedExternal(
            [String(metaObj.lastVideoId)],
            true,
            metaObj,
            profile,
            {
              contentId: plan.contentId,
              contentType: String(metaObj.type ?? 'series'),
              season: metaObj.lastEpisodeSeason as number | undefined,
              episode: metaObj.lastEpisodeNumber as number | undefined,
              title: String(metaObj.name ?? ''),
            },
            nextProgress,
          ).catch(() => undefined);
        }
        if (plan.droppedExternalContinueWatching) {
          void dropExternalPlaybackProgress(plan.droppedExternalContinueWatching);
        }
      }
      value = (plan && !preserveLastWatched) ? { droppedId: plan.contentId } : {};
      break;
    }
    case 'writeSettings':
      value = await writeSettings(p);
      break;
    case 'syncWatchedState':
      value = {};
      break;

    case 'fetchAddonManifest':
      value = await fetchAddonManifest(p, signal);
      break;
    case 'refreshInstalledAddons':
      value = await refreshInstalledAddons(p, signal);
      break;
    case 'fetchAddonResource':
      value = await fetchAddonResource(p, signal);
      break;
    case 'fetchPluginManifest':
      value = await fetchPluginManifestEffect(p, signal);
      break;

    case 'fetchYoutubeTrailerWatchConfig':
    case 'fetchYoutubeTrailerPlayer':
    case 'fetchYoutubeTrailerPlayerScript':
      value = await executeYoutubeTrailerRequest(p);
      break;

    case 'fetchCatalogPage':
      value = await fetchCatalogPage(p, signal);
      break;
    case 'fetchDiscoverPage':
      value = await fetchCatalogPage(p, signal);
      break;
    case 'runSearch':
      value = await runSearch(p, signal);
      break;
    case 'runDiscover':
      value = await runDiscover(p, signal);
      break;

    case 'fetchMetaDetail': {
      const [meta, mdblistRatings] = await Promise.all([fetchMetaDetail(p), fetchMdblistRatingsForDetail(p)]);
      value = { meta, mdblistRatings };
      break;
    }
    case 'fetchMetaDetailLookup':
      value = await fetchMetaDetail(p);
      break;
    case 'fetchDetailSecondary':
      value = await fetchDetailSecondary(p);
      break;
    case 'prefetchDetailStreams':
      value = await prefetchDetailStreams(p, signal);
      break;
    case 'fetchDetailStreams':
      value = await fetchDetailStreams(p, onStateUpdate, effect.generation, signal);
      break;
    case 'fetchSeasonEpisodes':
      value = await fetchSeasonEpisodes(p);
      break;
    case 'loadStreams':
      value = await fetchDetailStreams(p, undefined, undefined, signal);
      break;

    case 'fetchSubtitles':
      value = await fetchSubtitles(p);
      break;

    case 'resolveIntroImdbId':
      value = await resolveIntroImdbId(p);
      break;
    case 'fetchIntroSegments':
      value = await fetchIntroSegments(p);
      break;

    case 'runAuthFlow':
      value = await runAuthFlow(p);
      break;
    case 'exchangeAuthCode':
      value = await exchangeAuthCode(p);
      break;
    case 'refreshAuthToken':
      value = await refreshAuthToken(p);
      break;

    case 'runExternalSync':
      value = await syncExternalIntegrationNow(p);
      break;
    case 'syncExternalIntegration':
      value = { synced: false };
      break;
    case 'enqueueTraktScrobble':
      value = await enqueueTraktScrobble(p);
      break;

    case 'startTorrentStream':
      value = await startTorrentFromEffect(p);
      break;
    case 'stopTorrent':
      value = { stopped: await stopTorrentStream() };
      break;

    case 'enqueueOfflineDownload':
      value = await enqueueOfflineDownload(p);
      break;

    case 'notifyReleasedEpisodes':
      void notifyReleasedEpisodes(p);
      value = {};
      break;
    case 'updateCalendarWidget':
      value = {};
      break;
    case 'replaceExternalContinueWatching':
      value = await replaceExternalContinueWatching(p);
      break;

    case 'prepareDirectPlayback':
      value = await fetchDetailStreams(p, undefined, undefined, signal);
      break;

    default:
      value = null;
      break;
  }

  return value;
}

export async function executeEffect(
  effect: Effect,
  onStateUpdate?: (state: Partial<AppState>) => void,
  signal?: AbortSignal,
): Promise<EffectResult> {
  try {
    const abortController = new AbortController();
    const effectSignal = signal ? AbortSignal.any([signal, abortController.signal]) : abortController.signal;
    const run = Sentry.startSpan(
      { name: effect.type, op: 'fluxa.effect' },
      () => runEffect(effect, onStateUpdate, effectSignal),
    );
    const value = effect.timeoutMs && effect.timeoutMs > 0
      ? await new Promise<unknown>((resolve, reject) => {
          const timeoutId = window.setTimeout(() => {
            abortController.abort();
            reject(new Error(`effect timed out after ${effect.timeoutMs}ms`));
          }, effect.timeoutMs);
          void run.then(
            (result) => {
              window.clearTimeout(timeoutId);
              resolve(result);
            },
            (error) => {
              window.clearTimeout(timeoutId);
              reject(error);
            },
          );
        })
      : await run;
    return { effectId: effect.id, status: 'ok', value };
  } catch (err) {
    const message = err instanceof Error ? err.message : String(err);
    if (effect.type === 'runExternalSync') {
      void notify(t('notifications.trakt_sync_failed_title'), message);
    }
    return {
      effectId: effect.id,
      status: 'err',
      error: message,
    };
  }
}

async function runWithConcurrency<T>(items: T[], limit: number, worker: (item: T) => Promise<void>): Promise<void> {
  let next = 0;
  const workers = Array.from({ length: Math.min(limit, items.length) }, async () => {
    while (next < items.length) {
      const item = items[next++];
      await worker(item);
    }
  });
  await Promise.all(workers);
}

export async function pumpEffects(
  effects: Effect[],
  onStateUpdate: (state: Partial<AppState>) => void,
  signal?: AbortSignal,
): Promise<Partial<AppState> | null> {
  let lastState: Partial<AppState> | null = null;

  const groups = new Map<string, Effect[]>();
  for (const effect of effects) {
    const key = effect.dedupeKey ?? effect.id;
    const group = groups.get(key);
    if (group) group.push(effect);
    else groups.set(key, [effect]);
  }

  const complete = async (effect: Effect, result: EffectResult) => {
      let dispatchResult: Awaited<ReturnType<typeof completeEffect>> = null;
      try {
        dispatchResult = await completeEffect({ ...result, effectId: effect.id });
      } catch {
        return;
      }
      // The request may have belonged to a profile/session that was replaced
      // while completeEffect was in flight. Never publish that stale result.
      if (signal?.aborted || !dispatchResult) return;

      lastState = dispatchResult.state;
      onStateUpdate(dispatchResult.state);
      if (dispatchResult.effects.length > 0) {
        await pumpEffects(dispatchResult.effects, onStateUpdate, signal);
      }
  };

  const scheduled = Array.from(groups.values()).sort((left, right) => (left[0].priority ?? 100) - (right[0].priority ?? 100));
  const scheduledByGroup = new Map<string, Effect[][]>();
  for (const duplicates of scheduled) {
    const key = duplicates[0].groupId ?? duplicates[0].type;
    const group = scheduledByGroup.get(key);
    if (group) group.push(duplicates);
    else scheduledByGroup.set(key, [duplicates]);
  }
  await Promise.all(Array.from(scheduledByGroup.entries()).map(async ([groupId, entries]) => {
    const limit = groupId === 'addon' ? 6 : groupId === 'plugin' ? 2 : 4;
    await runWithConcurrency(entries, limit, async (duplicates) => {
      if (signal?.aborted) return;
      const primary = duplicates[0];
      const result = await executeEffect(primary, onStateUpdate, signal);
      await Promise.all(duplicates.map((effect) => complete(effect, result)));
    });
  }));

  return lastState;
}

async function runTrailerEffects(effects: Effect[], requestId?: string): Promise<YoutubeTrailerResolution | null> {
  let pending = effects;
  while (pending.length > 0) {
    const next: Effect[] = [];
    for (const effect of pending) {
      const result = await executeEffect(effect);
      const completion = await completeEffect(result);
      if (!completion) continue;
      const resolution = requestId ? completion.state.trailer?.resolutions?.[requestId] : undefined;
      if (resolution && typeof resolution === 'object' && (resolution as { status?: unknown }).status === 'ok') {
        return resolution as YoutubeTrailerResolution;
      }
      next.push(...completion.effects);
    }
    pending = next;
  }
  return null;
}

async function proxyTrailerUrls(resolution: YoutubeTrailerResolution): Promise<YoutubeTrailerResolution> {
  const streamUrl = await registerTrailerProxyUrl(resolution.streamUrl);
  const audioUrl = resolution.audioUrl ? await registerTrailerProxyUrl(resolution.audioUrl) : resolution.audioUrl;
  return { ...resolution, streamUrl, audioUrl };
}

export async function resolveYoutubeTrailer(videoId: string): Promise<YoutubeTrailerResolution | null> {
  const requestId = crypto.randomUUID();
  const dispatch = await dispatchAction(JSON.stringify({ type: 'trailerResolveRequested', requestId, videoId, maxHeight: 1080 }));
  if (!dispatch) return null;
  const immediate = dispatch.state.trailer?.resolutions?.[requestId];
  if (immediate && typeof immediate === 'object' && (immediate as { status?: unknown }).status === 'ok') {
    return proxyTrailerUrls(immediate as YoutubeTrailerResolution);
  }
  const resolved = await runTrailerEffects(dispatch.effects, requestId);
  return resolved ? proxyTrailerUrls(resolved) : null;
}

export async function prewarmYoutubeTrailerConfig(): Promise<void> {
  const dispatch = await dispatchAction(JSON.stringify({ type: 'trailerPrewarmRequested' }));
  if (dispatch) await runTrailerEffects(dispatch.effects);
}

export async function fetchStreamsForEpisode(
  episodeId: string,
  contentType: string,
): Promise<{ streams: unknown[] }> {
  const result = await fetchDetailStreams({
    id: episodeId,
    contentType,
    requestIds: [episodeId],
  });
  return result as { streams: unknown[] };
}

export async function fetchPlaybackSkipSegments(opts: {
  imdbId: string;
  tmdbId?: number;
  season: number;
  episode: number;
  title: string;
  useSkipSegments?: boolean;
  useAnimeSkip?: boolean;
  animeSkipClientId?: string;
}): Promise<{ segments: IntroSegmentResult[]; coverage: Record<string, string[]> }> {
  return fetchIntroSegments(opts) as Promise<{ segments: IntroSegmentResult[]; coverage: Record<string, string[]> }>;
}
