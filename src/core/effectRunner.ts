import { completeEffect, dispatchAction, enqueueOfflineDownload } from './engine';
import { startTorrentStream, stopTorrentStream } from './mpvPlayer';
import { loadLibrary, loadPrefs, saveLibrary, buildContinueWatching } from './libraryOps';
import { readHomeBootstrap } from './homeEffects';
import {
  applyLibraryCommand,
  readCalendarMonth,
  readDetailLocalState,
  readLibraryState,
  readPlaybackProgress,
  writePlaybackProgress,
  writeSettings,
} from './libraryEffects';
import { fetchAddonManifest, fetchAddonResource, refreshInstalledAddons } from './addonEffects';
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
  replaceExternalContinueWatching,
  syncExternalIntegrationNow,
} from './externalSync';
import { fetchIntroSegments, fetchSubtitles, resolveIntroImdbId, type IntroSegmentResult } from './introEffects';
import type { AppState, Effect, EffectResult } from './types';

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

export async function executeEffect(
  effect: Effect,
  onStateUpdate?: (state: AppState) => void,
): Promise<EffectResult> {
  const p = effect.payload;
  try {
    let value: unknown;

    switch (effect.type) {
      case 'readHomeBootstrap':
        value = await readHomeBootstrap(p);
        break;

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
        const id = ((p.meta as Record<string, unknown>) ?? {}).id as string | undefined;
        if (id) {
          const progressMap = (lib.progress as Record<string, unknown> | undefined) ?? {};
          delete progressMap[id];
          lib.progress = progressMap;
          lib.continueWatching = await buildContinueWatching(progressMap);
          const extCW = (lib.externalContinueWatching as Record<string, unknown>[] | undefined) ?? [];
          const droppedExternal = extCW.find((item) => item.id === id);
          lib.externalContinueWatching = extCW.filter((item) => item.id !== id);
          // Also clean lastWatchedEpisodes so the badge computer doesn't recreate a phantom entry
          const lastWatched = (lib.lastWatchedEpisodes as Record<string, unknown> | undefined) ?? {};
          delete lastWatched[id];
          lib.lastWatchedEpisodes = lastWatched;
          await saveLibrary(lib);
          // Fire-and-forget: remove progress on the external service too
          if (droppedExternal) {
            void dropExternalPlaybackProgress(droppedExternal);
          }
        }
        // Return droppedId so the engine can remove it from home.continueWatching in state
        value = id ? { droppedId: id } : {};
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

      case 'fetchCatalogPage':
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
        value = await fetchDetailStreams(p, onStateUpdate);
        break;
      case 'fetchSeasonEpisodes':
        value = await fetchSeasonEpisodes(p);
        break;
      case 'loadStreams':
        value = await fetchDetailStreams(p, onStateUpdate);
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

      // Platform-only side effects (no-ops on desktop)
      case 'updateCalendarWidget':
      case 'notifyReleasedEpisodes':
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

    return { effectId: effect.id, status: 'ok', value };
  } catch (err) {
    return {
      effectId: effect.id,
      status: 'err',
      error: err instanceof Error ? err.message : String(err),
    };
  }
}

export async function pumpEffects(
  effects: Effect[],
  onStateUpdate: (state: AppState) => void,
): Promise<AppState | null> {
  let lastState: AppState | null = null;

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
}): Promise<IntroSegmentResult[]> {
  return fetchIntroSegments(opts) as Promise<IntroSegmentResult[]>;
}
