import {
  coreNuvioCanonicalContentType,
  coreBuildHomeCollectionShelves,
  coreBuildMetadataFeedOptions,
  coreComputeContinueWatchingBadges,
  coreContinueWatchingForSource,
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
import type { AddonDescriptor, AppState } from './types';
import { startPerfSpan } from './performance';

const HOME_FEED_FETCH_CONCURRENCY = 8;
const CONTINUE_WATCHING_METADATA_CONCURRENCY = 8;
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
  const progressEntries = await Promise.all(Object.entries(progressMap).map(async ([key, entry]) => {
    const meta = (entry.meta as Record<string, unknown> | undefined) ?? {};
    const rawType = String(entry.contentType ?? meta.type ?? (entry.lastEpisodeSeason != null ? 'series' : 'movie'));
    return { key, entry, meta, contentType: await coreNuvioCanonicalContentType(rawType) };
  }));
  const libraryItems = progressEntries.map(({ key, entry, meta, contentType }) => {
    return {
      content_id: String(entry.contentId ?? meta.id ?? key),
      content_type: contentType,
      name: meta.name ?? null,
      poster: meta.poster ?? null,
      background: meta.background ?? null,
    };
  });
  const watchProgress = progressEntries.map(({ key, entry, contentType }) => ({
    content_id: String(entry.contentId ?? (entry.meta as Record<string, unknown> | undefined)?.id ?? key),
    content_type: contentType,
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
  const uniqueNeeds = [...new Map(needs.map((need) => [`${need.contentType}:${need.contentId}`, need])).values()];
  console.debug('[fluxa:home:continue-watching:metadata-needs]', JSON.stringify({ needs: needs.length, unique: uniqueNeeds.length }));
  const metadataPerf = startPerfSpan('home.continue-watching.metadata', { needs: needs.length, uniqueNeeds: uniqueNeeds.length });
  const fetchedMetadata = await runWithConcurrency(uniqueNeeds, CONTINUE_WATCHING_METADATA_CONCURRENCY, async (need) => {
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
  metadataPerf.end({ fetched: fetchedMetadata.length });
  const addonMetas = Object.fromEntries(fetchedMetadata.filter(([, meta]) => meta));
  const resolvePerf = startPerfSpan('home.continue-watching.resolve', { metadata: Object.keys(addonMetas).length });
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
  const result = (await buildContinueWatching(mapped?.progress ?? progressMap)) as Record<string, unknown>[];
  resolvePerf.end({ result: result.length });
  return result;
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
    const fetchedMetadata = await runWithConcurrency(metadataNeeds, CONTINUE_WATCHING_METADATA_CONCURRENCY, async (need) => {
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

export async function readHomeBootstrap(
  payload: Record<string, unknown>,
  signal?: AbortSignal,
  onStateUpdate?: (state: Partial<AppState>) => void,
): Promise<unknown> {
  const language = (payload.language as string | undefined) ?? 'en';
  const perf = startPerfSpan('home.bootstrap', { language, force: payload.force === true });
  const firstContentPerf = startPerfSpan('home.first-content', { parentTraceId: perf.traceId });
  const setupPerf = startPerfSpan('home.setup', { parentTraceId: perf.traceId });
  console.debug('[fluxa:web:home:start]', { force: payload.force === true, language });
  const cacheKey = `${HOME_BOOTSTRAP_CACHE_PREFIX}_${await effectRunnerLibraryKey()}_${language}`;
  if (!payload.force) {
    const cached = await storageRead<HomeBootstrapCache>(cacheKey);
    if (cached) {
      perf.end({ cacheHit: true, categories: cached.categories.length, continueWatching: cached.continueWatching.length });
      return { ...cached, stale: true };
    }
  }

  const [profile, enabledAddons, library, prefs] = await Promise.all([
    loadActiveProfile(),
    loadEnabledAddons(),
    loadLibrary(),
    loadPrefs(),
  ]);
  const addons = await withBuiltinTmdbAddon(enabledAddons, prefs);
  setupPerf.end({ addons: addons.length });

  const continueWatchingPerf = startPerfSpan('home.continue-watching', { parentTraceId: perf.traceId });
  const continueWatchingPromise = continueWatchingForSelectedSource(library, prefs, addons).then((items) => {
    continueWatchingPerf.end({ items: items.length });
    return items;
  });

  const collectionsPerf = startPerfSpan('home.collections', { parentTraceId: perf.traceId });
  const collectionsPromise = (async () => {
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
    collectionsPerf.end({ pinned: pinnedCollections.length, regular: regularCollections.length, hidden: hiddenFolderCategories.length });
    return { pinnedCollections, regularCollections, hiddenFolderCategories };
  })();

  const feedsPerf = startPerfSpan('home.feed-options', { parentTraceId: perf.traceId });
  const metadataFeeds = await metadataFeedOptions(addons);
  const selectedKeys = prefs.homeFeedToggles as string[] | undefined;
  const availableKeys = metadataFeeds.map((feed) => feed.key);
  // [] means "all enabled" (same convention as isFeedEnabled in Settings).
  // Only call the Rust filter when there are explicit key selections; otherwise show all.
  const effectiveKeys = selectedKeys?.length
    ? ((await coreEffectiveMetadataFeedSelection(selectedKeys, availableKeys)) ?? availableKeys)
    : availableKeys;
  const visibleFeeds = metadataFeeds.filter((feed) => effectiveKeys.includes(feed.key));
  feedsPerf.end({ feeds: metadataFeeds.length, visibleFeeds: visibleFeeds.length });
  console.debug('[fluxa:web:home:feeds]', {
    addons: addons.length,
    metadataFeeds: metadataFeeds.length,
    visibleFeeds: visibleFeeds.length,
  });

  const categoriesPerf = startPerfSpan('home.categories', { parentTraceId: perf.traceId, feeds: visibleFeeds.length });
  const categoryResults = await runWithConcurrency(visibleFeeds, HOME_FEED_FETCH_CONCURRENCY, async (feed) => {
    const feedPerf = startPerfSpan(`home.feed:${feed.key}`, { parentTraceId: categoriesPerf.traceId });
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
    feedPerf.end({ metas: metas.length });
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
  categoriesPerf.end({ categories: categories.length });

  const categoryBillboard = categories.length > 0 ? ((categories[0] as { items: unknown[] }).items[0] ?? null) : null;

  // Categories are usable before collections and Continue Watching finish.
  // Publish this first snapshot as soon as the feed requests complete.
  firstContentPerf.end({ categories: categories.length, feeds: visibleFeeds.length, aborted: Boolean(signal?.aborted) });
  if (!signal?.aborted) {
    onStateUpdate?.({
      home: {
        categories: categories as AppState['home']['categories'],
        metadataFeeds,
        billboard: categoryBillboard as AppState['home']['billboard'],
        isLoading: false,
      },
    });
  }

  void continueWatchingPromise.then((items) => {
    if (signal?.aborted) return;
    onStateUpdate?.({
      home: {
        categories: categories as AppState['home']['categories'],
        continueWatching: items as unknown as AppState['home']['continueWatching'],
        metadataFeeds,
        billboard: categoryBillboard as AppState['home']['billboard'],
        isLoading: false,
      },
    });
  });

  const { pinnedCollections, regularCollections, hiddenFolderCategories } = await collectionsPromise;

  const allCategories = [...pinnedCollections, ...categories, ...regularCollections, ...hiddenFolderCategories];

  const billboard = categoryBillboard;

  const bootstrap = { categories: allCategories, metadataFeeds, billboard };
  perf.end({ cacheHit: false, categories: allCategories.length, continueWatching: 'deferred', feeds: visibleFeeds.length });
  console.debug('[fluxa:web:home:end]', { categories: allCategories.length, continueWatching: 'deferred' });
  void continueWatchingPromise.then((continueWatching) => storageWrite(cacheKey, { ...bootstrap, continueWatching }));
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
    purpose: 'hero-description',
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
