import React, { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { ChevronLeft, LayoutGrid, Play, Plus, TriangleAlert } from 'lucide-react';
import type { Meta, MetaLink } from '../core/types';
import type { PosterPrefs } from '../core/posterPrefs';
import { ModernTabBar } from '../components/detail/DetailButtons';
import { VirtualizedPosterGrid } from '../components/VirtualizedPosterGrid';
import { coreInvoke } from '../core/engine';
import { t } from '../i18n';

interface Props {
  title: string;
  items: Meta[];
  groups?: Array<{ type: string; items: Meta[] }>;
  isLoading?: boolean;
  loadError?: boolean;
  onLoadMore?: () => void;
  isLoadingMore?: boolean;
  posterPrefs: PosterPrefs;
  onNavigateDetail: (meta: Meta) => void;
  onBack: () => void;
  onDispatch: (a: string) => void;
}

function labelForType(type: string): string {
  if (type === 'movie') return t('auto.movies');
  if (type === 'series') return t('auto.series');
  return type.charAt(0).toUpperCase() + type.slice(1);
}

const SCROLL_HOVER_IDLE_MS = 180;

export function CategoryGridScreen({
  title,
  items,
  groups,
  isLoading = false,
  loadError = false,
  onLoadMore,
  isLoadingMore = false,
  posterPrefs,
  onNavigateDetail,
  onBack,
  onDispatch,
}: Props) {
  const [hoveredMeta, setHoveredMeta] = useState<Meta | null>(null);
  const [selectedMeta, setSelectedMeta] = useState<Meta | null>(null);
  const [activeTab, setActiveTab] = useState('all');
  const isGridScrollingRef = useRef(false);
  const scrollIdleTimerRef = useRef<number | null>(null);

  const showTabs = (groups?.length ?? 0) > 1;
  const tabs = useMemo(
    () => [{ id: 'all', label: t('auto.all') }, ...(groups ?? []).map((g) => ({ id: g.type, label: labelForType(g.type) }))],
    [groups],
  );
  const gridItems = useMemo(
    () => (showTabs && activeTab !== 'all' ? (groups!.find((g) => g.type === activeTab)?.items ?? []) : items),
    [showTabs, activeTab, groups, items],
  );

  const panelMeta = hoveredMeta ?? selectedMeta;

  useEffect(() => {
    return () => {
      if (scrollIdleTimerRef.current != null) window.clearTimeout(scrollIdleTimerRef.current);
    };
  }, []);

  const handleGridScroll = useCallback(() => {
    isGridScrollingRef.current = true;
    if (hoveredMeta) setHoveredMeta(null);
    if (scrollIdleTimerRef.current != null) window.clearTimeout(scrollIdleTimerRef.current);
    scrollIdleTimerRef.current = window.setTimeout(() => {
      isGridScrollingRef.current = false;
      scrollIdleTimerRef.current = null;
    }, SCROLL_HOVER_IDLE_MS);
  }, [hoveredMeta]);

  const handlePosterHover = useCallback((meta: Meta | null): boolean => {
    if (isGridScrollingRef.current) return false;
    setHoveredMeta(meta);
    return true;
  }, []);

  const handlePosterClick = useCallback(
    (meta: Meta) => {
      setSelectedMeta((prev) => {
        if (prev?.id === meta.id) {
          onNavigateDetail(meta);
          return prev;
        }
        return meta;
      });
    },
    [onNavigateDetail],
  );

  return (
    <div className="category-screen" style={S.screen}>
      {/* Left: header + grid */}
      <div style={S.left}>
        <div className="category-header" style={S.header}>
          <button style={S.backBtn} onClick={onBack}>
            <ChevronLeft size={20} />
          </button>
          <h2 style={S.title}>{title}</h2>
          {!isLoading && (
            <span style={S.count}>
              {gridItems.length} {t('auto.titles')}
            </span>
          )}
        </div>

        {showTabs && (
          <div style={S.tabBarWrap}>
            <ModernTabBar tabs={tabs} active={activeTab} onChange={setActiveTab} />
          </div>
        )}

        {isLoading && items.length === 0 ? (
          <div style={S.loadingGrid}>
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
        ) : loadError && items.length === 0 ? (
          <div style={S.gridEmpty}>
            <TriangleAlert size={72} style={{ color: 'rgba(255,255,255,0.35)' }} />
            <p style={S.errorTitle}>{t('home.folder_load_failed')}</p>
            <p style={S.errorBody}>{t('home.folder_load_failed_body')}</p>
          </div>
        ) : (
          <VirtualizedPosterGrid
            items={gridItems}
            selectedId={panelMeta?.id ?? null}
            posterPrefs={posterPrefs}
            onHover={handlePosterHover}
            onClick={handlePosterClick}
            onScrollActivity={handleGridScroll}
            onNearEnd={onLoadMore}
            isLoadingMore={isLoadingMore}
          />
        )}
      </div>

      {/* Right: detail panel */}
      <div className="category-panel" style={S.right}>
        {panelMeta ? (
          <DetailPanel meta={panelMeta} onPlay={() => onNavigateDetail(panelMeta)} onDispatch={onDispatch} />
        ) : (
          <div style={S.panelEmpty}>
            <LayoutGrid size={40} style={{ color: 'rgba(255,255,255,0.12)' }} />
            <p style={S.panelEmptyText}>{t('discover.hover_title_hint')}</p>
          </div>
        )}
      </div>
    </div>
  );
}

function DetailPanel({ meta, onPlay, onDispatch }: { meta: Meta; onPlay: () => void; onDispatch: (a: string) => void }) {
  const [imgErr, setImgErr] = useState(false);
  const bgUrl = !imgErr ? (meta.background ?? meta.poster) : null;

  const [cast, setCast] = useState<string[]>([]);
  const [directors, setDirectors] = useState<string[]>([]);
  useEffect(() => {
    let cancelled = false;
    coreInvoke<{ cast: MetaLink[]; directors: MetaLink[] }>('classifyMetaLinks', JSON.stringify(meta.links ?? [])).then((result) => {
      if (cancelled) return;
      setCast((result?.cast ?? []).map((l) => l.name).slice(0, 4));
      setDirectors((result?.directors ?? []).map((l) => l.name).slice(0, 2));
    });
    return () => {
      cancelled = true;
    };
  }, [meta.links]);

  return (
    <div style={DP.wrap}>
      {bgUrl && (
        <div style={DP.bg}>
          <img src={bgUrl} alt="" decoding="async" style={DP.bgImg} onError={() => setImgErr(true)} />
          <div style={DP.bgFade} />
        </div>
      )}
      <div style={DP.content}>
        <h1 style={DP.title}>{meta.name}</h1>
        <div style={DP.metaRow}>
          {meta.runtime && <span style={DP.metaItem}>{meta.runtime}</span>}
          {meta.releaseInfo && <span style={DP.metaItem}>{meta.releaseInfo}</span>}
          {meta.imdbRating && (
            <div style={DP.imdbBadge}>
              <span style={DP.imdbLabel}>IMDb</span>
              <span style={DP.imdbVal}>{meta.imdbRating}</span>
            </div>
          )}
        </div>
        {meta.description && <p style={DP.desc}>{meta.description}</p>}
        {Array.isArray(meta.genres) && meta.genres.length > 0 && (
          <div style={DP.section}>
            <p style={DP.sectionLabel}>{t('auto.genres')}</p>
            <div style={{ display: 'flex', gap: '0.625rem', flexWrap: 'wrap' }}>
              {meta.genres.slice(0, 5).map((g) => (
                <span key={g} style={DP.genreChip}>
                  {g}
                </span>
              ))}
            </div>
          </div>
        )}
        {cast.length > 0 && (
          <div style={DP.section}>
            <p style={DP.sectionLabel}>{t('auto.cast')}</p>
            <div style={{ display: 'flex', flexWrap: 'wrap', gap: '0.375rem 1.25rem' }}>
              {cast.map((name) => (
                <span key={name} style={DP.castName}>
                  {name}
                </span>
              ))}
            </div>
          </div>
        )}
        {directors.length > 0 && (
          <div style={DP.section}>
            <p style={DP.sectionLabel}>{directors.length > 1 ? t('detail.directors') : t('detail.director')}</p>
            <div style={{ display: 'flex', flexWrap: 'wrap', gap: '0.375rem 1.25rem' }}>
              {directors.map((name) => (
                <span key={name} style={DP.castName}>
                  {name}
                </span>
              ))}
            </div>
          </div>
        )}
        <div style={DP.actions}>
          <button style={DP.playBtn} onClick={onPlay}>
            <Play size={15} fill="currentColor" strokeWidth={0} />
            {t('common.details')}
          </button>
          <PanelIconBtn
            title={t('discover.add_to_list')}
            icon={<Plus size={18} />}
            onClick={() => onDispatch(JSON.stringify({ type: 'toggleWatchlistRequested', item: meta }))}
          />
        </div>
      </div>
    </div>
  );
}

function PanelIconBtn({ title, icon, onClick }: { title: string; icon: React.ReactNode; onClick?: () => void }) {
  const [hovered, setHovered] = useState(false);
  return (
    <button
      title={title}
      style={{
        width: '2.375rem',
        height: '2.375rem',
        borderRadius: '50%',
        border: '1px solid rgba(255,255,255,0.15)',
        background: hovered ? 'rgba(255,255,255,0.14)' : 'rgba(255,255,255,0.06)',
        color: '#FFF',
        cursor: 'pointer',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        transition: 'background 0.15s',
        flexShrink: 0,
      }}
      onClick={onClick}
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
    >
      {icon}
    </button>
  );
}

const S: Record<string, React.CSSProperties> = {
  screen: {
    display: 'flex',
    width: 'calc(100% - 6.5rem)',
    height: 'calc(100% - 3.25rem)',
    marginLeft: '6.5rem',
    marginTop: '3.25rem',
    background: '#09091280',
    overflow: 'hidden',
  },
  left: {
    flex: 1,
    minWidth: 0,
    display: 'flex',
    flexDirection: 'column',
    overflow: 'hidden',
  },
  header: {
    display: 'flex',
    alignItems: 'center',
    gap: '0.75rem',
    padding: '1rem 1.5rem',
    flexShrink: 0,
    borderBottom: '1px solid rgba(255,255,255,0.05)',
  },
  backBtn: {
    width: '2.25rem',
    height: '2.25rem',
    borderRadius: '50%',
    border: '1px solid rgba(255,255,255,0.12)',
    background: 'rgba(255,255,255,0.06)',
    color: '#FFFFFF',
    cursor: 'pointer',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    flexShrink: 0,
    padding: 0,
  },
  title: {
    color: '#FFFFFF',
    fontSize: '1.125rem',
    fontWeight: 700,
    margin: 0,
    letterSpacing: '-0.01em',
  },
  count: {
    color: 'rgba(255,255,255,0.35)',
    fontSize: '0.8125rem',
    fontWeight: 500,
    marginLeft: '0.25rem',
  },
  tabBarWrap: {
    padding: '0.875rem 1.5rem 0',
    flexShrink: 0,
  },
  loadingGrid: {
    flex: 1,
    overflowY: 'hidden',
    display: 'grid',
    gridTemplateColumns: 'repeat(auto-fill, minmax(9.375rem, 1fr))',
    gap: '1.75rem 1.125rem',
    padding: '1.25rem 1.5rem 3.75rem',
    alignContent: 'start',
  },
  right: {
    width: '18.75rem',
    flexShrink: 0,
    background: '#0C0D18',
    borderLeft: '1px solid rgba(255,255,255,0.06)',
    overflowY: 'auto',
    scrollbarWidth: 'none',
    display: 'flex',
    flexDirection: 'column',
  },
  panelEmpty: {
    flex: 1,
    display: 'flex',
    flexDirection: 'column',
    alignItems: 'center',
    justifyContent: 'center',
    gap: '0.75rem',
    padding: '1.5rem',
  },
  panelEmptyText: {
    color: 'rgba(255,255,255,0.28)',
    fontSize: '0.8125rem',
    textAlign: 'center',
    margin: 0,
  },
  gridEmpty: {
    flex: 1,
    display: 'flex',
    flexDirection: 'column',
    alignItems: 'center',
    justifyContent: 'center',
    gap: '1.25rem',
    padding: '2rem',
  },
  errorTitle: {
    color: '#FFFFFF',
    fontSize: '1.5rem',
    fontWeight: 700,
    textAlign: 'center',
    margin: 0,
  },
  errorBody: {
    color: 'rgba(255,255,255,0.6)',
    fontSize: '1.0625rem',
    textAlign: 'center',
    margin: 0,
    maxWidth: '34rem',
    lineHeight: 1.6,
  },
};

const DP: Record<string, React.CSSProperties> = {
  wrap: { position: 'relative', display: 'flex', flexDirection: 'column', minHeight: '100%' },
  bg: { position: 'absolute', top: 0, left: 0, right: 0, height: '13.75rem', zIndex: 0, overflow: 'hidden' },
  bgImg: { width: '100%', height: '100%', objectFit: 'cover', objectPosition: 'center top', opacity: 0.4 },
  bgFade: { position: 'absolute', inset: 0, background: 'linear-gradient(to bottom, transparent 0%, #0C0D18 100%)' },
  content: { position: 'relative', zIndex: 1, padding: '11.25rem 1.25rem 2rem', display: 'flex', flexDirection: 'column', flex: 1 },
  title: { color: '#FFFFFF', fontSize: '1.625rem', fontWeight: 900, margin: '0 0 0.625rem', lineHeight: 1.1, letterSpacing: '-0.0187rem' },
  metaRow: { display: 'flex', alignItems: 'center', gap: '0.875rem', marginBottom: '0.875rem', flexWrap: 'wrap' },
  metaItem: { color: 'rgba(255,255,255,0.65)', fontSize: '0.8125rem', fontWeight: 600 },
  imdbBadge: {
    display: 'flex',
    alignItems: 'center',
    gap: '0.25rem',
    background: '#F5C518',
    borderRadius: '0.25rem',
    padding: '0.125rem 0.375rem',
  },
  imdbLabel: { color: '#000', fontSize: '0.625rem', fontWeight: 900 },
  imdbVal: { color: '#000', fontSize: '0.6875rem', fontWeight: 900 },
  desc: {
    color: 'rgba(255,255,255,0.65)',
    fontSize: '0.8125rem',
    lineHeight: '1.1875rem',
    margin: '0 0 1rem',
    display: '-webkit-box',
    WebkitLineClamp: 5,
    WebkitBoxOrient: 'vertical',
    overflow: 'hidden',
  },
  section: { marginBottom: '0.875rem' },
  sectionLabel: {
    color: 'rgba(255,255,255,0.3)',
    fontSize: '0.625rem',
    fontWeight: 700,
    letterSpacing: '0.075rem',
    textTransform: 'uppercase',
    margin: '0 0 0.375rem',
  },
  genreChip: { color: 'rgba(255,255,255,0.65)', fontSize: '0.8125rem' },
  castName: { color: 'rgba(255,255,255,0.6)', fontSize: '0.8125rem' },
  actions: { display: 'flex', alignItems: 'center', gap: '0.625rem', marginTop: 'auto', paddingTop: '1.25rem', flexWrap: 'wrap' },
  playBtn: {
    display: 'inline-flex',
    alignItems: 'center',
    gap: '0.4375rem',
    height: '2.375rem',
    padding: '0 1.125rem',
    background: '#FFFFFF',
    color: '#000000',
    border: 'none',
    borderRadius: '0.5rem',
    fontSize: '0.8125rem',
    fontWeight: 900,
    cursor: 'pointer',
    flexShrink: 0,
  },
};
