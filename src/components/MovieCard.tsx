import React, { useCallback, useEffect, useRef, useState } from 'react';
import { Plus, Check } from 'lucide-react';
import { t } from '../i18n';
import type { Meta } from '../core/types';
import type { PosterPrefs } from '../core/posterPrefs';
import { rpdbPosterUrl } from '../core/rpdb';
import { cardImageUrl } from '../core/imageSizes';
import { ContextMenu } from './ui/ContextMenu';
import { PosterPreviewCard } from './PosterPreviewCard';

interface Props {
  meta: Meta;
  width?: number;
  height?: number;
  radius?: number;
  hideTitle?: boolean;
  layout?: PosterPrefs['layout'];
  topTenRank?: number;
  addonIcon?: string;
  preview?: boolean;
  onClick?: (meta: Meta) => void;
  onDispatch?: (actionJson: string) => void | Promise<void>;
}

export const MovieCard = React.memo(
  function MovieCard({
    meta,
    width = 156,
    height = 234,
    radius = 12,
    hideTitle = false,
    layout = 'vertical',
    topTenRank,
    addonIcon,
    preview = false,
    onClick,
    onDispatch,
  }: Props) {
    const [imgError, setImgError] = useState(false);
    const [imgLoaded, setImgLoaded] = useState(false);
    const [rpdbFailed, setRpdbFailed] = useState(false);
    const [menuPoint, setMenuPoint] = useState<{ x: number; y: number } | null>(null);
    const [previewAnchor, setPreviewAnchor] = useState<DOMRect | null>(null);
    const previewTimerRef = useRef<number | null>(null);

    const clearPreviewTimer = useCallback(() => {
      if (previewTimerRef.current !== null) {
        window.clearTimeout(previewTimerRef.current);
        previewTimerRef.current = null;
      }
    }, []);

    useEffect(() => clearPreviewTimer, [clearPreviewTimer]);

    const openPreview = useCallback(
      (target: HTMLElement) => {
        if (!preview) return;
        clearPreviewTimer();
        previewTimerRef.current = window.setTimeout(() => setPreviewAnchor(target.getBoundingClientRect()), 650);
      },
      [preview, clearPreviewTimer],
    );

    const closePreview = useCallback(() => {
      clearPreviewTimer();
      setPreviewAnchor(null);
    }, [clearPreviewTimer]);

    const anyMeta = meta as unknown as Record<string, unknown>;
    const isWatched = anyMeta.watched === true || anyMeta.notWatched === false;
    const timeOffset = typeof anyMeta.timeOffset === 'number' ? anyMeta.timeOffset : 0;
    const duration = typeof anyMeta.duration === 'number' ? anyMeta.duration : 0;
    const progressRatio = timeOffset > 0 && duration > 0 ? timeOffset / duration : 0;

    // fontSize * ~0.72 = visual cap height, so to fill `height` visually: fontSize = height / 0.72 ≈ height * 1.4
    const rankFontSize = height * 1.4;
    const rankStroke = Math.max(3, rankFontSize * 0.018);
    const rankNumWidth =
      topTenRank != null ? (topTenRank >= 10 ? rankFontSize * 1.08 : topTenRank === 1 ? rankFontSize * 0.52 : rankFontSize * 0.64) : 0;
    const rankOverlap = rankNumWidth * 0.22;
    const outerWidth = topTenRank != null ? Math.round(rankNumWidth + width - rankOverlap) : width;

    return (
      <div style={{ width: outerWidth, display: 'flex', flexDirection: 'column', flexShrink: 0 }}>
        {/* Poster row — height is always just the poster height */}
        <div style={{ display: 'flex', alignItems: 'flex-end', height, position: 'relative', overflow: 'visible' }}>
          {topTenRank != null && (
            <div
              style={{
                position: 'absolute',
                left: 0,
                bottom: 0,
                width: Math.round(rankNumWidth),
                height,
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
                overflow: 'hidden',
                pointerEvents: 'none',
              }}
            >
              <span
                style={{
                  fontSize: rankFontSize,
                  fontWeight: 900,
                  fontFamily: '"Arial Black", "Arial Bold", Arial, sans-serif',
                  lineHeight: 1,
                  color: 'transparent',
                  WebkitTextStroke: `${rankStroke}px rgba(255,255,255,0.45)`,
                  userSelect: 'none',
                  letterSpacing: '-0.05em',
                  display: 'block',
                }}
              >
                {topTenRank}
              </span>
            </div>
          )}
          {/* Poster card */}
          <div
            role="button"
            tabIndex={0}
            className="movie-card"
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
            onClick={() => {
              closePreview();
              onClick?.(meta);
            }}
            onMouseEnter={(e) => openPreview(e.currentTarget)}
            onMouseLeave={closePreview}
            onKeyDown={(e) => {
              if (e.key === 'Enter' || e.key === ' ') {
                e.preventDefault();
                onClick?.(meta);
              }
            }}
            onContextMenu={(e) => {
              if (!onDispatch) return;
              e.preventDefault();
              setMenuPoint({ x: e.clientX, y: e.clientY });
            }}
          >
            {/* Poster image */}
            {(() => {
              const dpr = window.devicePixelRatio;
              const basePosterSrc =
                layout === 'horizontal'
                  ? cardImageUrl(meta.background, { kind: 'backdrop', displayWidth: width, dpr }) ||
                    cardImageUrl(meta.poster, { displayWidth: width, dpr })
                  : cardImageUrl(meta.poster, { displayWidth: width, dpr }) ||
                    cardImageUrl(meta.background, { kind: 'backdrop', displayWidth: width, dpr });
              const rpdbSrc = layout !== 'horizontal' ? rpdbPosterUrl(meta) : undefined;
              const posterSrc = rpdbSrc && !rpdbFailed ? rpdbSrc : basePosterSrc;
              if (posterSrc && !imgError) {
                return (
                  <img
                    src={posterSrc}
                    alt={meta.name}
                    loading="lazy"
                    decoding="async"
                    draggable={false}
                    style={{
                      width: '100%',
                      height: '100%',
                      objectFit: 'cover',
                      display: 'block',
                      opacity: imgLoaded ? 1 : 0,
                      transition: 'opacity 0.2s ease',
                    }}
                    onLoad={() => setImgLoaded(true)}
                    onError={() => {
                      if (posterSrc === rpdbSrc) setRpdbFailed(true);
                      else setImgError(true);
                    }}
                  />
                );
              }
              return (
                <div
                  style={{
                    width: '100%',
                    height: '100%',
                    background: '#12161D',
                    display: 'flex',
                    alignItems: 'center',
                    justifyContent: 'center',
                  }}
                >
                  {addonIcon ? (
                    <img src={addonIcon} alt="" style={{ width: '48%', height: '48%', objectFit: 'contain', opacity: 0.35 }} />
                  ) : (
                    <span style={{ color: 'rgba(255,255,255,0.2)', fontSize: '2rem', fontWeight: 900 }}>
                      {(meta.name ?? '').slice(0, 2).toUpperCase()}
                    </span>
                  )}
                </div>
              );
            })()}

            <div className="mc-dim" />

            {/* Watched overlay */}
            {isWatched && (
              <div
                className="mc-watched"
                style={{
                  position: 'absolute',
                  inset: 0,
                  background: 'rgba(0,0,0,0.55)',
                  display: 'flex',
                  flexDirection: 'column',
                  alignItems: 'center',
                  justifyContent: 'center',
                  gap: '0.625rem',
                  pointerEvents: 'none',
                }}
              >
                <WatchedCheckmark />
                <span
                  style={{
                    color: '#FFFFFF',
                    fontSize: '0.8125rem',
                    fontWeight: 700,
                    letterSpacing: '0.0313rem',
                  }}
                >
                  {t('auto.watched')}
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
                  height: '0.1875rem',
                  background: 'rgba(255,255,255,0.22)',
                }}
              >
                <div
                  style={{
                    height: '100%',
                    width: `${Math.min(progressRatio, 1) * 100}%`,
                    background: '#E85D3F',
                    borderRadius: '0 0.125rem 0.125rem 0',
                  }}
                />
              </div>
            )}

            {/* Upcoming badge */}
            {meta.releaseInfo && isUpcoming(meta.releaseInfo) && (
              <div
                style={{
                  position: 'absolute',
                  top: '0.5rem',
                  right: '0.5rem',
                  background: 'rgba(0,0,0,0.72)',
                  borderRadius: '0.3125rem',
                  padding: '0.125rem 0.4375rem',
                }}
              >
                <span
                  style={{
                    color: 'rgba(255,255,255,0.7)',
                    fontSize: '0.625rem',
                    fontWeight: 700,
                    letterSpacing: '0.0313rem',
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
              margin: '0.5rem 0 0',
              marginLeft: topTenRank != null ? Math.round(rankNumWidth - rankOverlap) : 0,
              width: topTenRank != null ? width : '100%',
              display: 'flex',
              flexDirection: 'column',
              alignItems: 'flex-start',
              gap: '0.125rem',
              minWidth: 0,
            }}
          >
            <p
              style={{
                color: '#FFFFFF',
                fontSize: '0.8125rem',
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
              <p style={{ color: 'rgba(255,255,255,0.45)', fontSize: '0.6875rem', fontWeight: 400, margin: 0, lineHeight: 1.3 }}>
                {meta.year}
              </p>
            )}
          </div>
        )}
        {previewAnchor && (
          <PosterPreviewCard
            meta={meta}
            anchor={previewAnchor}
            onOpenDetail={() => {
              closePreview();
              onClick?.(meta);
            }}
            onToggleLibrary={
              onDispatch
                ? () => {
                    void onDispatch(JSON.stringify({ type: 'toggleWatchlistRequested', item: meta }));
                    closePreview();
                  }
                : undefined
            }
            onMouseEnter={clearPreviewTimer}
            onMouseLeave={closePreview}
          />
        )}
        {onDispatch && (
          <ContextMenu
            point={menuPoint}
            onClose={() => setMenuPoint(null)}
            items={[
              {
                icon: <Plus size={15} />,
                label: t('common.add_to_library'),
                onSelect: () => onDispatch(JSON.stringify({ type: 'toggleWatchlistRequested', item: meta })),
              },
              {
                icon: <Check size={15} />,
                label: t('detail.mark_watched'),
                onSelect: () => onDispatch(JSON.stringify({ type: 'toggleLibraryStatusRequested', list: 'completed', item: meta })),
              },
            ]}
          />
        )}
      </div>
    );
  },
  (prev, next) => {
    if (prev.width !== next.width || prev.height !== next.height || prev.radius !== next.radius) return false;
    if (prev.hideTitle !== next.hideTitle || prev.layout !== next.layout) return false;
    if (prev.topTenRank !== next.topTenRank || prev.addonIcon !== next.addonIcon) return false;
    if (prev.preview !== next.preview) return false;
    if (prev.onClick !== next.onClick || prev.onDispatch !== next.onDispatch) return false;
    if (prev.meta === next.meta) return true;
    const pm = prev.meta as unknown as Record<string, unknown>;
    const nm = next.meta as unknown as Record<string, unknown>;
    if (pm.id !== nm.id || pm.name !== nm.name) return false;
    if (pm.poster !== nm.poster || pm.background !== nm.background || pm.year !== nm.year) return false;
    if (pm.releaseInfo !== nm.releaseInfo) return false;
    if (pm.watched !== nm.watched || pm.notWatched !== nm.notWatched) return false;
    if (pm.timeOffset !== nm.timeOffset || pm.duration !== nm.duration) return false;
    return true;
  },
);

function WatchedCheckmark() {
  return (
    <div
      style={{
        width: '2rem',
        height: '1.25rem',
        borderLeft: '0.25rem solid #FFFFFF',
        borderBottom: '0.25rem solid #FFFFFF',
        transform: 'rotate(-45deg)',
        boxSizing: 'border-box',
        marginBottom: '0.25rem',
      }}
    />
  );
}

function isUpcoming(releaseInfo?: string): boolean {
  if (!releaseInfo) return false;
  const year = parseInt(String(releaseInfo).slice(0, 4));
  return year > new Date().getFullYear();
}
