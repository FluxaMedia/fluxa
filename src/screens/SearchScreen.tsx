import React, { useState, useEffect, useMemo, useRef } from 'react';
import { ArrowLeft } from 'lucide-react';
import { appPrefs, prefBool } from '../core/appPrefs';
import { posterPrefsFromState } from '../core/posterPrefs';
import { addRecentSearch, clearRecentSearches, loadRecentSearches, removeRecentSearch, type RecentSearch } from '../core/searchHistory';
import type { AppState, HomeCategory, Meta } from '../core/types';
import { getLanguage, t } from '../i18n';
import { coreInvoke } from '../core/engine';
import { addSearchPartialHandler, type PartialSearchSource } from '../core/catalogEffects';
import { styles } from './searchStyles';
import { searchCacheKey, searchCacheGet, searchCacheSet, searchCacheDelete } from '../core/searchResultsCache';
import { LoadingShelves, SearchCategoryRow, RecentSearchChip, formatCatalogTitle, TypeChip, GenreCard } from './SearchScreenParts';

interface Props {
  state: Pick<AppState, 'search' | 'settings'>;
  onDispatch: (actionJson: string) => void;
  onNavigateDetail: (meta: Meta) => void;
  query: string;
  onQueryChange: (query: string) => void;
  onBack: () => void;
}

const TYPE_FILTERS = [
  { labelKey: 'auto.all', value: '' },
  { labelKey: 'auto.movies', value: 'movie' },
  { labelKey: 'auto.series', value: 'series' },
];

function groupCategoriesByAddon(categories: HomeCategory[]): HomeCategory[] {
  const groups = new Map<string, HomeCategory[]>();
  const order: string[] = [];
  for (const category of categories) {
    if (!groups.has(category.name)) {
      groups.set(category.name, []);
      order.push(category.name);
    }
    groups.get(category.name)!.push(category);
  }
  return order.flatMap((name) => (
    groups.get(name)!.sort((a, b) => (a.type === 'series' ? 1 : 0) - (b.type === 'series' ? 1 : 0))
  ));
}

const GENRE_CHIPS = [
  'genre.action', 'genre.thriller', 'genre.scifi', 'genre.comedy', 'genre.drama',
  'genre.horror', 'genre.animation', 'genre.documentary', 'genre.romance', 'genre.crime',
];

export const SearchScreen = React.memo(function SearchScreen({ state, onDispatch, onNavigateDetail, query, onQueryChange, onBack }: Props) {
  const [typeFilter, setTypeFilter] = useState('');
  const [recentSearches, setRecentSearches] = useState<RecentSearch[]>([]);
  const search = state.search;
  const posterPrefs = posterPrefsFromState(state, 0.85);
  const trimmedQuery = query.trim();
  const lastRecentQueryRef = useRef('');
  const [screenPlan, setScreenPlan] = useState<{
    query: string;
    queryEligible: boolean;
    shouldDispatch: boolean;
    shouldCache: boolean;
    categories: HomeCategory[];
    resultCount: number;
    categoryCount: number;
    isLoading: boolean;
  }>({ query: '', queryEligible: false, shouldDispatch: false, shouldCache: false, categories: [], resultCount: 0, categoryCount: 0, isLoading: false });
  const trimmedQueryRef = useRef(trimmedQuery);
  trimmedQueryRef.current = trimmedQuery;
  const [partialCategories, setPartialCategories] = useState<HomeCategory[]>([]);

  useEffect(() => addSearchPartialHandler((q, source: PartialSearchSource, items) => {
    if (q !== trimmedQueryRef.current) return;
    const meta = items as Meta[];
    setPartialCategories((current) => {
      const idx = current.findIndex((c) => c.id === source.id);
      if (idx === -1) {
        return [...current, { id: source.id, name: source.name ?? '', type: source.type ?? '', items: meta }];
      }
      const existing = current[idx];
      const seen = new Set(existing.items.map((m) => m.id));
      const added = meta.filter((m) => !seen.has(m.id));
      if (!added.length) return current;
      const updated = [...current];
      updated[idx] = { ...existing, items: [...existing.items, ...added] };
      return updated;
    });
  }), []);

  useEffect(() => {
    setPartialCategories([]);
  }, [trimmedQuery]);

  useEffect(() => {
    loadRecentSearches().then(setRecentSearches);
  }, []);

  const language = getLanguage();
  const cacheKey = searchCacheKey(trimmedQuery, language, typeFilter);
  const cachedCategories = searchCacheGet(cacheKey);
  useEffect(() => {
    let active = true;
    void coreInvoke<typeof screenPlan>('searchScreenPlan', JSON.stringify({
      query,
      searchQuery: search.query,
      searchCategories: search.categories ?? [],
      cachedCategories: cachedCategories ?? [],
      hasCache: cachedCategories != null,
      searchLoading: search.isLoading,
      typeFilter,
    })).then((plan) => {
      if (!active || !plan) return;
      if (plan.shouldCache) searchCacheSet(searchCacheKey(plan.query, language, typeFilter), search.categories as HomeCategory[]);
      setScreenPlan(plan);
    });
    return () => { active = false; };
  }, [query, search.query, search.categories, search.isLoading, cachedCategories, typeFilter, language]);

  useEffect(() => {
    if (!screenPlan.queryEligible || screenPlan.query !== trimmedQuery) return;
    if (lastRecentQueryRef.current !== screenPlan.query) {
      lastRecentQueryRef.current = screenPlan.query;
      void addRecentSearch(screenPlan.query, recentSearches).then(setRecentSearches);
    }
    if (screenPlan.shouldDispatch) onDispatch(JSON.stringify({ type: 'searchRequested', query: screenPlan.query, language: getLanguage() }));
  }, [screenPlan.query, screenPlan.queryEligible, screenPlan.shouldDispatch, trimmedQuery, onDispatch]);

  const categories = useMemo(() => groupCategoriesByAddon(screenPlan.categories), [screenPlan.categories]);
  const resultCount = screenPlan.resultCount;
  const isLoading = screenPlan.isLoading;
  const orderedPartialCategories = useMemo(() => groupCategoriesByAddon(partialCategories), [partialCategories]);

  const handleGenreClick = (genreKey: string) => {
    onQueryChange(t(genreKey));
  };

  const handleRecentClick = (recent: RecentSearch) => {
    const openDetail = prefBool(appPrefs(state), 'searchSuggestionsOpenDetail', false);
    if (openDetail && recent.meta) {
      onNavigateDetail(recent.meta);
      return;
    }
    onQueryChange(recent.query);
  };

  const handleRemoveRecent = (value: string) => {
    void removeRecentSearch(value, recentSearches).then(setRecentSearches);
  };

  const handleClearRecent = () => {
    void clearRecentSearches().then(setRecentSearches);
  };

  return (
    <div style={styles.screen}>
      <div style={styles.content}>
        <button style={styles.backBtn} onClick={onBack}>
          <ArrowLeft size={18} strokeWidth={2.2} />
          {t('auto.back')}
        </button>

        <div style={styles.header}>
          <p style={styles.eyebrow}>{t('auto.search_results')}</p>
          <h1 style={styles.title}>{query.trim() ? query.trim() : t('auto.search')}</h1>
          {query.trim().length >= 2 && !isLoading && (
            <p style={styles.subtitle}>{t('search.results_across_catalogs', resultCount, screenPlan.categoryCount)}</p>
          )}
        </div>

        <div style={styles.typeRow}>
          {TYPE_FILTERS.map((f) => (
            <TypeChip
              key={f.value}
              label={t(f.labelKey)}
              selected={typeFilter === f.value}
              onClick={() => setTypeFilter(f.value)}
            />
          ))}
        </div>

        {!query && recentSearches.length > 0 && (
          <>
            <div style={styles.sectionHeaderRow}>
              <p style={styles.sectionLabel}>{t('search.recent_searches')}</p>
              <button style={styles.clearRecentBtn} onClick={handleClearRecent}>{t('common.clear')}</button>
            </div>
            <div style={styles.recentGrid}>
              {recentSearches.map((item) => (
                <RecentSearchChip
                  key={item.query}
                  value={item.query}
                  onClick={() => handleRecentClick(item)}
                  onRemove={() => handleRemoveRecent(item.query)}
                />
              ))}
            </div>
          </>
        )}

        {!query && (
          <>
            <p style={styles.sectionLabel}>{t('search.browse_by_genre')}</p>
            <div style={styles.genreGrid}>
              {GENRE_CHIPS.map((g) => (
                <GenreCard key={g} genre={t(g)} onClick={() => handleGenreClick(g)} />
              ))}
            </div>
          </>
        )}

        {isLoading && orderedPartialCategories.length === 0 && (
          <LoadingShelves />
        )}

        {isLoading && orderedPartialCategories.length > 0 && (
          <div style={styles.categoryList}>
            {orderedPartialCategories.map((category) => (
              <SearchCategoryRow
                key={category.id}
                title={formatCatalogTitle(category.name, category.type)}
                items={category.items}
                onItemClick={onNavigateDetail}
                onDispatch={onDispatch}
                posterPrefs={posterPrefs}
              />
            ))}
          </div>
        )}

        {!isLoading && search.error && query.trim().length >= 2 && (
          <div style={styles.emptyState}>
            <p style={styles.emptyTitle}>{t('common.error')}</p>
            <p style={styles.emptyHint}>{search.error}</p>
            <button
              style={styles.retryBtn}
              onClick={() => {
                searchCacheDelete(cacheKey);
                onDispatch(JSON.stringify({ type: 'searchRequested', query: trimmedQuery, language: getLanguage() }));
              }}
            >
              {t('common.retry')}
            </button>
          </div>
        )}

        {!isLoading && !search.error && query.length >= 2 && resultCount === 0 && (
          <div style={styles.emptyState}>
            <p style={styles.emptyTitle}>{t('format.no_results_for', query)}</p>
            <p style={styles.emptyHint}>{t('search.try_shorter_or_genre')}</p>
            <div style={{ ...styles.genreGrid, marginTop: '1.5rem' }}>
              {GENRE_CHIPS.slice(0, 6).map((g) => (
                <GenreCard key={g} genre={t(g)} onClick={() => handleGenreClick(g)} />
              ))}
            </div>
          </div>
        )}

        {!isLoading && !search.error && categories.length > 0 && (
          <div style={styles.categoryList}>
            {categories.map((category) => (
              <SearchCategoryRow
                key={category.id}
                title={formatCatalogTitle(category.name, category.type)}
                items={category.items}
                onItemClick={onNavigateDetail}
                onDispatch={onDispatch}
                posterPrefs={posterPrefs}
              />
            ))}
          </div>
        )}
      </div>
    </div>
  );
}, (prev, next) =>
  prev.state.search === next.state.search &&
  prev.state.settings === next.state.settings &&
  prev.query === next.query &&
  prev.onDispatch === next.onDispatch &&
  prev.onNavigateDetail === next.onNavigateDetail &&
  prev.onQueryChange === next.onQueryChange &&
  prev.onBack === next.onBack,
);
