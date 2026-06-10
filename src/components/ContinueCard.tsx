import React from 'react';
import { Check, Info, X } from '@phosphor-icons/react';
import type { LibraryItem, Meta } from '../core/types';
import {
  selectContinueWatchingArtwork,
  formatEpisodeLine,
  formatRemaining,
  formatReleaseCountdown,
} from '../core/continueWatchingUtils';
import { t } from '../i18n';

export function ContinueCard({
  meta,
  isHorizontal,
  artworkPreference,
  onClick,
  onMarkWatched,
  onDrop,
}: {
  meta: Meta;
  isHorizontal: boolean;
  artworkPreference: string;
  onClick: (m: Meta) => void;
  onMarkWatched: (m: Meta) => void;
  onDrop: (m: Meta) => void;
}) {
  const [hovered, setHovered] = React.useState(false);
  const [imgError, setImgError] = React.useState(false);
  const [menuOpen, setMenuOpen] = React.useState(false);

  const lib = meta as unknown as LibraryItem & {
    lastEpisodeName?: string;
    lastEpisodeSeason?: number;
    lastEpisodeNumber?: number;
    lastEpisodeThumbnail?: string;
    continueWatchingPoster?: string;
    continueWatchingBackground?: string;
    continueWatchingBadge?: string;
    newEpisodeReleasedAt?: string;
    reason?: string;
  };
  const progress = lib.timeOffset && lib.duration ? lib.timeOffset / lib.duration : 0;
  const artwork = selectContinueWatchingArtwork(meta, artworkPreference, isHorizontal);
  const isUpNext = progress < 0.005 || progress >= 0.995;
  const isNewEpisode = lib.continueWatchingBadge === 'newEpisode';
  const scheduledText = lib.continueWatchingBadge === 'scheduledEpisode'
    ? formatReleaseCountdown(lib.newEpisodeReleasedAt)
    : null;
  const remainingText = !isUpNext && lib.timeOffset && lib.duration
    ? formatRemaining(lib.timeOffset, lib.duration)
    : null;
  const episodeLine = formatEpisodeLine(lib);

  return (
    <div
      role="button"
      tabIndex={0}
      style={{
        ...(isHorizontal ? cwStyles.landscapeCard : cwStyles.posterCard),
        boxShadow: hovered ? '0 0 0 2px rgba(255,255,255,0.44)' : 'none',
        transform: hovered ? 'translateY(-2px)' : 'translateY(0)',
      }}
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
      onFocus={() => setHovered(true)}
      onBlur={() => setHovered(false)}
      onClick={() => onClick(meta)}
      onKeyDown={(e) => {
        if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); onClick(meta); }
      }}
    >
      <div style={cwStyles.imageArea}>
        {artwork && !imgError ? (
          <img src={artwork} alt={meta.name} loading="lazy" decoding="async" style={cwStyles.artwork} onError={() => setImgError(true)} />
        ) : (
          <div style={cwStyles.thumbPlaceholder}>
            <span style={cwStyles.placeholderText}>{meta.name.slice(0, 1).toUpperCase()}</span>
          </div>
        )}
        {!isUpNext && progress > 0 && (
          <div style={cwStyles.imageProgressBg}>
            <div style={{ ...cwStyles.progressBar, width: `${Math.min(progress, 1) * 100}%` }} />
          </div>
        )}
        {isNewEpisode && meta.type === 'series'
          ? <div style={{ ...cwStyles.remainingBadge, ...cwStyles.newEpisodeBadge }}>{t('auto.new_episode')}</div>
          : scheduledText && meta.type === 'series'
          ? <div style={cwStyles.remainingBadge}>{scheduledText}</div>
          : isUpNext && meta.type === 'series'
          ? <div style={cwStyles.remainingBadge}>{t('auto.up_next')}</div>
          : remainingText
          ? <div style={cwStyles.remainingBadge}>{remainingText}</div>
          : null}
        {hovered && (
          <div style={{ position: 'absolute', inset: 0, display: 'flex', alignItems: 'center', justifyContent: 'center', pointerEvents: 'none' }}>
            <div style={{ width: 56, height: 56, borderRadius: '50%', background: 'rgba(35,35,35,0.88)', display: 'flex', alignItems: 'center', justifyContent: 'center' }}>
              <svg width="28" height="28" viewBox="0 0 24 24" fill="white" style={{ marginLeft: 3 }}><path d="M8 5v14l11-7z" /></svg>
            </div>
          </div>
        )}
      </div>

      <div style={cwStyles.footer}>
        <div style={cwStyles.metaStack}>
          <p style={cwStyles.name}>{meta.name}</p>
          <p style={cwStyles.episodeName}>{episodeLine || (meta.type === 'series' ? t('auto.up_next') : '')}</p>
        </div>
        <button
          type="button"
          aria-label={t('home.continue_watching_actions')}
          style={cwStyles.infoButton}
          onClick={(e) => { e.stopPropagation(); setMenuOpen((open) => !open); }}
          onKeyDown={(e) => e.stopPropagation()}
        >
          <Info size={14} />
        </button>
      </div>

      {menuOpen && (
        <div style={cwStyles.menu} onClick={(e) => e.stopPropagation()}>
          <button type="button" style={cwStyles.menuItem} onClick={() => { setMenuOpen(false); onMarkWatched(meta); }}>
            <Check size={15} />
            {t('detail.mark_watched')}
          </button>
          <button type="button" style={cwStyles.menuItem} onClick={() => { setMenuOpen(false); onDrop(meta); }}>
            <X size={15} />
            {t('home.drop_continue_watching')}
          </button>
        </div>
      )}

      {lib.reason?.toLowerCase() === 'trakt.tv' && (
        <div style={cwStyles.sourceBadge}>T</div>
      )}
    </div>
  );
}

const cwStyles: Record<string, React.CSSProperties> = {
  landscapeCard: { position: 'relative', width: 318, minWidth: 318, height: 218, borderRadius: 2, overflow: 'hidden', background: '#050506', cursor: 'pointer', transition: 'transform 0.16s ease', outline: 'none' },
  posterCard: { position: 'relative', width: 128, minWidth: 128, height: 192, borderRadius: 3, overflow: 'hidden', background: '#141922', cursor: 'pointer', transition: 'transform 0.16s ease', outline: 'none' },
  imageArea: { position: 'relative', width: '100%', height: 161, overflow: 'hidden', background: '#141922' },
  artwork: { width: '100%', height: '100%', objectFit: 'cover', display: 'block' },
  thumbPlaceholder: { width: '100%', height: '100%', background: '#1B212B', display: 'flex', alignItems: 'center', justifyContent: 'center' },
  placeholderText: { color: 'rgba(255,255,255,0.22)', fontSize: 48, fontWeight: 900 },
  footer: { height: 57, padding: '9px 10px 10px', display: 'flex', alignItems: 'center', justifyContent: 'space-between', gap: 8, background: '#050506' },
  metaStack: { flex: 1, minWidth: 0 },
  imageProgressBg: { position: 'absolute', left: 10, right: 10, bottom: 8, height: 4, borderRadius: 999, background: 'rgba(255,255,255,0.24)' },
  progressBar: { height: '100%', borderRadius: 999, background: '#e50914' },
  remainingBadge: { position: 'absolute', top: 8, right: 9, color: '#FFFFFF', fontSize: 12, fontWeight: 800, textShadow: '0 1px 5px rgba(0,0,0,0.88)', background: 'rgba(0,0,0,0.42)', borderRadius: 4, padding: '3px 6px' },
  newEpisodeBadge: { background: 'var(--primary-accent-color)', color: 'var(--primary-accent-foreground-color)', textShadow: 'none' },
  sourceBadge: { position: 'absolute', top: 8, right: 8, width: 22, height: 22, borderRadius: 999, background: 'rgba(0,0,0,0.72)', border: '1px solid rgba(255,255,255,0.14)', display: 'flex', alignItems: 'center', justifyContent: 'center', color: '#FFFFFF', fontSize: 11, fontWeight: 900 },
  episodeName: { color: 'rgba(255,255,255,0.68)', fontSize: 13, fontWeight: 600, margin: '4px 0 0', overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' },
  name: { color: '#FFFFFF', fontSize: 15, fontWeight: 800, margin: 0, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap', lineHeight: 1.12 },
  infoButton: { width: 27, height: 27, borderRadius: 999, border: '2px solid #FFFFFF', background: 'transparent', color: '#FFFFFF', display: 'flex', alignItems: 'center', justifyContent: 'center', flexShrink: 0, cursor: 'pointer', padding: 0 },
  menu: { position: 'absolute', right: 10, bottom: 52, zIndex: 30, width: 178, padding: 5, borderRadius: 8, border: '1px solid rgba(255,255,255,0.12)', background: 'rgba(13,15,22,0.98)', boxShadow: '0 8px 20px rgba(0,0,0,0.45)' },
  menuItem: { width: '100%', minHeight: 34, border: 'none', borderRadius: 6, background: 'transparent', color: '#FFFFFF', display: 'flex', alignItems: 'center', gap: 9, padding: '0 10px', cursor: 'pointer', fontSize: 12, fontWeight: 800, fontFamily: 'sans-serif', textAlign: 'left' },
};
