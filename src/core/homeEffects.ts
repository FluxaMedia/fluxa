import {
  coreBuildMetadataFeedOptions,
  coreComputeContinueWatchingBadges,
  coreDiscoverCatalogOptions,
  coreEffectiveMetadataFeedSelection,
  coreMergeContinueWatchingLists,
} from './engine';
import { buildResourceUrl } from './addonManifest';
import { loadActiveProfile, loadAddons, loadLibrary, loadPrefs } from './libraryOps';
import { fetchPlannedResources } from './fetchPlanning';
import { tryFetchJson } from './httpClient';
import type { AddonDescriptor, Video } from './types';

interface MetadataFeedOption {
  key: string;
  label: string;
  homeTitle?: string;
  transportUrl: string;
  type: string;
  id: string;
  genre?: string | null;
}

export interface DiscoverCatalogOption {
  key: string;
  label: string;
  transportUrl: string;
  type: string;
  id: string;
  genres?: string[];
  requiresGenre?: boolean;
}

async function metadataFeedOptions(addons: AddonDescriptor[]): Promise<MetadataFeedOption[]> {
  const options = ((await coreBuildMetadataFeedOptions(addons)) ?? []) as MetadataFeedOption[];
  // Pre-index by transportUrl to avoid O(n×m) scan per option
  const addonByUrl = new Map(addons.map((a) => [a.transportUrl, a]));
  return options.map((option) => {
    const catalog = addonByUrl.get(option.transportUrl)
      ?.manifest
      ?.catalogs
      ?.find((item) => item.type === option.type && item.id === option.id);
    const genreExtra = catalog?.extra?.find((extra) => extra.name === 'genre');
    const defaultGenre = (genreExtra as { default?: unknown } | undefined)?.default;
    const firstOption = genreExtra?.options?.[0];
    const genre = typeof option.genre === 'string' && option.genre.trim()
      ? option.genre
      : typeof defaultGenre === 'string' && defaultGenre.trim()
        ? defaultGenre
        : genreExtra?.isRequired && typeof firstOption === 'string'
          ? firstOption
          : null;
    return { ...option, genre };
  });
}

export async function discoverCatalogOptions(addons: AddonDescriptor[], selectedType: string): Promise<DiscoverCatalogOption[]> {
  return ((await coreDiscoverCatalogOptions(addons, selectedType)) ?? []) as DiscoverCatalogOption[];
}

async function fetchMetaVideosForSeries(id: string, addons: AddonDescriptor[]): Promise<Video[]> {
  try {
    const values = await fetchPlannedResources({ kind: 'metaDetail', addons, contentType: 'series', id });
    const meta = (values.find((value) => (value as { meta?: unknown }).meta) as { meta?: { videos?: Video[] } } | undefined)?.meta;
    return Array.isArray(meta?.videos) ? meta.videos : [];
  } catch {
    return [];
  }
}

async function refreshReleasedContinueWatching(
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
  await Promise.all([...seriesIds].map(async (id) => {
    const videos = await fetchMetaVideosForSeries(id, addons);
    if (videos.length > 0) videosBySeriesId[id] = videos;
  }));

  // All decision logic lives in Rust
  const result = await coreComputeContinueWatchingBadges(
    JSON.stringify(items),
    JSON.stringify(videosBySeriesId),
    JSON.stringify(lastWatched),
    Date.now(),
  );
  return (result ?? []) as Record<string, unknown>[];
}

export async function readHomeBootstrap(
  payload: Record<string, unknown>,
): Promise<unknown> {
  const addons = await loadAddons();
  const library = await loadLibrary();
  const prefs = await loadPrefs();
  const language = (payload.language as string | undefined) ?? 'en';

  const localContinueWatching = (library.continueWatching as Record<string, unknown>[] | undefined) ?? [];
  const externalContinueWatching = (library.externalContinueWatching as Record<string, unknown>[] | undefined) ?? [];

  const progressMap = (library.progress as Record<string, unknown> | undefined) ?? {};
  const mergedCWRaw = await coreMergeContinueWatchingLists(
    JSON.stringify(localContinueWatching),
    JSON.stringify(externalContinueWatching),
    JSON.stringify(progressMap),
  );
  const mergedCW = (mergedCWRaw ?? []) as Record<string, unknown>[];
  const continueWatching = await refreshReleasedContinueWatching(mergedCW, library, addons);

  const categories: unknown[] = [];

  const metadataFeeds = await metadataFeedOptions(addons);
  const selectedKeys = prefs.homeFeedToggles as string[] | undefined;
  const availableKeys = metadataFeeds.map((feed) => feed.key);
  // [] means "all enabled" (same convention as isFeedEnabled in Settings).
  // Only call the Rust filter when there are explicit key selections; otherwise show all.
  const effectiveKeys = selectedKeys?.length
    ? ((await coreEffectiveMetadataFeedSelection(selectedKeys, availableKeys)) ?? availableKeys)
    : availableKeys;
  const visibleFeeds = metadataFeeds.filter((feed) => effectiveKeys.includes(feed.key));

  for (const feed of visibleFeeds) {
      const extraJson = feed.genre ? JSON.stringify({ genre: feed.genre }) : undefined;
      const url = await buildResourceUrl(feed.transportUrl, 'catalog', feed.type, feed.id, extraJson);
      const data = (await tryFetchJson(url)) as { metas?: unknown[] } | null;
      if (!data?.metas?.length) continue;
      categories.push({
        id: feed.key,
        name: feed.homeTitle ?? feed.label,
        semanticName: feed.homeTitle ?? feed.label,
        type: feed.type,
        items: data.metas,
        addonName: feed.label.split(' - ')[0] ?? feed.label,
        transportUrl: feed.transportUrl,
        catalogId: feed.id,
      });
  }

  // Collection shelf rows — mirrors Android HomeCatalogFeedCoordinator.buildUserCollectionHomeCategories.
  // Visible "collection" shelves contain folder tiles; hidden "collection_folder" categories hold
  // resolved transportUrls so the click handler can fetch catalog data without re-scanning addons.
  const profile = await loadActiveProfile();
  const pinnedCollections: unknown[] = [];
  const regularCollections: unknown[] = [];
  const hiddenFolderCategories: unknown[] = [];
  const libraryCollections = (profile as Record<string, unknown> | null)?.libraryCollections;
  const addonList = addons as unknown as Array<Record<string, unknown>>;

  function resolveTransportUrl(source: Record<string, unknown>): string | null {
    const srcAddonId = typeof source.addonId === 'string' ? source.addonId : null;
    const srcCatalogId = typeof source.catalogId === 'string' ? source.catalogId : null;
    const srcType = typeof source.type === 'string' ? source.type.trim().toLowerCase() : null;
    const normalizeType = (v: string | null) => {
      if (!v) return null;
      if (v === 'movies') return 'movie';
      if (v === 'series' || v === 'tv' || v === 'show' || v === 'shows') return 'series';
      return v;
    };
    for (const addon of addonList) {
      const manifest = addon.manifest as Record<string, unknown> | undefined;
      if (!manifest) continue;
      const addonId = typeof manifest.id === 'string' ? manifest.id : '';
      const tUrl = typeof addon.transportUrl === 'string' ? addon.transportUrl : '';
      if (srcAddonId && !(addonId.toLowerCase() === srcAddonId.toLowerCase() || tUrl.toLowerCase().includes(srcAddonId.toLowerCase()))) continue;
      const cats = Array.isArray(manifest.catalogs) ? manifest.catalogs as Array<Record<string, unknown>> : [];
      const match = cats.some((cat) =>
        cat.id === srcCatalogId &&
        normalizeType(typeof cat.type === 'string' ? cat.type : null) === normalizeType(srcType)
      );
      if (match) return tUrl;
    }
    return null;
  }

  if (Array.isArray(libraryCollections)) {
    for (const col of libraryCollections) {
      if (!col || typeof col !== 'object') continue;
      const c = col as Record<string, unknown>;
      if (!c.showOnHome) continue;
      const folders = Array.isArray(c.folders) ? c.folders : [];
      if (!folders.length) continue;

      const folderTiles: unknown[] = [];

      for (const f of folders) {
        if (!f || typeof f !== 'object') continue;
        const folder = f as Record<string, unknown>;
        const folderId = typeof folder.id === 'string' ? folder.id : String(Math.random());
        const folderTitle = typeof folder.title === 'string' ? folder.title : '';
        if (!folderTitle) continue;

        const rawSources = Array.isArray(folder.catalogSources) ? folder.catalogSources as Array<Record<string, unknown>> : [];
        const resolvedSources = rawSources
          .filter((s) => s && typeof s === 'object' && typeof s.catalogId === 'string')
          .flatMap((s) => {
            const tUrl = resolveTransportUrl(s);
            if (!tUrl) return [];
            return [{ transportUrl: tUrl, catalogId: String(s.catalogId), type: typeof s.type === 'string' ? s.type : 'movie', genre: typeof folder.genre === 'string' ? folder.genre : undefined }];
          });

        // fallback: single catalogId with no sources array
        if (!resolvedSources.length && typeof folder.catalogId === 'string') {
          const tUrl = resolveTransportUrl({ catalogId: folder.catalogId, type: 'movie' });
          if (tUrl) resolvedSources.push({ transportUrl: tUrl, catalogId: folder.catalogId, type: 'movie', genre: typeof folder.genre === 'string' ? folder.genre : undefined });
        }

        if (resolvedSources.length) {
          // hidden category — same pattern as Android's collection_folder HomeCategory
          hiddenFolderCategories.push({
            id: folderId,
            name: folderTitle,
            type: 'collection_folder',
            items: [],
            catalogSources: resolvedSources,
            addonGenre: typeof folder.genre === 'string' ? folder.genre : undefined,
            canLoadMore: false,
          });
        }

        const imgUrl = typeof folder.coverImageUrl === 'string' ? folder.coverImageUrl : typeof folder.imageUrl === 'string' ? folder.imageUrl : undefined;
        const focusGifEnabled = folder.focusGifEnabled !== false;
        folderTiles.push({
          id: folderId,
          type: 'catalog_folder',
          name: folderTitle,
          poster: imgUrl,
          background: typeof folder.heroBackdropUrl === 'string' ? folder.heroBackdropUrl : imgUrl,
          logo: typeof folder.titleLogoUrl === 'string' ? folder.titleLogoUrl : undefined,
          releaseInfo: typeof folder.catalogTitle === 'string' ? folder.catalogTitle : undefined,
          reason: typeof folder.shape === 'string' ? folder.shape : 'poster',
          focusGifUrl: focusGifEnabled && typeof folder.focusGifUrl === 'string' ? folder.focusGifUrl : undefined,
        });
      }

      if (!folderTiles.length) continue;

      const shelf = {
        id: typeof c.id === 'string' ? c.id : String(Math.random()),
        name: typeof c.title === 'string' ? c.title : '',
        type: 'collection',
        items: folderTiles,
        canLoadMore: false,
      };

      if (c.pinToTop) {
        pinnedCollections.push(shelf);
      } else {
        regularCollections.push(shelf);
      }
    }
  }

  // hidden collection_folder categories go at the end (invisible to the shelf renderer)
  const allCategories = [...pinnedCollections, ...categories, ...regularCollections, ...hiddenFolderCategories];

  const billboard =
    categories.length > 0
      ? ((categories[0] as { items: unknown[] }).items[0] ?? null)
      : null;

  return { categories: allCategories, continueWatching, metadataFeeds, billboard };
}
