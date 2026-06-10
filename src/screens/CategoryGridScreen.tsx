import React, { useCallback, useEffect, useRef, useState } from 'react';
import { CaretLeft as ChevronLeft } from '@phosphor-icons/react';
import type { Meta } from '../core/types';
import type { PosterPrefs } from '../core/posterPrefs';
import { t } from '../i18n';

interface Props {
  title: string;
  items: Meta[];
  posterPrefs: PosterPrefs;
  onNavigateDetail: (meta: Meta) => void;
  onBack: () => void;
  onDispatch: (a: string) => void;
}

const GRID_PADDING_X = 24;
const GRID_PADDING_TOP = 20;
const GRID_PADDING_BOTTOM = 60;
const GRID_GAP_X = 18;
const GRID_GAP_Y = 28;
const GRID_MIN_COLUMN_WIDTH = 150;
const GRID_OVERSCAN_ROWS = 3;
const SCROLL_HOVER_IDLE_MS = 180;
const SCROLL_IMAGE_IDLE_MS = 240;

export function CategoryGridScreen({ title, items, posterPrefs, onNavigateDetail, onBack, onDispatch }: Props) {
  const [hoveredMeta, setHoveredMeta] = useState<Meta | null>(null);
  const [selectedMeta, setSelectedMeta] = useState<Meta | null>(null);
  const isGridScrollingRef = useRef(false);
  const scrollIdleTimerRef = useRef<number | null>(null);

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

  const handlePosterClick = useCallback((meta: Meta) => {
    setSelectedMeta(meta);
    onNavigateDetail(meta);
  }, [onNavigateDetail]);

  return (
    <div style={S.screen}>
      {/* Left: header + grid */}
      <div style={S.left}>
        <div style={S.header}>
          <button style={S.backBtn} onClick={onBack}>
            <ChevronLeft size={20} />
          </button>
          <h2 style={S.title}>{title}</h2>
          <span style={S.count}>{items.length} {t('auto.titles') ?? 'titles'}</span>
        </div>

        <VirtualizedPosterGrid
          items={items}
          selectedId={panelMeta?.id ?? null}
          posterPrefs={posterPrefs}
          onHover={handlePosterHover}
          onClick={handlePosterClick}
          onScrollActivity={handleGridScroll}
        />
      </div>

      {/* Right: detail panel */}
      <div style={S.right}>
        {panelMeta ? (
          <DetailPanel meta={panelMeta} onPlay={() => onNavigateDetail(panelMeta)} onDispatch={onDispatch} />
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

function DetailPanel({ meta, onPlay, onDispatch }: { meta: Meta; onPlay: () => void; onDispatch: (a: string) => void }) {
  const [imgErr, setImgErr] = useState(false);
  const bgUrl = !imgErr ? (meta.background ?? meta.poster) : null;

  const cast = (meta.links ?? [])
    .filter((l) => l.category.toLowerCase().includes('cast') || l.category.toLowerCase() === 'actor')
    .map((l) => l.name)
    .slice(0, 4);

  const directors = (meta.links ?? [])
    .filter((l) => l.category.toLowerCase().includes('director'))
    .map((l) => l.name)
    .slice(0, 2);

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
        {meta.genres && meta.genres.length > 0 && (
          <div style={DP.section}>
            <p style={DP.sectionLabel}>{t('auto.genres')}</p>
            <div style={{ display: 'flex', gap: 10, flexWrap: 'wrap' }}>
              {meta.genres.slice(0, 5).map((g) => (
                <span key={g} style={DP.genreChip}>{g}</span>
              ))}
            </div>
          </div>
        )}
        {cast.length > 0 && (
          <div style={DP.section}>
            <p style={DP.sectionLabel}>{t('auto.cast')}</p>
            <div style={{ display: 'flex', flexWrap: 'wrap', gap: '6px 20px' }}>
              {cast.map((name) => <span key={name} style={DP.castName}>{name}</span>)}
            </div>
          </div>
        )}
        {directors.length > 0 && (
          <div style={DP.section}>
            <p style={DP.sectionLabel}>{directors.length > 1 ? t('detail.directors') : t('detail.director')}</p>
            <div style={{ display: 'flex', flexWrap: 'wrap', gap: '6px 20px' }}>
              {directors.map((name) => <span key={name} style={DP.castName}>{name}</span>)}
            </div>
          </div>
        )}
        <div style={DP.actions}>
          <button style={DP.playBtn} onClick={onPlay}>
            <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor"><path d="M8 5v14l11-7z" /></svg>
            {t('common.details')}
          </button>
          <PanelIconBtn
            title={t('discover.add_to_list')}
            icon={<svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor"><path d="M19 13H13v6h-2v-6H5v-2h6V5h2v6h6v2z" /></svg>}
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
        width: 38, height: 38, borderRadius: '50%',
        border: '1px solid rgba(255,255,255,0.15)',
        background: hovered ? 'rgba(255,255,255,0.14)' : 'rgba(255,255,255,0.06)',
        color: '#FFF', cursor: 'pointer',
        display: 'flex', alignItems: 'center', justifyContent: 'center',
        transition: 'background 0.15s', flexShrink: 0,
      }}
      onClick={onClick}
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
    >
      {icon}
    </button>
  );
}



function VirtualizedPosterGrid({ items, selectedId, posterPrefs, onHover, onClick, onScrollActivity }: {
  items: Meta[];
  selectedId: string | null;
  posterPrefs: PosterPrefs;
  onHover: (m: Meta | null) => boolean;
  onClick: (m: Meta) => void;
  onScrollActivity: () => void;
}) {
  const scrollRef = useRef<HTMLDivElement>(null);
  const rafRef = useRef<number | null>(null);
  const imageIdleTimerRef = useRef<number | null>(null);
  const [viewport, setViewport] = useState({ width: 0, height: 0, scrollTop: 0 });
  const [renderImages, setRenderImages] = useState(true);

  useEffect(() => {
    const node = scrollRef.current;
    if (!node) return;
    const update = () => setViewport({ width: node.clientWidth, height: node.clientHeight, scrollTop: node.scrollTop });
    update();
    const observer = new ResizeObserver(update);
    observer.observe(node);
    return () => observer.disconnect();
  }, []);

  useEffect(() => {
    return () => {
      if (rafRef.current != null) window.cancelAnimationFrame(rafRef.current);
      if (imageIdleTimerRef.current != null) window.clearTimeout(imageIdleTimerRef.current);
    };
  }, []);

  const cardExtraHeight = posterPrefs.hideTitles ? 0 : 23;
  const itemHeight = posterPrefs.height + cardExtraHeight;
  const availableWidth = Math.max(0, viewport.width - GRID_PADDING_X * 2);
  const columns = Math.max(1, Math.floor((availableWidth + GRID_GAP_X) / (GRID_MIN_COLUMN_WIDTH + GRID_GAP_X)));
  const columnWidth = columns > 0
    ? Math.max(GRID_MIN_COLUMN_WIDTH, (availableWidth - GRID_GAP_X * (columns - 1)) / columns)
    : GRID_MIN_COLUMN_WIDTH;
  const rowStep = itemHeight + GRID_GAP_Y;
  const rowCount = Math.ceil(items.length / columns);
  const totalHeight = GRID_PADDING_TOP + GRID_PADDING_BOTTOM + Math.max(0, rowCount * itemHeight + Math.max(0, rowCount - 1) * GRID_GAP_Y);
  const startRow = Math.max(0, Math.floor((viewport.scrollTop - GRID_PADDING_TOP) / rowStep) - GRID_OVERSCAN_ROWS);
  const endRow = Math.min(rowCount, Math.ceil((viewport.scrollTop + viewport.height - GRID_PADDING_TOP) / rowStep) + GRID_OVERSCAN_ROWS);

  const visible: Array<{ item: Meta; row: number; col: number }> = [];
  for (let row = startRow; row < endRow; row++) {
    for (let col = 0; col < columns; col++) {
      const item = items[row * columns + col];
      if (!item) continue;
      visible.push({ item, row, col });
    }
  }

  const handleScroll = () => {
    onScrollActivity();
    setRenderImages(false);
    if (imageIdleTimerRef.current != null) window.clearTimeout(imageIdleTimerRef.current);
    imageIdleTimerRef.current = window.setTimeout(() => {
      imageIdleTimerRef.current = null;
      setRenderImages(true);
    }, SCROLL_IMAGE_IDLE_MS);

    const node = scrollRef.current;
    if (!node || rafRef.current != null) return;
    rafRef.current = window.requestAnimationFrame(() => {
      rafRef.current = null;
      setViewport((c) => c.scrollTop === node.scrollTop ? c : { width: node.clientWidth, height: node.clientHeight, scrollTop: node.scrollTop });
    });
  };

  return (
    <div ref={scrollRef} style={S.virtualGrid} onScroll={handleScroll}>
      <div style={{ position: 'relative', height: totalHeight, minHeight: '100%' }}>
        {visible.map(({ item, row, col }) => {
          const left = GRID_PADDING_X + col * (columnWidth + GRID_GAP_X) + Math.max(0, (columnWidth - posterPrefs.width) / 2);
          const top = GRID_PADDING_TOP + row * rowStep;
          return (
            <div key={item.id} style={{ position: 'absolute', left, top, width: posterPrefs.width, height: itemHeight }}>
              <PosterCard
                meta={item}
                selected={selectedId === item.id}
                posterPrefs={posterPrefs}
                renderImage={renderImages}
                onHover={onHover}
                onClick={onClick}
              />
            </div>
          );
        })}
      </div>
    </div>
  );
}

const PosterCard = React.memo(function PosterCard({ meta, selected, posterPrefs, renderImage, onHover, onClick }: {
  meta: Meta; selected: boolean; posterPrefs: PosterPrefs; renderImage: boolean;
  onHover: (m: Meta | null) => boolean; onClick: (m: Meta) => void;
}) {
  const [imgErr, setImgErr] = useState(false);
  const src = posterPrefs.layout === 'horizontal'
    ? meta.background || meta.poster
    : meta.poster || meta.background;

  return (
    <div
      style={{ width: posterPrefs.width, cursor: 'pointer' }}
      onMouseEnter={() => onHover(meta)}
      onMouseLeave={() => onHover(null)}
      onClick={() => onClick(meta)}
    >
      <div style={{
        width: posterPrefs.width, height: posterPrefs.height,
        borderRadius: posterPrefs.radius, overflow: 'hidden', background: '#1B212B',
        outline: selected ? '3px solid rgba(255,255,255,0.9)' : 'none',
      }}>
        {renderImage && src && !imgErr ? (
          <img src={src} alt={meta.name} loading="lazy" decoding="async"
            style={{ width: '100%', height: '100%', objectFit: 'cover', display: 'block' }}
            onError={() => setImgErr(true)} />
        ) : (
          <div style={{ width: '100%', height: '100%', display: 'flex', alignItems: 'center', justifyContent: 'center', background: 'linear-gradient(135deg, #1B212B, #12161D)' }}>
            <span style={{ color: 'rgba(255,255,255,0.2)', fontSize: 24, fontWeight: 900, fontFamily: 'sans-serif' }}>
              {meta.name.slice(0, 2).toUpperCase()}
            </span>
          </div>
        )}
      </div>
      {!posterPrefs.hideTitles && (
        <p style={{ color: 'rgba(255,255,255,0.72)', fontSize: 12, fontWeight: 600, margin: '7px 0 0', textAlign: 'center', fontFamily: 'sans-serif', overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>
          {meta.name}
        </p>
      )}
    </div>
  );
});



const S: Record<string, React.CSSProperties> = {
  screen: {
    display: 'flex',
    width: 'calc(100% - 104px)',
    height: 'calc(100% - 52px)',
    marginLeft: 104,
    marginTop: 52,
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
    gap: 12,
    padding: '16px 24px',
    flexShrink: 0,
    borderBottom: '1px solid rgba(255,255,255,0.05)',
  },
  backBtn: {
    width: 36, height: 36, borderRadius: '50%',
    border: '1px solid rgba(255,255,255,0.12)',
    background: 'rgba(255,255,255,0.06)',
    color: '#FFFFFF', cursor: 'pointer',
    display: 'flex', alignItems: 'center', justifyContent: 'center',
    flexShrink: 0, padding: 0,
  },
  title: {
    color: '#FFFFFF', fontSize: 18, fontWeight: 700,
    margin: 0, fontFamily: 'sans-serif', letterSpacing: '-0.01em',
  },
  count: {
    color: 'rgba(255,255,255,0.35)', fontSize: 13,
    fontWeight: 500, fontFamily: 'sans-serif', marginLeft: 4,
  },
  virtualGrid: {
    flex: 1,
    overflowY: 'auto',
    overflowX: 'hidden',
    position: 'relative',
    scrollbarWidth: 'thin',
    scrollbarColor: 'rgba(255,255,255,0.1) transparent',
    contain: 'strict',
    willChange: 'scroll-position',
  },
  right: {
    width: 300, flexShrink: 0,
    background: '#0C0D18',
    borderLeft: '1px solid rgba(255,255,255,0.06)',
    overflowY: 'auto', scrollbarWidth: 'none',
    display: 'flex', flexDirection: 'column',
  },
  panelEmpty: {
    flex: 1, display: 'flex', flexDirection: 'column',
    alignItems: 'center', justifyContent: 'center', gap: 12, padding: 24,
  },
  panelEmptyText: {
    color: 'rgba(255,255,255,0.28)', fontSize: 13,
    fontFamily: 'sans-serif', textAlign: 'center', margin: 0,
  },
};

const DP: Record<string, React.CSSProperties> = {
  wrap: { position: 'relative', display: 'flex', flexDirection: 'column', minHeight: '100%' },
  bg: { position: 'absolute', top: 0, left: 0, right: 0, height: 220, zIndex: 0, overflow: 'hidden' },
  bgImg: { width: '100%', height: '100%', objectFit: 'cover', objectPosition: 'center top', opacity: 0.4 },
  bgFade: { position: 'absolute', inset: 0, background: 'linear-gradient(to bottom, transparent 0%, #0C0D18 100%)' },
  content: { position: 'relative', zIndex: 1, padding: '180px 20px 32px', display: 'flex', flexDirection: 'column', flex: 1 },
  title: { color: '#FFFFFF', fontSize: 26, fontWeight: 900, margin: '0 0 10px', fontFamily: 'sans-serif', lineHeight: 1.1, letterSpacing: '-0.3px' },
  metaRow: { display: 'flex', alignItems: 'center', gap: 14, marginBottom: 14, flexWrap: 'wrap' },
  metaItem: { color: 'rgba(255,255,255,0.65)', fontSize: 13, fontWeight: 600, fontFamily: 'sans-serif' },
  imdbBadge: { display: 'flex', alignItems: 'center', gap: 4, background: '#F5C518', borderRadius: 4, padding: '2px 6px' },
  imdbLabel: { color: '#000', fontSize: 10, fontWeight: 900, fontFamily: 'sans-serif' },
  imdbVal: { color: '#000', fontSize: 11, fontWeight: 900, fontFamily: 'sans-serif' },
  desc: { color: 'rgba(255,255,255,0.65)', fontSize: 13, lineHeight: '19px', margin: '0 0 16px', fontFamily: 'sans-serif', display: '-webkit-box', WebkitLineClamp: 5, WebkitBoxOrient: 'vertical', overflow: 'hidden' },
  section: { marginBottom: 14 },
  sectionLabel: { color: 'rgba(255,255,255,0.3)', fontSize: 10, fontWeight: 700, letterSpacing: '1.2px', textTransform: 'uppercase', margin: '0 0 6px', fontFamily: 'sans-serif' },
  genreChip: { color: 'rgba(255,255,255,0.65)', fontSize: 13, fontFamily: 'sans-serif' },
  castName: { color: 'rgba(255,255,255,0.6)', fontSize: 13, fontFamily: 'sans-serif' },
  actions: { display: 'flex', alignItems: 'center', gap: 10, marginTop: 'auto', paddingTop: 20, flexWrap: 'wrap' },
  playBtn: { display: 'inline-flex', alignItems: 'center', gap: 7, height: 38, padding: '0 18px', background: '#FFFFFF', color: '#000000', border: 'none', borderRadius: 8, fontSize: 13, fontWeight: 900, cursor: 'pointer', fontFamily: 'sans-serif', flexShrink: 0 },
};
