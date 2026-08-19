import React, { useCallback, useEffect, useRef, useState } from 'react';
import { ChevronLeft, ChevronRight, Clock, X } from 'lucide-react';
import { MovieCard } from '../components/MovieCard';
import type { PosterPrefs } from '../core/posterPrefs';
import type { Meta } from '../core/types';
import { t } from '../i18n';
import { styles } from './searchStyles';

export function LoadingShelves() {
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

export function SearchCategoryRow({
  title,
  items,
  onItemClick,
  onDispatch,
  posterPrefs,
}: {
  title: string;
  items: Meta[];
  onItemClick: (meta: Meta) => void;
  onDispatch: (actionJson: string) => void;
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
    <div style={styles.category} onMouseEnter={() => setHovered(true)} onMouseLeave={() => setHovered(false)}>
      <p style={styles.categoryTitle}>{title}</p>
      <div style={{ position: 'relative' }}>
        {hovered && canScrollLeft && (
          <SearchScrollArrow direction="left" onClick={() => scrollRef.current?.scrollBy({ left: -520, behavior: 'smooth' })} />
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
              onDispatch={onDispatch}
            />
          ))}
        </div>
        {hovered && canScrollRight && (
          <SearchScrollArrow direction="right" onClick={() => scrollRef.current?.scrollBy({ left: 520, behavior: 'smooth' })} />
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

export function RecentSearchChip({ value, onClick, onRemove }: { value: string; onClick: () => void; onRemove: () => void }) {
  const [hovered, setHovered] = useState(false);
  return (
    <div
      style={{
        ...styles.recentChip,
        background: hovered ? 'rgba(255,255,255,0.09)' : 'rgba(255,255,255,0.045)',
        borderColor: hovered ? 'rgba(255,255,255,0.18)' : 'rgba(255,255,255,0.08)',
      }}
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
    >
      <button style={styles.recentChipMain} onClick={onClick}>
        <Clock size={15} color="rgba(255,255,255,0.45)" />
        <span style={styles.recentChipText}>{value}</span>
      </button>
      <button
        title={t('common.remove')}
        style={styles.recentChipRemove}
        onClick={(e) => {
          e.stopPropagation();
          onRemove();
        }}
      >
        <X size={14} />
      </button>
    </div>
  );
}

export function formatCatalogTitle(name: string, type: string): string {
  let label: string;
  if (type === 'movie') label = t('auto.movies');
  else if (type === 'series') label = t('auto.series');
  else if (type) label = type.charAt(0).toUpperCase() + type.slice(1);
  else return name;
  return `${name} - ${label}`;
}

export function TypeChip({ label, selected, onClick }: { label: string; selected: boolean; onClick: () => void }) {
  const [hovered, setHovered] = useState(false);
  return (
    <button
      style={{
        height: '2.125rem',
        padding: '0 1rem',
        borderRadius: '62.4375rem',
        border: selected ? 'none' : '1px solid rgba(255,255,255,0.12)',
        background: selected ? '#FFFFFF' : hovered ? 'rgba(255,255,255,0.08)' : 'transparent',
        color: selected ? '#000000' : '#FFFFFF',
        fontSize: '0.8125rem',
        fontWeight: 700,
        cursor: 'pointer',
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

export function GenreCard({ genre, onClick }: { genre: string; onClick: () => void }) {
  const [hovered, setHovered] = useState(false);
  return (
    <button
      style={{
        height: '4rem',
        borderRadius: '0.75rem',
        border: '1px solid rgba(255,255,255,0.08)',
        background: hovered ? 'rgba(255,255,255,0.1)' : 'rgba(255,255,255,0.04)',
        color: '#FFFFFF',
        fontSize: '0.875rem',
        fontWeight: 700,
        cursor: 'pointer',
        transition: 'background 0.15s',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        boxShadow: hovered ? '0 0 0 0.0938rem rgba(255,255,255,0.3)' : 'none',
      }}
      onClick={onClick}
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
    >
      {genre}
    </button>
  );
}
