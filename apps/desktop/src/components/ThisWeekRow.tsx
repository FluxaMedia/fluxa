import React from 'react';
import type { Meta } from '../core/types';
import { continueWatchingCardFields } from '../core/continueWatchingUtils';
import { ContinueCard } from './ContinueCard';
import { t } from '../i18n';
import { useDragScroll } from '../hooks/useDragScroll';

const ROW_PADDING_LEFT = '2rem';

export const ThisWeekRow = React.memo(function ThisWeekRow({
  items,
  artworkPreference,
  onItemClick,
}: {
  items: Meta[];
  artworkPreference: string;
  onItemClick: (m: Meta) => void;
}) {
  const scrollRef = React.useRef<HTMLDivElement>(null);
  const dragScroll = useDragScroll(scrollRef);
  const [canScrollLeft, setCanScrollLeft] = React.useState(false);
  const [canScrollRight, setCanScrollRight] = React.useState(false);
  const [cardFields, setCardFields] = React.useState<Map<string, { artwork: string | null; episodeLine: string }>>(new Map());

  React.useEffect(() => {
    let cancelled = false;
    void continueWatchingCardFields(items, artworkPreference, true).then((fields) => {
      if (!cancelled) setCardFields(fields);
    });
    return () => {
      cancelled = true;
    };
  }, [items, artworkPreference]);

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

  const noop = React.useCallback(() => {}, []);

  return (
    <div className="this-week-section" style={twStyles.section}>
      <div className="this-week-header" style={twStyles.header}>
        <p style={twStyles.title}>{t('auto.this_week')}</p>
        <div style={{ display: 'flex', alignItems: 'center', gap: '0.375rem' }}>
          {canScrollLeft && (
            <button style={twStyles.arrowBtn} onClick={() => scroll('left')} aria-label={t('common.scroll_left')}>
              <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                <path d="M15 18l-6-6 6-6v12z" />
              </svg>
            </button>
          )}
          {canScrollRight && (
            <button style={twStyles.arrowBtn} onClick={() => scroll('right')} aria-label={t('common.scroll_right')}>
              <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                <path d="M9 18l6-6-6-6v12z" />
              </svg>
            </button>
          )}
        </div>
      </div>
      <div ref={scrollRef} className="this-week-scroll" style={twStyles.scroll} {...dragScroll}>
        {items.map((meta) => (
          <ContinueCard
            key={meta.id}
            meta={meta}
            isHorizontal
            artwork={cardFields.get(meta.id)?.artwork ?? null}
            episodeLine={cardFields.get(meta.id)?.episodeLine ?? null}
            remainingFormat="time"
            progressDirection="remaining"
            dismissing={false}
            hideActions
            onClick={onItemClick}
            onGoToDetails={onItemClick}
            onStartOver={noop}
            onPlayManually={noop}
            onMarkWatched={noop}
            onDrop={noop}
            onDismissAnimationEnd={noop}
          />
        ))}
      </div>
    </div>
  );
});

const twStyles: Record<string, React.CSSProperties> = {
  section: { position: 'relative', zIndex: 1, paddingTop: '0.5rem', marginBottom: 0 },
  header: {
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'space-between',
    paddingLeft: ROW_PADDING_LEFT,
    paddingRight: '2rem',
    marginBottom: '0.75rem',
  },
  title: { color: 'var(--fluxa-text-primary)', fontSize: '1.125rem', fontWeight: 700, margin: 0, letterSpacing: '-0.01em' },
  arrowBtn: {
    display: 'inline-flex',
    alignItems: 'center',
    justifyContent: 'center',
    width: '1.75rem',
    height: '1.75rem',
    borderRadius: '62.4375rem',
    border: '1px solid var(--fluxa-border)',
    background: 'var(--fluxa-fill)',
    color: 'var(--fluxa-text-secondary)',
    transition: 'opacity 0.15s',
    padding: 0,
  },
  scroll: {
    display: 'flex',
    gap: '1.125rem',
    overflowX: 'auto',
    paddingLeft: ROW_PADDING_LEFT,
    paddingRight: '2.5rem',
    paddingBottom: '1rem',
    paddingTop: '0.25rem',
    scrollbarWidth: 'none',
  },
};
