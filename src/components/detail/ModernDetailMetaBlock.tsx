import React, { useState } from 'react';
import { t } from '../../i18n';
import { color, fontSize, weight } from '../../design';
import { MS } from './detailStyles';
import { AgeBadge } from './DetailButtons';

export function ModernDetailMetaBlock({
  certification,
  metaDetails,
  description,
}: {
  certification?: string;
  metaDetails: string[];
  description?: string;
}) {
  const [expanded, setExpanded] = useState(false);
  const [primary, ...secondary] = metaDetails;
  return (
    <div style={MS.metaBlock}>
      {(metaDetails.length > 0 || certification) && (
        <div style={{ ...MS.metaChipRow, marginBottom: '0.75rem' }}>
          {primary && <span style={{ fontSize: fontSize.base, fontWeight: weight.semibold, color: color.textStrong }}>{primary}</span>}
          {certification && <AgeBadge label={certification} />}
          {secondary.map((detail) => (
            <span key={detail} style={{ fontSize: fontSize.base, fontWeight: weight.medium, color: color.textMuted }}>
              {detail}
            </span>
          ))}
        </div>
      )}
      {description && (
        <>
          <p style={expanded ? { ...MS.descText, WebkitLineClamp: 'unset', overflow: 'visible' } : MS.descText}>{description}</p>
          {description.length > 180 && (
            <button
              onClick={() => setExpanded((value) => !value)}
              style={{
                background: 'none',
                border: 'none',
                color: color.textBody,
                fontSize: fontSize.base,
                fontWeight: weight.bold,
                cursor: 'pointer',
                padding: 0,
              }}
            >
              {expanded ? t('detail.read_less') : t('detail.read_more')}
            </button>
          )}
        </>
      )}
    </div>
  );
}
