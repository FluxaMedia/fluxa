import React, { useState } from 'react';
import type { Meta } from '../core/types';
import type { PosterPrefs } from '../core/posterPrefs';

interface Props {
  meta: Meta;
  width?: number;
  height?: number;
  radius?: number;
  hideTitle?: boolean;
  layout?: PosterPrefs['layout'];
  topTenRank?: number;
  onClick?: (meta: Meta) => void;
}

export const MovieCard = React.memo(function MovieCard({
  meta,
  width = 156,
  height = 234,
  radius = 12,
  hideTitle = false,
  layout = 'vertical',
  topTenRank,
  onClick,
}: Props) {
  const [hovered, setHovered] = useState(false);
  const [imgError, setImgError] = useState(false);

  const anyMeta = meta as unknown as Record<string, unknown>;
  const isWatched = anyMeta.watched === true || anyMeta.notWatched === false;
  const timeOffset = typeof anyMeta.timeOffset === 'number' ? anyMeta.timeOffset : 0;
  const duration = typeof anyMeta.duration === 'number' ? anyMeta.duration : 0;
  const progressRatio = timeOffset > 0 && duration > 0 ? timeOffset / duration : 0;

  // fontSize * ~0.72 = visual cap height, so to fill `height` visually: fontSize = height / 0.72 ≈ height * 1.4
  const numTotalHeight = height;
  const rankFontSize = height * 1.4;
  const rankStroke = Math.max(3, rankFontSize * 0.018);
  const rankNumWidth = topTenRank != null
    ? (topTenRank >= 10 ? rankFontSize * 1.08 : topTenRank === 1 ? rankFontSize * 0.52 : rankFontSize * 0.64)
    : 0;
  const rankOverlap = rankNumWidth * 0.22;
  const outerWidth = topTenRank != null ? Math.round(rankNumWidth + width - rankOverlap) : width;

  return (
    <div style={{ width: outerWidth, display: 'flex', flexDirection: 'column', flexShrink: 0 }}>
      {/* Poster row — height is always just the poster height */}
      <div style={{ display: 'flex', alignItems: 'flex-end', height, position: 'relative', overflow: 'visible' }}>
        {topTenRank != null && (
          <div style={{
            position: 'absolute',
            left: 0,
            bottom: 0,
            width: Math.round(rankNumWidth),
            height: numTotalHeight,
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            overflow: 'hidden',
            pointerEvents: 'none',
          }}>
            <span style={{
              fontSize: rankFontSize,
              fontWeight: 900,
              fontFamily: '"Arial Black", "Arial Bold", Arial, sans-serif',
              lineHeight: 1,
              color: 'transparent',
              WebkitTextStroke: `${rankStroke}px rgba(255,255,255,0.45)`,
              userSelect: 'none',
              letterSpacing: '-0.05em',
              display: 'block',
            }}>{topTenRank}</span>
          </div>
        )}
      {/* Poster card */}
      <div
        role="button"
        tabIndex={0}
        style={{
          width,
          height,
          borderRadius: radius,
          overflow: 'hidden',
          position: 'relative',
          background: '#12161D',
          border: '1px solid transparent',
          transform: 'none',
          transition: 'border-color 0.18s ease-out',
          boxShadow: 'none',
          zIndex: 1,
          cursor: 'pointer',
          outline: 'none',
          flexShrink: 0,
          marginLeft: topTenRank != null ? Math.round(rankNumWidth - rankOverlap) : 0,
        }}
        onMouseEnter={() => setHovered(true)}
        onMouseLeave={() => setHovered(false)}
        onFocus={() => setHovered(true)}
        onBlur={() => setHovered(false)}
        onClick={() => onClick?.(meta)}
        onKeyDown={(e) => {
          if (e.key === 'Enter' || e.key === ' ') {
            e.preventDefault();
            onClick?.(meta);
          }
        }}
      >
        {/* Poster image */}
        {(layout === 'horizontal' ? meta.background || meta.poster : meta.poster || meta.background) && !imgError ? (
          <img
            src={(layout === 'horizontal' ? meta.background || meta.poster : meta.poster || meta.background) ?? ''}
            alt={meta.name}
            loading="lazy"
            decoding="async"
            style={{
              width: '100%',
              height: '100%',
              objectFit: 'cover',
              display: 'block',
            }}
            onError={() => setImgError(true)}
          />
        ) : (
          <div
            style={{
              width: '100%',
              height: '100%',
              background: 'linear-gradient(135deg, #1B212B, #12161D)',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
            }}
          >
            <span
              style={{
                color: 'rgba(255,255,255,0.2)',
                fontSize: 32,
                fontWeight: 900,
              }}
            >
              {meta.name.slice(0, 2).toUpperCase()}
            </span>
          </div>
        )}

        {hovered && (
          <div
            style={{
              position: 'absolute',
              inset: 0,
              background: 'rgba(0,0,0,0.28)',
              pointerEvents: 'none',
            }}
          />
        )}

        {/* Watched overlay */}
        {isWatched && (
          <div
            style={{
              position: 'absolute',
              inset: 0,
              background: 'rgba(0,0,0,0.55)',
              display: 'flex',
              flexDirection: 'column',
              alignItems: 'center',
              justifyContent: 'center',
              gap: 10,
              opacity: hovered ? 0 : 1,
              transition: 'opacity 0.25s ease',
              pointerEvents: 'none',
            }}
          >
            <WatchedCheckmark />
            <span
              style={{
                color: '#FFFFFF',
                fontSize: 13,
                fontWeight: 700,
                letterSpacing: '0.5px',
              }}
            >
              Watched
            </span>
          </div>
        )}

        {/* Progress bar */}
        {progressRatio > 0.01 && progressRatio < 0.99 && (
          <div
            style={{
              position: 'absolute',
              bottom: 0,
              left: 0,
              right: 0,
              height: 3,
              background: 'rgba(255,255,255,0.22)',
            }}
          >
            <div
              style={{
                height: '100%',
                width: `${Math.min(progressRatio, 1) * 100}%`,
                background: '#E85D3F',
                borderRadius: '0 2px 2px 0',
              }}
            />
          </div>
        )}

        {/* Upcoming badge */}
        {meta.releaseInfo && isUpcoming(meta.releaseInfo) && (
          <div
            style={{
              position: 'absolute',
              top: 8,
              right: 8,
              background: 'rgba(0,0,0,0.72)',
              borderRadius: 5,
              padding: '2px 7px',
            }}
          >
            <span
              style={{
                color: 'rgba(255,255,255,0.7)',
                fontSize: 10,
                fontWeight: 700,
                letterSpacing: '0.5px',
              }}
            >
              UPCOMING
            </span>
          </div>
        )}
      </div>
      </div>

      {!hideTitle && (
        <div
          style={{
            margin: '8px 0 0',
            marginLeft: topTenRank != null ? Math.round(rankNumWidth - rankOverlap) : 0,
            width: topTenRank != null ? width : '100%',
            display: 'flex',
            flexDirection: 'column',
            alignItems: 'flex-start',
            gap: 2,
            minWidth: 0,
          }}
        >
          <p
            style={{
              color: '#FFFFFF',
              fontSize: 13,
              fontWeight: 700,
              margin: 0,
              overflow: 'hidden',
              textOverflow: 'ellipsis',
              whiteSpace: 'nowrap',
              lineHeight: 1.3,
              width: '100%',
              textAlign: 'left',
            }}
            title={meta.name}
          >
            {meta.name}
          </p>
          {meta.year && (
            <p style={{ color: 'rgba(255,255,255,0.45)', fontSize: 11, fontWeight: 400, margin: 0, lineHeight: 1.3 }}>
              {meta.year}
            </p>
          )}
        </div>
      )}
    </div>
  );
});

function WatchedCheckmark() {
  return (
    <div
      style={{
        width: 32,
        height: 20,
        borderLeft: '4px solid #FFFFFF',
        borderBottom: '4px solid #FFFFFF',
        transform: 'rotate(-45deg)',
        boxSizing: 'border-box',
        marginBottom: 4,
      }}
    />
  );
}

function isUpcoming(releaseInfo?: string): boolean {
  if (!releaseInfo) return false;
  const year = parseInt(releaseInfo.slice(0, 4));
  return year > new Date().getFullYear();
}
