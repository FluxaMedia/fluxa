import React, { useCallback, useEffect, useRef, useState } from 'react';
import { CaretLeft as ChevronLeft, CaretRight as ChevronRight } from '@phosphor-icons/react';
import { MovieCard } from './MovieCard';
import type { Meta } from '../core/types';
import type { PosterPrefs } from '../core/posterPrefs';

const ROW_PADDING_LEFT = 32;
const MAX_ROW_ITEMS = 32;

interface Props {
  title: string;
  items: Meta[];
  onItemClick?: (meta: Meta) => void;
  onViewAll?: () => void;
  isLoading?: boolean;
  cardWidth?: number;
  cardHeight?: number;
  posterPrefs?: PosterPrefs;
  topTenEnabled?: boolean;
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
}: Props) {
  const scrollRef = useRef<HTMLDivElement>(null);
  const [hovered, setHovered] = useState(false);
  const [canScrollLeft, setCanScrollLeft] = useState(false);
  const [canScrollRight, setCanScrollRight] = useState(false);
  const width = cardWidth ?? posterPrefs?.width ?? 156;
  const height = cardHeight ?? posterPrefs?.height ?? 234;
  const radius = posterPrefs?.radius ?? 12;
  const layout = posterPrefs?.layout ?? 'vertical';
  const hideTitle = posterPrefs?.hideTitles ?? false;
  const visibleItems = items.length > MAX_ROW_ITEMS ? items.slice(0, MAX_ROW_ITEMS) : items;

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

  if (!isLoading && items.length === 0) return null;

  return (
    <div
      style={styles.row}
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
    >
      <div style={styles.header}>
        <p style={styles.title}>{title}</p>
        <button style={styles.viewAll} onClick={onViewAll}>
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
        <div ref={scrollRef} style={styles.scroll}>
          {isLoading
            ? Array.from({ length: 10 }).map((_, i) => (
                <SkeletonCard key={i} width={width} height={height} radius={radius} delay={i * 0.06} />
              ))
            : visibleItems.map((meta, idx) => (
                <MovieCard
                  key={meta.id}
                  meta={meta}
                  width={width}
                  height={height}
                  radius={radius}
                  layout={layout}
                  hideTitle={hideTitle}
                  topTenRank={topTenEnabled && idx < 10 ? idx + 1 : undefined}
                  onClick={onItemClick}
                />
              ))}
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

function SkeletonCard({ width, height, radius, delay }: { width: number; height: number; radius: number; delay: number }) {
  return (
    <div style={{ width, display: 'flex', flexDirection: 'column', gap: 8, flexShrink: 0 }}>
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
          height: 13,
          width: '70%',
          margin: '0 auto',
          borderRadius: 4,
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
    paddingTop: 8,
    marginBottom: 8,
  },
  header: {
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'space-between',
    paddingLeft: ROW_PADDING_LEFT,
    paddingRight: 32,
    marginBottom: 12,
  },
  viewAll: {
    display: 'inline-flex',
    alignItems: 'center',
    gap: 4,
    height: 28,
    padding: '0 10px 0 12px',
    border: '1px solid rgba(255,255,255,0.10)',
    borderRadius: 999,
    background: 'rgba(255,255,255,0.06)',
    color: 'rgba(255,255,255,0.70)',
    fontSize: 12,
    fontWeight: 700,
    cursor: 'pointer',
    fontFamily: 'sans-serif',
    flexShrink: 0,
  },
  title: {
    color: '#FFFFFF',
    fontSize: 18,
    fontWeight: 700,
    margin: 0,
    letterSpacing: '-0.01em',
    lineHeight: 1.2,
  },
  scroll: {
    display: 'flex',
    gap: 10,
    overflowX: 'auto',
    overflowY: 'visible',
    paddingLeft: ROW_PADDING_LEFT,
    paddingRight: 40,
    paddingBottom: 24,
    paddingTop: 8,
    scrollbarWidth: 'none',
    msOverflowStyle: 'none',
  },
};
