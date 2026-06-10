import React from 'react';
import type { AppState, Meta } from '../core/types';
import { markContinueWatchingItemWatched, dropContinueWatchingItem } from '../core/continueWatchingUtils';
import { ContinueCard } from './ContinueCard';
import { t } from '../i18n';

const ROW_PADDING_LEFT = 32;

export function ContinueWatchingRow({
  items,
  state,
  onItemClick,
  onDispatch,
}: {
  items: Meta[];
  state: AppState;
  onItemClick: (m: Meta) => void;
  onDispatch: (actionJson: string) => void | Promise<void>;
}) {
  const prefs = (state.settings?.values ?? {}) as Record<string, unknown>;
  const layout = String(prefs.resolvedContinueWatchingLayout ?? prefs.continueWatchingLayout ?? 'horizontal');
  const artworkPreference = String(prefs.continueWatchingArtwork ?? 'episode');
  const isHorizontal = layout !== 'vertical';
  const scrollRef = React.useRef<HTMLDivElement>(null);
  const [canScrollLeft, setCanScrollLeft] = React.useState(false);
  const [canScrollRight, setCanScrollRight] = React.useState(true);
  const [dismissedIds, setDismissedIds] = React.useState<Set<string>>(new Set());

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
  }, [updateArrows, items.length]);

  const scroll = (dir: 'left' | 'right') => {
    scrollRef.current?.scrollBy({ left: dir === 'right' ? 660 : -660, behavior: 'smooth' });
  };

  return (
    <div style={cwStyles.section}>
      <div style={cwStyles.header}>
        <p style={cwStyles.title}>{t('auto.continue_watching')}</p>
        <div style={{ display: 'flex', alignItems: 'center', gap: 6 }}>
          <button
            style={{ ...cwStyles.arrowBtn, opacity: canScrollLeft ? 1 : 0.28, cursor: canScrollLeft ? 'pointer' : 'default' }}
            onClick={() => canScrollLeft && scroll('left')}
            aria-label="Scroll left"
          >
            <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor"><path d="M15 18l-6-6 6-6v12z" /></svg>
          </button>
          <button
            style={{ ...cwStyles.arrowBtn, opacity: canScrollRight ? 1 : 0.28, cursor: canScrollRight ? 'pointer' : 'default' }}
            onClick={() => canScrollRight && scroll('right')}
            aria-label="Scroll right"
          >
            <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor"><path d="M9 18l6-6-6-6v12z" /></svg>
          </button>
        </div>
      </div>
      <div ref={scrollRef} style={cwStyles.scroll}>
        {items.filter((meta) => !dismissedIds.has(meta.id)).map((meta) => (
          <ContinueCard
            key={meta.id}
            meta={meta}
            isHorizontal={isHorizontal}
            artworkPreference={artworkPreference}
            onClick={onItemClick}
            onMarkWatched={(item) => {
              setDismissedIds((prev) => new Set([...prev, item.id]));
              void markContinueWatchingItemWatched(item, onDispatch);
            }}
            onDrop={(item) => {
              setDismissedIds((prev) => new Set([...prev, item.id]));
              void dropContinueWatchingItem(item, onDispatch);
            }}
          />
        ))}
      </div>
    </div>
  );
}

const cwStyles: Record<string, React.CSSProperties> = {
  section: { position: 'relative', zIndex: 1, paddingTop: 8, marginBottom: 0 },
  header: { display: 'flex', alignItems: 'center', justifyContent: 'space-between', paddingLeft: ROW_PADDING_LEFT, paddingRight: 32, marginBottom: 12 },
  title: { color: '#FFFFFF', fontSize: 18, fontWeight: 700, margin: 0, letterSpacing: '-0.01em' },
  arrowBtn: { display: 'inline-flex', alignItems: 'center', justifyContent: 'center', width: 28, height: 28, borderRadius: 999, border: '1px solid rgba(255,255,255,0.10)', background: 'rgba(255,255,255,0.06)', color: 'rgba(255,255,255,0.76)', transition: 'opacity 0.15s', padding: 0 },
  scroll: { display: 'flex', gap: 18, overflowX: 'auto', paddingLeft: ROW_PADDING_LEFT, paddingRight: 40, paddingBottom: 16, paddingTop: 4, scrollbarWidth: 'none' },
};
