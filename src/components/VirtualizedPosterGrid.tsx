import React, { useEffect, useRef, useState } from 'react';
import type { PosterPrefs } from '../core/posterPrefs';
import type { Meta } from '../core/types';

const GRID_PADDING_X = 24;
const GRID_PADDING_TOP = 20;
const GRID_PADDING_BOTTOM = 60;
const GRID_GAP_X = 18;
const GRID_GAP_Y = 28;
const GRID_MIN_COLUMN_WIDTH = 150;
const GRID_OVERSCAN_ROWS = 3;

export function VirtualizedPosterGrid({
  items,
  selectedId,
  posterPrefs,
  onHover,
  onClick,
  onScrollActivity,
}: {
  items: Meta[];
  selectedId: string | null;
  posterPrefs: PosterPrefs;
  onHover: (m: Meta | null) => boolean;
  onClick: (m: Meta) => void;
  onScrollActivity: () => void;
}) {
  const scrollRef = useRef<HTMLDivElement>(null);
  const rafRef = useRef<number | null>(null);
  const revealTimerRef = useRef<number | null>(null);
  const columnsRef = useRef(1);
  const [viewport, setViewport] = useState({ width: 0, height: 0, scrollTop: 0 });
  const [revealedRows, setRevealedRows] = useState(Infinity);

  useEffect(() => {
    const node = scrollRef.current;
    if (!node) return;
    const update = () => {
      setViewport({ width: node.clientWidth, height: node.clientHeight, scrollTop: node.scrollTop });
    };
    update();
    const observer = new ResizeObserver(update);
    observer.observe(node);
    return () => observer.disconnect();
  }, []);

  useEffect(() => {
    if (revealTimerRef.current != null) {
      window.clearInterval(revealTimerRef.current);
      revealTimerRef.current = null;
    }
    if (items.length === 0) { setRevealedRows(Infinity); return; }
    setRevealedRows(0);
    let revealed = 0;
    revealTimerRef.current = window.setInterval(() => {
      revealed += 1;
      const maxRows = Math.ceil(items.length / Math.max(1, columnsRef.current));
      if (revealed >= maxRows) {
        window.clearInterval(revealTimerRef.current!);
        revealTimerRef.current = null;
        setRevealedRows(Infinity);
      } else {
        setRevealedRows(revealed);
      }
    }, 80);
    return () => { if (revealTimerRef.current != null) window.clearInterval(revealTimerRef.current); };
  }, [items]);

  useEffect(() => {
    return () => {
      if (rafRef.current != null) window.cancelAnimationFrame(rafRef.current);
      if (revealTimerRef.current != null) window.clearInterval(revealTimerRef.current);
    };
  }, []);

  const cardExtraHeight = posterPrefs.hideTitles ? 0 : 23;
  const itemHeight = posterPrefs.height + cardExtraHeight;
  const availableWidth = Math.max(0, viewport.width - GRID_PADDING_X * 2);
  const columns = Math.max(1, Math.floor((availableWidth + GRID_GAP_X) / (GRID_MIN_COLUMN_WIDTH + GRID_GAP_X)));
  columnsRef.current = columns;
  const columnWidth = columns > 0
    ? Math.max(GRID_MIN_COLUMN_WIDTH, (availableWidth - GRID_GAP_X * (columns - 1)) / columns)
    : GRID_MIN_COLUMN_WIDTH;
  const rowStep = itemHeight + GRID_GAP_Y;
  const rowCount = Math.ceil(items.length / columns);
  const totalHeight = GRID_PADDING_TOP + GRID_PADDING_BOTTOM + Math.max(0, rowCount * itemHeight + Math.max(0, rowCount - 1) * GRID_GAP_Y);
  const startRow = Math.max(0, Math.floor((viewport.scrollTop - GRID_PADDING_TOP) / rowStep) - GRID_OVERSCAN_ROWS);
  const endRow = Math.min(
    rowCount,
    Math.ceil((viewport.scrollTop + viewport.height - GRID_PADDING_TOP) / rowStep) + GRID_OVERSCAN_ROWS,
  );

  const visible: Array<{ item: Meta; index: number; row: number; col: number }> = [];
  for (let row = startRow; row < endRow; row += 1) {
    for (let col = 0; col < columns; col += 1) {
      const index = row * columns + col;
      const item = items[index];
      if (!item) continue;
      visible.push({ item, index, row, col });
    }
  }

  const handleScroll = () => {
    onScrollActivity();
    const node = scrollRef.current;
    if (!node || rafRef.current != null) return;
    rafRef.current = window.requestAnimationFrame(() => {
      rafRef.current = null;
      setViewport((current) =>
        current.scrollTop === node.scrollTop
          ? current
          : { width: node.clientWidth, height: node.clientHeight, scrollTop: node.scrollTop },
      );
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
        willChange: 'scroll-position',
      }}
      onScroll={handleScroll}
    >
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
                showImage={row <= revealedRows}
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

const PosterCard = React.memo(function PosterCard({
  meta,
  selected,
  posterPrefs,
  showImage,
  onHover,
  onClick,
}: {
  meta: Meta;
  selected: boolean;
  posterPrefs: PosterPrefs;
  showImage: boolean;
  onHover: (m: Meta | null) => boolean;
  onClick: (m: Meta) => void;
}) {
  const [imgErr, setImgErr] = useState(false);

  return (
    <div
      style={{ width: posterPrefs.width, cursor: 'pointer' }}
      onMouseEnter={() => onHover(meta)}
      onMouseLeave={() => onHover(null)}
      onClick={() => onClick(meta)}
    >
      <div
        style={{
          width: posterPrefs.width,
          height: posterPrefs.height,
          borderRadius: posterPrefs.radius,
          overflow: 'hidden',
          background: '#1B212B',
          outline: selected ? '3px solid rgba(255,255,255,0.9)' : 'none',
          outlineOffset: 0,
        }}
      >
        {showImage && (posterPrefs.layout === 'horizontal' ? meta.background || meta.poster : meta.poster || meta.background) && !imgErr ? (
          <img
            src={(posterPrefs.layout === 'horizontal' ? meta.background || meta.poster : meta.poster || meta.background) ?? ''}
            alt={meta.name}
            loading="lazy"
            decoding="async"
            style={{ width: '100%', height: '100%', objectFit: 'cover', display: 'block' }}
            onError={() => setImgErr(true)}
          />
        ) : (
          <div style={{ width: '100%', height: '100%', display: 'flex', alignItems: 'center', justifyContent: 'center', background: 'linear-gradient(135deg, #1B212B, #12161D)' }}>
            <span style={{ color: 'rgba(255,255,255,0.2)', fontSize: 24, fontWeight: 900, fontFamily: 'sans-serif' }}>
              {meta.name.slice(0, 2).toUpperCase()}
            </span>
          </div>
        )}
      </div>
      {!posterPrefs.hideTitles && (
        <p style={{
          color: 'rgba(255,255,255,0.72)',
          fontSize: 12,
          fontWeight: 600,
          margin: '7px 0 0',
          textAlign: 'center',
          fontFamily: 'sans-serif',
          overflow: 'hidden',
          textOverflow: 'ellipsis',
          whiteSpace: 'nowrap',
        }}>
          {meta.name}
        </p>
      )}
    </div>
  );
});
