import React, { useEffect, useRef, useState } from 'react';
import type { PosterPrefs } from '../core/posterPrefs';
import type { Meta } from '../core/types';
import { cardImageUrl } from '../core/imageSizes';

const GRID_PADDING_X = 24;
const GRID_PADDING_TOP = 12;
const GRID_PADDING_BOTTOM = 60;
const GRID_GAP_X = 18;
const GRID_GAP_Y = 28;
const GRID_MIN_COLUMN_WIDTH = 150;
const GRID_OVERSCAN_ROWS = 2;
const NEAR_END_THRESHOLD_PX = 800;

export const VirtualizedPosterGrid = React.memo(function VirtualizedPosterGrid({
  items,
  selectedId,
  selectedIds,
  posterPrefs,
  onHover,
  onClick,
  onScrollActivity,
  onNearEnd,
  resetKey,
  isLoadingMore = false,
}: {
  items: Meta[];
  selectedId: string | null;
  selectedIds?: Set<string>;
  posterPrefs: PosterPrefs;
  onHover: (m: Meta | null) => boolean;
  onClick: (m: Meta) => void;
  onScrollActivity: () => void;
  onNearEnd?: () => void;
  resetKey?: string;
  isLoadingMore?: boolean;
}) {
  const scrollRef = useRef<HTMLDivElement>(null);
  const rafRef = useRef<number | null>(null);
  const [viewport, setViewport] = useState({ width: 0, height: 0, scrollTop: 0 });

  useEffect(() => {
    const node = scrollRef.current;
    if (!node) return;
    let raf: number | null = null;
    const update = () => {
      setViewport({ width: node.clientWidth, height: node.clientHeight, scrollTop: node.scrollTop });
    };
    const onResize = () => {
      if (raf != null) cancelAnimationFrame(raf);
      raf = requestAnimationFrame(() => {
        raf = null;
        update();
      });
    };
    update();
    const observer = new ResizeObserver(onResize);
    observer.observe(node);
    return () => {
      observer.disconnect();
      if (raf != null) cancelAnimationFrame(raf);
    };
  }, []);

  useEffect(() => {
    return () => {
      if (rafRef.current != null) window.cancelAnimationFrame(rafRef.current);
    };
  }, []);

  useEffect(() => {
    const node = scrollRef.current;
    if (!node) return;
    node.scrollTop = 0;
    setViewport((current) => (current.scrollTop === 0 ? current : { ...current, scrollTop: 0 }));
  }, [resetKey]);

  const narrow = viewport.width > 0 && viewport.width <= 820;
  const paddingX = narrow ? 16 : GRID_PADDING_X;
  const gapX = narrow ? 12 : GRID_GAP_X;
  const availableWidth = Math.max(0, viewport.width - paddingX * 2);
  const minColumnWidth = narrow
    ? Math.min(GRID_MIN_COLUMN_WIDTH, Math.max(96, Math.floor((availableWidth - gapX) / 2)))
    : GRID_MIN_COLUMN_WIDTH;
  const columns = Math.max(1, Math.floor((availableWidth + gapX) / (minColumnWidth + gapX)));
  const columnWidth = columns > 0 ? Math.max(minColumnWidth, (availableWidth - gapX * (columns - 1)) / columns) : minColumnWidth;
  const cardWidth = narrow ? Math.min(posterPrefs.width, Math.floor(columnWidth)) : posterPrefs.width;
  const cardHeight =
    cardWidth === posterPrefs.width ? posterPrefs.height : Math.round(posterPrefs.height * (cardWidth / posterPrefs.width));
  const cardPrefs = cardWidth === posterPrefs.width ? posterPrefs : { ...posterPrefs, width: cardWidth, height: cardHeight };
  const cardExtraHeight = posterPrefs.hideTitles ? 0 : 40;
  const itemHeight = cardHeight + cardExtraHeight;
  const rowStep = itemHeight + GRID_GAP_Y;
  const placeholderCount = isLoadingMore ? columns : 0;
  const slotCount = items.length + placeholderCount;
  const rowCount = Math.ceil(slotCount / columns);
  const totalHeight = GRID_PADDING_TOP + GRID_PADDING_BOTTOM + Math.max(0, rowCount * itemHeight + Math.max(0, rowCount - 1) * GRID_GAP_Y);
  const startRow = Math.max(0, Math.floor((viewport.scrollTop - GRID_PADDING_TOP) / rowStep) - GRID_OVERSCAN_ROWS);
  const endRow = Math.min(rowCount, Math.ceil((viewport.scrollTop + viewport.height - GRID_PADDING_TOP) / rowStep) + GRID_OVERSCAN_ROWS);

  const visible: Array<{ item: Meta; index: number; row: number; col: number }> = [];
  const placeholders: Array<{ row: number; col: number }> = [];
  for (let row = startRow; row < endRow; row += 1) {
    for (let col = 0; col < columns; col += 1) {
      const index = row * columns + col;
      const item = items[index];
      if (item) visible.push({ item, index, row, col });
      else if (index < slotCount) placeholders.push({ row, col });
    }
  }

  const handleScroll = () => {
    onScrollActivity();
    const node = scrollRef.current;
    if (!node || rafRef.current != null) return;
    rafRef.current = window.requestAnimationFrame(() => {
      rafRef.current = null;
      setViewport((current) =>
        current.scrollTop === node.scrollTop ? current : { width: node.clientWidth, height: node.clientHeight, scrollTop: node.scrollTop },
      );
      if (node.scrollTop + node.clientHeight >= node.scrollHeight - NEAR_END_THRESHOLD_PX) {
        onNearEnd?.();
      }
    });
  };

  return (
    <div
      ref={scrollRef}
      style={{
        flex: 1,
        overflowY: 'auto',
        overflowX: 'hidden',
        position: 'relative',
        scrollbarWidth: 'thin',
        scrollbarColor: 'rgba(255,255,255,0.1) transparent',
        contain: 'strict',
      }}
      onScroll={handleScroll}
    >
      <div style={{ position: 'relative', height: totalHeight, minHeight: '100%' }}>
        {visible.map(({ item, row, col }) => {
          const left = paddingX + col * (columnWidth + gapX) + Math.max(0, (columnWidth - cardWidth) / 2);
          const top = GRID_PADDING_TOP + row * rowStep;
          return (
            <div
              key={item.id}
              style={{
                position: 'absolute',
                left: 0,
                top: 0,
                width: cardWidth,
                height: itemHeight,
                transform: `translate3d(${left}px, ${top}px, 0)`,
              }}
            >
              <PosterCard
                meta={item}
                selected={selectedId === item.id || selectedIds?.has(item.id) === true}
                posterPrefs={cardPrefs}
                onHover={onHover}
                onClick={onClick}
              />
            </div>
          );
        })}
        {placeholders.map(({ row, col }) => {
          const left = paddingX + col * (columnWidth + gapX) + Math.max(0, (columnWidth - cardWidth) / 2);
          const top = GRID_PADDING_TOP + row * rowStep;
          return (
            <div
              key={`ph-${row}-${col}`}
              style={{
                position: 'absolute',
                left: 0,
                top: 0,
                width: cardWidth,
                height: cardHeight,
                borderRadius: posterPrefs.radius,
                background: '#1B212B',
                transform: `translate3d(${left}px, ${top}px, 0)`,
                animation: 'pulse 1.6s ease-in-out infinite',
                animationDelay: `${(col % 8) * 0.07}s`,
              }}
            />
          );
        })}
      </div>
    </div>
  );
});

const PosterCard = React.memo(function PosterCard({
  meta,
  selected,
  posterPrefs,
  onHover,
  onClick,
}: {
  meta: Meta;
  selected: boolean;
  posterPrefs: PosterPrefs;
  onHover: (m: Meta | null) => boolean;
  onClick: (m: Meta) => void;
}) {
  const [imgErr, setImgErr] = useState(false);
  const dpr = window.devicePixelRatio;
  const displayWidth = posterPrefs.width;
  const imgSrc =
    posterPrefs.layout === 'horizontal'
      ? cardImageUrl(meta.background, { kind: 'backdrop', displayWidth, dpr }) || cardImageUrl(meta.poster, { displayWidth, dpr })
      : cardImageUrl(meta.poster, { displayWidth, dpr }) || cardImageUrl(meta.background, { kind: 'backdrop', displayWidth, dpr });

  return (
    <div
      role="button"
      tabIndex={0}
      style={{ width: posterPrefs.width, cursor: 'pointer' }}
      onMouseEnter={() => onHover(meta)}
      onMouseLeave={() => onHover(null)}
      onClick={() => onClick(meta)}
      onKeyDown={(e) => {
        if (e.key === 'Enter' || e.key === ' ') {
          e.preventDefault();
          onClick(meta);
        }
      }}
      aria-label={meta.name}
    >
      <div
        style={{
          width: posterPrefs.width,
          height: posterPrefs.height,
          borderRadius: posterPrefs.radius,
          overflow: 'hidden',
          background: '#1B212B',
          boxShadow: selected ? 'inset 0 0 0 0.125rem rgba(255,255,255,0.55), 0 0 0.75rem rgba(255,255,255,0.08)' : 'none',
        }}
      >
        {imgSrc && !imgErr ? (
          <img
            src={imgSrc}
            alt={meta.name}
            loading="lazy"
            decoding="async"
            style={{ width: '100%', height: '100%', objectFit: 'cover', display: 'block' }}
            onError={() => setImgErr(true)}
          />
        ) : (
          <div
            style={{
              width: '100%',
              height: '100%',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              background: '#1B212B',
            }}
          >
            <span style={{ color: 'rgba(255,255,255,0.2)', fontSize: '1.5rem', fontWeight: 900 }}>
              {(meta.name ?? '').slice(0, 2).toUpperCase()}
            </span>
          </div>
        )}
      </div>
      {!posterPrefs.hideTitles && (
        <>
          <p
            style={{
              color: '#fff',
              fontSize: '0.875rem',
              fontWeight: 700,
              margin: '0.375rem 0 0',
              textAlign: 'center',
              overflow: 'hidden',
              textOverflow: 'ellipsis',
              whiteSpace: 'nowrap',
            }}
          >
            {meta.name}
          </p>
          {(meta.year ?? meta.releaseInfo) && (
            <p
              style={{
                color: 'rgba(255,255,255,0.45)',
                fontSize: '0.75rem',
                fontWeight: 400,
                margin: '0.125rem 0 0',
                textAlign: 'center',
              }}
            >
              {meta.year ?? meta.releaseInfo}
            </p>
          )}
        </>
      )}
    </div>
  );
});
