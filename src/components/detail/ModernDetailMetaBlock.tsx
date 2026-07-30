import React from 'react';
import { MS } from './detailStyles';
import { GenreTag } from './ModernDetailParts';
import { RatingsRow } from './RatingBadge';

export function ModernDetailMetaBlock({
  mdblistRatings,
  metaGenres,
  onNavigateGenre,
  metaDetails,
  description,
}: {
  mdblistRatings?: Record<string, number> | null;
  metaGenres: string[];
  onNavigateGenre?: (genre: string) => void;
  metaDetails: string[];
  description?: string;
}) {
  const hasMdblistRatings = mdblistRatings != null && Object.keys(mdblistRatings).length > 0;
  return (
    <div style={MS.metaBlock}>
      {hasMdblistRatings && (
        <div style={{ marginBottom: '0.625rem' }}>
          <RatingsRow ratings={mdblistRatings} />
        </div>
      )}
      {(metaGenres.length > 0 || metaDetails.length > 0) && (
        <p style={MS.metaInfoLine}>
          {metaGenres.map((g, i) => (
            <React.Fragment key={g}>
              <GenreTag label={g} onClick={() => onNavigateGenre?.(g)} />
              {(i < metaGenres.length - 1 || metaDetails.length > 0) && <span style={MS.metaDot}> • </span>}
            </React.Fragment>
          ))}
          {metaDetails.length > 0 && <span style={MS.metaDetailsText}>{metaDetails.join(' • ')}</span>}
        </p>
      )}
      {description && <p style={MS.descText}>{description}</p>}
    </div>
  );
}
