import React, { useMemo } from 'react';
import type { Meta } from '../core/types';
import { t } from '../i18n';
import { cardImageUrl } from '../core/imageSizes';
import { useInViewport } from '../hooks/useInViewport';
import { useGifSlot } from '../hooks/useGifSlot';
import { useDragScroll } from '../hooks/useDragScroll';
import { usePosterSrc } from '../hooks/usePosterSrc';

const ROW_PADDING_LEFT = '2rem';

const SCROLL_GAP = 12;
const BUFFER = 3;

const dominantColorCache = new Map<string, string | null>();

function useDominantColor(src: string | null, enabled: boolean): string | null {
  const [color, setColor] = React.useState<string | null>(() => (src ? (dominantColorCache.get(src) ?? null) : null));

  React.useEffect(() => {
    if (!enabled || !src) return;
    const cached = dominantColorCache.get(src);
    if (cached !== undefined) {
      setColor(cached);
      return;
    }
    let cancelled = false;
    const img = new Image();
    img.crossOrigin = 'anonymous';
    img.onload = () => {
      if (cancelled) return;
      try {
        const canvas = document.createElement('canvas');
        canvas.width = 16;
        canvas.height = 16;
        const ctx = canvas.getContext('2d');
        if (!ctx) throw new Error('no ctx');
        ctx.drawImage(img, 0, 0, 16, 16);
        const data = ctx.getImageData(0, 0, 16, 16).data;
        let r = 0;
        let g = 0;
        let b = 0;
        let count = 0;
        for (let i = 0; i < data.length; i += 4) {
          r += data[i];
          g += data[i + 1];
          b += data[i + 2];
          count++;
        }
        const rgb = `rgb(${Math.round(r / count)}, ${Math.round(g / count)}, ${Math.round(b / count)})`;
        dominantColorCache.set(src, rgb);
        if (!cancelled) setColor(rgb);
      } catch {
        dominantColorCache.set(src, null);
        if (!cancelled) setColor(null);
      }
    };
    img.onerror = () => {
      dominantColorCache.set(src, null);
      if (!cancelled) setColor(null);
    };
    img.src = src;
    return () => {
      cancelled = true;
    };
  }, [src, enabled]);

  return color;
}

function cardWidth(folder: Meta): number {
  const shape = (((folder as unknown as Record<string, unknown>).reason as string | undefined) ?? 'poster').toLowerCase();
  if (shape === 'wide' || shape === 'landscape') return 280;
  if (shape === 'square') return 150;
  return 156;
}

export const CollectionShelfRow = React.memo(function CollectionShelfRow({
  title,
  folders,
  onFolderClick,
  addonIcon,
  gifAutoplayEnabled = true,
  focusGlowEnabled = false,
}: {
  title: string;
  folders: Meta[];
  onFolderClick: (f: Meta) => void;
  addonIcon?: string;
  gifAutoplayEnabled?: boolean;
  focusGlowEnabled?: boolean;
}) {
  const scrollRef = React.useRef<HTMLDivElement>(null);
  const dragScroll = useDragScroll(scrollRef);
  const [canScrollLeft, setCanScrollLeft] = React.useState(false);
  const [canScrollRight, setCanScrollRight] = React.useState(true);
  const [scrollLeft, setScrollLeft] = React.useState(0);
  const [containerWidth, setContainerWidth] = React.useState(() => window.innerWidth);
  const scrollRafRef = React.useRef<number | null>(null);

  const slotWidth = useMemo(() => (folders.length > 0 ? cardWidth(folders[0]) + SCROLL_GAP : 168), [folders]);

  const updateArrows = React.useCallback(() => {
    const el = scrollRef.current;
    if (!el) return;
    setCanScrollLeft(el.scrollLeft > 4);
    setCanScrollRight(el.scrollLeft + el.clientWidth < el.scrollWidth - 4);
  }, []);

  React.useEffect(() => {
    const el = scrollRef.current;
    if (!el) return;
    updateArrows();
    el.addEventListener('scroll', updateArrows, { passive: true });
    const ro = new ResizeObserver(() => {
      if (el) setContainerWidth(el.clientWidth);
    });
    ro.observe(el);
    return () => {
      el.removeEventListener('scroll', updateArrows);
      ro.disconnect();
    };
  }, [updateArrows, folders.length]);

  const handleScroll = React.useCallback(() => {
    const el = scrollRef.current;
    if (!el || scrollRafRef.current != null) return;
    scrollRafRef.current = window.requestAnimationFrame(() => {
      scrollRafRef.current = null;
      updateArrows();
      setScrollLeft((current) => (el.scrollLeft === current ? current : el.scrollLeft));
    });
  }, [updateArrows]);

  React.useEffect(() => {
    return () => {
      if (scrollRafRef.current != null) window.cancelAnimationFrame(scrollRafRef.current);
    };
  }, []);

  const scroll = (dir: 'left' | 'right') => {
    scrollRef.current?.scrollBy({ left: dir === 'right' ? 660 : -660, behavior: 'smooth' });
  };

  const startIdx = Math.max(0, Math.floor(scrollLeft / slotWidth) - BUFFER);
  const endIdx = Math.min(folders.length - 1, Math.ceil((scrollLeft + containerWidth) / slotWidth) + BUFFER);

  const visibleFolders = useMemo(() => folders.slice(startIdx, endIdx + 1), [folders, startIdx, endIdx]);

  const beforeWidth = startIdx > 0 ? startIdx * slotWidth - SCROLL_GAP : 0;
  const afterCount = folders.length - endIdx - 1;
  const afterWidth = afterCount > 0 ? afterCount * slotWidth - SCROLL_GAP : 0;

  return (
    <div style={collStyles.section}>
      <div style={headerStyles.header}>
        <p style={headerStyles.title}>{title}</p>
        <div style={{ display: 'flex', alignItems: 'center', gap: '0.375rem' }}>
          <button
            style={{ ...headerStyles.arrowBtn, opacity: canScrollLeft ? 1 : 0.28, cursor: canScrollLeft ? 'pointer' : 'default' }}
            onClick={() => canScrollLeft && scroll('left')}
            aria-label={t('common.scroll_left')}
          >
            <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
              <path d="M15 18l-6-6 6-6v12z" />
            </svg>
          </button>
          <button
            style={{ ...headerStyles.arrowBtn, opacity: canScrollRight ? 1 : 0.28, cursor: canScrollRight ? 'pointer' : 'default' }}
            onClick={() => canScrollRight && scroll('right')}
            aria-label={t('common.scroll_right')}
          >
            <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
              <path d="M9 18l6-6-6-6v12z" />
            </svg>
          </button>
        </div>
      </div>
      <div ref={scrollRef} style={collStyles.scroll} onScroll={handleScroll} {...dragScroll}>
        {beforeWidth > 0 && <div style={{ width: beforeWidth, flexShrink: 0 }} />}
        {visibleFolders.map((folder) => (
          <FolderTileCard
            key={folder.id}
            folder={folder}
            onClick={onFolderClick}
            addonIcon={addonIcon}
            gifAutoplayEnabled={gifAutoplayEnabled}
            focusGlowEnabled={focusGlowEnabled}
          />
        ))}
        {afterWidth > 0 && <div style={{ width: afterWidth, flexShrink: 0 }} />}
      </div>
    </div>
  );
});

const FolderTileCard = React.memo(function FolderTileCard({
  folder,
  onClick,
  addonIcon,
  gifAutoplayEnabled,
  focusGlowEnabled,
}: {
  folder: Meta;
  onClick: (f: Meta) => void;
  addonIcon?: string;
  gifAutoplayEnabled: boolean;
  focusGlowEnabled: boolean;
}) {
  const [hovered, setHovered] = React.useState(false);
  const [gifError, setGifError] = React.useState(false);
  const wrapperRef = React.useRef<HTMLDivElement>(null);
  const inViewport = useInViewport(wrapperRef, '150px');

  const shape = (((folder as unknown as Record<string, unknown>).reason as string | undefined) ?? 'poster').toLowerCase();
  const isWide = shape === 'wide' || shape === 'landscape';
  const isSquare = shape === 'square';
  const w = cardWidth(folder);

  const imgStyle: React.CSSProperties = isWide
    ? { width: w, minWidth: w, height: '9.875rem' }
    : isSquare
      ? { width: w, minWidth: w, height: '9.375rem' }
      : { width: w, minWidth: w, height: '14.625rem' };

  const staticUrl =
    cardImageUrl(folder.poster, { displayWidth: w, dpr: window.devicePixelRatio }) ??
    cardImageUrl(folder.background, { kind: 'backdrop', displayWidth: w, dpr: window.devicePixelRatio });
  const { src: staticSrc } = usePosterSrc(staticUrl);
  const hasStatic = !!staticSrc;
  const gifEligible =
    !!folder.focusGifUrl && folder.focusGifEnabled !== false && inViewport && (gifAutoplayEnabled || hovered) && !gifError;
  const hasGifSlot = useGifSlot(folder.id, gifEligible);
  const wantsMotion = gifEligible && hasGifSlot;
  const showPlaceholder = !hasStatic && !wantsMotion;
  const glowColor = useDominantColor(staticSrc ?? null, focusGlowEnabled && hovered);

  React.useEffect(() => {
    setGifError(false);
  }, [folder.id, folder.focusGifUrl]);

  return (
    <div
      ref={wrapperRef}
      role="button"
      tabIndex={0}
      className="folder-tile"
      style={{ ...collStyles.tileWrapper, width: imgStyle.width, minWidth: imgStyle.minWidth }}
      onClick={() => onClick(folder)}
      onKeyDown={(e) => {
        if (e.key === 'Enter' || e.key === ' ') {
          e.preventDefault();
          onClick(folder);
        }
      }}
      onPointerEnter={() => setHovered(true)}
      onPointerLeave={() => setHovered(false)}
    >
      <div
        className="folder-card"
        data-motion-url={folder.focusGifUrl ?? undefined}
        style={{
          ...collStyles.card,
          ...imgStyle,
          boxShadow: glowColor ? `0 0 1.5rem 0.125rem ${glowColor}` : undefined,
          transition: 'box-shadow 0.2s ease',
        }}
      >
        {hasStatic && <img src={staticSrc} alt={folder.name} loading="lazy" decoding="async" draggable={false} style={collStyles.img} />}
        {wantsMotion && (
          <img
            src={folder.focusGifUrl}
            alt=""
            loading="lazy"
            draggable={false}
            style={{ ...collStyles.img, position: 'absolute', inset: 0 }}
            onError={() => setGifError(true)}
          />
        )}
        {showPlaceholder && (
          <div style={collStyles.namePlaceholder}>
            {folder.coverEmoji ? (
              <span style={{ fontSize: '2.5rem' }}>{folder.coverEmoji}</span>
            ) : addonIcon ? (
              <img src={addonIcon} alt="" style={{ width: '48%', height: '48%', objectFit: 'contain', opacity: 0.35 }} />
            ) : (
              <span style={collStyles.namePlaceholderText}>{folder.name.slice(0, 1).toUpperCase()}</span>
            )}
          </div>
        )}
      </div>
      {!folder.hideTitle && <p style={collStyles.folderName}>{folder.name}</p>}
    </div>
  );
});

const headerStyles: Record<string, React.CSSProperties> = {
  header: {
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'space-between',
    paddingLeft: ROW_PADDING_LEFT,
    paddingRight: '2rem',
    marginBottom: '0.75rem',
  },
  title: {
    color: '#FFFFFF',
    fontSize: '1.125rem',
    fontWeight: 700,
    margin: 0,
    letterSpacing: '-0.01em',
  },
  arrowBtn: {
    display: 'inline-flex',
    alignItems: 'center',
    justifyContent: 'center',
    width: '1.75rem',
    height: '1.75rem',
    borderRadius: '62.4375rem',
    border: '1px solid rgba(255,255,255,0.10)',
    background: 'rgba(255,255,255,0.06)',
    color: 'rgba(255,255,255,0.76)',
    transition: 'opacity 0.15s',
    padding: 0,
  },
};

const collStyles: Record<string, React.CSSProperties> = {
  section: {
    position: 'relative',
    zIndex: 1,
    paddingTop: '0.5rem',
    marginBottom: '0.25rem',
    contain: 'layout style',
  },
  scroll: {
    display: 'flex',
    gap: '0.75rem',
    overflowX: 'auto',
    paddingLeft: ROW_PADDING_LEFT,
    paddingRight: '2.5rem',
    paddingBottom: '1rem',
    paddingTop: '0.25rem',
    scrollbarWidth: 'none',
    willChange: 'transform',
  },
  tileWrapper: {
    display: 'flex',
    flexDirection: 'column',
    gap: '0.375rem',
    cursor: 'pointer',
    outline: 'none',
    flexShrink: 0,
  },
  card: {
    position: 'relative',
    borderRadius: '0.5rem',
    overflow: 'hidden',
    background: '#141922',
  },
  img: {
    width: '100%',
    height: '100%',
    objectFit: 'cover',
    display: 'block',
  },
  namePlaceholder: {
    width: '100%',
    height: '100%',
    background: '#1B212B',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
  },
  namePlaceholderText: {
    color: 'rgba(255,255,255,0.22)',
    fontSize: '3rem',
    fontWeight: 900,
  },
  folderName: {
    color: '#FFFFFF',
    fontSize: '0.8125rem',
    fontWeight: 700,
    margin: 0,
    overflow: 'hidden',
    textOverflow: 'ellipsis',
    whiteSpace: 'nowrap',
  },
};
