import React from 'react';
import type { Meta } from '../core/types';

const ROW_PADDING_LEFT = 32;

export function CollectionShelfRow({
  title,
  folders,
  onFolderClick,
}: {
  title: string;
  folders: Meta[];
  onFolderClick: (f: Meta) => void;
}) {
  const scrollRef = React.useRef<HTMLDivElement>(null);
  const [canScrollLeft, setCanScrollLeft] = React.useState(false);
  const [canScrollRight, setCanScrollRight] = React.useState(true);
  const [hoveredId, setHoveredId] = React.useState<string | null>(null);
  const hoveredIdRef = React.useRef<string | null>(null);

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
    return () => el.removeEventListener('scroll', updateArrows);
  }, [updateArrows, folders.length]);

  const handleScroll = React.useCallback(() => {
    updateArrows();
    hoveredIdRef.current = null;
    setHoveredId(null);
  }, [updateArrows]);

  const handleTileHover = React.useCallback((fid: string | null) => {
    if (hoveredIdRef.current === fid) return;
    hoveredIdRef.current = fid;
    setHoveredId(fid);
  }, []);

  const scroll = (dir: 'left' | 'right') => {
    scrollRef.current?.scrollBy({ left: dir === 'right' ? 660 : -660, behavior: 'smooth' });
  };

  return (
    <div style={collStyles.section}>
      <div style={headerStyles.header}>
        <p style={headerStyles.title}>{title}</p>
        <div style={{ display: 'flex', alignItems: 'center', gap: 6 }}>
          <button
            style={{ ...headerStyles.arrowBtn, opacity: canScrollLeft ? 1 : 0.28, cursor: canScrollLeft ? 'pointer' : 'default' }}
            onClick={() => canScrollLeft && scroll('left')}
            aria-label="Scroll left"
          >
            <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor"><path d="M15 18l-6-6 6-6v12z"/></svg>
          </button>
          <button
            style={{ ...headerStyles.arrowBtn, opacity: canScrollRight ? 1 : 0.28, cursor: canScrollRight ? 'pointer' : 'default' }}
            onClick={() => canScrollRight && scroll('right')}
            aria-label="Scroll right"
          >
            <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor"><path d="M9 18l6-6-6-6v12z"/></svg>
          </button>
        </div>
      </div>
      <div
        ref={scrollRef}
        style={collStyles.scroll}
        onScroll={handleScroll}
      >
        {folders.map((folder) => (
          <FolderTileCard
            key={folder.id}
            folder={folder}
            isHovered={hoveredId === folder.id}
            scrollRoot={scrollRef}
            onClick={onFolderClick}
            onHoverChange={handleTileHover}
          />
        ))}
      </div>
    </div>
  );
}

function FolderTileCard({
  folder,
  isHovered,
  scrollRoot,
  onClick,
  onHoverChange,
}: {
  folder: Meta;
  isHovered: boolean;
  scrollRoot: React.RefObject<HTMLDivElement | null>;
  onClick: (f: Meta) => void;
  onHoverChange: (fid: string | null) => void;
}) {
  const [imgError, setImgError] = React.useState(false);
  const [isVisible, setIsVisible] = React.useState(false);
  const cardRef = React.useRef<HTMLDivElement>(null);

  const shape = ((folder as unknown as Record<string,unknown>).reason as string | undefined ?? 'poster').toLowerCase();
  const isWide = shape === 'wide' || shape === 'landscape';
  const isSquare = shape === 'square';

  const imgStyle: React.CSSProperties = isWide
    ? { width: 280, minWidth: 280, height: 158 }
    : isSquare
    ? { width: 150, minWidth: 150, height: 150 }
    : { width: 156, minWidth: 156, height: 234 };

  // IntersectionObserver: play GIF when card is in view, show static poster otherwise
  React.useEffect(() => {
    const el = cardRef.current;
    const root = scrollRoot.current;
    if (!el || !folder.focusGifUrl) return;
    const observer = new IntersectionObserver(
      (entries) => { setIsVisible(entries[0]?.isIntersecting ?? false); },
      { root, threshold: 0.5 },
    );
    observer.observe(el);
    return () => observer.disconnect();
  }, [folder.focusGifUrl, scrollRoot]);

  const staticImg = folder.poster ?? folder.background;
  const gifUrl = folder.focusGifUrl;
  const displayUrl = gifUrl && isVisible ? gifUrl : staticImg;

  return (
    <div
      ref={cardRef}
      role="button"
      tabIndex={0}
      style={{ ...collStyles.tileWrapper, width: imgStyle.width, minWidth: imgStyle.minWidth }}
      onClick={() => onClick(folder)}
      onKeyDown={(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); onClick(folder); } }}
      onPointerEnter={() => onHoverChange(folder.id)}
      onPointerLeave={() => onHoverChange(null)}
    >
      <div style={{
        ...collStyles.card,
        ...imgStyle,
        boxShadow: isHovered ? '0 0 0 2px var(--primary-accent-color, rgba(255,255,255,0.44))' : 'none',
        transform: isHovered ? 'translateY(-2px) scale(1.02)' : 'none',
      }}>
        {displayUrl && !imgError ? (
          <img
            key={displayUrl}
            src={displayUrl}
            alt={folder.name}
            loading="lazy"
            decoding="async"
            style={collStyles.img}
            onError={() => setImgError(true)}
          />
        ) : (
          <div style={collStyles.namePlaceholder}>
            <span style={collStyles.namePlaceholderText}>{folder.name.slice(0, 1).toUpperCase()}</span>
          </div>
        )}
      </div>
      <p style={collStyles.folderName}>{folder.name}</p>
    </div>
  );
}

const headerStyles: Record<string, React.CSSProperties> = {
  header: {
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'space-between',
    paddingLeft: ROW_PADDING_LEFT,
    paddingRight: 32,
    marginBottom: 12,
  },
  title: {
    color: '#FFFFFF',
    fontSize: 18,
    fontWeight: 700,
    margin: 0,
    letterSpacing: '-0.01em',
  },
  arrowBtn: {
    display: 'inline-flex',
    alignItems: 'center',
    justifyContent: 'center',
    width: 28,
    height: 28,
    borderRadius: 999,
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
    paddingTop: 8,
    marginBottom: 4,
  },
  scroll: {
    display: 'flex',
    gap: 12,
    overflowX: 'auto',
    paddingLeft: ROW_PADDING_LEFT,
    paddingRight: 40,
    paddingBottom: 16,
    paddingTop: 4,
    scrollbarWidth: 'none',
  },
  tileWrapper: {
    display: 'flex',
    flexDirection: 'column',
    gap: 6,
    cursor: 'pointer',
    outline: 'none',
    flexShrink: 0,
  },
  card: {
    position: 'relative',
    borderRadius: 8,
    overflow: 'hidden',
    background: '#141922',
    transition: 'transform 0.16s ease, box-shadow 0.16s ease',
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
    background: 'linear-gradient(135deg, #1e2535 0%, #141922 100%)',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
  },
  namePlaceholderText: {
    color: 'rgba(255,255,255,0.22)',
    fontSize: 48,
    fontWeight: 900,
  },
  folderName: {
    color: '#FFFFFF',
    fontSize: 13,
    fontWeight: 700,
    margin: 0,
    overflow: 'hidden',
    textOverflow: 'ellipsis',
    whiteSpace: 'nowrap',
  },
};
