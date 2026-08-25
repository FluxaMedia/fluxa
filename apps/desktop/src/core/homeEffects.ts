import {
  coreBuildHomeCollectionShelves,
  coreBuildMetadataFeedOptions,
  coreComputeContinueWatchingBadges,
  coreContinueWatchingForSource,
  coreDiscoverCatalogOptions,
  coreEffectiveMetadataFeedSelection,
  coreInvoke,
  coreNuvioResolveContinueWatching,
  coreNuvioProgressMetaNeeds,
  coreResolveFeedOptionGenre,
  storageRead,
  storageWrite,
} from './engine';
import { platformInvoke as invoke } from '../platform/invoke';
import { buildResourceUrl } from './addonManifest';
import { buildContinueWatching, effectRunnerLibraryKey, loadActiveProfile, loadEnabledAddons, loadLibrary, loadPrefs } from './libraryOps';
import { fetchBuiltinCatalog, isBuiltinTmdbAddon, withBuiltinTmdbAddon } from './tmdbAddon';
import { fetchPlannedResources, fetchVideosForSeries, runWithConcurrency } from './fetchPlanning';
import { tryFetchJson } from './httpClient';
import { loadProviderLibraries, type LibraryProvider } from './providerLibraries';
import { nuvioPullCollections, nuvioPullLibrary, nuvioPullWatchProgress } from './nuvioApi';
import { coreNuvioImportMergePlan, coreNuvioMapCollections } from './engineCoreLibrary';
import { fetchMetaDetail } from './detailEffects';
import type { AddonDescriptor } from './types';

const HOME_FEED_FETCH_CONCURRENCY = 6;
const HOME_BOOTSTRAP_CACHE_PREFIX = 'home_bootstrap_v1';

interface MetadataFeedOption {
  key: string;
  label: string;
  homeTitle?: string;
  transportUrl: string;
  type: string;
  id: string;
  genre?: string | null;
}

interface HomeBootstrapCache {
  categories: unknown[];
  continueWatching: Record<string, unknown>[];
  metadataFeeds: MetadataFeedOption[];
  billboard: unknown;
}

interface ContinueWatchingSourcePlan {
  source: string;
  provider: string | null;
}

async function selectedContinueWatchingSource(prefs: Record<string, unknown>): Promise<ContinueWatchingSourcePlan> {
  return (
    (await coreInvoke<ContinueWatchingSourcePlan>(
      'continueWatchingSourcePlan',
      JSON.stringify({ source: prefs.continueWatchingSource }),
    )) ?? { source: 'local', provider: null }
  );
}

export interface DiscoverCatalogOption {
  key: string;
  label: string;
  transportUrl: string;
  type: string;
  id: string;
  extras?: Array<{
    name: string;
    options: string[];
    isRequired?: boolean;
  }>;
}

async function metadataFeedOptions(addons: AddonDescriptor[]): Promise<MetadataFeedOption[]> {
  const options = ((await coreBuildMetadataFeedOptions(addons)) ?? []) as MetadataFeedOption[];
  const addonsJson = JSON.stringify(addons);
  return Promise.all(
    options.map(async (option) => {
      const genre = await coreResolveFeedOptionGenre(JSON.stringify(option), addonsJson);
      return { ...option, genre };
    }),
  );
}

export async function discoverCatalogOptions(addons: AddonDescriptor[], selectedType: string): Promise<DiscoverCatalogOption[]> {
  const withBuiltin = await withBuiltinTmdbAddon(addons, await loadPrefs());
  return ((await coreDiscoverCatalogOptions(withBuiltin, selectedType)) ?? []) as DiscoverCatalogOption[];
}

export async function refreshReleasedContinueWatching(
  items: Record<string, unknown>[],
  library: Record<string, unknown>,
  addons: AddonDescriptor[],
): Promise<Record<string, unknown>[]> {
  // Fetch addon videos for every series candidate (I/O — must stay in platform)
  const lastWatched = (library.lastWatchedEpisodes as Record<string, unknown> | undefined) ?? {};
  const seriesIds = new Set<string>();
  for (const item of items) {
    if (item.type === 'series') seriesIds.add(String(item.id ?? item._id ?? ''));
  }
  for (const id of Object.keys(lastWatched)) seriesIds.add(id);

  const videosBySeriesId: Record<string, unknown[]> = {};
  const seriesIdList = [...seriesIds];
  const fetchedVideos = await runWithConcurrency(seriesIdList, 3, (id) => fetchVideosForSeries(id, addons));
  fetchedVideos.forEach((videos, index) => {
    if (videos.length > 0) videosBySeriesId[seriesIdList[index]] = videos;
  });

  // All decision logic lives in Rust
  const result = await coreComputeContinueWatchingBadges(
    JSON.stringify(items),
    JSON.stringify(videosBySeriesId),
    JSON.stringify(lastWatched),
    Date.now(),
  );
  return (result ?? []) as Record<string, unknown>[];
}

async function continueWatchingFromCompactProgress(
  library: Record<string, unknown>,
  addons: AddonDescriptor[],
): Promise<Record<string, unknown>[]> {
  const progressMap = (library.progress as Record<string, Record<string, unknown>> | undefined) ?? {};
  const libraryItems = Object.entries(progressMap).map(([key, entry]) => {
    const meta = (entry.meta as Record<string, unknown> | undefined) ?? {};
    return {
      content_id: String(entry.contentId ?? meta.id ?? key),
      content_type: String(entry.contentType ?? meta.type ?? (entry.lastEpisodeSeason != null ? 'series' : 'movie')),
      name: meta.name ?? null,
      poster: meta.poster ?? null,
      background: meta.background ?? null,
    };
  });
  const watchProgress = Object.entries(progressMap).map(([key, entry]) => ({
    content_id: String(entry.contentId ?? (entry.meta as Record<string, unknown> | undefined)?.id ?? key),
    content_type: String(
      entry.contentType ??
        (entry.meta as Record<string, unknown> | undefined)?.type ??
        (entry.lastEpisodeSeason != null ? 'series' : 'movie'),
    ),
    video_id: entry.videoId ?? entry.lastVideoId ?? null,
    season: entry.season ?? entry.lastEpisodeSeason ?? null,
    episode: entry.episode ?? entry.lastEpisodeNumber ?? null,
    position: entry.position ?? entry.timeOffset ?? 0,
    duration: entry.duration ?? 0,
    last_watched:
      typeof entry.lastWatched === 'number' ? entry.lastWatched : Date.parse(String(entry.lastWatched ?? entry.savedAt ?? '')) || 0,
    progress_key: entry.progressKey ?? key,
  }));
  if (watchProgress.length === 0) return [];

  const needs = (await coreNuvioProgressMetaNeeds(watchProgress, libraryItems)) ?? [];
  const fetchedMetadata = await runWithConcurrency(needs, 3, async (need) => {
    const values = await fetchPlannedResources({
      kind: 'metaDetail',
      addons,
      contentType: need.contentType,
      id: need.contentId,
    }).catch(() => []);
    const result = values.find((value) => value && typeof value === 'object' && 'meta' in value) as
      { meta?: Record<string, unknown> } | undefined;
    return [need.contentId, result?.meta ?? null] as const;
  });
  const addonMetas = Object.fromEntries(fetchedMetadata.filter(([, meta]) => meta));
  const resolved = await coreNuvioResolveContinueWatching(watchProgress, addonMetas);
  const mapped = await coreNuvioImportMergePlan({
    progress: progressMap,
    watched: (library.watched as Record<string, boolean> | undefined) ?? {},
    library: libraryItems,
    addonMetas,
    watchProgress: resolved ?? watchProgress,
    watchHistory: [],
    categories: ['continueWatching'],
  });
  return (await buildContinueWatching(mapped?.progress ?? progressMap)) as Record<string, unknown>[];
}

export async function continueWatchingForSelectedSource(
  library: Record<string, unknown>,
  prefs: Record<string, unknown>,
  addons: AddonDescriptor[],
): Promise<Record<string, unknown>[]> {
  const profile = await loadActiveProfile();
  const effectivePrefs = profile?.nuvioAccessToken ? { ...prefs, continueWatchingSource: 'nuvio' } : prefs;
  const requestedSource = String(effectivePrefs.continueWatchingSource ?? 'local');
  void invoke('debug_log', { msg: `cw-source: resolving plan source=${requestedSource}` });
  const plan = await selectedContinueWatchingSource(effectivePrefs);
  const provider = plan.provider;
  void invoke('debug_log', { msg: `cw-source: resolved plan source=${plan.source} provider=${provider ?? 'local'}` });

  if (provider === 'nuvio') {
    if (!profile?.nuvioAccessToken) return [];
    const profileId = profile.nuvioProfileIndex ?? 1;
    const [libraryItems, progressItems] = await Promise.all([
      nuvioPullLibrary(profile.nuvioAccessToken, profileId),
      nuvioPullWatchProgress(profile.nuvioAccessToken, profileId),
    ]);
    const metadataNeeds = (await coreNuvioProgressMetaNeeds(progressItems, libraryItems)) ?? [];
    const fetchedMetadata = await runWithConcurrency(metadataNeeds, 3, async (need) => {
      const values = await fetchPlannedResources({
        kind: 'metaDetail',
        addons,
        contentType: need.contentType,
        id: need.contentId,
      }).catch(() => []);
      const result = values.find((value) => value && typeof value === 'object' && 'meta' in value) as
        { meta?: Record<string, unknown> } | undefined;
      return [need.contentId, result?.meta ?? null] as const;
    });
    const metaById: Record<string, unknown> = Object.fromEntries(
      libraryItems
        .filter((item) => item.content_id)
        .map((item) => [String(item.content_id), { name: item.name, poster: item.poster, background: item.background }]),
    );
    for (const [id, detail] of fetchedMetadata) {
      if (detail) metaById[id] = { ...(metaById[id] as Record<string, unknown> | undefined), ...detail };
    }
    return (
      (await coreContinueWatchingForSource({
        source: 'nuvio',
        watchProgress: progressItems,
        metaById,
        prefs,
      })) ?? []
    );
  }

  if (provider) {
    void invoke('debug_log', { msg: `cw-source: loading provider library provider=${provider}` });
    const libraries = await loadProviderLibraries();
    const items = libraries[provider as LibraryProvider]?.watching ?? [];
    void invoke('debug_log', {
      msg: `cw-source: loaded provider library provider=${provider} count=${items.length} ids=${items.map((item) => item.id ?? item._id).join(',')}`,
    });
    return (await coreContinueWatchingForSource({ source: provider, providerWatching: items })) ?? [];
  }

  return continueWatchingFromCompactProgress(library, addons);
}

export async function readHomeBootstrap(payload: Record<string, unknown>, signal?: AbortSignal): Promise<unknown> {
  const language = (payload.language as string | undefined) ?? 'en';
  console.debug('[fluxa:web:home:start]', { force: payload.force === true, language });
  const cacheKey = `${HOME_BOOTSTRAP_CACHE_PREFIX}_${await effectRunnerLibraryKey()}_${language}`;
  if (!payload.force) {
    const cached = await storageRead<HomeBootstrapCache>(cacheKey);
    if (cached) return { ...cached, stale: true };
    return { stale: true };
  }

  const profile = await loadActiveProfile();
  const enabledAddons = await loadEnabledAddons();
  const library = await loadLibrary();
  const prefs = await loadPrefs();
  const addons = await withBuiltinTmdbAddon(enabledAddons, prefs);

  const continueWatching = await continueWatchingForSelectedSource(library, prefs, addons);

  const metadataFeeds = await metadataFeedOptions(addons);
  const selectedKeys = prefs.homeFeedToggles as string[] | undefined;
  const availableKeys = metadataFeeds.map((feed) => feed.key);
  // [] means "all enabled" (same convention as isFeedEnabled in Settings).
  // Only call the Rust filter when there are explicit key selections; otherwise show all.
  const effectiveKeys = selectedKeys?.length
    ? ((await coreEffectiveMetadataFeedSelection(selectedKeys, availableKeys)) ?? availableKeys)
    : availableKeys;
  const visibleFeeds = metadataFeeds.filter((feed) => effectiveKeys.includes(feed.key));
  console.debug('[fluxa:web:home:feeds]', {
    addons: addons.length,
    metadataFeeds: metadataFeeds.length,
    visibleFeeds: visibleFeeds.length,
  });

  const categoryResults = await runWithConcurrency(visibleFeeds, HOME_FEED_FETCH_CONCURRENCY, async (feed) => {
    const extra = feed.genre ? { genre: feed.genre } : {};
    const url = isBuiltinTmdbAddon(feed.transportUrl)
      ? null
      : await buildResourceUrl(feed.transportUrl, 'catalog', feed.type, feed.id, JSON.stringify(extra));
    const startedAt = performance.now();
    console.debug('[fluxa:web:home:catalog:start]', { feed: feed.key, url });
    const data = isBuiltinTmdbAddon(feed.transportUrl)
      ? await fetchBuiltinCatalog(feed.type, extra, String(prefs.tmdbApiKey ?? ''), language, signal)
      : ((await tryFetchJson(url!, { signal })) as { metas?: unknown[] } | null);
    console.debug('[fluxa:web:home:catalog:end]', {
      feed: feed.key,
      url,
      metas: Array.isArray(data?.metas) ? data.metas.length : 0,
      elapsedMs: Math.round(performance.now() - startedAt),
    });
    const metas = Array.isArray(data?.metas) ? data.metas : [];
    if (metas.length === 0) return null;
    const items = metas.map((m) =>
      m && typeof m === 'object'
        ? { ...(m as Record<string, unknown>), sourceAddonTransportUrl: feed.transportUrl, sourceAddonCatalogType: feed.type }
        : m,
    );
    return {
      id: feed.key,
      name: feed.homeTitle ?? feed.label,
      semanticName: feed.homeTitle ?? feed.label,
      type: feed.type,
      items,
      addonName: feed.label.split(' - ')[0] ?? feed.label,
      transportUrl: feed.transportUrl,
      catalogId: feed.id,
    };
  });
  const categories = categoryResults.filter((c): c is NonNullable<typeof c> => c !== null);

  let collectionProfile = profile ?? {};
  if (profile?.nuvioAccessToken) {
    const remoteCollections = await nuvioPullCollections(profile.nuvioAccessToken, profile.nuvioProfileIndex ?? 1).catch(() => []);
    const rawCollections = (remoteCollections[0]?.collections_json ?? []) as unknown[];
    const mappedCollections = await coreNuvioMapCollections(rawCollections);
    collectionProfile = { ...profile, libraryCollections: mappedCollections ?? [] };
  }
  const collectionShelves = await coreBuildHomeCollectionShelves(JSON.stringify(collectionProfile), JSON.stringify(addons));
  const pinnedCollections = collectionShelves?.pinnedShelves ?? [];
  const regularCollections = collectionShelves?.regularShelves ?? [];
  const hiddenFolderCategories = collectionShelves?.hiddenFolderCategories ?? [];

  const allCategories = [...pinnedCollections, ...categories, ...regularCollections, ...hiddenFolderCategories];

  const billboard = categories.length > 0 ? ((categories[0] as { items: unknown[] }).items[0] ?? null) : null;

  const bootstrap = { categories: allCategories, continueWatching, metadataFeeds, billboard };
  console.debug('[fluxa:web:home:end]', { categories: allCategories.length, continueWatching: continueWatching.length });
  void storageWrite(cacheKey, bootstrap);
  return bootstrap;
}

const HERO_DESCRIPTION_CACHE_KEY = 'hero_description_cache';
const HERO_DESCRIPTION_CACHE_TTL_MS = 24 * 60 * 60 * 1000;
const HERO_DESCRIPTION_CACHE_MAX_AGE_MS = 30 * 24 * 60 * 60 * 1000;

type HeroDescriptionCache = Record<string, { fetchedAt: number; description: string | null }>;

let heroDescriptionCachePromise: Promise<HeroDescriptionCache> | null = null;
function loadHeroDescriptionCache(): Promise<HeroDescriptionCache> {
  if (!heroDescriptionCachePromise) {
    heroDescriptionCachePromise = storageRead<HeroDescriptionCache>(HERO_DESCRIPTION_CACHE_KEY).then((cache) => cache ?? {});
  }
  return heroDescriptionCachePromise;
}
void loadHeroDescriptionCache();

export async function fetchHeroDescription(item: { id: string; type: string; sourceAddonTransportUrl?: string }): Promise<string | null> {
  const cacheKey = `${item.type}:${item.id}`;
  const now = Date.now();
  const cache = await loadHeroDescriptionCache();
  const cached = cache[cacheKey];
  if (cached && now - cached.fetchedAt < HERO_DESCRIPTION_CACHE_TTL_MS) {
    return cached.description;
  }

  const detail = (await fetchMetaDetail({
    id: item.id,
    contentType: item.type,
    sourceAddonTransportUrl: item.sourceAddonTransportUrl,
  }).catch(() => null)) as { description?: string } | null;
  const shortened = detail?.description
    ? ((await coreInvoke<string>('shortenSynopsis', JSON.stringify({ text: detail.description }))) ?? null)
    : null;

  cache[cacheKey] = { fetchedAt: now, description: shortened };
  for (const [key, entry] of Object.entries(cache)) {
    if (now - entry.fetchedAt > HERO_DESCRIPTION_CACHE_MAX_AGE_MS) delete cache[key];
  }
  void storageWrite(HERO_DESCRIPTION_CACHE_KEY, cache);

  return shortened;
}
