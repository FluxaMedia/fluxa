import React from 'react';
import { MovieCard } from '../MovieCard';
import { t } from '../../i18n';
import type { Meta, MetaLink, Trailer } from '../../core/types';
import type { posterPrefsFromState } from '../../core/posterPrefs';
import { MS, S } from './detailStyles';
import { CastAvatar, type NormalizedCastMember } from './castSection';
import { TrailerCarousel, type TrailerMetadata } from './TrailerCarousel';
import { SimilarSourcePicker } from './ModernDetailParts';

export function DetailsTabContent({
  displayMeta,
  castMembers,
  directorLinks,
  peopleImages,
  displayTrailers,
  trailerMetadata,
}: {
  displayMeta: Meta;
  castMembers: NormalizedCastMember[];
  directorLinks: MetaLink[];
  peopleImages: Record<string, string>;
  displayTrailers: Trailer[];
  trailerMetadata: TrailerMetadata;
}) {
  return (
    <div style={{ ...MS.detailsTab, minHeight: '12.5rem' }}>
      {displayMeta.description && (
        <div style={MS.detailsSection}>
          <h3 style={MS.detailsSectionTitle}>{t('detail.summary')}</h3>
          <p style={MS.detailsText}>{displayMeta.description}</p>
        </div>
      )}
      {displayMeta.awards && (
        <div style={MS.detailsSection}>
          <h3 style={MS.detailsSectionTitle}>{t('detail.awards')}</h3>
          <p style={{ ...MS.detailsText, color: '#54D17A', fontWeight: 700 }}>{displayMeta.awards}</p>
        </div>
      )}
      {(castMembers.length > 0 || directorLinks.length > 0) && (
        <div style={MS.detailsSection}>
          <h3 style={MS.detailsSectionTitle}>{t('detail.cast_crew')}</h3>
          <div style={S.castRow}>
            {directorLinks.map((l) => <CastAvatar key={`dir-${l.name}`} name={l.name} role={t('detail.director')} imageUrl={peopleImages[l.name]} />)}
            {castMembers.map((member) => <CastAvatar key={`cast-${member.name}:${member.role ?? ''}`} name={member.name} role={member.role || t('detail.actor')} imageUrl={member.imageUrl ?? peopleImages[member.name]} />)}
          </div>
        </div>
      )}
      {displayTrailers.length > 0 && (
        <div style={MS.detailsSection}>
          <h3 style={MS.detailsSectionTitle}>{t('auto.trailers')}</h3>
          <TrailerCarousel trailers={displayTrailers} trailerMetadata={trailerMetadata} />
        </div>
      )}
    </div>
  );
}

export function RelatedTabContent({
  similarSource,
  onChangeSimilarSource,
  similarItems,
  poster,
  onNavigateDetail,
}: {
  similarSource: string;
  onChangeSimilarSource: (source: string) => void;
  similarItems: Meta[];
  poster: ReturnType<typeof posterPrefsFromState>;
  onNavigateDetail: (meta: Meta) => void;
}) {
  return (
    <div style={{ ...MS.relatedSection, minHeight: '12.5rem' }}>
      <SimilarSourcePicker value={similarSource} onChange={onChangeSimilarSource} />
      {similarItems.length === 0 ? (
        <p style={MS.episodeCount}>{t('auto.no_similar_titles')}</p>
      ) : (
        <div style={MS.relatedGrid}>
          {similarItems.slice(0, 24).map((item) => (
            <MovieCard key={`${item.type}:${item.id}`} meta={item} width={poster.width} height={poster.height} radius={poster.radius} hideTitle={poster.hideTitles} layout={poster.layout} onClick={onNavigateDetail} />
          ))}
        </div>
      )}
    </div>
  );
}
