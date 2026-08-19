import React, { useCallback, useEffect, useLayoutEffect, useMemo, useRef, useState } from 'react';
import { HeroSection } from '../components/HeroSection';
import { ShelfRow } from '../components/ShelfRow';
import { CategoryGridScreen } from './CategoryGridScreen';
import { ContinueWatchingRow } from '../components/ContinueWatchingRow';
import { ThisWeekRow } from '../components/ThisWeekRow';
import { partitionThisWeek } from '../core/continueWatchingUtils';
import { CollectionShelfRow } from '../components/CollectionShelfRow';
import { posterPrefsFromState } from '../core/posterPrefs';
import { appPrefs, prefBool, prefString } from '../core/appPrefs';
import { buildResourceUrl } from '../core/addonManifest';
import { coreInvoke, coreResolveTransportUrl, httpFetchText } from '../core/engine';

import { prewarmYoutubeTrailerConfig } from '../core/effectRunner';
import { fetchContentLogo, fetchTmdbTrailers } from '../core/detailEffects';
import { fetchHeroDescription } from '../core/homeEffects';
import type { AppState, HomeCategory, Meta, NuvioRemoteCollectionSource, Trailer } from '../core/types';
import { getLanguage, t } from '../i18n';
import { useInViewport } from '../hooks/useInViewport';
import { loadNuvioCollectionSource } from '../core/collectionSources';
import { fetchBuiltinCatalog } from '../core/tmdbAddon';
import { loadPrefs } from '../core/libraryOps';

const ROW_PLACEHOLDER_HEIGHT = 340;

function LazyRow({ children }: { children: React.ReactNode }) {
  const ref = useRef<HTMLDivElement>(null);
  const inViewport = useInViewport(ref, '1000px');
  const shownRef = useRef(false);
  if (inViewport) shownRef.current = true;
  return (
    <div
      ref={ref}
      style={
        {
          contentVisibility: 'auto',
          containIntrinsicSize: `100% ${ROW_PLACEHOLDER_HEIGHT}px`,
          minHeight: shownRef.current ? undefined : ROW_PLACEHOLDER_HEIGHT,
        } as React.CSSProperties
      }
    >
      {shownRef.current ? children : null}
    </div>
  );
}

interface Props {
  state: Pick<AppState, 'home' | 'settings' | 'addons'>;
  onDispatch: (actionJson: string) => void | Promise<void>;
  onNavigateDetail: (meta: Meta) => void;
  onPlay: (meta: Meta) => void;
  onResume: (meta: Meta) => void;
  onStartOver: (meta: Meta) => void;
  onPlayManually: (meta: Meta) => void;
  onOpenSettings?: () => void;
  // Home stays mounted while hidden (it's the heaviest screen — re-mounting it on every
  // visit was costing a visible stutter), so this tells HeroSection to pause its
  // auto-slide interval rather than keep cycling backdrop images forever in the background.
  isActive: boolean;
  onScrolledChange?: (scrolled: boolean) => void;
  // Bumped when the user clicks "Home" while already on the home route, so we can
  // exit a folder's "view all" grid rather than doing nothing.
  resetKey?: number;
  deferStaleRefresh?: boolean;
}

interface FolderItemsResult {
  items: Meta[];
  groups: Array<{ type: string; items: Meta[] }>;
}

type FolderSourceBatch = { type: string; items: Meta[] };

type AddonFolderSource = { transportUrl: string; catalogId: string; type: string; genre?: string };
type FolderSource = AddonFolderSource | NuvioRemoteCollectionSource;

interface FolderSourceState {
  skip: number;
  exhausted: boolean;
  duplicateStreak: number;
  items: Meta[];
}

function initFolderSourceState(): FolderSourceState {
  return { skip: 0, exhausted: false, duplicateStreak: 0, items: [] };
}

async function loadFolderSourcePage(source: FolderSource, skip: number): Promise<FolderSourceBatch> {
  const plan = await coreInvoke<{
    kind: 'remote' | 'builtinTmdb' | 'addon';
    type: string;
    page?: number;
    transportUrl?: string;
    catalogId?: string;
    extra?: Record<string, unknown>;
  }>('folderSourcePagePlan', JSON.stringify({ source, skip }));
  if (!plan) return { type: 'movie', items: [] };
  if (plan.kind === 'remote') {
    return { type: plan.type, items: await loadNuvioCollectionSource(source as NuvioRemoteCollectionSource, plan.page) };
  }

  if (plan.kind === 'builtinTmdb') {
    const prefs = await loadPrefs();
    const { metas } = await fetchBuiltinCatalog(plan.type, plan.extra ?? {}, String(prefs.tmdbApiKey ?? ''), getLanguage());
    return { type: plan.type, items: metas as Meta[] };
  }

  const extraJson = Object.keys(plan.extra ?? {}).length ? JSON.stringify(plan.extra) : undefined;
  const url = await buildResourceUrl(plan.transportUrl!, 'catalog', plan.type, plan.catalogId!, extraJson);
  try {
    const res = await httpFetchText(url);
    if (res.statusCode === 200) {
      const data = JSON.parse(res.body) as { metas?: unknown };
      return { type: plan.type, items: Array.isArray(data?.metas) ? (data.metas as Meta[]) : [] };
    }
  } catch {
    /* skip failed source */
  }
  return { type: plan.type, items: [] as Meta[] };
}

export const HomeScreen = React.memo(
  function HomeScreen({
    state,
    onDispatch,
    onNavigateDetail,
    onPlay,
    onResume,
    onStartOver,
    onPlayManually,
    onOpenSettings,
    isActive,
    onScrolledChange,
    resetKey,
    deferStaleRefresh,
  }: Props) {
    const home = state.home;
    const [viewAllCategory, setViewAllCategory] = useState<{
      title: string;
      items: Meta[];
      groups?: Array<{ type: string; items: Meta[] }>;
    } | null>(null);
    const [folderLoading, setFolderLoading] = useState(false);
    const [folderError, setFolderError] = useState(false);
    const [folderLoadingMore, setFolderLoadingMore] = useState(false);
    const [folderPaginated, setFolderPaginated] = useState(false);
    const folderSourcesRef = useRef<FolderSource[]>([]);
    const folderSourceStatesRef = useRef<FolderSourceState[]>([]);
    const scrollRef = useRef<HTMLDivElement>(null);
    const savedScrollRef = useRef(0);
    const [heroScrolledPast, setHeroScrolledPast] = useState(false);

    const [catalogExtra, setCatalogExtra] = useState<Record<string, Meta[]>>({});
    const catalogExtraRef = useRef(catalogExtra);
    const catalogNoMoreRef = useRef<Set<string>>(new Set());
    const pendingCatalogPageIdRef = useRef<string | null>(null);
    const [loadingMoreCategoryId, setLoadingMoreCategoryId] = useState<string | null>(null);
    const refreshStartedRef = useRef(false);

    useEffect(() => {
      catalogExtraRef.current = catalogExtra;
    }, [catalogExtra]);

    useEffect(() => {
      if (!home.isLoading) return;
      setCatalogExtra({});
      catalogNoMoreRef.current.clear();
      pendingCatalogPageIdRef.current = null;
      setLoadingMoreCategoryId(null);
    }, [home.isLoading]);

    const handleLoadMoreCategory = useCallback(
      (cat: HomeCategory) => {
        if (!cat.transportUrl || !cat.catalogId) return;
        if (catalogNoMoreRef.current.has(cat.id)) return;
        if (pendingCatalogPageIdRef.current) return;
        pendingCatalogPageIdRef.current = cat.id;
        setLoadingMoreCategoryId(cat.id);
        const skip = cat.items.length + (catalogExtraRef.current[cat.id]?.length ?? 0);
        onDispatch(
          JSON.stringify({
            type: 'catalogPageRequested',
            categoryId: cat.id,
            transportUrl: cat.transportUrl,
            contentType: cat.type,
            catalogId: cat.catalogId,
            skip,
            genre: cat.addonGenre ?? null,
          }),
        );
      },
      [onDispatch],
    );

    useEffect(() => {
      const paging = home.paging;
      const pendingId = pendingCatalogPageIdRef.current;
      if (!paging || !pendingId || paging.categoryId !== pendingId || paging.isLoading) return;
      pendingCatalogPageIdRef.current = null;
      setLoadingMoreCategoryId(null);
      const items = Array.isArray(paging.items) ? paging.items : [];
      if (paging.error || items.length === 0) {
        catalogNoMoreRef.current.add(pendingId);
        return;
      }
      setCatalogExtra((prev) => ({
        ...prev,
        [pendingId]: [...(prev[pendingId] ?? []), ...items],
      }));
    }, [home.paging]);

    useEffect(() => {
      if (resetKey === undefined) return;
      setViewAllCategory(null);
      setFolderPaginated(false);
      folderSourcesRef.current = [];
      folderSourceStatesRef.current = [];
      refreshStartedRef.current = false;
    }, [resetKey]);

    useLayoutEffect(() => {
      if (!viewAllCategory && scrollRef.current) scrollRef.current.scrollTop = savedScrollRef.current;
    }, [viewAllCategory]);

    useEffect(() => {
      const el = scrollRef.current;
      if (!el) return;
      const handleScroll = () => {
        onScrolledChange?.(el.scrollTop > 40);
        const past = el.scrollTop > 100;
        setHeroScrolledPast((prev) => (prev === past ? prev : past));
      };
      handleScroll();
      el.addEventListener('scroll', handleScroll, { passive: true });
      return () => el.removeEventListener('scroll', handleScroll);
    }, [onScrolledChange]);

    const handleFolderTileClick = useCallback(
      async (folderMeta: Meta) => {
        const allCats = (home.categories ?? []) as HomeCategory[];
        const folderCat = allCats.find((c) => c.id === folderMeta.id && c.type === 'collection_folder');
        const tileSources = (folderMeta as Meta & { collectionSources?: unknown[] }).collectionSources ?? [];
        const categorySources = folderCat?.catalogSources ?? [];
        const rawSources = categorySources.length > 0 ? categorySources : tileSources;
        const installedAddons = state.addons.installed ?? [];
        const sources = (
          await Promise.all(
            (rawSources as Record<string, unknown>[]).map(async (source) => {
              if (typeof source.transportUrl === 'string') return source as unknown as FolderSource;
              if (source.provider === 'trakt' || source.provider === 'tmdb') return source as unknown as FolderSource;
              const transportUrl = await coreResolveTransportUrl(JSON.stringify(source), JSON.stringify(installedAddons));
              return transportUrl ? ({ ...source, transportUrl } as unknown as FolderSource) : null;
            }),
          )
        ).filter((source): source is FolderSource => source !== null);
        if (!sources.length) return;
        savedScrollRef.current = scrollRef.current?.scrollTop ?? 0;
        setViewAllCategory({ title: folderMeta.name, items: [] });
        setFolderError(false);
        setFolderLoading(true);
        folderSourcesRef.current = sources;
        folderSourceStatesRef.current = sources.map(() => initFolderSourceState());
        setFolderPaginated(true);
        try {
          const batches = await Promise.all(sources.map((source) => loadFolderSourcePage(source, 0)));
          folderSourceStatesRef.current = await Promise.all(
            folderSourceStatesRef.current.map(
              async (state, index) =>
                (await coreInvoke<FolderSourceState>('folderPageState', JSON.stringify({ state, batch: batches[index] }))) ?? state,
            ),
          );
          const { items, groups } = (await coreInvoke<FolderItemsResult>(
            'mergeFolderSources',
            JSON.stringify(folderSourceStatesRef.current.map((state) => state.items)),
          )) ?? { items: [], groups: [] };
          setViewAllCategory({ title: folderMeta.name, items, groups });
          setFolderError(items.length === 0);
        } finally {
          setFolderLoading(false);
        }
      },
      [home.categories, state.addons.installed],
    );

    const handleLoadMoreFolder = useCallback(async () => {
      const sources = folderSourcesRef.current ?? [];
      const states = folderSourceStatesRef.current;
      if (!sources.length || folderLoadingMore) return;
      if (states.every((s) => s.exhausted)) return;
      setFolderLoadingMore(true);
      try {
        const batches = await Promise.all(
          sources.map(async (source, i) => {
            if (!states[i].exhausted) return loadFolderSourcePage(source, states[i].skip);
            const plan = await coreInvoke<{ type: string }>('folderSourcePagePlan', JSON.stringify({ source, skip: states[i].skip }));
            return { type: plan?.type ?? 'movie', items: [] };
          }),
        );
        folderSourceStatesRef.current = await Promise.all(
          states.map(async (state, index) =>
            state.exhausted
              ? state
              : ((await coreInvoke<FolderSourceState>('folderPageState', JSON.stringify({ state, batch: batches[index] }))) ?? state),
          ),
        );
        const { items, groups } = (await coreInvoke<FolderItemsResult>(
          'mergeFolderSources',
          JSON.stringify(folderSourceStatesRef.current.map((state) => state.items)),
        )) ?? { items: [], groups: [] };
        setViewAllCategory((prev) => (prev ? { ...prev, items, groups } : prev));
      } finally {
        setFolderLoadingMore(false);
      }
    }, [folderLoadingMore]);

    useEffect(() => {
      if (deferStaleRefresh) return;
      const hasData = (home.categories?.length ?? 0) > 0 || !!home.billboard || (home.continueWatching?.length ?? 0) > 0;
      if (!hasData && !home.isLoading) {
        onDispatch(JSON.stringify({ type: 'homeLoadRequested', language: getLanguage() }));
      }
    }, [deferStaleRefresh, home.billboard, home.categories, home.continueWatching, home.isLoading, onDispatch]);

    useEffect(() => {
      if (!home.isStale || deferStaleRefresh || refreshStartedRef.current) return;
      refreshStartedRef.current = true;
      const timer = window.setTimeout(() => {
        void onDispatch(JSON.stringify({ type: 'homeLoadRequested', force: true, language: getLanguage() }));
      }, 300);
      return () => window.clearTimeout(timer);
    }, [deferStaleRefresh, home.isStale, onDispatch]);

    const continueWatching = useMemo(() => (home.continueWatching ?? []) as Meta[], [home.continueWatching]);
    const posterPrefs = useMemo(() => posterPrefsFromState(state), [state.settings?.values]);
    const prefs = useMemo(() => appPrefs(state), [state.settings?.values]);
    const [heroTrailers, setHeroTrailers] = useState<Record<string, Trailer[]>>({});
    const [fetchedHeroTrailerIds, setFetchedHeroTrailerIds] = useState<string[]>([]);
    const [heroLogos, setHeroLogos] = useState<Record<string, string>>({});
    const [fetchedHeroLogoIds, setFetchedHeroLogoIds] = useState<string[]>([]);
    const [heroDescriptions, setHeroDescriptions] = useState<Record<string, string>>({});
    const [homePlan, setHomePlan] = useState<{
      categories: HomeCategory[];
      billboard: Meta | null;
      slides: Meta[];
      trailerTargets: Meta[];
      logoTargets: Meta[];
      showHero: boolean;
      autoplayTrailer: boolean;
    }>({ categories: [], billboard: null, slides: [], trailerTargets: [], logoTargets: [], showHero: true, autoplayTrailer: false });
    useEffect(() => {
      let active = true;
      void coreInvoke<typeof homePlan>(
        'homeHeroPlan',
        JSON.stringify({
          categories: home.categories ?? [],
          billboard: home.billboard ?? null,
          prefs,
          fetchedTrailers: heroTrailers,
          fetchedIds: fetchedHeroTrailerIds,
          fetchedLogos: heroLogos,
          fetchedLogoIds: fetchedHeroLogoIds,
        }),
      ).then((plan) => {
        if (!active || !plan) return;
        setHomePlan(plan);
      });
      return () => {
        active = false;
      };
    }, [home.categories, home.billboard, prefs]);
    const resolvedHomePlan = useMemo(() => {
      const resolve = (item: Meta | null): Meta | null => {
        if (!item) return null;
        const trailers = heroTrailers[item.id];
        const logo = heroLogos[item.id];
        return trailers || logo ? { ...item, ...(trailers ? { trailers } : {}), ...(logo ? { logo } : {}) } : item;
      };
      return {
        ...homePlan,
        billboard: resolve(homePlan.billboard),
        slides: homePlan.slides.map((item) => resolve(item) ?? item),
      };
    }, [heroLogos, heroTrailers, homePlan]);
    const categories = resolvedHomePlan.categories;
    const categoryItems = useMemo(
      () => new Map(categories.map((cat) => [cat.id, catalogExtra[cat.id]?.length ? [...cat.items, ...catalogExtra[cat.id]] : cat.items])),
      [categories, catalogExtra],
    );
    const nearEndCallbacks = useMemo(() => {
      const map = new Map<string, () => void>();
      for (const cat of categories) map.set(cat.id, () => handleLoadMoreCategory(cat));
      return map;
    }, [categories, handleLoadMoreCategory]);
    const billboard = resolvedHomePlan.billboard;
    const heroSlides = resolvedHomePlan.slides;
    const autoplayTrailerEnabled = resolvedHomePlan.autoplayTrailer;

    useEffect(() => {
      if (!autoplayTrailerEnabled) return;
      prewarmYoutubeTrailerConfig().catch((err) => console.error('prewarmYoutubeTrailerConfig failed', err));
    }, [autoplayTrailerEnabled]);

    useEffect(() => {
      const apiKey = prefString(prefs, 'tmdbApiKey');
      const targets = resolvedHomePlan.trailerTargets;
      if (!targets.length) return;
      let cancelled = false;
      const language = getLanguage();
      Promise.all(
        targets.map(async (item) => {
          const trailers = (await fetchTmdbTrailers({ contentType: item.type, id: item.id, language, apiKey })) as Trailer[];
          return [item.id, trailers] as const;
        }),
      )
        .then((results) => {
          if (cancelled) return;
          setFetchedHeroTrailerIds((current) => Array.from(new Set([...current, ...targets.map((item) => item.id)])));
          const found = results.filter(([, trailers]) => trailers.length);
          if (!found.length) return;
          setHeroTrailers((prev) => ({ ...prev, ...Object.fromEntries(found) }));
        })
        .catch((err) => console.error('hero trailer fetch failed', err));
      return () => {
        cancelled = true;
      };
    }, [resolvedHomePlan.trailerTargets, prefs]);

    useEffect(() => {
      const apiKey = prefString(prefs, 'tmdbApiKey');
      const fanartApiKey = prefString(prefs, 'fanartApiKey');
      const targets = resolvedHomePlan.logoTargets;
      if (!targets.length) return;
      let cancelled = false;
      const language = getLanguage();
      Promise.all(
        targets.map(async (item) => {
          const logo = await fetchContentLogo(item.id, item.type, language, apiKey, fanartApiKey).catch(() => undefined);
          return [item.id, logo ?? null] as const;
        }),
      )
        .then((results) => {
          if (cancelled) return;
          setFetchedHeroLogoIds((current) => Array.from(new Set([...current, ...targets.map((item) => item.id)])));
          const found = results.filter((entry): entry is [string, string] => !!entry[1]);
          if (!found.length) return;
          setHeroLogos((prev) => ({ ...prev, ...Object.fromEntries(found) }));
        })
        .catch((err) => console.error('hero logo fetch failed', err));
      return () => {
        cancelled = true;
      };
    }, [resolvedHomePlan.logoTargets, prefs]);

    const heroDescriptionRequestedRef = useRef<Set<string>>(new Set());
    const heroItemsSignature = `${billboard?.id ?? ''}|${heroSlides.map((s) => s.id).join(',')}`;
    const heroDescriptionTargets = useMemo(() => {
      const seen = new Set<string>();
      const targets: Meta[] = [];
      for (const item of [billboard, ...heroSlides]) {
        if (!item || seen.has(item.id) || item.description) continue;
        seen.add(item.id);
        if (!heroDescriptionRequestedRef.current.has(item.id)) targets.push(item);
      }
      return targets;
      // eslint-disable-next-line react-hooks/exhaustive-deps
    }, [heroItemsSignature]);

    useEffect(() => {
      const targets = heroDescriptionTargets;
      if (!targets.length) return;
      for (const item of targets) heroDescriptionRequestedRef.current.add(item.id);
      let cancelled = false;
      Promise.all(
        targets.map(async (item) => {
          const description = await fetchHeroDescription(item).catch(() => null);
          return [item.id, description] as const;
        }),
      )
        .then((results) => {
          if (cancelled) return;
          const found = results.filter((entry): entry is [string, string] => !!entry[1]);
          if (!found.length) return;
          setHeroDescriptions((prev) => ({ ...prev, ...Object.fromEntries(found) }));
        })
        .catch((err) => console.error('hero description fetch failed', err));
      return () => {
        cancelled = true;
      };
    }, [heroDescriptionTargets]);

    const billboardWithTrailer =
      billboard && !billboard.description && heroDescriptions[billboard.id]
        ? { ...billboard, description: heroDescriptions[billboard.id] }
        : billboard;
    const heroSlidesWithTrailers = useMemo(
      () =>
        heroSlides.map((item) =>
          !item.description && heroDescriptions[item.id] ? { ...item, description: heroDescriptions[item.id] } : item,
        ),
      [heroSlides, heroDescriptions],
    );

    const heroPendingLogoIds = useMemo(
      () => new Set(resolvedHomePlan.logoTargets.map((item) => item.id).filter((id) => !fetchedHeroLogoIds.includes(id))),
      [resolvedHomePlan.logoTargets, fetchedHeroLogoIds],
    );

    const addonIconByName = useMemo(() => {
      const map = new Map<string, string>();
      for (const addon of state.addons.installed ?? []) {
        if (addon.name && addon.logo) map.set(addon.name, addon.logo);
      }
      return map;
    }, [state.addons.installed]);
    const showHero = resolvedHomePlan.showHero;
    const showContinueWatching = prefBool(prefs, 'continueWatchingEnabled', true);
    const gifAutoplayEnabled = prefBool(prefs, 'gifAutoplayEnabled', false);
    const topTenFeedKeys = useMemo(() => {
      const raw = prefs.topTenFeedToggles;
      return new Set<string>(Array.isArray(raw) ? (raw as string[]) : []);
    }, [prefs.topTenFeedToggles]);

    const handleViewAll = useCallback((title: string, items: Meta[]) => {
      savedScrollRef.current = scrollRef.current?.scrollTop ?? 0;
      setViewAllCategory({ title, items });
      setFolderPaginated(false);
      folderSourcesRef.current = [];
      folderSourceStatesRef.current = [];
    }, []);

    const handleAddToWatchlist = useCallback(
      (meta: Meta) => onDispatch(JSON.stringify({ type: 'libraryAddRequested', meta })),
      [onDispatch],
    );

    const cwSettingsValues = state.settings?.values as Record<string, unknown> | undefined;
    const cwLayout = String(cwSettingsValues?.resolvedContinueWatchingLayout ?? cwSettingsValues?.continueWatchingLayout ?? 'horizontal');
    const cwArtwork = String(cwSettingsValues?.continueWatchingArtwork ?? 'episode');
    const cwRemainingFormat = String(cwSettingsValues?.continueWatchingRemainingFormat ?? 'time');
    const cwProgressDirection = String(cwSettingsValues?.continueWatchingProgressDirection ?? 'remaining');
    const keepScheduled = prefBool(prefs, 'continueWatchingKeepScheduled', false);
    const showThisWeek = prefBool(prefs, 'continueWatchingShowThisWeek', true);
    const [{ thisWeek, continueWatching: cwItems }, setCwPartition] = useState<{ thisWeek: Meta[]; continueWatching: Meta[] }>({
      thisWeek: [],
      continueWatching,
    });
    useEffect(() => {
      let cancelled = false;
      partitionThisWeek(continueWatching, keepScheduled || !showThisWeek).then((result) => {
        if (!cancelled) setCwPartition(result);
      });
      return () => {
        cancelled = true;
      };
    }, [continueWatching, keepScheduled, showThisWeek]);

    if (home.isLoading && !billboard && categories.length === 0 && (home.continueWatching?.length ?? 0) === 0) {
      return <LoadingSkeleton />;
    }

    if (home.error && !billboard && categories.length === 0 && continueWatching.length === 0) {
      return (
        <HomeStateMessage
          title={t('common.error')}
          body={home.error}
          primaryLabel={t('common.retry')}
          onPrimary={() => onDispatch(JSON.stringify({ type: 'homeLoadRequested', force: true, language: getLanguage() }))}
        />
      );
    }

    if (!home.isLoading && !billboard && categories.length === 0 && continueWatching.length === 0) {
      return <EmptyHome onOpenSettings={onOpenSettings} />;
    }

    if (viewAllCategory) {
      return (
        <CategoryGridScreen
          title={viewAllCategory.title}
          items={viewAllCategory.items}
          groups={viewAllCategory.groups}
          isLoading={folderLoading}
          loadError={!folderLoading && folderError}
          onLoadMore={folderPaginated ? handleLoadMoreFolder : undefined}
          isLoadingMore={folderLoadingMore}
          posterPrefs={posterPrefs}
          onNavigateDetail={onNavigateDetail}
          onBack={() => {
            setViewAllCategory(null);
            setFolderPaginated(false);
            folderSourcesRef.current = [];
            folderSourceStatesRef.current = [];
          }}
          onDispatch={onDispatch}
        />
      );
    }

    return (
      <div ref={scrollRef} className="home-screen" style={styles.screen}>
        {billboardWithTrailer && showHero && (
          <HeroSection
            meta={billboardWithTrailer}
            slides={heroSlidesWithTrailers}
            preferSeasonPosters={prefBool(prefs, 'homeSeasonPostersOnHero', true)}
            onPlay={onPlay}
            onDetails={onNavigateDetail}
            onAddToWatchlist={handleAddToWatchlist}
            isActive={isActive && !heroScrolledPast}
            autoplayTrailer={autoplayTrailerEnabled}
            autoplayTrailerDelaySecs={Number(prefString(prefs, 'homeHeroAutoplayTrailerDelaySecs', '2'))}
            preferredSubtitleLanguage={prefString(prefs, 'preferredSubtitleLanguage', 'none')}
            secondarySubtitleLanguage={prefString(prefs, 'secondarySubtitleLanguage', 'none')}
            pendingLogoIds={heroPendingLogoIds}
          />
        )}

        <div
          className="home-shelves"
          style={{ ...styles.shelves, marginTop: billboardWithTrailer && showHero ? styles.shelves.marginTop : 0 }}
        >
          {showContinueWatching && cwItems.length > 0 && (
            <ContinueWatchingRow
              items={cwItems}
              cwLayout={cwLayout}
              artworkPreference={cwArtwork}
              remainingFormat={cwRemainingFormat}
              progressDirection={cwProgressDirection}
              onItemClick={onResume}
              onNavigateDetail={onNavigateDetail}
              onStartOver={onStartOver}
              onPlayManually={onPlayManually}
              onDispatch={onDispatch}
            />
          )}
          {showContinueWatching && showThisWeek && thisWeek.length > 0 && (
            <ThisWeekRow items={thisWeek} artworkPreference={cwArtwork} onItemClick={onNavigateDetail} />
          )}
          {categories.map((cat) => (
            <LazyRow key={cat.id}>
              {cat.type === 'collection' ? (
                <CollectionShelfRow
                  title={cat.name}
                  folders={cat.items}
                  onFolderClick={handleFolderTileClick}
                  addonIcon={cat.addonName ? addonIconByName.get(cat.addonName) : undefined}
                  gifAutoplayEnabled={gifAutoplayEnabled}
                />
              ) : (
                <ShelfRow
                  title={formatCatalogTitle(cat.name, cat.type)}
                  items={categoryItems.get(cat.id) ?? cat.items}
                  onItemClick={onNavigateDetail}
                  onViewAll={handleViewAll}
                  isLoading={cat.items.length === 0 && !!home.isLoading}
                  posterPrefs={posterPrefs}
                  topTenEnabled={topTenFeedKeys.has(cat.id)}
                  addonIcon={cat.addonName ? addonIconByName.get(cat.addonName) : undefined}
                  onNearEnd={nearEndCallbacks.get(cat.id)}
                  isLoadingMore={loadingMoreCategoryId === cat.id}
                  onDispatch={onDispatch}
                />
              )}
            </LazyRow>
          ))}
        </div>
      </div>
    );
  },
  (prev, next) =>
    prev.state.home === next.state.home &&
    prev.state.settings === next.state.settings &&
    prev.state.addons === next.state.addons &&
    prev.onDispatch === next.onDispatch &&
    prev.onNavigateDetail === next.onNavigateDetail &&
    prev.onPlay === next.onPlay &&
    prev.onResume === next.onResume &&
    prev.onStartOver === next.onStartOver &&
    prev.onPlayManually === next.onPlayManually &&
    prev.onOpenSettings === next.onOpenSettings &&
    prev.isActive === next.isActive &&
    prev.resetKey === next.resetKey,
);

function formatCatalogTitle(name: string, type: string): string {
  let label: string;
  if (type === 'movie') label = t('auto.movies');
  else if (type === 'series') label = t('auto.series');
  else if (type) label = type.charAt(0).toUpperCase() + type.slice(1);
  else return name;
  return `${name} - ${label}`;
}

function LoadingSkeleton() {
  const box: React.CSSProperties = { background: '#12161D', borderRadius: '0.625rem', animation: 'pulse 1.6s ease-in-out infinite' };
  return (
    <div style={{ width: '100%', height: '100%', background: '#040508', overflow: 'hidden' }}>
      <div style={{ ...box, width: '100%', height: HOME_HERO_HEIGHT, borderRadius: 0 }} />
      {[0, 1].map((row) => (
        <div key={row} style={{ padding: '1.75rem 3.625rem 0' }}>
          <div style={{ ...box, width: '11.25rem', height: '1.125rem', marginBottom: '1rem', animationDelay: `${row * 0.2}s` }} />
          <div style={{ display: 'flex', gap: '1.125rem', overflow: 'hidden' }}>
            {Array.from({ length: 8 }).map((_, i) => (
              <div
                key={i}
                style={{ ...box, width: '9.375rem', height: '14.0625rem', flexShrink: 0, animationDelay: `${(row * 8 + i) * 0.06}s` }}
              />
            ))}
          </div>
        </div>
      ))}
    </div>
  );
}

function HomeStateMessage({
  title,
  body,
  primaryLabel,
  onPrimary,
}: {
  title: string;
  body: string;
  primaryLabel?: string;
  onPrimary?: () => void;
}) {
  return (
    <div className="home-empty" style={styles.empty}>
      <p style={styles.emptyTitle}>{title}</p>
      <p style={styles.emptyText}>{body}</p>
      {primaryLabel && onPrimary && (
        <button style={styles.emptyButton} onClick={onPrimary}>
          {primaryLabel}
        </button>
      )}
    </div>
  );
}

function EmptyHome({ onOpenSettings }: { onOpenSettings?: () => void }) {
  return (
    <HomeStateMessage
      title={t('home.no_catalog_providers')}
      body={t('home.add_catalog_addon')}
      primaryLabel={onOpenSettings ? t('auto.add_ons') : undefined}
      onPrimary={onOpenSettings}
    />
  );
}

const HOME_HERO_OVERLAP = '7.5rem';
const HOME_HERO_HEIGHT = `clamp(45.5rem, calc(66vh + ${HOME_HERO_OVERLAP}), 61.5rem)`;

const styles: Record<string, React.CSSProperties> = {
  screen: {
    height: '100%',
    overflowY: 'auto',
    overflowX: 'hidden',
    background: '#040508',
    scrollbarWidth: 'none',
    ['--hero-height' as string]: HOME_HERO_HEIGHT,
  },
  shelves: {
    position: 'relative',
    marginTop: `-${HOME_HERO_OVERLAP}`,
    paddingTop: '0.5rem',
    paddingBottom: '5rem',
  },
  empty: {
    height: '100%',
    display: 'flex',
    flexDirection: 'column',
    alignItems: 'center',
    justifyContent: 'center',
    gap: '1rem',
    background: '#040508',
  },
  emptyTitle: {
    color: '#FFFFFF',
    fontSize: '2rem',
    fontWeight: 800,
    margin: 0,
  },
  emptyText: {
    color: 'rgba(255,255,255,0.5)',
    fontSize: '1rem',
    textAlign: 'center',
    lineHeight: 1.6,
    margin: 0,
  },
  emptyButton: {
    height: '2.625rem',
    padding: '0 1.125rem',
    borderRadius: '62.4375rem',
    border: '1px solid rgba(255,255,255,0.14)',
    background: '#FFFFFF',
    color: '#000000',
    fontSize: '0.8125rem',
    fontWeight: 850,
    cursor: 'pointer',
  },
};
