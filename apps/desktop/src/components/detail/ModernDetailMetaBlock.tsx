import React, { useEffect, useRef, useState } from 'react';
import { t } from '../../i18n';
import { color, fontSize, weight } from '../../design';
import { MS } from './detailStyles';
import { AgeBadge } from './DetailButtons';
import { GenreTag } from './ModernDetailParts';

export function ModernDetailMetaBlock({
  certification,
  metaDetails,
  description,
  genres,
  ratings,
  onNavigateGenre,
}: {
  certification?: string;
  metaDetails: string[];
  description?: string;
  genres: string[];
  ratings?: React.ReactNode;
  onNavigateGenre?: (genre: string) => void;
}) {
  const [expanded, setExpanded] = useState(false);
  const [clamped, setClamped] = useState(false);
  const [unclamp, setUnclamp] = useState(false);
  const descRef = useRef<HTMLParagraphElement | null>(null);
  useEffect(() => {
    setExpanded(false);
    setUnclamp(false);
    const el = descRef.current;
    if (!el) return;
    const measure = () => {
      const line = parseFloat(getComputedStyle(el).lineHeight) || 20;
      const hidden = el.scrollHeight - el.clientHeight;
      if (hidden > line * 1.05) {
        setClamped(true);
      } else if (hidden > 2) {
        setUnclamp(true);
        setClamped(false);
      } else {
        setClamped(false);
      }
    };
    measure();
    const observer = new ResizeObserver(measure);
    observer.observe(el);
    return () => observer.disconnect();
  }, [description]);
  const [primary, ...secondary] = metaDetails;
  return (
    <div style={MS.metaBlock}>
      {(metaDetails.length > 0 || certification || genres.length > 0) && (
        <div style={{ ...MS.metaChipRow, marginBottom: '0.75rem' }}>
          {primary && <span style={{ fontSize: fontSize.base, fontWeight: weight.semibold, color: color.textStrong }}>{primary}</span>}
          {certification && <AgeBadge label={certification} />}
          {secondary.map((detail) => (
            <span key={detail} style={{ fontSize: fontSize.base, fontWeight: weight.medium, color: color.textMuted }}>
              {detail}
            </span>
          ))}
          {genres.map((genre) => (
            <GenreTag key={genre} label={genre} onClick={() => onNavigateGenre?.(genre)} />
          ))}
        </div>
      )}
      {ratings && <div style={{ marginBottom: '0.875rem' }}>{ratings}</div>}
      {description && (
        <>
          <p ref={descRef} style={expanded || unclamp ? { ...MS.descText, WebkitLineClamp: 'unset', overflow: 'visible' } : MS.descText}>
            {description}
          </p>
          {!unclamp && (clamped || expanded) && (
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
