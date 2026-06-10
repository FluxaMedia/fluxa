import React, { useState } from 'react';
import { t } from '../../i18n';
import type { Video } from '../../core/types';
import { MS } from './detailStyles';
import { formatEpDate } from './EpisodePanel';

export function epReleaseCountdown(date?: string): string {
  if (!date) return '';
  const diff = new Date(date).getTime() - Date.now();
  if (diff <= 0) return t('format.remaining_almost_done');
  const mins = Math.floor(diff / 60000);
  const hours = Math.floor(mins / 60);
  const days = Math.floor(hours / 24);
  if (days > 0) return hours % 24 > 0 ? `${days}d ${hours % 24}h` : `${days}d`;
  if (hours > 0) return mins % 60 > 0 ? `${hours}h ${mins % 60}m` : `${hours}h`;
  return `${mins}m`;
}

function episodeContentRating(episode: Video): string | null {
  const record = episode as unknown as Record<string, unknown>;
  const cr = record.contentRating ?? record.certification ?? record.rated ?? record.ageRating;
  return typeof cr === 'string' && cr.trim() ? cr.trim() : null;
}

export function ModernEpisodeCard({ episode, number, isWatched, progressPct, minutesRemaining, cwBadge, cwScheduledDate, onClick, onToggleWatched }: {
  episode: Video; number: number; isWatched: boolean; progressPct: number; minutesRemaining: number;
  cwBadge?: string | null; cwScheduledDate?: string; onClick: () => void; onToggleWatched?: () => void;
}) {
  const [hovered, setHovered] = useState(false);
  const [thumbErr, setThumbErr] = useState(false);
  const [watchBtnHovered, setWatchBtnHovered] = useState(false);
  const title = episode.title?.trim() || episode.name?.trim() || `Episode ${episode.episode ?? episode.number ?? number}`;
  const desc = (episode as unknown as { overview?: string }).overview;
  const dateStr = episode.released ? formatEpDate(episode.released) : null;
  const runtime = (episode as unknown as { runtime?: string }).runtime;
  const contentRating = episodeContentRating(episode);

  void watchBtnHovered;

  return (
    <div style={MS.epCard} onClick={onClick} onMouseEnter={() => setHovered(true)} onMouseLeave={() => setHovered(false)}>
      <div style={MS.epThumb}>
        {episode.thumbnail && !thumbErr ? (
          <img src={episode.thumbnail} alt="" style={{ width: '100%', height: '100%', objectFit: 'cover', display: 'block' }} onError={() => setThumbErr(true)} />
        ) : (
          <div style={MS.epThumbPlaceholder}>
            <svg width="32" height="32" viewBox="0 0 24 24" fill="rgba(255,255,255,0.07)">
              <path d="M18 4l2 4h-3l-2-4h-2l2 4h-3l-2-4H8l2 4H7L5 4H4c-1.1 0-2 .9-2 2v12c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V4h-4z" />
            </svg>
          </div>
        )}
        {minutesRemaining > 0 && <div style={MS.epTimeRemaining}>{t('format.remaining_minutes', minutesRemaining)}</div>}
        {progressPct > 0 && (
          <div style={MS.epProgressTrack}>
            <div style={{ ...MS.epProgressFill, width: `${progressPct}%`, background: '#e50914' }} />
          </div>
        )}
        {cwBadge && !minutesRemaining && (
          <div style={{
            position: 'absolute', top: 7, left: 7, zIndex: 3,
            background: cwBadge === 'newEpisode' ? '#e50914' : 'rgba(0,0,0,0.68)',
            backdropFilter: cwBadge !== 'newEpisode' ? 'blur(4px)' : undefined,
            color: '#fff', fontSize: 11, fontWeight: 800,
            padding: '3px 7px', borderRadius: 4,
            textShadow: cwBadge === 'newEpisode' ? 'none' : '0 1px 4px rgba(0,0,0,0.8)',
            letterSpacing: '0.2px',
          }}>
            {cwBadge === 'newEpisode' ? t('auto.new_episode')
              : cwBadge === 'upNext' ? t('auto.up_next')
              : cwBadge === 'scheduledEpisode' ? epReleaseCountdown(cwScheduledDate)
              : null}
          </div>
        )}
        {hovered && (
          <div style={MS.epHoverOverlay}>
            <div style={{ width: 56, height: 56, borderRadius: '50%', background: 'rgba(35,35,35,0.88)', display: 'flex', alignItems: 'center', justifyContent: 'center' }}>
              <svg width="28" height="28" viewBox="0 0 24 24" fill="white" style={{ marginLeft: 3 }}><path d="M8 5v14l11-7z" /></svg>
            </div>
          </div>
        )}
        {onToggleWatched && (
          <button
            style={{ position: 'absolute', top: 5, right: 5, width: 32, height: 32, borderRadius: '50%', border: 'none', padding: 0, background: 'rgba(0,0,0,0.52)', backdropFilter: 'blur(4px)', display: 'flex', alignItems: 'center', justifyContent: 'center', cursor: 'pointer', zIndex: 4, flexShrink: 0 }}
            onClick={(e) => { e.stopPropagation(); onToggleWatched(); }}
            onMouseEnter={(e) => { e.stopPropagation(); setWatchBtnHovered(true); }}
            onMouseLeave={(e) => { e.stopPropagation(); setWatchBtnHovered(false); }}
            title={isWatched ? t('detail.mark_unwatched') : t('detail.mark_watched')}
          >
            {isWatched ? (
              <svg width="22" height="22" viewBox="0 0 24 24" fill="rgba(255,255,255,0.9)"><path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-2 15-5-5 1.41-1.41L10 14.17l7.59-7.59L19 8l-9 9z" /></svg>
            ) : (
              <svg width="22" height="22" viewBox="0 0 24 24" fill="rgba(255,255,255,0.55)"><path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm0 18c-4.42 0-8-3.58-8-8s3.58-8 8-8 8 3.58 8 8-3.58 8-8 8z" /></svg>
            )}
          </button>
        )}
      </div>
      <h3 style={{ ...MS.epTitle, color: hovered ? 'rgba(255,255,255,0.82)' : '#FFFFFF' }}>{number}. {title}</h3>
      {desc && <p style={MS.epDesc}>{desc}</p>}
      <div style={MS.epMetaRow}>
        {contentRating && <span style={MS.epRatingBadge}>{contentRating}</span>}
        {(runtime || dateStr) && <span style={MS.epMetaText}>{[runtime, dateStr].filter(Boolean).join('  ')}</span>}
      </div>
    </div>
  );
}
