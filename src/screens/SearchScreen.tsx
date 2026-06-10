import React, { useCallback, useState, useEffect, useRef } from 'react';
import { CaretLeft as ChevronLeft, CaretRight as ChevronRight } from '@phosphor-icons/react';
import { MovieCard } from '../components/MovieCard';
import { posterPrefsFromState, type PosterPrefs } from '../core/posterPrefs';
import type { AppState, HomeCategory, Meta } from '../core/types';
import { getLanguage, t } from '../i18n';

interface Props {
  state: AppState;
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

const GENRE_CHIPS = [
  'genre.action', 'genre.thriller', 'genre.scifi', 'genre.comedy', 'genre.drama',
  'genre.horror', 'genre.animation', 'genre.documentary', 'genre.romance', 'genre.crime',
];

const NAV_RAIL_WIDTH = 104;

export function SearchScreen({ state, onDispatch, onNavigateDetail, query, onQueryChange, onBack }: Props) {
  const [typeFilter, setTypeFilter] = useState('');
  const search = state.search;
  const posterPrefs = posterPrefsFromState(state, 0.85);

  useEffect(() => {
    if (query.trim().length >= 2) {
      onDispatch(JSON.stringify({ type: 'searchRequested', query: query.trim(), language: getLanguage() }));
    }
  }, [query, onDispatch]);

  const handleGenreClick = (genreKey: string) => {
    onQueryChange(t(genreKey));
  };

  const categories = ((search.categories ?? []) as HomeCategory[])
    .map((category) => ({
      ...category,
      items: typeFilter
        ? category.items.filter((meta) => meta.type === typeFilter)
        : category.items,
    }))
    .filter((category) => category.items.length > 0);
  const resultCount = categories.reduce((sum, category) => sum + category.items.length, 0);

  return (
    <div style={styles.screen}>
      <div style={styles.content}>
        <button style={styles.backBtn} onClick={onBack}>
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none">
            <path d="M19 12H5M5 12l7 7M5 12l7-7" stroke="currentColor" strokeWidth="2.2" strokeLinecap="round" strokeLinejoin="round"/>
          </svg>
          {t('auto.back')}
        </button>

        <div style={styles.header}>
          <p style={styles.eyebrow}>{t('auto.search_results')}</p>
          <h1 style={styles.title}>{query.trim() ? query.trim() : t('auto.search')}</h1>
          {query.trim().length >= 2 && !search.isLoading && (
            <p style={styles.subtitle}>{t('search.results_across_catalogs', resultCount, categories.length)}</p>
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

        {search.isLoading && (
          <LoadingShelves />
        )}

        {!search.isLoading && query.length >= 2 && resultCount === 0 && (
          <div style={styles.emptyState}>
            <p style={styles.emptyTitle}>{t('format.no_results_for', query)}</p>
            <p style={styles.emptyHint}>{t('search.try_shorter_or_genre')}</p>
            <div style={{ ...styles.genreGrid, marginTop: 24 }}>
              {GENRE_CHIPS.slice(0, 6).map((g) => (
                <GenreCard key={g} genre={t(g)} onClick={() => handleGenreClick(g)} />
              ))}
            </div>
          </div>
        )}

        {!search.isLoading && categories.length > 0 && (
          <div style={styles.categoryList}>
            {categories.map((category) => (
              <SearchCategoryRow
                key={category.id}
                title={formatCatalogTitle(category.name, category.type)}
                items={category.items}
                onItemClick={onNavigateDetail}
                posterPrefs={posterPrefs}
              />
            ))}
          </div>
        )}
      </div>
    </div>
  );
}

function LoadingShelves() {
  return (
    <div style={styles.categoryList}>
      {Array.from({ length: 3 }).map((_, row) => (
        <div key={row} style={styles.category}>
          <div style={{ ...styles.skeletonTitle, width: 220 - row * 24 }} />
          <div style={styles.categoryScroll}>
            {Array.from({ length: 7 }).map((_, i) => (
              <div key={i} style={{ ...styles.skeletonCard, animationDelay: `${(row * 7 + i) * 0.04}s` }} />
            ))}
          </div>
        </div>
      ))}
    </div>
  );
}

function SearchCategoryRow({
  title,
  items,
  onItemClick,
  posterPrefs,
}: {
  title: string;
  items: Meta[];
  onItemClick: (meta: Meta) => void;
  posterPrefs: PosterPrefs;
}) {
  const scrollRef = useRef<HTMLDivElement>(null);
  const [hovered, setHovered] = useState(false);
  const [canScrollLeft, setCanScrollLeft] = useState(false);
  const [canScrollRight, setCanScrollRight] = useState(false);

  const checkScroll = useCallback(() => {
    const el = scrollRef.current;
    if (!el) return;
    setCanScrollLeft(el.scrollLeft > 4);
    setCanScrollRight(el.scrollLeft + el.clientWidth < el.scrollWidth - 4);
  }, []);

  useEffect(() => {
    const el = scrollRef.current;
    if (!el) return;
    checkScroll();
    el.addEventListener('scroll', checkScroll, { passive: true });
    const ro = new ResizeObserver(checkScroll);
    ro.observe(el);
    return () => {
      el.removeEventListener('scroll', checkScroll);
      ro.disconnect();
    };
  }, [checkScroll, items.length]);

  return (
    <div
      style={styles.category}
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
    >
      <p style={styles.categoryTitle}>{title}</p>
      <div style={{ position: 'relative' }}>
        {hovered && canScrollLeft && (
          <SearchScrollArrow
            direction="left"
            onClick={() => scrollRef.current?.scrollBy({ left: -520, behavior: 'smooth' })}
          />
        )}
        <div ref={scrollRef} style={styles.categoryScroll}>
          {items.map((meta) => (
            <MovieCard
              key={`${title}:${meta.id}`}
              meta={meta}
              width={posterPrefs.width}
              height={posterPrefs.height}
              radius={posterPrefs.radius}
              layout={posterPrefs.layout}
              hideTitle={posterPrefs.hideTitles}
              onClick={onItemClick}
            />
          ))}
        </div>
        {hovered && canScrollRight && (
          <SearchScrollArrow
            direction="right"
            onClick={() => scrollRef.current?.scrollBy({ left: 520, behavior: 'smooth' })}
          />
        )}
      </div>
    </div>
  );
}

function SearchScrollArrow({ direction, onClick }: { direction: 'left' | 'right'; onClick: () => void }) {
  const [hovered, setHovered] = useState(false);
  const isLeft = direction === 'left';
  return (
    <div
      style={{
        position: 'absolute',
        top: 0,
        bottom: 0,
        [direction]: 0,
        width: 90,
        zIndex: 3,
        display: 'flex',
        alignItems: 'center',
        justifyContent: isLeft ? 'flex-start' : 'flex-end',
        background: isLeft
          ? 'linear-gradient(to right, rgba(4,5,8,0.9) 30%, transparent 100%)'
          : 'linear-gradient(to left, rgba(4,5,8,0.9) 30%, transparent 100%)',
        pointerEvents: 'none',
      }}
    >
      <button
        style={{
          width: 38,
          height: 38,
          borderRadius: '50%',
          border: '1px solid rgba(255,255,255,0.16)',
          background: hovered ? 'rgba(255,255,255,0.18)' : 'rgba(14,15,22,0.9)',
          color: '#fff',
          cursor: 'pointer',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          pointerEvents: 'auto',
          margin: isLeft ? '0 0 0 10px' : '0 10px 0 0',
          transition: 'background 0.15s',
          flexShrink: 0,
          boxShadow: '0 2px 12px rgba(0,0,0,0.5)',
          padding: 0,
        }}
        onClick={onClick}
        onMouseEnter={() => setHovered(true)}
        onMouseLeave={() => setHovered(false)}
      >
        {isLeft ? <ChevronLeft size={18} /> : <ChevronRight size={18} />}
      </button>
    </div>
  );
}

function formatCatalogTitle(name: string, type: string): string {
  let label: string;
  if (type === 'movie') label = t('auto.movies');
  else if (type === 'series') label = t('auto.series');
  else if (type) label = type.charAt(0).toUpperCase() + type.slice(1);
  else return name;
  return `${name} - ${label}`;
}



function TypeChip({ label, selected, onClick }: { label: string; selected: boolean; onClick: () => void }) {
  const [hovered, setHovered] = useState(false);
  return (
    <button
      style={{
        height: 34,
        padding: '0 16px',
        borderRadius: 999,
        border: selected ? 'none' : '1px solid rgba(255,255,255,0.12)',
        background: selected ? '#FFFFFF' : hovered ? 'rgba(255,255,255,0.08)' : 'transparent',
        color: selected ? '#000000' : '#FFFFFF',
        fontSize: 13,
        fontWeight: 700,
        cursor: 'pointer',
        fontFamily: 'sans-serif',
        transition: 'background 0.15s, color 0.15s',
        flexShrink: 0,
      }}
      onClick={onClick}
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
    >
      {label}
    </button>
  );
}

function GenreCard({ genre, onClick }: { genre: string; onClick: () => void }) {
  const [hovered, setHovered] = useState(false);
  return (
    <button
      style={{
        height: 64,
        borderRadius: 12,
        border: '1px solid rgba(255,255,255,0.08)',
        background: hovered ? 'rgba(255,255,255,0.1)' : 'rgba(255,255,255,0.04)',
        color: '#FFFFFF',
        fontSize: 14,
        fontWeight: 700,
        cursor: 'pointer',
        fontFamily: 'sans-serif',
        transition: 'background 0.15s',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        boxShadow: hovered ? '0 0 0 1.5px rgba(255,255,255,0.3)' : 'none',
      }}
      onClick={onClick}
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
    >
      {genre}
    </button>
  );
}

function SearchResultRow({ meta, onClick }: { meta: Meta; onClick: (meta: Meta) => void }) {
  const [hovered, setHovered] = useState(false);
  const [imgErr, setImgErr] = useState(false);

  const metaChips: string[] = [];
  if (meta.releaseInfo) metaChips.push(meta.releaseInfo);
  metaChips.push(meta.type === 'movie' ? t('auto.movie') : t('auto.series'));
  if (meta.genres?.[0]) metaChips.push(meta.genres[0]);

  return (
    <div
      style={{
        display: 'flex',
        alignItems: 'center',
        gap: 16,
        padding: '10px 12px',
        borderRadius: 12,
        cursor: 'pointer',
        background: hovered ? 'rgba(255,255,255,0.06)' : 'rgba(255,255,255,0.03)',
        border: '1px solid rgba(255,255,255,0.05)',
        transition: 'background 0.15s',
      }}
      onClick={() => onClick(meta)}
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
    >
      {/* Poster */}
      <div style={{ width: 54, height: 78, borderRadius: 8, overflow: 'hidden', flexShrink: 0, background: 'rgba(255,255,255,0.05)' }}>
        {meta.poster && !imgErr ? (
          <img
            src={meta.poster}
            alt={meta.name}
            style={{ width: '100%', height: '100%', objectFit: 'cover' }}
            onError={() => setImgErr(true)}
          />
        ) : (
          <div style={{ width: '100%', height: '100%', display: 'flex', alignItems: 'center', justifyContent: 'center' }}>
            <span style={{ color: 'rgba(255,255,255,0.2)', fontSize: 16, fontWeight: 900, fontFamily: 'sans-serif' }}>
              {meta.name.slice(0, 2).toUpperCase()}
            </span>
          </div>
        )}
      </div>

      {/* Info */}
      <div style={{ flex: 1, minWidth: 0 }}>
        <p style={{ color: '#FFFFFF', fontSize: 15, fontWeight: 700, margin: '0 0 5px', overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap', fontFamily: 'sans-serif' }}>
          {meta.name}
        </p>
        <p style={{ color: 'rgba(255,255,255,0.45)', fontSize: 13, fontWeight: 600, margin: 0, fontFamily: 'sans-serif' }}>
          {metaChips.join('  ·  ')}
        </p>
      </div>

      {/* Arrow */}
      <svg width="18" height="18" viewBox="0 0 24 24" fill="rgba(255,255,255,0.2)" style={{ flexShrink: 0 }}>
        <path d="M10 6L8.59 7.41 13.17 12l-4.58 4.59L10 18l6-6z"/>
      </svg>
    </div>
  );
}

const styles: Record<string, React.CSSProperties> = {
  screen: {
    display: 'flex',
    flexDirection: 'column',
    height: '100%',
    background: '#040508',
    overflow: 'hidden',
  },
  topBar: {
    padding: '24px 32px 16px',
    flexShrink: 0,
  },
  searchBox: {
    display: 'flex',
    alignItems: 'center',
    gap: 12,
    height: 52,
    background: 'rgba(255,255,255,0.06)',
    border: '1px solid rgba(255,255,255,0.1)',
    borderRadius: 14,
    padding: '0 16px',
  },
  input: {
    flex: 1,
    background: 'transparent',
    border: 'none',
    outline: 'none',
    color: '#FFFFFF',
    fontSize: 16,
    fontWeight: 600,
    fontFamily: 'sans-serif',
  },
  clearBtn: {
    width: 28,
    height: 28,
    background: 'transparent',
    border: 'none',
    borderRadius: '50%',
    cursor: 'pointer',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    flexShrink: 0,
    padding: 0,
  },
  content: {
    flex: 1,
    overflowY: 'auto',
    padding: `34px 36px 44px ${NAV_RAIL_WIDTH + 42}px`,
    scrollbarWidth: 'none',
  },
  header: {
    marginBottom: 22,
  },
  backBtn: {
    display: 'inline-flex',
    alignItems: 'center',
    gap: 7,
    height: 34,
    padding: '0 12px',
    borderRadius: 999,
    border: '1px solid rgba(255,255,255,0.1)',
    background: 'rgba(255,255,255,0.06)',
    color: 'rgba(255,255,255,0.76)',
    fontSize: 12,
    fontWeight: 750,
    marginBottom: 18,
  },
  eyebrow: {
    color: 'rgba(255,255,255,0.46)',
    fontSize: 12,
    fontWeight: 800,
    margin: '0 0 6px',
    fontFamily: 'sans-serif',
    textTransform: 'uppercase',
    letterSpacing: '0.8px',
  },
  title: {
    color: '#FFFFFF',
    fontSize: 34,
    lineHeight: '39px',
    fontWeight: 900,
    margin: 0,
    fontFamily: 'sans-serif',
  },
  subtitle: {
    color: 'rgba(255,255,255,0.52)',
    fontSize: 13,
    fontWeight: 600,
    margin: '8px 0 0',
    fontFamily: 'sans-serif',
  },
  typeRow: {
    display: 'flex',
    gap: 8,
    marginBottom: 28,
    flexWrap: 'wrap',
  },
  sectionLabel: {
    color: 'rgba(255,255,255,0.55)',
    fontSize: 13,
    fontWeight: 700,
    margin: '0 0 14px',
    fontFamily: 'sans-serif',
    textTransform: 'uppercase',
    letterSpacing: '0.5px',
  },
  genreGrid: {
    display: 'grid',
    gridTemplateColumns: 'repeat(auto-fill, minmax(140px, 1fr))',
    gap: 10,
    marginBottom: 32,
  },
  resultList: {
    display: 'flex',
    flexDirection: 'column',
    gap: 8,
  },
  categoryList: {
    display: 'flex',
    flexDirection: 'column',
    gap: 4,
    paddingBottom: 36,
  },
  category: {
    paddingTop: 12,
  },
  categoryTitle: {
    color: '#FFFFFF',
    fontSize: 20,
    fontWeight: 800,
    margin: '0 0 12px',
    fontFamily: 'sans-serif',
  },
  categoryScroll: {
    display: 'flex',
    gap: 14,
    overflowX: 'auto',
    overflowY: 'visible',
    padding: '8px 36px 28px 0',
    scrollbarWidth: 'none',
  },
  skeletonTitle: {
    height: 20,
    borderRadius: 6,
    background: 'rgba(255,255,255,0.05)',
    marginBottom: 14,
    animation: 'pulse 1.6s ease-in-out infinite',
  },
  skeletonCard: {
    width: 132,
    height: 198,
    borderRadius: 12,
    background: 'rgba(255,255,255,0.045)',
    flexShrink: 0,
    animation: 'pulse 1.6s ease-in-out infinite',
  },
  emptyState: {
    paddingTop: 40,
  },
  emptyTitle: {
    color: '#FFFFFF',
    fontSize: 18,
    fontWeight: 700,
    margin: '0 0 6px',
    fontFamily: 'sans-serif',
  },
  emptyHint: {
    color: 'rgba(255,255,255,0.4)',
    fontSize: 14,
    margin: 0,
    fontFamily: 'sans-serif',
  },
};
