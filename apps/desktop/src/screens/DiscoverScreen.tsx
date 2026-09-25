import React, { memo, useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { posterPrefsFromState } from '../core/posterPrefs';
import type { AppState, Meta } from '../core/types';
import { getLanguage, t } from '../i18n';
import { FilterDropdown } from '../components/FilterDropdown';
import { GlobalSearchBar } from '../components/GlobalSearchBar';
import { VirtualizedPosterGrid } from '../components/VirtualizedPosterGrid';
import { coreInvoke } from '../core/engine';
import { SearchScreen } from './SearchScreen';

interface Props {
  state: Pick<AppState, 'addons' | 'discover' | 'settings' | 'home' | 'search'>;
  onDispatch: (actionJson: string) => void;
  onNavigateDetail: (meta: Meta) => void;
  onBack: () => void;
  initialGenre?: string | null;
  query?: string;
  onQueryChange?: (query: string) => void;
}

const discoverResultsCache = new Map<string, Meta[]>();
const DISCOVER_CONTENT_GUTTER = 42;

interface DiscoverCatalog {
  key: string;
  label: string;
  type: string;
  transportUrl?: string;
  id?: string;
  extras?: Array<{
    name: string;
    options: string[];
    isRequired?: boolean;
  }>;
}

function DiscoverScreenInner({ state, onDispatch, onNavigateDetail, initialGenre, query = '', onQueryChange }: Props) {
  const discover = state.discover;
  const [contentType, setContentType] = useState<string>('movie');
  const [selectedCatalogKey, setSelectedCatalogKey] = useState<string | null>(null);
  const [extraValue, setExtraValue] = useState<string | null>(initialGenre ?? null);
  const [selectionPlan, setSelectionPlan] = useState<{
    catalogs: DiscoverCatalog[];
    selectedCatalogKey: string | null;
    selectedCatalog: DiscoverCatalog | null;
    selectedExtra: NonNullable<DiscoverCatalog['extras']>[number] | null;
    extraValue: string | null;
    key: string;
  }>({ catalogs: [], selectedCatalogKey: null, selectedCatalog: null, selectedExtra: null, extraValue: null, key: '||' });
  const catalogs = selectionPlan.catalogs;
  const selectedCatalog = selectionPlan.selectedCatalog;
  const selectedExtra = selectionPlan.selectedExtra;
  const key = selectionPlan.key;
  const cachedResults = discoverResultsCache.get(key) ?? null;
  const lastDispatchedKeyRef = useRef<string | null>(null);
  const posterPrefs = useMemo(() => posterPrefsFromState(state, 0.88), [state.settings?.values]);

  useEffect(() => {
    setSelectedCatalogKey(null);
    setExtraValue(initialGenre ?? null);
    onDispatch(JSON.stringify({ type: 'discoverCatalogFiltersRequested', contentType }));
  }, [contentType]);

  useEffect(() => {
    let active = true;
    void coreInvoke<typeof selectionPlan>(
      'discoverSelectionPlan',
      JSON.stringify({
        catalogs: discover.catalogs ?? [],
        contentType,
        selectedCatalogKey,
        extraValue,
      }),
    ).then((plan) => {
      if (!active || !plan) return;
      setSelectionPlan(plan);
      if (plan.selectedCatalogKey !== selectedCatalogKey) setSelectedCatalogKey(plan.selectedCatalogKey);
      if (plan.extraValue !== extraValue) setExtraValue(plan.extraValue);
    });
    return () => {
      active = false;
    };
  }, [discover.catalogs, contentType, selectedCatalogKey, extraValue]);

  useEffect(() => {
    if (!selectedCatalog || discoverResultsCache.has(key)) return;
    const timer = window.setTimeout(() => {
      lastDispatchedKeyRef.current = key;
      onDispatch(
        JSON.stringify({
          type: 'discoverRequested',
          contentType,
          filters: {
            catalogKey: selectedCatalog.key,
            transportUrl: selectedCatalog.transportUrl,
            extra: selectedExtra && extraValue ? { [selectedExtra.name]: extraValue } : {},
          },
          language: getLanguage(),
        }),
      );
    }, 200);
    return () => window.clearTimeout(timer);
  }, [contentType, selectedCatalog, selectedExtra, extraValue, key]);

  const results = useMemo(() => (discover.results ?? []) as Meta[], [discover.results]);
  const resultsMatchCurrentKey = lastDispatchedKeyRef.current === key;
  if (results.length > 0 && resultsMatchCurrentKey) {
    discoverResultsCache.set(key, results);
  }

  const isFinal = !discover.isLoading && resultsMatchCurrentKey && results.length > 0;
  const baseResults = isFinal ? results : (cachedResults ?? []);
  const isWaitingForResults = !!selectedCatalog && !cachedResults && !resultsMatchCurrentKey;
  const isLoading = discover.isLoading || discover.catalogsLoading || isWaitingForResults;

  const [pagingExtra, setPagingExtra] = useState<Record<string, Meta[]>>({});
  const pagingNoMoreRef = useRef<Set<string>>(new Set());
  const pendingPagingKeyRef = useRef<string | null>(null);

  useEffect(() => {
    pendingPagingKeyRef.current = null;
  }, [key]);

  const [displayResults, setDisplayResults] = useState<Meta[]>([]);
  useEffect(() => {
    let active = true;
    void coreInvoke<{ items: Meta[] }>(
      'mergeDiscoverPages',
      JSON.stringify({
        baseItems: baseResults,
        existingItems: pagingExtra[key] ?? [],
        incomingItems: [],
      }),
    )
      .then((plan) => {
        if (active) setDisplayResults(plan?.items ?? []);
      })
      .catch(() => {
        if (active) setDisplayResults(baseResults);
      });
    return () => {
      active = false;
    };
  }, [baseResults, pagingExtra, key]);

  const handleLoadMore = useCallback(() => {
    if (!selectedCatalog?.transportUrl || !selectedCatalog.id) return;
    if (isLoading || pagingNoMoreRef.current.has(key) || pendingPagingKeyRef.current) return;
    pendingPagingKeyRef.current = key;
    onDispatch(
      JSON.stringify({
        type: 'discoverPageRequested',
        transportUrl: selectedCatalog.transportUrl,
        contentType: selectedCatalog.type,
        catalogId: selectedCatalog.id,
        skip: displayResults.length,
        genre: extraValue,
      }),
    );
  }, [selectedCatalog, key, extraValue, displayResults.length, isLoading, onDispatch]);

  useEffect(() => {
    const paging = discover.paging;
    const pendingKey = pendingPagingKeyRef.current;
    if (!paging || !pendingKey || paging.isLoading) return;
    pendingPagingKeyRef.current = null;
    const items = Array.isArray(paging.items) ? paging.items : [];
    if (paging.error) {
      pagingNoMoreRef.current.add(pendingKey);
      return;
    }
    void (async () => {
      const existing = pagingExtra[pendingKey] ?? [];
      const plan = await coreInvoke<{ appendedItems: Meta[]; exhausted: boolean }>(
        'mergeDiscoverPages',
        JSON.stringify({
          baseItems: baseResults,
          existingItems: existing,
          incomingItems: items,
        }),
      );
      if (!plan || plan.exhausted) pagingNoMoreRef.current.add(pendingKey);
      if (!plan?.appendedItems.length) return;
      setPagingExtra((prev) => ({
        ...prev,
        [pendingKey]: [...(prev[pendingKey] ?? []), ...plan.appendedItems],
      }));
    })().catch(() => pagingNoMoreRef.current.add(pendingKey));
  }, [discover.paging, baseResults]);

  const [contentTypes, setContentTypes] = useState<string[]>(['movie', 'series']);
  useEffect(() => {
    void coreInvoke<string[]>('discoverContentTypes', JSON.stringify(state.addons?.installed ?? [])).then((types) => {
      if (types) setContentTypes(types);
    });
  }, [state.addons?.installed]);
  const typeOptions = useMemo(() => {
    return contentTypes.map((ty) => ({
      value: ty,
      label: ty === 'movie' ? t('auto.movies') : ty === 'series' ? t('auto.series') : ty.charAt(0).toUpperCase() + ty.slice(1),
    }));
  }, [contentTypes]);

  const handlePosterClick = useCallback((meta: Meta) => onNavigateDetail(meta), [onNavigateDetail]);
  const isSearching = query.trim().length > 0;

  return (
    <div className="discover-screen" style={S.screen}>
      <div style={S.left}>
        <div className="discover-searchbar" style={S.searchbar}>
          <GlobalSearchBar
            query={query}
            onSearch={(nextQuery) => onQueryChange?.(nextQuery)}
            onBack={() => onQueryChange?.('')}
            state={state}
            onDispatch={onDispatch}
            onNavigateDetail={onNavigateDetail}
            alwaysOpen
            wide
          />
        </div>

        {isSearching ? (
          <div style={S.searchResults}>
            <SearchScreen
              state={state}
              onDispatch={onDispatch}
              onNavigateDetail={onNavigateDetail}
              query={query}
              onQueryChange={(nextQuery) => onQueryChange?.(nextQuery)}
              onBack={() => onQueryChange?.('')}
              embedded
            />
          </div>
        ) : (
          <>
            <div className="discover-filterbar" style={S.filterBar}>
              <FilterDropdown
                value={typeOptions.find((o) => o.value === contentType)?.label ?? contentType}
                options={typeOptions}
                onSelect={setContentType}
              />
              <FilterDropdown
                value={selectedCatalog?.label ?? t('discover.catalog')}
                options={catalogs.map((catalog) => ({ value: catalog.key, label: catalog.label }))}
                onSelect={(v) => {
                  setSelectedCatalogKey(v);
                  setExtraValue(null);
                }}
              />
              {selectedExtra && (
                <FilterDropdown
                  value={extraValue ?? selectedExtra.name}
                  options={[
                    { value: '__all__', label: t('discover.all_filter_values', selectedExtra.name) },
                    ...selectedExtra.options.map((option) => ({ value: option, label: option })),
                  ]}
                  onSelect={(v) => setExtraValue(v === '__all__' ? null : v)}
                />
              )}
              {isLoading && <div style={S.loadingDot} />}
            </div>

            {isLoading && displayResults.length === 0 ? (
              <div className="discover-loading-grid" style={S.loadingGrid}>
                {Array.from({ length: 24 }).map((_, i) => (
                  <div
                    key={i}
                    style={{
                      borderRadius: '0.625rem',
                      background: '#222222',
                      aspectRatio: '2/3',
                      animation: 'pulse 1.6s ease-in-out infinite',
                      animationDelay: `${(i % 8) * 0.07}s`,
                    }}
                  />
                ))}
              </div>
            ) : displayResults.length === 0 ? (
              <div style={S.empty}>
                <p style={S.emptyTitle}>{t('discover.no_content')}</p>
                <p style={S.emptyHint}>{t('discover.install_addons_hint')}</p>
              </div>
            ) : (
              <VirtualizedPosterGrid
                resetKey={key}
                items={displayResults}
                selectedId={null}
                posterPrefs={posterPrefs}
                onHover={() => true}
                onClick={handlePosterClick}
                onScrollActivity={() => {}}
                onNearEnd={handleLoadMore}
                paddingX={DISCOVER_CONTENT_GUTTER}
              />
            )}
          </>
        )}
      </div>
    </div>
  );
}

const S: Record<string, React.CSSProperties> = {
  screen: {
    display: 'flex',
    width: '100%',
    height: '100%',
    marginLeft: 0,
    marginTop: 0,
    background: '#09091280',
    overflow: 'hidden',
  },
  left: { flex: 1, minWidth: 0, display: 'flex', flexDirection: 'column', overflow: 'hidden' },
  searchbar: {
    display: 'flex',
    justifyContent: 'flex-start',
    position: 'relative',
    zIndex: 40,
    background: 'var(--fluxa-background)',
    padding: '0.875rem 2.625rem 0.75rem',
    flexShrink: 0,
    borderBottom: '1px solid rgba(255,255,255,0.05)',
  },
  searchResults: { flex: 1, minHeight: 0, overflow: 'hidden' },
  filterBar: {
    display: 'flex',
    alignItems: 'center',
    gap: '0.625rem',
    padding: '0.625rem 2.625rem',
    flexShrink: 0,
    borderBottom: '1px solid rgba(255,255,255,0.05)',
  },
  loadingDot: {
    marginLeft: 'auto',
    width: '0.375rem',
    height: '0.375rem',
    borderRadius: '50%',
    background: 'rgba(255,255,255,0.25)',
    animation: 'pulse 1.2s ease-in-out infinite',
    flexShrink: 0,
  },
  loadingGrid: {
    flex: 1,
    overflowY: 'auto',
    display: 'grid',
    gridTemplateColumns: 'repeat(auto-fill, minmax(9.375rem, 1fr))',
    gap: '1.75rem 1.125rem',
    padding: '1.25rem 2.625rem 3.75rem',
    alignContent: 'start',
    scrollbarWidth: 'thin',
    scrollbarColor: 'rgba(255,255,255,0.1) transparent',
    contain: 'layout paint style',
  },
  empty: { flex: 1, display: 'flex', flexDirection: 'column', alignItems: 'center', justifyContent: 'center', gap: '0.625rem' },
  emptyTitle: { color: '#FFFFFF', fontSize: '1.25rem', fontWeight: 700, margin: 0 },
  emptyHint: { color: 'rgba(255,255,255,0.4)', fontSize: '0.875rem', margin: 0, textAlign: 'center' },
};

export const DiscoverScreen = memo(
  DiscoverScreenInner,
  (prev, next) =>
    prev.state.discover === next.state.discover &&
    prev.state.home === next.state.home &&
    prev.state.search === next.state.search &&
    prev.state.settings === next.state.settings &&
    prev.state.addons === next.state.addons &&
    prev.onDispatch === next.onDispatch &&
    prev.onNavigateDetail === next.onNavigateDetail &&
    prev.initialGenre === next.initialGenre &&
    prev.query === next.query &&
    prev.onQueryChange === next.onQueryChange,
);
