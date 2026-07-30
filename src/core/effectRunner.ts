import * as Sentry from '@sentry/react';
import { completeEffect, coreInvoke, coreMergeContinueWatchingLists, dispatchAction, enqueueOfflineDownload, httpExecuteText, libraryContinueWatchingDelete, libraryProgressDelete, registerTrailerProxyUrl } from './engine';
import { startTorrentStream, stopTorrentStream } from './mpvPlayer';
import { effectRunnerLibraryKey, loadActiveProfile, loadAddons, loadLibrary, loadPrefs, saveLibrary, persistLastWatchedEpisode } from './libraryOps';
import { readHomeBootstrap, refreshReleasedContinueWatching } from './homeEffects';
import { invalidateCalendarCache } from './libraryEffects';
import {
  applyLibraryCommand,
  notifyReleasedEpisodes,
  readCalendarMonth,
  readDetailLocalState,
  readLibraryState,
  readPlaybackProgress,
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
  const url = await startTorrentStream(JSON.stringify(stream), title, prefs);
  return { url };
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
  const videos = await fetchVideosForSeries(id, await loadAddons());
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
): Promise<unknown> {
  const p = effect.payload;
  let value: unknown;

  switch (effect.type) {
    case 'readHomeBootstrap':
      value = await readHomeBootstrap(p);
      break;

    case 'refreshContinueWatching': {
      const lib = await loadLibrary();
      const addons = await loadAddons();
      const prefs = await loadPrefs();
      const localCW = (lib.continueWatching as Record<string, unknown>[] | undefined) ?? [];
      const externalCW = (lib.externalContinueWatching as Record<string, unknown>[] | undefined) ?? [];
      const progressMap = (lib.progress as Record<string, unknown> | undefined) ?? {};
      const mergedCWRaw = await coreMergeContinueWatchingLists(
        JSON.stringify(localCW),
        JSON.stringify(externalCW),
        JSON.stringify(progressMap),
        prefs.syncCwSourceOfTruth as string | undefined,
        prefs.syncCwRanking as string | undefined,
      );
      const mergedCW = (mergedCWRaw ?? []) as Record<string, unknown>[];
      const continueWatching = await refreshReleasedContinueWatching(
        mergedCW,
        lib as Record<string, unknown>,
        addons,
      );
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
      value = {};
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
      value = await fetchAddonManifest(p);
      break;
    case 'refreshInstalledAddons':
      value = await refreshInstalledAddons(p);
      break;
    case 'fetchAddonResource':
      value = await fetchAddonResource(p);
      break;
    case 'fetchPluginManifest':
      value = await fetchPluginManifestEffect(p);
      break;

    case 'fetchYoutubeTrailerWatchConfig':
    case 'fetchYoutubeTrailerPlayer':
    case 'fetchYoutubeTrailerPlayerScript':
      value = await executeYoutubeTrailerRequest(p);
      break;

    case 'fetchCatalogPage':
      value = await fetchCatalogPage(p);
      break;
    case 'fetchDiscoverPage':
      value = await fetchCatalogPage(p);
      break;
    case 'runSearch':
      value = await runSearch(p);
      break;
    case 'runDiscover':
      value = await runDiscover(p);
      break;

    case 'fetchMetaDetail':
    case 'fetchMetaDetailLookup':
      value = await fetchMetaDetail(p);
      break;
    case 'fetchDetailSecondary':
      value = await fetchDetailSecondary(p);
      break;
    case 'prefetchDetailStreams':
      value = await prefetchDetailStreams(p);
      break;
    case 'fetchDetailStreams':
      value = await fetchDetailStreams(p, onStateUpdate, effect.generation);
      break;
    case 'fetchSeasonEpisodes':
      value = await fetchSeasonEpisodes(p);
      break;
    case 'loadStreams':
      value = await fetchDetailStreams(p);
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
      value = await fetchDetailStreams(p);
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
): Promise<EffectResult> {
  try {
    const value = await Sentry.startSpan(
      { name: effect.type, op: 'fluxa.effect' },
      () => runEffect(effect, onStateUpdate),
    );
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

export async function pumpEffects(
  effects: Effect[],
  onStateUpdate: (state: Partial<AppState>) => void,
): Promise<Partial<AppState> | null> {
  let lastState: Partial<AppState> | null = null;

  // Execute effects concurrently per batch, then feed completions back
  await Promise.all(
    effects.map(async (effect) => {
      const result = await executeEffect(effect, onStateUpdate);
      let dispatchResult: Awaited<ReturnType<typeof completeEffect>> = null;
      try {
        dispatchResult = await completeEffect(result);
      } catch {
      }
      if (dispatchResult) {
        lastState = dispatchResult.state;
        onStateUpdate(dispatchResult.state);
        if (dispatchResult.effects.length > 0) {
          await pumpEffects(dispatchResult.effects, onStateUpdate);
        }
      }
    }),
  );

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
  season: number;
  episode: number;
  title: string;
  useIntroDb?: boolean;
  useAniSkip?: boolean;
  useAnimeSkip?: boolean;
  animeSkipClientId?: string;
}): Promise<IntroSegmentResult[]> {
  return fetchIntroSegments(opts) as Promise<IntroSegmentResult[]>;
}
