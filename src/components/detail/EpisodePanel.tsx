import React, { useEffect, useMemo, useRef, useState } from 'react';
import { color, fade, radius } from '../../design';
import { Check, ChevronDown, Circle } from 'lucide-react';
import { t } from '../../i18n';
import { EP, S, spinnerStyle } from './detailStyles';
import { InlineSourceList } from './SourcePanel';
import type { Meta, Stream, Video } from '../../core/types';

export type ProgressEntry = { meta: { id: string }; timeOffset: number; duration: number; lastVideoId?: string };

export function seasonLabel(season: number): string {
  return season === 0 ? t('auto.extras') : t('format.season_number', season);
}

export function formatEpDate(released: string): string {
  try {
    const d = new Date(released);
    return d.toLocaleDateString('en-US', { month: 'short', day: 'numeric', year: 'numeric' });
  } catch {
    return released;
  }
}

export function episodeRating(episode: Video): string | null {
  const record = episode as unknown as Record<string, unknown>;
  const direct = record.imdbRating ?? record.rating;
  if (typeof direct === 'number') return direct.toFixed(1).replace(/\.0$/, '');
  if (typeof direct === 'string' && direct.trim()) return direct.trim();
  const ratings = record.ratings as Record<string, unknown> | undefined;
  const imdb = ratings?.imdb;
  if (typeof imdb === 'number') return imdb.toFixed(1).replace(/\.0$/, '');
  if (typeof imdb === 'string' && imdb.trim()) return imdb.trim();
  return null;
}

export function mergeEpisodes(primary: Video[], secondary: Video[]): Video[] {
  const byId = new Map<string, Video>();
  for (const episode of [...primary, ...secondary]) {
    const id = episode.id || `${episode.season ?? 1}:${episode.episode ?? episode.number ?? byId.size}`;
    byId.set(id, { ...byId.get(id), ...episode });
  }
  return [...byId.values()];
}

export function ImdbBadge({ rating, compact = false }: { rating: string | number; compact?: boolean }) {
  return (
    <span style={{ ...S.imdbBadge, ...(compact ? S.imdbBadgeCompact : null) }}>
      <span style={{ ...S.imdbLogo, ...(compact ? S.imdbLogoCompact : null) }}>IMDb</span>
      <span style={{ ...S.imdbRating, ...(compact ? S.imdbRatingCompact : null) }}>{rating}</span>
    </span>
  );
}

export function SeasonDropdown({
  seasons,
  selected,
  onChange,
  buttonStyle,
  seasonWatched,
  hideButtonIndicator,
}: {
  seasons: number[];
  selected: number;
  onChange: (s: number) => void;
  buttonStyle?: React.CSSProperties;
  seasonWatched?: Record<number, boolean>;
  hideButtonIndicator?: boolean;
}) {
  const [open, setOpen] = useState(false);
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!open) return;
    const handler = (e: MouseEvent) => {
      if (ref.current && !ref.current.contains(e.target as Node)) setOpen(false);
    };
    document.addEventListener('mousedown', handler);
    return () => document.removeEventListener('mousedown', handler);
  }, [open]);

  const selectedIsWatched = seasonWatched?.[selected] === true;

  return (
    <div ref={ref} style={{ position: 'relative' }}>
      <button style={{ ...EP.seasonBtn, ...buttonStyle }} onClick={() => setOpen((o) => !o)}>
        <span>{seasonLabel(selected)}</span>
        {seasonWatched && !hideButtonIndicator && (
          <span style={{ display: 'flex', alignItems: 'center', marginLeft: '0.125rem' }}>
            {selectedIsWatched ? <Check size={14} color={color.textStrong} /> : <Circle size={14} color={color.textDim} />}
          </span>
        )}
        <ChevronDown
          size={14}
          color="currentColor"
          style={{ transform: open ? 'rotate(180deg)' : 'none', transition: 'transform 0.2s', flexShrink: 0 }}
        />
      </button>
      {open && (
        <div style={EP.seasonMenu}>
          {seasons.map((s) => {
            const isWatched = seasonWatched?.[s] === true;
            return (
              <button
                key={s}
                style={{
                  ...EP.seasonMenuItem,
                  background: s === selected ? color.fillActive : 'transparent',
                  fontWeight: s === selected ? 700 : 500,
                  display: 'flex',
                  alignItems: 'center',
                  gap: '0.5rem',
                }}
                onClick={() => {
                  onChange(s);
                  setOpen(false);
                }}
              >
                <span style={{ flex: 1 }}>{seasonLabel(s)}</span>
                {isWatched ? (
                  <Check size={15} color={color.textStrong} style={{ flexShrink: 0 }} />
                ) : (
                  <Circle size={15} color={color.textDim} style={{ flexShrink: 0 }} />
                )}
              </button>
            );
          })}
        </div>
      )}
    </div>
  );
}

export function EpisodeRow({
  episode,
  number,
  selected,
  isWatched,
  progressPct,
  onClick,
  onToggleWatched,
}: {
  episode: Video;
  number: number;
  selected: boolean;
  isWatched: boolean;
  progressPct: number;
  onClick: () => void;
  onToggleWatched?: () => void;
}) {
  const [hovered, setHovered] = useState(false);
  const [thumbErr, setThumbErr] = useState(false);
  const title = episode.title?.trim() || episode.name?.trim() || `Episode ${episode.episode ?? episode.number ?? number}`;
  const dateStr = episode.released ? formatEpDate(episode.released) : null;
  const desc = (episode as unknown as { overview?: string }).overview;
  const rating = episodeRating(episode);

  return (
    <div
      style={{
        ...EP.row,
        background: selected ? color.fillHover : hovered ? color.fill : 'transparent',
        contentVisibility: 'auto',
        containIntrinsicSize: '5.75rem',
      }}
      onClick={onClick}
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
    >
      <div style={EP.thumb}>
        {episode.thumbnail && !thumbErr ? (
          <img
            src={episode.thumbnail}
            alt=""
            loading="lazy"
            decoding="async"
            style={{ width: '100%', height: '100%', objectFit: 'cover' }}
            onError={() => setThumbErr(true)}
          />
        ) : (
          <div style={EP.thumbPlaceholder}>
            <svg width="22" height="22" viewBox="0 0 24 24" fill={color.fillHover}>
              <path d="M18 4l2 4h-3l-2-4h-2l2 4h-3l-2-4H8l2 4H7L5 4H4c-1.1 0-2 .9-2 2v12c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V4h-4z" />
            </svg>
          </div>
        )}
        {progressPct > 0 && (
          <div style={{ position: 'absolute', bottom: 0, left: 0, right: 0, height: '0.1875rem', background: fade.shade(0.4) }}>
            <div style={{ height: '100%', width: `${progressPct}%`, background: color.imdb, borderRadius: `0 ${radius.xs} ${radius.xs} 0` }} />
          </div>
        )}
        {(hovered || selected) && (
          <div style={EP.thumbOverlay}>
            <svg width="22" height="22" viewBox="0 0 24 24" fill="white">
              <path d="M8 5v14l11-7z" />
            </svg>
          </div>
        )}
      </div>
      <div style={{ flex: 1, minWidth: 0, paddingRight: '0.25rem' }}>
        <div style={{ display: 'flex', alignItems: 'baseline', justifyContent: 'space-between', gap: '0.5rem', marginBottom: '0.25rem' }}>
          <p style={EP.epTitle}>
            {number}. {title}
          </p>
          <div style={{ ...EP.epMetaRight, alignItems: 'center' }}>
            {rating && <ImdbBadge rating={rating} compact />}
            {dateStr && <span style={EP.epDate}>{dateStr}</span>}
            {onToggleWatched && (
              <button
                style={{
                  background: 'none',
                  border: 'none',
                  padding: '0.125rem 0.25rem',
                  cursor: 'pointer',
                  display: 'flex',
                  alignItems: 'center',
                  flexShrink: 0,
                }}
                title={isWatched ? t('detail.mark_unwatched') : t('detail.mark_watched')}
                onClick={(e) => {
                  e.stopPropagation();
                  onToggleWatched();
                }}
              >
                {isWatched ? (
                  <svg width="18" height="18" viewBox="0 0 24 24" fill={color.light}>
                    <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-2 15-5-5 1.41-1.41L10 14.17l7.59-7.59L19 8l-9 9z" />
                  </svg>
                ) : (
                  <svg width="18" height="18" viewBox="0 0 24 24" fill={color.fillStrong}>
                    <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm0 18c-4.42 0-8-3.58-8-8s3.58-8 8-8 8 3.58 8 8-3.58 8-8 8z" />
                  </svg>
                )}
              </button>
            )}
          </div>
        </div>
        {desc && <p style={EP.epDesc}>{desc}</p>}
      </div>
    </div>
  );
}

export function EpisodePanel({
  metaId,
  meta,
  seasons,
  selectedSeason,
  onSeasonChange,
  episodes,
  selectedEpisode,
  showSources,
  streams,
  isLoadingStreams,
  isLoadingEpisodes,
  availableAddons,
  playbackFailure,
  streamAddonCount,
  onBackToEpisodes,
  onEpisodeClick,
  onPlaySource,
  onAddonChange,
  watchedMap,
  progressMap,
  blurUnwatchedEpisodes: _blurUnwatchedEpisodes,
  detailSeasonSelectorMode: _detailSeasonSelectorMode,
  episodeCardsLayout: _episodeCardsLayout,
  onToggleEpisodeWatched,
  onMarkSeason,
  seasonWatchedMap,
}: {
  metaId: string;
  meta: Meta;
  seasons: number[];
  selectedSeason: number;
  onSeasonChange: (s: number) => void;
  episodes: Video[];
  selectedEpisode: Video | null;
  showSources: boolean;
  streams: Stream[];
  isLoadingStreams: boolean;
  isLoadingEpisodes?: boolean;
  availableAddons: string[];
  playbackFailure?: string | null;
  streamAddonCount: number;
  onBackToEpisodes: () => void;
  onEpisodeClick: (ep: Video) => void;
  onPlaySource: (stream: Stream) => void;
  onAddonChange?: (addon: string | null) => void;
  watchedMap: Record<string, boolean>;
  progressMap: Record<string, ProgressEntry>;
  blurUnwatchedEpisodes: boolean;
  detailSeasonSelectorMode: string;
  episodeCardsLayout: string;
  onToggleEpisodeWatched?: (ep: Video, isWatched: boolean) => void;
  onMarkSeason?: (seasons: number[], watched: boolean) => void;
  seasonWatchedMap?: Record<number, boolean>;
}) {
  const [epSearch, setEpSearch] = useState('');
  const seasonIndex = seasons.indexOf(selectedSeason);
  const hasPrev = seasonIndex > 0;
  const hasNext = seasonIndex < seasons.length - 1;

  const filtered = useMemo(
    () =>
      epSearch
        ? episodes.filter((ep) =>
            (ep.title ?? ep.name ?? `Episode ${ep.episode ?? ep.number ?? ''}`).toLowerCase().includes(epSearch.toLowerCase()),
          )
        : episodes,
    [epSearch, episodes],
  );

  const metaProgress = progressMap[metaId];

  return (
    <div style={EP.panel}>
      <div style={EP.seasonNav}>
        <button
          style={{ ...EP.navBtn, opacity: hasPrev ? 1 : 0.25, cursor: hasPrev ? 'pointer' : 'default' }}
          onClick={() => hasPrev && onSeasonChange(seasons[seasonIndex - 1])}
          disabled={!hasPrev}
        >
          <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
            <path d="M15.41 7.41L14 6l-6 6 6 6 1.41-1.41L10.83 12z" />
          </svg>
          <span>{t('common.prev')}</span>
        </button>
        <SeasonDropdown seasons={seasons} selected={selectedSeason} onChange={onSeasonChange} seasonWatched={seasonWatchedMap} />
        <button
          style={{ ...EP.navBtn, opacity: hasNext ? 1 : 0.25, cursor: hasNext ? 'pointer' : 'default' }}
          onClick={() => hasNext && onSeasonChange(seasons[seasonIndex + 1])}
          disabled={!hasNext}
        >
          <span>{t('common.next')}</span>
          <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
            <path d="M10 6L8.59 7.41 13.17 12l-4.58 4.59L10 18l6-6z" />
          </svg>
        </button>
        {onMarkSeason && (
          <button
            style={{ ...EP.navBtn, marginLeft: 'auto' }}
            title={seasonWatchedMap?.[selectedSeason] ? t('detail.mark_season_unwatched') : t('detail.mark_season_watched')}
            onClick={() => onMarkSeason([selectedSeason], !seasonWatchedMap?.[selectedSeason])}
          >
            {seasonWatchedMap?.[selectedSeason] ? (
              <svg width="16" height="16" viewBox="0 0 24 24" fill={color.light}>
                <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-2 15-5-5 1.41-1.41L10 14.17l7.59-7.59L19 8l-9 9z" />
              </svg>
            ) : (
              <svg width="16" height="16" viewBox="0 0 24 24" fill={color.fillStrong}>
                <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm0 18c-4.42 0-8-3.58-8-8s3.58-8 8-8 8 3.58 8 8-3.58 8-8 8z" />
              </svg>
            )}
            <span>{seasonWatchedMap?.[selectedSeason] ? t('detail.mark_season_unwatched') : t('detail.mark_season_watched')}</span>
          </button>
        )}
      </div>

      {showSources && selectedEpisode ? (
        <InlineSourceList
          episode={selectedEpisode}
          meta={meta}
          streams={streams}
          isLoading={isLoadingStreams}
          availableAddons={availableAddons}
          playbackFailure={playbackFailure}
          streamAddonCount={streamAddonCount}
          onBack={onBackToEpisodes}
          onPlay={onPlaySource}
          onAddonChange={onAddonChange}
        />
      ) : (
        <>
          <div style={EP.searchRow}>
            <svg width="14" height="14" viewBox="0 0 24 24" fill={color.fillStrong} style={{ flexShrink: 0 }}>
              <path d="M15.5 14h-.79l-.28-.27A6.47 6.47 0 0 0 16 9.5 6.5 6.5 0 1 0 9.5 16c1.61 0 3.09-.59 4.23-1.57l.27.28v.79l5 4.99L20.49 19l-4.99-5zm-6 0C7.01 14 5 11.99 5 9.5S7.01 5 9.5 5 14 7.01 14 9.5 11.99 14 9.5 14z" />
            </svg>
            <input
              type="text"
              placeholder={t('detail.search_episodes')}
              value={epSearch}
              onChange={(e) => setEpSearch(e.target.value)}
              style={EP.searchInput}
            />
          </div>
          <div style={EP.list}>
            {filtered.length === 0 ? (
              isLoadingEpisodes ? (
                <div style={{ display: 'flex', justifyContent: 'center', padding: '2rem' }}>
                  <div style={spinnerStyle} />
                </div>
              ) : (
                <div style={EP.empty}>{t('detail.no_episodes_found')}</div>
              )
            ) : (
              filtered.map((ep, i) => {
                const isWatched = watchedMap[ep.id] === true;
                const showProgress = !isWatched && metaProgress?.lastVideoId === ep.id && (metaProgress.duration ?? 0) > 0;
                const progressPct = showProgress ? Math.min(100, Math.round((metaProgress!.timeOffset / metaProgress!.duration) * 100)) : 0;
                return (
                  <EpisodeRow
                    key={ep.id}
                    episode={ep}
                    number={i + 1}
                    selected={selectedEpisode?.id === ep.id}
                    isWatched={isWatched}
                    progressPct={progressPct}
                    onClick={() => onEpisodeClick(ep)}
                    onToggleWatched={onToggleEpisodeWatched ? () => onToggleEpisodeWatched(ep, isWatched) : undefined}
                  />
                );
              })
            )}
          </div>
        </>
      )}
    </div>
  );
}
