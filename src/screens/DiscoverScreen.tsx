import React, { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { posterPrefsFromState, type PosterPrefs } from '../core/posterPrefs';
import type { AppState, Meta } from '../core/types';
import { getLanguage, t } from '../i18n';
import { FilterDropdown } from '../components/FilterDropdown';
import { DiscoverDetailPanel } from '../components/DiscoverDetailPanel';
import { VirtualizedPosterGrid } from '../components/VirtualizedPosterGrid';

interface Props {
  state: AppState;
  onDispatch: (actionJson: string) => void;
  onNavigateDetail: (meta: Meta) => void;
  onBack: () => void;
  initialGenre?: string | null;
}

const SORT_OPTIONS = [
  { value: 'popular', labelKey: 'metadata.popular' },
  { value: 'top', labelKey: 'auto.top_rated' },
  { value: 'newest', labelKey: 'sort.release_date_desc' },
];

const FALLBACK_GENRES = [
  'Action', 'Adventure', 'Animation', 'Comedy', 'Crime',
  'Documentary', 'Drama', 'Fantasy', 'Horror', 'Mystery',
  'Romance', 'Sci-Fi', 'Thriller',
];

const SCROLL_HOVER_IDLE_MS = 180;

// Module-level: survives remounts so navigating away and back skips re-fetch
let lastDiscoverFetch: { contentType: string; sortBy: string; genre: string | null } | null = null;

export function DiscoverScreen({ state, onDispatch, onNavigateDetail, initialGenre }: Props) {
  const discover = state.discover;
  const [contentType, setContentType] = useState<string>('movie');
  const [sortBy, setSortBy] = useState<string>('popular');
  const [genre, setGenre] = useState<string | null>(initialGenre ?? null);
  const [hoveredMeta, setHoveredMeta] = useState<Meta | null>(null);
  const [selectedMeta, setSelectedMeta] = useState<Meta | null>(null);
  const isGridScrollingRef = useRef(false);
  const scrollIdleTimerRef = useRef<number | null>(null);
  const posterPrefs = useMemo(() => posterPrefsFromState(state), [state.settings?.values]);

  const panelMeta = hoveredMeta ?? selectedMeta;

  const genreOptions = (discover.filters ?? [])
    .find((f) => f.name.toLowerCase() === 'genre')
    ?.options ?? FALLBACK_GENRES;

  useEffect(() => {
    const hasData = (discover.results?.length ?? 0) > 0;
    const sameParams = lastDiscoverFetch?.contentType === contentType
      && lastDiscoverFetch?.sortBy === sortBy
      && lastDiscoverFetch?.genre === genre;
    if (hasData && sameParams) return;
    lastDiscoverFetch = { contentType, sortBy, genre };
    onDispatch(JSON.stringify({ type: 'discoverRequested', contentType, sortBy, genre, language: getLanguage() }));
  }, [contentType, sortBy, genre]);

  const results = useMemo(() => (discover.results ?? []) as Meta[], [discover.results]);

  const handleGridScroll = useCallback(() => {
    isGridScrollingRef.current = true;
    if (hoveredMeta) setHoveredMeta(null);
    if (scrollIdleTimerRef.current != null) window.clearTimeout(scrollIdleTimerRef.current);
    scrollIdleTimerRef.current = window.setTimeout(() => {
      isGridScrollingRef.current = false;
      scrollIdleTimerRef.current = null;
    }, SCROLL_HOVER_IDLE_MS);
  }, [hoveredMeta]);

  useEffect(() => {
    return () => { if (scrollIdleTimerRef.current != null) window.clearTimeout(scrollIdleTimerRef.current); };
  }, []);

  const handlePosterHover = useCallback((meta: Meta | null): boolean => {
    if (isGridScrollingRef.current) return false;
    setHoveredMeta(meta);
    return true;
  }, []);

  const handlePosterClick = useCallback((meta: Meta) => {
    setSelectedMeta(meta);
    onNavigateDetail(meta);
  }, [onNavigateDetail]);

  return (
    <div style={S.screen}>
      <div style={S.left}>
        <div style={S.filterBar}>
          <FilterDropdown
            value={contentType === 'movie' ? t('auto.movie') : t('auto.series')}
            options={[{ value: 'movie', label: t('auto.movie') }, { value: 'series', label: t('auto.series') }]}
            onSelect={(v) => { setContentType(v); setGenre(null); }}
          />
          <FilterDropdown
            value={t(SORT_OPTIONS.find((s) => s.value === sortBy)?.labelKey ?? 'metadata.popular')}
            options={SORT_OPTIONS.map((s) => ({ value: s.value, label: t(s.labelKey) }))}
            onSelect={(v) => setSortBy(v)}
          />
          <FilterDropdown
            value={genre ?? t('auto.genre')}
            options={[{ value: '__all__', label: t('search.all_genres') }, ...genreOptions.map((g) => ({ value: g, label: g }))]}
            onSelect={(v) => setGenre(v === '__all__' ? null : v)}
          />
        </div>

        {discover.isLoading ? (
          <div style={S.loadingGrid} onScroll={handleGridScroll}>
            {Array.from({ length: 24 }).map((_, i) => (
              <div key={i} style={{ borderRadius: 10, background: '#1B212B', aspectRatio: '2/3', animation: 'pulse 1.6s ease-in-out infinite', animationDelay: `${(i % 8) * 0.07}s` }} />
            ))}
          </div>
        ) : results.length === 0 ? (
          <div style={S.empty}>
            <p style={S.emptyTitle}>{t('discover.no_content')}</p>
            <p style={S.emptyHint}>{t('discover.install_addons_hint')}</p>
          </div>
        ) : (
          <VirtualizedPosterGrid
            items={results}
            selectedId={panelMeta?.id ?? null}
            posterPrefs={posterPrefs}
            onHover={handlePosterHover}
            onClick={handlePosterClick}
            onScrollActivity={handleGridScroll}
          />
        )}
      </div>

      <div style={S.right}>
        {panelMeta ? (
          <DiscoverDetailPanel meta={panelMeta} onPlay={() => onNavigateDetail(panelMeta)} onDispatch={onDispatch} />
        ) : (
          <div style={S.panelEmpty}>
            <svg width="40" height="40" viewBox="0 0 24 24" fill="rgba(255,255,255,0.12)">
              <path d="M4 8h4V4H4v4zm6 12h4v-4h-4v4zm-6 0h4v-4H4v4zm0-6h4v-4H4v4zm6 0h4v-4h-4v4zm6-10v4h4V4h-4zm-6 4h4V4h-4v4zm6 6h4v-4h-4v4zm0 6h4v-4h-4v4z" />
            </svg>
            <p style={S.panelEmptyText}>{t('discover.hover_title_hint')}</p>
          </div>
        )}
      </div>
    </div>
  );
}

const S: Record<string, React.CSSProperties> = {
  screen: { display: 'flex', width: 'calc(100% - 104px)', height: 'calc(100% - 52px)', marginLeft: 104, marginTop: 52, background: '#09091280', overflow: 'hidden' },
  left: { flex: 1, minWidth: 0, display: 'flex', flexDirection: 'column', overflow: 'hidden' },
  filterBar: { display: 'flex', alignItems: 'center', gap: 10, padding: '16px 24px', flexShrink: 0, borderBottom: '1px solid rgba(255,255,255,0.05)' },
  loadingGrid: { flex: 1, overflowY: 'auto', display: 'grid', gridTemplateColumns: 'repeat(auto-fill, minmax(150px, 1fr))', gap: '28px 18px', padding: '20px 24px 60px', alignContent: 'start', scrollbarWidth: 'thin', scrollbarColor: 'rgba(255,255,255,0.1) transparent', contain: 'layout paint style' },
  empty: { flex: 1, display: 'flex', flexDirection: 'column', alignItems: 'center', justifyContent: 'center', gap: 10 },
  emptyTitle: { color: '#FFFFFF', fontSize: 20, fontWeight: 700, margin: 0, fontFamily: 'sans-serif' },
  emptyHint: { color: 'rgba(255,255,255,0.4)', fontSize: 14, margin: 0, fontFamily: 'sans-serif', textAlign: 'center' },
  right: { width: 300, flexShrink: 0, background: '#0C0D18', borderLeft: '1px solid rgba(255,255,255,0.06)', overflowY: 'auto', scrollbarWidth: 'none', display: 'flex', flexDirection: 'column' },
  panelEmpty: { flex: 1, display: 'flex', flexDirection: 'column', alignItems: 'center', justifyContent: 'center', gap: 12, padding: 24 },
  panelEmptyText: { color: 'rgba(255,255,255,0.28)', fontSize: 13, fontFamily: 'sans-serif', textAlign: 'center', margin: 0 },
};
