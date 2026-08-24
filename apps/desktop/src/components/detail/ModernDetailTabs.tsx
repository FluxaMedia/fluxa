import React from 'react';
import { color, fade } from '../../design';
import { MovieCard } from '../MovieCard';
import { t } from '../../i18n';
import type { DetailState, LibraryItem, Meta, MetaLink, Trailer, Video } from '../../core/types';
import type { posterPrefsFromState } from '../../core/posterPrefs';
import { MS, S, spinnerStyle } from './detailStyles';
import { MetaFactsSection } from './MetaFacts';
import { CastAvatar, type NormalizedCastMember } from './castSection';
import { TrailerCarousel, type TrailerMetadata } from './TrailerCarousel';
import { ModernEpisodeCard } from './ModernEpisodeCard';
import type { ProgressEntry } from './EpisodePanel';

export function DetailsTabContent({
  displayMeta,
  castMembers,
  directorLinks,
  peopleImages,
  displayTrailers,
  trailerMetadata,
  creators,
}: {
  displayMeta: Meta;
  castMembers: NormalizedCastMember[];
  directorLinks: MetaLink[];
  peopleImages: Record<string, string>;
  displayTrailers: Trailer[];
  trailerMetadata: TrailerMetadata;
  creators: string[];
}) {
  return (
    <div style={MS.detailsTab}>
      {displayMeta.awards && (
        <div style={MS.detailsSection}>
          <h3 style={MS.detailsSectionTitle}>{t('detail.awards')}</h3>
          <p style={{ ...MS.detailsText, color: color.success, fontWeight: 700 }}>{displayMeta.awards}</p>
        </div>
      )}
      {(castMembers.length > 0 || directorLinks.length > 0 || creators.length > 0) && (
        <div style={MS.detailsSection}>
          <h3 style={MS.detailsSectionTitle}>{t('detail.cast_crew')}</h3>
          <div style={S.castRow}>
            {creators.map((name) => (
              <CastAvatar key={`creator-${name}`} name={name} role={t('detail.fact_creator')} imageUrl={peopleImages[name]} />
            ))}
            {directorLinks.map((l) => (
              <CastAvatar key={`dir-${l.name}`} name={l.name} role={t('detail.director')} imageUrl={peopleImages[l.name]} />
            ))}
            {castMembers.map((member) => (
              <CastAvatar
                key={`cast-${member.name}:${member.role ?? ''}`}
                name={member.name}
                role={member.role || t('detail.actor')}
                imageUrl={member.imageUrl ?? peopleImages[member.name]}
              />
            ))}
          </div>
        </div>
      )}
      {displayTrailers.length > 0 && (
        <div style={MS.detailsSection}>
          <h3 style={MS.detailsSectionTitle}>{t('auto.trailers')}</h3>
          <TrailerCarousel trailers={displayTrailers} trailerMetadata={trailerMetadata} />
        </div>
      )}
      <MetaFactsSection displayMeta={displayMeta} />
    </div>
  );
}

export function EpisodesTabContent({
  detail,
  filteredEps,
  watchedMap,
  progressMap,
  metaId,
  continueWatchingEntry,
  episodeGridStyle,
  blurUnwatchedEpisodes,
  spoilerHideEpisodeInfo,
  onEpisodeClick,
  toggleEpisodeWatched,
  seasonControls,
}: {
  detail: DetailState;
  filteredEps: Video[];
  watchedMap: Record<string, boolean>;
  progressMap: Record<string, ProgressEntry>;
  metaId: string;
  continueWatchingEntry?: LibraryItem | null;
  episodeGridStyle: React.CSSProperties;
  blurUnwatchedEpisodes: boolean;
  spoilerHideEpisodeInfo: boolean;
  onEpisodeClick: (ep: Video) => void;
  toggleEpisodeWatched: (ep: Video, isWatched: boolean) => void;
  seasonControls?: React.ReactNode;
}) {
  return (
    <div style={MS.episodeSection}>
      <div style={MS.sectionHeaderRow}>
        <h3 style={{ ...MS.detailsSectionTitle, margin: 0 }}>{t('auto.episodes')}</h3>
        {seasonControls}
      </div>
      {detail.isLoading && filteredEps.length === 0 ? (
        <div style={{ display: 'flex', justifyContent: 'center', padding: '2.5rem' }}>
          <div style={spinnerStyle} />
        </div>
      ) : (
        <>
          <p style={MS.episodeCount}>{t('format.episode_count', filteredEps.length)}</p>
          <div className="detail-episode-grid" style={episodeGridStyle}>
            {filteredEps.map((ep, i) => {
              const isWatched = watchedMap[ep.id] === true;
              const metaProgress = progressMap[metaId];
              const showProg = !isWatched && metaProgress?.lastVideoId === ep.id && (metaProgress.duration ?? 0) > 0;
              const progressPct = isWatched
                ? 100
                : showProg
                  ? Math.min(99, Math.round((metaProgress!.timeOffset / metaProgress!.duration) * 100))
                  : 0;
              const minutesRemaining = showProg ? Math.max(0, Math.round((metaProgress!.duration - metaProgress!.timeOffset) / 60)) : 0;
              const isCwEp = continueWatchingEntry?.lastVideoId === ep.id;
              const cwBadge = isCwEp ? (continueWatchingEntry?.continueWatchingBadge ?? null) : null;
              const cwScheduledDate =
                cwBadge === 'scheduledEpisode'
                  ? (continueWatchingEntry as LibraryItem & { newEpisodeReleasedAt?: string })?.newEpisodeReleasedAt
                  : undefined;
              return (
                <ModernEpisodeCard
                  key={ep.id}
                  episode={ep}
                  number={i + 1}
                  isWatched={isWatched}
                  progressPct={progressPct}
                  minutesRemaining={minutesRemaining}
                  cwBadge={cwBadge}
                  cwScheduledDate={cwScheduledDate}
                  blurUnwatched={blurUnwatchedEpisodes}
                  spoilerHide={spoilerHideEpisodeInfo}
                  onClick={() => onEpisodeClick(ep)}
                  onToggleWatched={() => toggleEpisodeWatched(ep, isWatched)}
                />
              );
            })}
          </div>
        </>
      )}
    </div>
  );
}

export function RelatedTabContent({
  similarItems,
  poster,
  onNavigateDetail,
  sourcePicker,
}: {
  similarItems: Meta[];
  poster: ReturnType<typeof posterPrefsFromState>;
  onNavigateDetail: (meta: Meta) => void;
  sourcePicker?: React.ReactNode;
}) {
  return (
    <div style={MS.relatedSection}>
      <div style={MS.sectionHeaderRow}>
        <h3 style={{ ...MS.detailsSectionTitle, margin: 0 }}>{t('auto.similar_titles')}</h3>
        {sourcePicker}
      </div>
      {similarItems.length === 0 ? (
        <p style={MS.episodeCount}>{t('auto.no_similar_titles')}</p>
      ) : (
        <div className="detail-related-grid" style={MS.relatedGrid}>
          {similarItems.map((item) => (
            <div key={`${item.type}:${item.id}`} style={{ flexShrink: 0 }}>
              <MovieCard
                meta={item}
                width={poster.width}
                height={poster.height}
                radius={poster.radius}
                hideTitle={poster.hideTitles}
                layout={poster.layout}
                preview={poster.hoverPreview}
                onClick={onNavigateDetail}
              />
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
