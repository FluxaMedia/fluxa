import React from 'react';
import { X } from 'lucide-react';
import type { Meta } from '../core/types';
import { markContinueWatchingItemWatched, dropContinueWatchingItem, continueWatchingCardFields } from '../core/continueWatchingUtils';
import { ContinueCard } from './ContinueCard';
import { t } from '../i18n';
import { useDragScroll } from '../hooks/useDragScroll';
import { ConfirmDialog } from './ConfirmDialog';

const ROW_PADDING_LEFT = '2rem';

let lastCardFieldsKey: string | null = null;
let lastCardFields: Map<string, { artwork: string | null; episodeLine: string }> = new Map();

function externalSourceLabel(meta: Meta): string {
  const item = meta as unknown as Record<string, unknown>;
  const rawSource = typeof item.reason === 'string' ? item.reason : typeof item.source === 'string' ? item.source : '';
  if (rawSource === 'trakt') return t('brand.trakt');
  if (rawSource === 'simkl') return t('brand.simkl');
  return rawSource;
}

function liveEpisodeLine(meta: Meta): string | null {
  const item = meta as unknown as {
    lastEpisodeName?: string;
    lastEpisodeSeason?: number;
    lastEpisodeNumber?: number;
    lastVideoId?: string;
  };
  const matchedVideo = item.lastVideoId ? meta.videos?.find((v) => v.id === item.lastVideoId) : undefined;

  let season = matchedVideo?.season;
  let number = matchedVideo?.episode ?? matchedVideo?.number;
  let name = matchedVideo?.name ?? matchedVideo?.title;

  if ((season == null || number == null) && item.lastVideoId) {
    const parts = item.lastVideoId.split(':');
    if (parts.length >= 3) {
      const s = Number(parts[parts.length - 2]);
      const e = Number(parts[parts.length - 1]);
      if (s > 0 && e > 0) {
        if (season == null) season = s;
        if (number == null) number = e;
      }
    }
  }
  if (season == null) season = item.lastEpisodeSeason;
  if (number == null) number = item.lastEpisodeNumber;
  if (name == null) name = item.lastEpisodeName;

  if (season == null || number == null) return null;
  const code = `${t('format.season_abbrev')}${season}:${t('format.episode_abbrev')}${number}`;
  return name?.trim() ? `${code} ${name.trim()}` : code;
}

export const ContinueWatchingRow = React.memo(
  function ContinueWatchingRow({
    items,
    cwLayout,
    artworkPreference,
    remainingFormat,
    progressDirection,
    onItemClick,
    onNavigateDetail,
    onStartOver,
    onPlayManually,
    onDispatch,
  }: {
    items: Meta[];
    cwLayout: string;
    artworkPreference: string;
    remainingFormat: string;
    progressDirection: string;
    onItemClick: (m: Meta) => void;
    onNavigateDetail: (m: Meta) => void;
    onStartOver: (m: Meta) => void;
    onPlayManually: (m: Meta) => void;
    onDispatch: (actionJson: string) => void | Promise<void>;
  }) {
    const isHorizontal = cwLayout !== 'vertical';
    const scrollRef = React.useRef<HTMLDivElement>(null);
    const dragScroll = useDragScroll(scrollRef);
    const [dismissingIds, setDismissingIds] = React.useState<Set<string>>(new Set());
    const [dismissedIds, setDismissedIds] = React.useState<Set<string>>(new Set());
    const [pendingIds, setPendingIds] = React.useState<Set<string>>(new Set());
    const [aboutItem, setAboutItem] = React.useState<{ meta: Meta; artwork: string | null } | null>(null);
    const [dropTarget, setDropTarget] = React.useState<Meta | null>(null);
    const markWatchedVideoIds = React.useRef<Map<string, string | null>>(new Map());

    React.useEffect(() => {
      if (markWatchedVideoIds.current.size === 0) return;
      const toReveal: string[] = [];
      const toDismiss: string[] = [];
      for (const [id, prevId] of markWatchedVideoIds.current.entries()) {
        const cur = items.find((m) => m.id === id);
        const curId = (cur as unknown as { lastVideoId?: string | null } | undefined)?.lastVideoId ?? null;
        if (cur && curId !== prevId) {
          toReveal.push(id);
        } else {
          toDismiss.push(id);
        }
        markWatchedVideoIds.current.delete(id);
      }
      if (toReveal.length > 0) {
        setPendingIds((prev) => {
          const n = new Set(prev);
          for (const id of toReveal) n.delete(id);
          return n;
        });
        setDismissingIds((prev) => {
          const n = new Set(prev);
          for (const id of toReveal) n.delete(id);
          return n;
        });
      }
      if (toDismiss.length > 0) {
        setPendingIds((prev) => {
          const n = new Set(prev);
          for (const id of toDismiss) n.delete(id);
          return n;
        });
        setDismissedIds((prev) => {
          const n = new Set(prev);
          for (const id of toDismiss) n.add(id);
          return n;
        });
      }
    }, [items]);

    const visibleItems = React.useMemo(() => items.filter((meta) => !dismissedIds.has(meta.id)), [items, dismissedIds]);

    const [cardFields, setCardFields] = React.useState(lastCardFields);
    const cardFieldsKey = React.useMemo(
      () =>
        `${artworkPreference}|${isHorizontal}|${visibleItems
          .map((m) => {
            const item = m as unknown as {
              lastVideoId?: string;
              lastEpisodeName?: string;
              lastEpisodeSeason?: number;
              lastEpisodeNumber?: number;
              lastEpisodeThumbnail?: string;
            };
            return [
              m.id,
              m.poster ?? '',
              m.background ?? '',
              item.lastVideoId ?? '',
              item.lastEpisodeName ?? '',
              item.lastEpisodeSeason ?? '',
              item.lastEpisodeNumber ?? '',
              item.lastEpisodeThumbnail ?? '',
            ].join(':');
          })
          .join(',')}`,
      [artworkPreference, isHorizontal, visibleItems],
    );
    React.useEffect(() => {
      if (cardFieldsKey === lastCardFieldsKey) {
        setCardFields(lastCardFields);
        return;
      }
      let cancelled = false;
      const load = () => {
        void continueWatchingCardFields(visibleItems, artworkPreference, isHorizontal)
          .then((fields) => {
            if (cancelled) return;
            const resolvedFields = new Map(fields);
            for (const item of visibleItems) {
              const current = resolvedFields.get(item.id);
              const cached = lastCardFields.get(item.id);
              if (current && !current.artwork && cached?.artwork) {
                resolvedFields.set(item.id, { ...current, artwork: cached.artwork });
              }
            }
            lastCardFieldsKey = cardFieldsKey;
            lastCardFields = resolvedFields;
            setCardFields(resolvedFields);
          })
          .catch(() => {
            if (!cancelled) setTimeout(load, 1000);
          });
      };
      load();
      return () => {
        cancelled = true;
      };
    }, [cardFieldsKey, visibleItems, artworkPreference, isHorizontal]);

    const startDismiss = (item: Meta, action: () => void) => {
      setDismissingIds((prev) => {
        const n = new Set(prev);
        n.add(item.id);
        return n;
      });
      action();
    };

    return (
      <div style={cwStyles.section}>
        <div style={cwStyles.header}>
          <p style={cwStyles.title}>{t('auto.continue_watching')}</p>
        </div>
        <div ref={scrollRef} style={cwStyles.scroll} {...dragScroll}>
          {visibleItems.map((meta) => (
            <ContinueCard
              key={meta.id}
              meta={meta}
              isHorizontal={isHorizontal}
              artwork={cardFields.get(meta.id)?.artwork ?? null}
              episodeLine={cardFields.get(meta.id)?.episodeLine ?? liveEpisodeLine(meta)}
              remainingFormat={remainingFormat}
              progressDirection={progressDirection}
              dismissing={dismissingIds.has(meta.id)}
              pending={pendingIds.has(meta.id)}
              onClick={onItemClick}
              onGoToDetails={onNavigateDetail}
              onStartOver={onStartOver}
              onPlayManually={onPlayManually}
              onShowAbout={(item, artwork) => setAboutItem({ meta: item, artwork })}
              onMarkWatched={(item) => {
                markWatchedVideoIds.current.set(item.id, (item as unknown as { lastVideoId?: string | null }).lastVideoId ?? null);
                startDismiss(item, () => void markContinueWatchingItemWatched(item, onDispatch));
              }}
              onDrop={(item) => setDropTarget(item)}
              onDismissAnimationEnd={(item) => {
                setDismissingIds((prev) => {
                  const n = new Set(prev);
                  n.delete(item.id);
                  return n;
                });
                if (markWatchedVideoIds.current.has(item.id)) {
                  setPendingIds((prev) => {
                    const n = new Set(prev);
                    n.add(item.id);
                    return n;
                  });
                } else {
                  setDismissedIds((prev) => new Set([...prev, item.id]));
                }
              }}
            />
          ))}
        </div>
        {aboutItem && <ContinueWatchingAboutDialog meta={aboutItem.meta} artwork={aboutItem.artwork} onClose={() => setAboutItem(null)} />}
        {dropTarget && (
          <ConfirmDialog
            title={t('home.drop_continue_watching_confirm_title', dropTarget.name)}
            body={
              externalSourceLabel(dropTarget)
                ? t('home.drop_continue_watching_confirm_body_external', externalSourceLabel(dropTarget))
                : t('home.drop_continue_watching_confirm_body')
            }
            confirmLabel={t('home.drop_continue_watching')}
            cancelLabel={t('common.cancel')}
            destructive
            onCancel={() => setDropTarget(null)}
            onConfirm={() => {
              const item = dropTarget;
              setDropTarget(null);
              startDismiss(item, () => void dropContinueWatchingItem(item, onDispatch));
            }}
          />
        )}
      </div>
    );
  },
  (prev, next) => {
    if (
      prev.cwLayout !== next.cwLayout ||
      prev.artworkPreference !== next.artworkPreference ||
      prev.remainingFormat !== next.remainingFormat ||
      prev.progressDirection !== next.progressDirection ||
      prev.onItemClick !== next.onItemClick ||
      prev.onDispatch !== next.onDispatch ||
      prev.onNavigateDetail !== next.onNavigateDetail ||
      prev.onStartOver !== next.onStartOver ||
      prev.onPlayManually !== next.onPlayManually
    )
      return false;
    if (prev.items === next.items) return true;
    if (prev.items.length !== next.items.length) return false;
    return prev.items.every((item, i) => {
      const pi = item as unknown as Record<string, unknown>;
      const ni = next.items[i] as unknown as Record<string, unknown>;
      return (
        pi.id === ni.id &&
        pi.timeOffset === ni.timeOffset &&
        pi.duration === ni.duration &&
        pi.poster === ni.poster &&
        pi.background === ni.background &&
        pi.lastVideoId === ni.lastVideoId &&
        pi.lastEpisodeName === ni.lastEpisodeName &&
        pi.lastEpisodeSeason === ni.lastEpisodeSeason &&
        pi.lastEpisodeNumber === ni.lastEpisodeNumber &&
        pi.lastEpisodeThumbnail === ni.lastEpisodeThumbnail &&
        pi.continueWatchingBadge === ni.continueWatchingBadge &&
        pi.resumeProgressPercent === ni.resumeProgressPercent
      );
    });
  },
);

function ContinueWatchingAboutDialog({ meta, artwork, onClose }: { meta: Meta; artwork: string | null; onClose: () => void }) {
  const item = meta as unknown as Record<string, unknown>;
  const source = externalSourceLabel(meta) || t('home.continue_watching_about_source_unknown');
  const rows = [
    [t('home.continue_watching_about_id'), meta.id],
    [t('home.continue_watching_about_video_id'), item.lastVideoId],
    [t('home.continue_watching_about_season'), item.lastEpisodeSeason],
    [t('home.continue_watching_about_episode'), item.lastEpisodeNumber],
    [t('home.continue_watching_about_source'), source],
    [
      t('home.continue_watching_about_progress'),
      typeof item.timeOffset === 'number' && typeof item.duration === 'number' ? `${item.timeOffset} / ${item.duration}` : undefined,
    ],
    [t('home.continue_watching_about_artwork'), artwork],
    [t('home.continue_watching_about_poster'), meta.poster],
    [t('home.continue_watching_about_background'), meta.background],
  ].filter(([, value]) => value != null && value !== '') as Array<[string, string | number]>;

  React.useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key === 'Escape') onClose();
    };
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [onClose]);

  return (
    <div style={cwStyles.aboutBackdrop} role="presentation" onClick={onClose}>
      <div
        role="dialog"
        aria-modal="true"
        aria-label={t('home.continue_watching_about_title')}
        style={cwStyles.aboutDialog}
        onClick={(event) => event.stopPropagation()}
      >
        <div style={cwStyles.aboutHeader}>
          <div style={{ minWidth: 0 }}>
            <p style={cwStyles.aboutTitle}>{t('home.continue_watching_about_title')}</p>
            <p style={cwStyles.aboutName}>{meta.name}</p>
          </div>
          <button type="button" style={cwStyles.aboutClose} onClick={onClose} aria-label={t('common.close')}>
            <X size={16} />
          </button>
        </div>
        <div style={cwStyles.aboutRows}>
          {rows.map(([label, value]) => (
            <div key={label} style={cwStyles.aboutRow}>
              <span style={cwStyles.aboutLabel}>{label}</span>
              <span style={cwStyles.aboutValue}>{String(value)}</span>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}

const cwStyles: Record<string, React.CSSProperties> = {
  section: { position: 'relative', zIndex: 1, paddingTop: '0.5rem', marginBottom: 0 },
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
    textShadow: '0 0.0625rem 0.375rem rgba(0,0,0,0.9)',
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
  aboutBackdrop: {
    position: 'fixed',
    inset: 0,
    zIndex: 10000,
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    padding: '1.5rem',
    background: 'rgba(0,0,0,0.68)',
    backdropFilter: 'blur(0.25rem)',
  },
  aboutDialog: {
    width: 'min(42rem, 100%)',
    maxHeight: 'min(38rem, calc(100vh - 3rem))',
    overflowY: 'auto',
    borderRadius: '0.75rem',
    border: '1px solid rgba(255,255,255,0.14)',
    background: '#16181e',
    boxShadow: '0 1.5rem 5rem rgba(0,0,0,0.5)',
    padding: '1.25rem',
  },
  aboutHeader: { display: 'flex', alignItems: 'flex-start', justifyContent: 'space-between', gap: '1rem', marginBottom: '1rem' },
  aboutTitle: { margin: 0, color: '#fff', fontSize: '1rem', fontWeight: 700 },
  aboutName: { margin: '0.25rem 0 0', color: 'rgba(255,255,255,0.58)', fontSize: '0.8125rem', overflowWrap: 'anywhere' },
  aboutClose: {
    flex: '0 0 auto',
    width: '1.875rem',
    height: '1.875rem',
    border: 'none',
    borderRadius: '0.375rem',
    background: 'rgba(255,255,255,0.09)',
    color: '#fff',
    fontSize: '1.25rem',
    lineHeight: 1,
    cursor: 'pointer',
  },
  aboutRows: { display: 'grid', gap: '0.5rem' },
  aboutRow: {
    display: 'grid',
    gridTemplateColumns: '9.5rem minmax(0, 1fr)',
    gap: '0.75rem',
    padding: '0.625rem 0.75rem',
    borderRadius: '0.375rem',
    background: 'rgba(255,255,255,0.045)',
  },
  aboutLabel: { color: 'rgba(255,255,255,0.52)', fontSize: '0.75rem', fontWeight: 600 },
  aboutValue: {
    color: 'rgba(255,255,255,0.9)',
    fontSize: '0.75rem',
    fontFamily: 'ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace',
    overflowWrap: 'anywhere',
  },
};
