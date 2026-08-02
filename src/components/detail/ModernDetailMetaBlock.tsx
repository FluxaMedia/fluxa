import React, { useState } from 'react';
import { MS } from './detailStyles';
import { GenreTag } from './ModernDetailParts';
import { RatingsRow } from './RatingBadge';
import { t } from '../../i18n';

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
  const [expanded, setExpanded] = useState(false);
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
      {description && (
        <>
          <p style={expanded ? { ...MS.descText, WebkitLineClamp: 'unset', overflow: 'visible' } : MS.descText}>{description}</p>
          {description.length > 180 && (
            <button
              onClick={() => setExpanded((value) => !value)}
              style={{ background: 'none', border: 'none', color: 'rgba(255,255,255,0.7)', fontSize: '0.8125rem', fontWeight: 700, cursor: 'pointer', padding: 0, marginTop: '-0.375rem', marginBottom: '0.75rem' }}
            >
              {expanded ? t('detail.read_less') : t('detail.read_more')}
            </button>
          )}
        </>
      )}
    </div>
  );
}
