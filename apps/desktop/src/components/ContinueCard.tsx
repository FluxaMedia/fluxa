import React from 'react';
import { Check, Info, Play, RotateCcw, Trash2 } from 'lucide-react';
import type { LibraryItem, Meta } from '../core/types';
import { formatAirDay, formatReleaseCountdown, formatRemaining, formatWatched } from '../core/continueWatchingUtils';
import { ContextMenu } from './ui/ContextMenu';
import { usePosterSrc } from '../hooks/usePosterSrc';
import { t } from '../i18n';

function resolveBadge(
  badge: string | undefined,
  type: string,
  isUpNext: boolean,
  remainingText: string | null,
  scheduledText: string | null,
): { text: string; variant: 'new' | 'default' } | null {
  if (badge === 'newEpisode' && type === 'series') {
    return { text: t('auto.new_episode'), variant: 'new' };
  }
  if (scheduledText && type === 'series') {
    return { text: scheduledText, variant: 'default' };
  }
  if (isUpNext && type === 'series') {
    return { text: t('auto.up_next'), variant: 'default' };
  }
  if (remainingText) return { text: remainingText, variant: 'default' };
  return null;
}

export function ContinueCard({
  meta,
  isHorizontal,
  artwork: artworkProp,
  episodeLine,
  remainingFormat,
  progressDirection,
  dismissing,
  pending,
  hideActions,
  onClick,
  onGoToDetails,
  onStartOver,
  onPlayManually,
  onShowAbout,
  onMarkWatched,
  onDrop,
  onDismissAnimationEnd,
}: {
  meta: Meta;
  isHorizontal: boolean;
  artwork: string | null;
  episodeLine: string | null;
  remainingFormat: string;
  progressDirection: string;
  dismissing: boolean;
  pending?: boolean;
  hideActions?: boolean;
  onClick: (m: Meta) => void;
  onGoToDetails: (m: Meta) => void;
  onStartOver: (m: Meta) => void;
  onPlayManually: (m: Meta) => void;
  onShowAbout?: (m: Meta, artwork: string | null) => void;
  onMarkWatched: (m: Meta) => void;
  onDrop: (m: Meta) => void;
  onDismissAnimationEnd: (m: Meta) => void;
}) {
  const [hovered, setHovered] = React.useState(false);
  const [menuPoint, setMenuPoint] = React.useState<{ x: number; y: number } | null>(null);
  const [imgError, setImgError] = React.useState(false);
  const [imgLoaded, setImgLoaded] = React.useState(false);
  const [artworkOverride, setArtworkOverride] = React.useState<string | null>(null);
  const artworkImgRef = React.useRef<HTMLImageElement | null>(null);
  const artwork = artworkOverride ?? artworkProp;
  const { src: artworkSrc, failed: posterFailed } = usePosterSrc(artwork);

  const lib = meta as unknown as LibraryItem & {
    lastEpisodeName?: string;
    lastEpisodeSeason?: number;
    lastEpisodeNumber?: number;
    lastEpisodeThumbnail?: string;
    continueWatchingPoster?: string;
    continueWatchingBackground?: string;
    continueWatchingBadge?: string;
    newEpisodeReleasedAt?: string;
    unwatchedAhead?: number;
    resumeProgressPercent?: number;
  };

  React.useEffect(() => {
    setArtworkOverride(null);
  }, [artworkProp]);

  React.useEffect(() => {
    setImgError(false);
    setImgLoaded(false);
  }, [artwork]);

  const fallbackToSeriesArt = React.useCallback(() => {
    const m = meta as unknown as {
      poster?: string | null;
      background?: string | null;
    };
    const fallback = m.poster || m.background || null;
    if (fallback && fallback !== artwork) setArtworkOverride(fallback);
    else setImgError(true);
  }, [artwork, meta]);

  React.useEffect(() => {
    if (posterFailed) fallbackToSeriesArt();
  }, [posterFailed, fallbackToSeriesArt]);

  React.useEffect(() => {
    const img = artworkImgRef.current;
    if (img?.complete && img.naturalWidth > 0) setImgLoaded(true);
  }, [artworkSrc]);

  const timeOffset = typeof lib.timeOffset === 'number' && Number.isFinite(lib.timeOffset) ? lib.timeOffset : null;
  const duration = typeof lib.duration === 'number' && Number.isFinite(lib.duration) && lib.duration > 0 ? lib.duration : null;
  const hasProgress = timeOffset !== null && duration !== null;
  const resumePercent =
    typeof lib.resumeProgressPercent === 'number' && Number.isFinite(lib.resumeProgressPercent) ? lib.resumeProgressPercent / 100 : null;
  const percentOnly = !hasProgress && resumePercent !== null;
  const progress = hasProgress ? timeOffset / duration : resumePercent;
  const isUpNext =
    meta.type === 'series' && (lib.continueWatchingBadge === 'upNext' || (progress !== null && (progress <= 0 || progress >= 0.995)));
  const remainingText =
    !isUpNext && progress !== null
      ? progressDirection === 'watched'
        ? remainingFormat === 'percent' || percentOnly
          ? t('format.watched_percent', Math.round(progress * 100))
          : formatWatched(timeOffset ?? 0)
        : remainingFormat === 'percent' || percentOnly
          ? t('format.remaining_percent', Math.round((1 - progress) * 100))
          : formatRemaining(timeOffset ?? 0, duration ?? 0)
      : null;
  const scheduledText = lib.continueWatchingBadge === 'scheduledEpisode' ? formatReleaseCountdown(lib.newEpisodeReleasedAt) : null;
  const badge = resolveBadge(lib.continueWatchingBadge, meta.type, isUpNext, remainingText, scheduledText);
  const isScheduled = lib.continueWatchingBadge === 'scheduledEpisode';
  const cornerText = isScheduled
    ? formatAirDay(lib.newEpisodeReleasedAt)
    : lib.unwatchedAhead && lib.unwatchedAhead > 0
      ? `+${lib.unwatchedAhead}`
      : null;
  return (
    <div
      role="button"
      tabIndex={0}
      style={{
        ...(isHorizontal ? cwStyles.landscapeCard : cwStyles.posterCard),
        opacity: dismissing || pending ? 0 : 1,
        transform: dismissing ? 'translateX(-0.75rem)' : hovered ? 'translateY(-0.125rem)' : 'translateY(0)',
        transition: dismissing ? 'opacity 0.22s ease, transform 0.22s ease' : 'opacity 0.22s ease, transform 0.16s ease',
        boxShadow: hovered && !dismissing && !pending ? '0 0 0 0.125rem rgba(255,255,255,0.44)' : 'none',
        pointerEvents: dismissing || pending ? 'none' : undefined,
      }}
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
      onFocus={() => setHovered(true)}
      onBlur={() => setHovered(false)}
      onClick={() => !dismissing && !pending && onClick(meta)}
      onKeyDown={(e) => {
        if (e.key === 'Enter' || e.key === ' ') {
          e.preventDefault();
          if (!dismissing && !pending) onClick(meta);
        }
      }}
      onContextMenu={(e) => {
        if (dismissing || pending || hideActions) return;
        e.preventDefault();
        setMenuPoint({ x: e.clientX, y: e.clientY });
      }}
      onTransitionEnd={(e) => {
        if (dismissing && e.propertyName === 'opacity') {
          onDismissAnimationEnd(meta);
        }
      }}
    >
      <div style={isHorizontal ? cwStyles.landscapeImageArea : cwStyles.posterImageArea}>
        {artwork && artworkSrc && !imgError ? (
          <img
            ref={artworkImgRef}
            src={artworkSrc}
            alt={meta.name}
            loading="lazy"
            decoding="async"
            draggable={false}
            style={{
              ...cwStyles.artwork,
              opacity: imgLoaded ? 1 : 0,
              transition: 'opacity 0.22s ease',
            }}
            onLoad={() => setImgLoaded(true)}
          />
        ) : (
          <div style={cwStyles.thumbPlaceholder}>
            <span style={cwStyles.placeholderText}>{meta.name.slice(0, 1).toUpperCase()}</span>
          </div>
        )}
        {isHorizontal && <div style={cwStyles.landscapeShade} />}
        {!isUpNext && progress !== null && progress > 0 && (
          <div style={isHorizontal ? cwStyles.landscapeProgressBg : cwStyles.imageProgressBg}>
            <div
              style={{
                ...cwStyles.progressBar,
                width: `${Math.min(progress, 1) * 100}%`,
              }}
            />
          </div>
        )}
        {badge && (
          <div style={badge.variant === 'new' ? { ...cwStyles.remainingBadge, ...cwStyles.newEpisodeBadge } : cwStyles.remainingBadge}>
            {badge.text}
          </div>
        )}
        {cornerText && <div style={cwStyles.cornerBadge}>{cornerText}</div>}

        <div
          style={{
            position: 'absolute',
            inset: 0,
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            pointerEvents: 'none',
            opacity: hovered && !dismissing && !pending ? 1 : 0,
            transition: 'opacity 0.16s ease',
          }}
        >
          <div
            style={{
              width: '3.5rem',
              height: '3.5rem',
              borderRadius: '50%',
              background: 'var(--fluxa-fill-active)',
              backdropFilter: 'blur(0.375rem)',
              border: '1px solid var(--fluxa-border-strong)',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
            }}
          >
            <svg width="26" height="26" viewBox="0 0 24 24" fill="var(--fluxa-text-primary)" style={{ marginLeft: '0.1875rem' }}>
              <path d="M8 5v14l11-7z" />
            </svg>
          </div>
        </div>
      </div>

      <div style={isHorizontal ? { ...cwStyles.footer, ...cwStyles.landscapeFooter } : cwStyles.footer}>
        <div style={cwStyles.metaStack}>
          <p style={cwStyles.name}>{meta.name}</p>
          <p style={cwStyles.episodeName}>{episodeLine ?? (meta.type === 'series' ? t('auto.up_next') : '')}</p>
        </div>
      </div>
      <ContextMenu
        point={menuPoint}
        onClose={() => setMenuPoint(null)}
        items={[
          {
            icon: <Info size={15} />,
            label: t('home.view_details'),
            onSelect: () => onGoToDetails(meta),
          },
          ...(onShowAbout
            ? [
                {
                  icon: <Info size={15} />,
                  label: t('home.continue_watching_about'),
                  onSelect: () => onShowAbout(meta, artwork),
                },
              ]
            : []),
          {
            icon: <Play size={15} />,
            label: t('home.play_manually'),
            onSelect: () => onPlayManually(meta),
          },
          {
            icon: <RotateCcw size={15} />,
            label: t('detail.resume_dialog_start_over'),
            onSelect: () => onStartOver(meta),
          },
          {
            icon: <Check size={15} />,
            label: t('detail.mark_watched'),
            onSelect: () => onMarkWatched(meta),
          },
          {
            icon: <Trash2 size={15} />,
            label: t('common.remove'),
            onSelect: () => onDrop(meta),
            danger: true,
          },
        ]}
      />
    </div>
  );
}

const cwStyles: Record<string, React.CSSProperties> = {
  landscapeCard: {
    position: 'relative',
    width: '19.875rem',
    minWidth: '19.875rem',
    height: '11.25rem',
    borderRadius: '0.5rem',
    overflow: 'hidden',
    background: 'var(--fluxa-surface)',
    cursor: 'pointer',
    outline: 'none',
  },
  posterCard: {
    position: 'relative',
    width: '8rem',
    minWidth: '8rem',
    height: '13.625rem',
    borderRadius: '0.1875rem',
    overflow: 'hidden',
    background: 'var(--fluxa-surface-raised)',
    cursor: 'pointer',
    outline: 'none',
  },
  landscapeImageArea: {
    position: 'relative',
    width: '100%',
    height: '100%',
    overflow: 'hidden',
    background: 'var(--fluxa-surface-raised)',
  },
  posterImageArea: {
    position: 'relative',
    width: '100%',
    height: '10.0625rem',
    overflow: 'hidden',
    background: 'var(--fluxa-surface-raised)',
  },
  artwork: {
    width: '100%',
    height: '100%',
    objectFit: 'cover',
    display: 'block',
  },
  thumbPlaceholder: {
    width: '100%',
    height: '100%',
    background: 'var(--fluxa-surface-raised)',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
  },
  placeholderText: {
    color: 'var(--fluxa-text-faint)',
    fontSize: '3rem',
    fontWeight: 900,
  },
  footer: {
    position: 'relative',
    height: '3.5625rem',
    padding: '0.5625rem 0.625rem 0.625rem',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'space-between',
    gap: '0.5rem',
    background: 'var(--fluxa-background)',
  },
  landscapeFooter: {
    position: 'absolute',
    zIndex: 1,
    right: 0,
    bottom: 0,
    left: 0,
    height: '4.25rem',
    padding: '1.85rem 0.75rem 1rem',
    alignItems: 'flex-end',
    background: 'linear-gradient(transparent, var(--fluxa-scrim))',
  },
  metaStack: { flex: 1, minWidth: 0 },
  imageProgressBg: {
    position: 'absolute',
    left: '0.625rem',
    right: '0.625rem',
    bottom: '0.5rem',
    height: '0.25rem',
    borderRadius: '62.4375rem',
    background: 'var(--fluxa-fill-active)',
  },
  landscapeShade: {
    position: 'absolute',
    inset: 0,
    background: 'linear-gradient(180deg, transparent 34%, var(--fluxa-scrim) 100%)',
    pointerEvents: 'none',
  },
  landscapeProgressBg: {
    position: 'absolute',
    zIndex: 2,
    right: '0.75rem',
    bottom: '0.38rem',
    left: '0.75rem',
    height: '0.3125rem',
    borderRadius: '62.4375rem',
    background: 'var(--fluxa-fill-strong)',
    boxShadow: '0 0 0.375rem rgba(0,0,0,0.5)',
  },
  progressBar: {
    height: '100%',
    borderRadius: '62.4375rem',
    background: 'var(--primary-accent-color)',
    boxShadow: '0 0 0.375rem var(--fluxa-accent-shadow)',
  },
  remainingBadge: {
    position: 'absolute',
    top: '0.5rem',
    right: '0.5625rem',
    color: 'var(--fluxa-text-primary)',
    fontSize: '0.6875rem',
    fontWeight: 750,
    textShadow: '0 1px 0.3125rem rgba(0,0,0,0.88)',
    background: 'var(--fluxa-scrim)',
    borderRadius: '0.3125rem',
    padding: '0.2rem 0.4rem',
  },
  newEpisodeBadge: {
    background: 'var(--primary-accent-color)',
    color: 'var(--primary-accent-foreground-color)',
    textShadow: 'none',
  },
  cornerBadge: {
    position: 'absolute',
    top: '0.5rem',
    left: '0.5625rem',
    color: 'var(--fluxa-text-primary)',
    fontSize: '0.75rem',
    fontWeight: 800,
    textShadow: '0 1px 0.3125rem rgba(0,0,0,0.88)',
    background: 'var(--fluxa-scrim)',
    borderRadius: '0.25rem',
    padding: '0.1875rem 0.375rem',
  },

  episodeName: {
    color: 'var(--fluxa-text-secondary)',
    fontSize: '0.78rem',
    fontWeight: 650,
    margin: '0.38rem 0 0',
    overflow: 'hidden',
    textOverflow: 'ellipsis',
    whiteSpace: 'nowrap',
  },
  name: {
    color: 'var(--fluxa-text-primary)',
    fontSize: '0.95rem',
    fontWeight: 800,
    margin: 0,
    overflow: 'hidden',
    textOverflow: 'ellipsis',
    whiteSpace: 'nowrap',
    lineHeight: 1.12,
  },
};
