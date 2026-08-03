import React, { useCallback, useEffect, useRef, useState } from 'react';
import { ChevronLeft, ChevronRight } from 'lucide-react';
import { MovieCard } from './MovieCard';
import type { Meta } from '../core/types';
import type { PosterPrefs } from '../core/posterPrefs';
import { useDragScroll } from '../hooks/useDragScroll';

const ROW_PADDING_LEFT = '2rem';
const NEAR_END_THRESHOLD_PX = 1200;
const SKELETON_INDICES = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
const CARD_GAP_PX = 10;
const OVERSCAN_CARDS = 4;

interface Props {
  title: string;
  items: Meta[];
  onItemClick?: (meta: Meta) => void;
  onViewAll?: (title: string, items: Meta[]) => void;
  isLoading?: boolean;
  cardWidth?: number;
  cardHeight?: number;
  posterPrefs?: PosterPrefs;
  topTenEnabled?: boolean;
  addonIcon?: string;
  onNearEnd?: () => void;
  isLoadingMore?: boolean;
  onDispatch?: (actionJson: string) => void | Promise<void>;
}

export const ShelfRow = React.memo(function ShelfRow({
  title,
  items,
  onItemClick,
  onViewAll,
  isLoading,
  cardWidth,
  cardHeight,
  posterPrefs,
  topTenEnabled = false,
  addonIcon,
  onNearEnd,
  isLoadingMore,
  onDispatch,
}: Props) {
  const scrollRef = useRef<HTMLDivElement>(null);
  const dragScroll = useDragScroll(scrollRef);
  const [hovered, setHovered] = useState(false);
  const [canScrollLeft, setCanScrollLeft] = useState(false);
  const [canScrollRight, setCanScrollRight] = useState(false);
  const width = cardWidth ?? posterPrefs?.width ?? 156;
  const height = cardHeight ?? posterPrefs?.height ?? 234;
  const radius = posterPrefs?.radius ?? 12;
  const layout = posterPrefs?.layout ?? 'vertical';
  const hideTitle = posterPrefs?.hideTitles ?? false;

  const [viewport, setViewport] = useState({ clientWidth: 0, scrollLeft: 0 });
  const rafRef = useRef<number | null>(null);

  const checkScroll = useCallback(() => {
    const el = scrollRef.current;
    if (!el) return;
    setCanScrollLeft(el.scrollLeft > 4);
    setCanScrollRight(el.scrollLeft + el.clientWidth < el.scrollWidth - 4);
    if (onNearEnd && el.scrollWidth - (el.scrollLeft + el.clientWidth) < NEAR_END_THRESHOLD_PX) {
      onNearEnd();
    }
  }, [onNearEnd]);

  const updateViewport = useCallback(() => {
    const el = scrollRef.current;
    if (!el) return;
    setViewport({ clientWidth: el.clientWidth, scrollLeft: el.scrollLeft });
  }, []);

  useEffect(() => {
    const el = scrollRef.current;
    if (!el) return;
    checkScroll();
    updateViewport();
    const handleScroll = () => {
      checkScroll();
      if (rafRef.current != null) return;
      rafRef.current = window.requestAnimationFrame(() => {
        rafRef.current = null;
        updateViewport();
      });
    };
    el.addEventListener('scroll', handleScroll, { passive: true });
    const ro = new ResizeObserver(updateViewport);
    ro.observe(el);
    return () => {
      el.removeEventListener('scroll', handleScroll);
      ro.disconnect();
      if (rafRef.current != null) window.cancelAnimationFrame(rafRef.current);
    };
  }, [checkScroll, updateViewport, items.length]);

  const canVirtualize = !topTenEnabled && !isLoading;
  const step = width + CARD_GAP_PX;
  const startIndex = canVirtualize
    ? Math.max(0, Math.floor(viewport.scrollLeft / step) - OVERSCAN_CARDS)
    : 0;
  const endIndex = canVirtualize
    ? Math.min(items.length, Math.ceil((viewport.scrollLeft + viewport.clientWidth) / step) + OVERSCAN_CARDS)
    : items.length;
  const visibleItems = canVirtualize ? items.slice(startIndex, endIndex) : items;
  const leadSpacerWidth = canVirtualize ? startIndex * step : 0;
  const tailSpacerWidth = canVirtualize ? Math.max(0, (items.length - endIndex) * step - CARD_GAP_PX) : 0;

  if (!isLoading && items.length === 0) return null;

  return (
    <div
      style={styles.row}
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
    >
      <div style={styles.header}>
        <p style={styles.title}>{title}</p>
        <button style={styles.viewAll} onClick={() => onViewAll?.(title, items)}>
          View All
          <ChevronRight size={14} />
        </button>
      </div>
      <div style={{ position: 'relative', overflow: 'visible' }}>
        {hovered && canScrollLeft && (
          <ScrollArrow
            direction="left"
            onClick={() => scrollRef.current?.scrollBy({ left: -520, behavior: 'smooth' })}
          />
        )}
        <div ref={scrollRef} style={styles.scroll} {...dragScroll}>
          {isLoading
            ? SKELETON_INDICES.map((i) => (
                <SkeletonCard key={i} width={width} height={height} radius={radius} delay={i * 0.06} />
              ))
            : <>
                {leadSpacerWidth > 0 && <div style={{ width: leadSpacerWidth, flexShrink: 0 }} />}
                {visibleItems.map((meta, relIdx) => {
                  const idx = startIndex + relIdx;
                  return (
                    <MovieCard
                      key={`${meta.id}-${idx}`}
                      meta={meta}
                      width={width}
                      height={height}
                      radius={radius}
                      layout={layout}
                      hideTitle={hideTitle}
                      topTenRank={topTenEnabled && idx < 10 ? idx + 1 : undefined}
                      addonIcon={addonIcon}
                      onClick={onItemClick}
                      onDispatch={onDispatch}
                    />
                  );
                })}
                {tailSpacerWidth > 0 && <div style={{ width: tailSpacerWidth, flexShrink: 0 }} />}
              </>}
          {!isLoading && isLoadingMore && (
            <SkeletonCard key="loading-more" width={width} height={height} radius={radius} delay={0} />
          )}
        </div>
        {hovered && canScrollRight && (
          <ScrollArrow
            direction="right"
            onClick={() => scrollRef.current?.scrollBy({ left: 520, behavior: 'smooth' })}
          />
        )}
      </div>
    </div>
  );
}, (prev, next) => {
  if (prev.title !== next.title) return false;
  if (prev.isLoading !== next.isLoading) return false;
  if (prev.topTenEnabled !== next.topTenEnabled) return false;
  if (prev.addonIcon !== next.addonIcon) return false;
  if (prev.posterPrefs !== next.posterPrefs) return false;
  if (prev.cardWidth !== next.cardWidth || prev.cardHeight !== next.cardHeight) return false;
  if (prev.onItemClick !== next.onItemClick || prev.onViewAll !== next.onViewAll) return false;
  if (prev.onNearEnd !== next.onNearEnd || prev.isLoadingMore !== next.isLoadingMore) return false;
  if (prev.onDispatch !== next.onDispatch) return false;
  if (prev.items === next.items) return true;
  if (prev.items.length !== next.items.length) return false;
  return prev.items.every((item, i) => item.id === next.items[i]?.id);
});

function ScrollArrow({ direction, onClick }: { direction: 'left' | 'right'; onClick: () => void }) {
  const [hovered, setHovered] = useState(false);
  const isLeft = direction === 'left';
  return (
    <div
      style={{
        position: 'absolute',
        top: 0,
        bottom: 0,
        [direction]: 0,
        width: '5.625rem',
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
          width: '2.375rem',
          height: '2.375rem',
          borderRadius: '50%',
          border: '1px solid rgba(255,255,255,0.16)',
          background: hovered ? 'rgba(255,255,255,0.18)' : 'rgba(14,15,22,0.9)',
          color: '#fff',
          cursor: 'pointer',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          pointerEvents: 'auto',
          margin: isLeft ? '0 0 0 0.625rem' : '0 0.625rem 0 0',
          transition: 'background 0.15s',
          flexShrink: 0,
          boxShadow: '0 0.125rem 0.75rem rgba(0,0,0,0.5)',
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

function SkeletonCard({ width, height, radius, delay }: { width: number; height: number; radius: number; delay: number }) {
  return (
    <div style={{ width, display: 'flex', flexDirection: 'column', gap: '0.5rem', flexShrink: 0 }}>
      <div
        style={{
          width,
          height,
          borderRadius: radius,
          background: 'linear-gradient(90deg, rgba(255,255,255,0.06) 25%, rgba(255,255,255,0.12) 37%, rgba(255,255,255,0.06) 63%)',
          backgroundSize: '400% 100%',
          animation: `skeleton-shimmer 1.4s cubic-bezier(0.23, 1, 0.32, 1) ${delay}s infinite`,
        }}
      />
      <div
        style={{
          height: '0.8125rem',
          width: '70%',
          margin: '0 auto',
          borderRadius: '0.25rem',
          background: 'rgba(255,255,255,0.08)',
        }}
      />
    </div>
  );
}

const styles: Record<string, React.CSSProperties> = {
  row: {
    position: 'relative',
    zIndex: 1,
    paddingTop: '0.5rem',
    marginBottom: '0.5rem',
    contain: 'layout style',
  },
  header: {
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'space-between',
    paddingLeft: ROW_PADDING_LEFT,
    paddingRight: '2rem',
    marginBottom: '0.75rem',
  },
  viewAll: {
    display: 'inline-flex',
    alignItems: 'center',
    gap: '0.25rem',
    height: '1.75rem',
    padding: '0 0.625rem 0 0.75rem',
    border: '1px solid rgba(255,255,255,0.10)',
    borderRadius: '62.4375rem',
    background: 'rgba(255,255,255,0.06)',
    color: 'rgba(255,255,255,0.70)',
    fontSize: '0.75rem',
    fontWeight: 700,
    cursor: 'pointer',
    flexShrink: 0,
  },
  title: {
    color: '#FFFFFF',
    fontSize: '1.125rem',
    fontWeight: 700,
    margin: 0,
    letterSpacing: '-0.01em',
    lineHeight: 1.2,
  },
  scroll: {
    display: 'flex',
    gap: '0.625rem',
    overflowX: 'auto',
    overflowY: 'visible',
    paddingLeft: ROW_PADDING_LEFT,
    paddingRight: '2.5rem',
    paddingBottom: '1.5rem',
    paddingTop: '0.5rem',
    scrollbarWidth: 'none',
    msOverflowStyle: 'none',
    willChange: 'transform',
  },
};
