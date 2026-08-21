import React from 'react';
import { t } from '../../i18n';
import type { Meta, MetaLink } from '../../core/types';
import { SectionLabel, color, radius, space } from '../../design';
import { MS } from './detailStyles';
import { GenreTag } from './ModernDetailParts';
import { RatingsRow } from './RatingBadge';
import type { NormalizedCastMember } from './castSection';
import { formatEpDate } from './EpisodePanel';
import { dedupedWatchProviders, WatchProviderLogo, openWatchProvidersLink } from './watchProvidersSection';

type ScoreRow = { label: string; value: string; fill: number };

function buildScoreRows(
  displayMeta: Meta,
  omdbRatings?: { rottenTomatoes?: string; metascore?: string } | null,
): ScoreRow[] {
  const rows: ScoreRow[] = [];
  if (displayMeta.imdbRating) {
    const value = Number(displayMeta.imdbRating);
    rows.push({ label: 'IMDb', value: value.toFixed(1), fill: Math.min(100, value * 10) });
  }
  if (omdbRatings?.rottenTomatoes) {
    rows.push({
      label: 'Rotten Tomatoes',
      value: omdbRatings.rottenTomatoes,
      fill: Math.min(100, parseInt(omdbRatings.rottenTomatoes, 10) || 0),
    });
  }
  if (omdbRatings?.metascore) {
    rows.push({ label: 'Metascore', value: omdbRatings.metascore, fill: Math.min(100, parseInt(omdbRatings.metascore, 10) || 0) });
  }
  return rows;
}

function Fact({ label, value }: { label: string; value: string }) {
  return (
    <div style={MS.railRow}>
      <span style={MS.railKey}>{label}</span>
      <span style={MS.railValue}>{value}</span>
    </div>
  );
}

export function ModernDetailRail({
  displayMeta,
  posterUrl,
  mdblistRatings,
  omdbRatings,
  castMembers,
  directorLinks,
  peopleImages,
  onNavigateGenre,
  onSeeAllCast,
}: {
  displayMeta: Meta;
  posterUrl?: string;
  mdblistRatings?: Record<string, number> | null;
  omdbRatings?: { rottenTomatoes?: string; metascore?: string } | null;
  castMembers: NormalizedCastMember[];
  directorLinks: MetaLink[];
  peopleImages: Record<string, string>;
  onNavigateGenre?: (genre: string) => void;
  onSeeAllCast: () => void;
}) {
  const hasMdblistRatings = mdblistRatings != null && Object.keys(mdblistRatings).length > 0;
  const scoreRows = hasMdblistRatings ? [] : buildScoreRows(displayMeta, omdbRatings);
  const genres = Array.isArray(displayMeta.genres) ? displayMeta.genres.slice(0, 6) : [];
  const visibleCast = castMembers.slice(0, 5);
  const creators = displayMeta.createdBy?.length ? displayMeta.createdBy : displayMeta.director;
  const watchProviders = dedupedWatchProviders(displayMeta.watchProviders).slice(0, 5);
  const nextEpisode = displayMeta.nextEpisodeToAir;
  const language = displayMeta.originalLanguage?.toUpperCase();

  return (
    <aside className="detail-rail" style={MS.railCol}>
      {posterUrl && <img src={posterUrl} alt="" style={MS.railPoster} />}

      {(hasMdblistRatings || scoreRows.length > 0) && (
        <div style={MS.railGroup}>
          <SectionLabel>{t('detail.ratings')}</SectionLabel>
          {hasMdblistRatings ? (
            <RatingsRow ratings={mdblistRatings} />
          ) : (
            scoreRows.map((row) => (
              <div key={row.label} style={{ display: 'flex', flexDirection: 'column', gap: space[1] }}>
                <div style={MS.railRow}>
                  <span style={MS.railKey}>{row.label}</span>
                  <span style={MS.railScoreValue}>{row.value}</span>
                </div>
                <span style={MS.railTrack}>
                  <span style={{ ...MS.railTrackFill, width: `${row.fill}%` }} />
                </span>
              </div>
            ))
          )}
        </div>
      )}

      {nextEpisode?.airDate && (
        <div style={MS.railGroup}>
          <SectionLabel>{t('detail.next_episode')}</SectionLabel>
          <span style={MS.railNextValue}>{formatEpDate(nextEpisode.airDate)}</span>
          <span style={MS.railCastNames}>
            {t('format.season_episode_short', nextEpisode.season, nextEpisode.episode)}
            {nextEpisode.name ? ` · ${nextEpisode.name}` : ''}
          </span>
        </div>
      )}

      {watchProviders.length > 0 && (
        <div style={MS.railGroup}>
          <SectionLabel>{t('detail.watch_providers')}</SectionLabel>
          <div style={{ display: 'flex', flexWrap: 'wrap', gap: space[1.5] }}>
            {watchProviders.map((provider) => (
              <WatchProviderLogo
                key={provider.name}
                name={provider.name}
                logo={provider.logo}
                size="2.25rem"
                onClick={() => openWatchProvidersLink(displayMeta.watchProviders?.link)}
              />
            ))}
          </div>
        </div>
      )}

      {visibleCast.length > 0 && (
        <div style={MS.railGroup}>
          <SectionLabel>{t('auto.cast')}</SectionLabel>
          <div style={{ display: 'flex', gap: space[1.5] }}>
            {visibleCast.map((member) => {
              const image = member.imageUrl ?? peopleImages[member.name];
              return image ? (
                <img
                  key={member.name}
                  src={image}
                  alt={member.name}
                  title={member.name}
                  style={{
                    width: '2.125rem',
                    height: '2.125rem',
                    borderRadius: radius.circle,
                    objectFit: 'cover',
                    border: `1px solid ${color.line}`,
                  }}
                />
              ) : (
                <span
                  key={member.name}
                  title={member.name}
                  style={{
                    width: '2.125rem',
                    height: '2.125rem',
                    borderRadius: radius.circle,
                    background: color.fillHover,
                    border: `1px solid ${color.line}`,
                    flexShrink: 0,
                  }}
                />
              );
            })}
          </div>
          <span style={MS.railCastNames}>{visibleCast.map((member) => member.name).join(', ')}</span>
          {castMembers.length > visibleCast.length && (
            <button style={MS.railSeeAll} onClick={onSeeAllCast}>
              {t('detail.see_all')}
            </button>
          )}
        </div>
      )}

      {genres.length > 0 && (
        <div style={MS.railGroup}>
          <SectionLabel>{t('auto.genres')}</SectionLabel>
          <div style={{ display: 'flex', flexWrap: 'wrap', gap: `${space[1]} ${space[2]}` }}>
            {genres.map((genre) => (
              <GenreTag key={genre} label={genre} onClick={() => onNavigateGenre?.(genre)} />
            ))}
          </div>
        </div>
      )}

      <div style={MS.railGroup}>
        {creators?.length ? <Fact label={t('detail.fact_creator')} value={creators.slice(0, 2).join(', ')} /> : null}
        {directorLinks.length > 0 && !creators?.length ? (
          <Fact label={t('detail.director')} value={directorLinks.map((link) => link.name).join(', ')} />
        ) : null}
        {displayMeta.network ? <Fact label={t('detail.fact_network')} value={displayMeta.network} /> : null}
        {language ? <Fact label={t('detail.fact_language')} value={language} /> : null}
        {displayMeta.status ? <Fact label={t('detail.fact_status')} value={displayMeta.status} /> : null}
        {displayMeta.productionCountries?.length ? (
          <Fact label={t('detail.fact_country')} value={displayMeta.productionCountries.slice(0, 2).join(', ')} />
        ) : null}
      </div>
    </aside>
  );
}
