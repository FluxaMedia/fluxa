import React, { useEffect, useMemo, useState } from 'react';
import { Info, Play, Plus } from '@phosphor-icons/react';
import { seasonPosterUrl } from '../core/seasonPosters';
import type { Meta } from '../core/types';
import { t } from '../i18n';

interface Props {
  meta: Meta;
  slides?: Meta[];
  onPlay?: (meta: Meta) => void;
  onDetails?: (meta: Meta) => void;
  onAddToWatchlist?: (meta: Meta) => void;
  preferSeasonPosters?: boolean;
}

const SLIDE_INTERVAL_MS = 6500;
const PANEL_LEFT = 120;

export function HeroSection({ meta, slides, onPlay, onDetails, onAddToWatchlist, preferSeasonPosters = false }: Props) {
  const items = useMemo(() => {
    const seen = new Set<string>();
    return [meta, ...(slides ?? [])].filter((item) => {
      const key = item.id || item.name;
      if (seen.has(key)) return false;
      seen.add(key);
      return !!(item.background || item.poster || seasonPosterUrl(item));
    });
  }, [meta, slides]);

  const [activeIndex, setActiveIndex] = useState(0);
  const [bgError, setBgError] = useState(false);
  const [logoError, setLogoError] = useState(false);

  const activeMeta = items[activeIndex] ?? meta;
  const canSlide = items.length > 1;
  const imageUrl = (preferSeasonPosters ? seasonPosterUrl(activeMeta) : undefined) ?? activeMeta.background ?? activeMeta.poster;
  const bgUrl = !bgError ? imageUrl : null;
  const logoUrl = !logoError ? activeMeta.logo : null;

  const imdbNum = activeMeta.imdbRating != null ? Number(activeMeta.imdbRating) : NaN;
  const releaseYear = activeMeta.year ?? parseReleaseYear(activeMeta.releaseInfo);
  const tagline = readOptionalString(activeMeta, ['tagline', 'tagLine', 'slogan']);
  const awards = readOptionalString(activeMeta, ['awards']);
  const certification = readOptionalString(activeMeta, ['certification', 'contentRating', 'rating']);
  const network = readOptionalString(activeMeta, ['network', 'studio', 'broadcaster']);

  const metaParts: string[] = [];
  if (releaseYear) metaParts.push(String(releaseYear));
  if (activeMeta.runtime) metaParts.push(String(activeMeta.runtime));
  if (network) metaParts.push(network);

  const genreLine = (activeMeta.genres ?? [])
    .filter((g): g is string => typeof g === 'string' && g.length > 0)
    .slice(0, 5);

  useEffect(() => {
    setBgError(false);
    setLogoError(false);
  }, [activeMeta.id, imageUrl, activeMeta.logo]);

  useEffect(() => {
    if (!canSlide) return;
    const id = window.setInterval(() => {
      setActiveIndex((idx) => (idx + 1) % items.length);
    }, SLIDE_INTERVAL_MS);
    return () => window.clearInterval(id);
  }, [canSlide, items.length]);

  const goTo = (index: number) => {
    if (!canSlide) return;
    setActiveIndex((index + items.length) % items.length);
  };

  return (
    <div style={styles.hero}>
      {/* Background image */}
      {bgUrl && (
        <img
          key={activeMeta.id || activeMeta.name}
          src={bgUrl}
          alt=""
          decoding="async"
          style={styles.backdrop}
          onError={() => setBgError(true)}
        />
      )}

      {/* Gradient overlays */}
      <div style={styles.gradientTop} />
      <div style={styles.gradientLeft} />
      <div style={styles.gradientBottom} />

      {/* Hero panel */}
      <div style={styles.panel}>
        {/* Logo or fallback title */}
        {logoUrl ? (
          <img
            src={logoUrl}
            alt={activeMeta.name}
            decoding="async"
            style={styles.logo}
            onError={() => setLogoError(true)}
          />
        ) : (
          <h1 style={styles.title}>{String(activeMeta.name ?? '')}</h1>
        )}

        {/* Tagline */}
        {tagline && <p style={styles.tagline}>{tagline}</p>}

        {/* Meta info row */}
        {metaParts.length > 0 && (
          <p style={styles.metaLine}>{metaParts.join(' · ')}</p>
        )}

        {/* Genre tags */}
        {(certification || genreLine.length > 0) && (
          <div style={styles.genreRow}>
            {certification && (
              <span style={styles.certBadge}>{certification}</span>
            )}
            {genreLine.map((g) => (
              <span key={g} style={styles.genrePill}>{g}</span>
            ))}
          </div>
        )}

        {/* Description */}
        {activeMeta.description && (
          <p style={styles.description}>{activeMeta.description}</p>
        )}

        {awards && <p style={styles.awards}>{awards}</p>}

        {/* Action row */}
        <div style={{ display: 'flex', alignItems: 'center', gap: 10, marginTop: 24, alignSelf: 'flex-start' }}>
          <button style={styles.watchBtn} onClick={() => onPlay?.(activeMeta)}>
            <Play size={13} weight="fill" />
            {t('common.play')}
          </button>
          <HeroIconBtn onClick={() => onAddToWatchlist?.(activeMeta)} title={t('auto.my_list')}>
            <Plus size={20} weight="bold" />
          </HeroIconBtn>
          <HeroIconBtn onClick={() => onDetails?.(activeMeta)} title={t('auto.info')}>
            <Info size={20} weight="bold" />
          </HeroIconBtn>
        </div>
      </div>

      {/* Navigation arrows */}
      {canSlide && (
        <>
          <NavArrow direction="left" onClick={() => goTo(activeIndex - 1)} />
          <NavArrow direction="right" onClick={() => goTo(activeIndex + 1)} />
          <div style={styles.indicators}>
            {items.map((item, i) => (
              <button
                key={item.id || item.name}
                aria-label={`Show ${item.name}`}
                style={{
                  ...styles.indicator,
                  ...(i === activeIndex ? styles.indicatorActive : null),
                }}
                onClick={() => goTo(i)}
              />
            ))}
          </div>
        </>
      )}
    </div>
  );
}

function NavArrow({ direction, onClick }: { direction: 'left' | 'right'; onClick: () => void }) {
  const [hovered, setHovered] = useState(false);
  return (
    <button
      aria-label={direction === 'left' ? 'Previous' : 'Next'}
      style={{
        position: 'absolute',
        top: 'calc(29vh - 40px)',
        ...(direction === 'left' ? { left: 20 } : { right: 14 }),
        transform: 'translateY(-50%)',
        background: 'transparent',
        border: 'none',
        color: hovered ? 'rgba(255,255,255,0.9)' : 'rgba(255,255,255,0.35)',
        fontSize: 48,
        fontWeight: 300,
        fontFamily: 'system-ui, sans-serif',
        width: 60,
        height: 70,
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        cursor: 'pointer',
        padding: 0,
        zIndex: 15,
        lineHeight: 1,
        transition: 'color 0.3s ease, transform 0.3s ease, text-shadow 0.3s ease',
        textShadow: hovered
          ? '0 0 15px rgba(255,255,255,0.8), 0 0 25px rgba(255,255,255,0.5), 2px 2px 3px rgba(0,0,0,0.9)'
          : '2px 2px 4px rgba(0,0,0,0.8)',
      }}
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
      onClick={(e) => { e.stopPropagation(); onClick(); }}
    >
      {direction === 'left' ? '‹' : '›'}
    </button>
  );
}

function HeroIconBtn({ onClick, title, children }: { onClick?: () => void; title?: string; children: React.ReactNode }) {
  const [hovered, setHovered] = useState(false);
  return (
    <button
      title={title}
      onClick={onClick}
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
      style={{
        width: 42,
        height: 42,
        borderRadius: '50%',
        background: hovered ? 'rgba(255,255,255,0.18)' : 'rgba(255,255,255,0.10)',
        border: '2px solid rgba(255,255,255,0.55)',
        color: '#FFFFFF',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        cursor: 'pointer',
        padding: 0,
        flexShrink: 0,
        transition: 'background 0.15s',
        boxShadow: '0 4px 16px rgba(0,0,0,0.35)',
      }}
    >
      {children}
    </button>
  );
}

function parseReleaseYear(releaseInfo?: string): number | null {
  const match = releaseInfo?.match(/\b(19|20)\d{2}\b/);
  return match ? Number(match[0]) : null;
}

function readOptionalString(meta: Meta, keys: string[]): string | null {
  const record = meta as unknown as Record<string, unknown>;
  for (const key of keys) {
    const value = record[key];
    if (typeof value === 'string' && value.trim()) return value.trim();
  }
  return null;
}

const styles: Record<string, React.CSSProperties> = {
  hero: {
    position: 'relative',
    width: '100%',
    height: '100vh',
    minHeight: 720,
    maxHeight: 1080,
    overflow: 'hidden',
    flexShrink: 0,
    background: '#040508',
  },
  backdrop: {
    position: 'absolute',
    top: 0,
    right: 0,
    width: '100%',
    height: '100%',
    objectFit: 'cover',
    objectPosition: 'center 20%',
    display: 'block',
    userSelect: 'none',
    pointerEvents: 'none',
  },
  gradientTop: {
    position: 'absolute',
    inset: 0,
    background: 'linear-gradient(to bottom, rgba(4,5,8,0.55) 0%, rgba(4,5,8,0.20) 12%, rgba(4,5,8,0.00) 28%)',
    pointerEvents: 'none',
    zIndex: 1,
  },
  gradientLeft: {
    position: 'absolute',
    inset: 0,
    background: [
      'linear-gradient(to right,',
      'rgba(4,5,8,1.00) 0%,',
      'rgba(4,5,8,0.99) 22%,',
      'rgba(4,5,8,0.96) 34%,',
      'rgba(4,5,8,0.88) 46%,',
      'rgba(4,5,8,0.72) 56%,',
      'rgba(4,5,8,0.40) 68%,',
      'rgba(4,5,8,0.10) 80%,',
      'rgba(4,5,8,0.00) 90%)',
    ].join(' '),
    maskImage: 'linear-gradient(to bottom, black 0%, black 72%, rgba(0,0,0,0.65) 86%, transparent 100%)',
    WebkitMaskImage: 'linear-gradient(to bottom, black 0%, black 72%, rgba(0,0,0,0.65) 86%, transparent 100%)',
    pointerEvents: 'none',
    zIndex: 1,
  },
  gradientBottom: {
    position: 'absolute',
    inset: 0,
    background: [
      'linear-gradient(to bottom,',
      'rgba(4,5,8,0.00) 0%,',
      'rgba(4,5,8,0.00) 52%,',
      'rgba(4,5,8,0.30) 70%,',
      'rgba(4,5,8,0.76) 88%,',
      '#040508 100%)',
    ].join(' '),
    maskImage: 'linear-gradient(to right, black 0%, black 46%, rgba(0,0,0,0.34) 60%, transparent 72%)',
    WebkitMaskImage: 'linear-gradient(to right, black 0%, black 46%, rgba(0,0,0,0.34) 60%, transparent 72%)',
    pointerEvents: 'none',
    zIndex: 1,
  },
  panel: {
    position: 'absolute',
    top: 112,
    left: PANEL_LEFT,
    maxWidth: 580,
    display: 'flex',
    flexDirection: 'column',
    gap: 0,
    zIndex: 10,
  },
  logo: {
    height: 'clamp(80px, 13vh, 200px)' as unknown as number,
    maxWidth: 540,
    objectFit: 'contain',
    objectPosition: 'left center',
    filter: 'drop-shadow(0 4px 12px rgba(0,0,0,0.65)) drop-shadow(0 0 1px rgba(255,255,255,0.25))',
    userSelect: 'none',
    marginBottom: 22,
  },
  title: {
    color: '#FFFFFF',
    fontSize: 'clamp(2.4rem, 5vw, 5rem)' as unknown as number,
    fontWeight: 900,
    lineHeight: 1.0,
    margin: '0 0 22px 0',
    fontFamily: "'Montserrat', sans-serif",
    textShadow: '0 4px 8px rgba(0,0,0,0.6)',
    letterSpacing: '-0.01em',
  },
  tagline: {
    color: 'rgba(255,255,255,0.88)',
    fontSize: '1.1rem',
    fontWeight: 700,
    fontStyle: 'italic',
    margin: '0 0 20px 0',
    textShadow: '0 2px 8px rgba(0,0,0,0.7)',
    lineHeight: 1.3,
  },
  ratingsRow: {
    display: 'flex',
    alignItems: 'center',
    gap: 10,
    marginBottom: 14,
    flexWrap: 'wrap' as const,
  },
  imdbBadge: {
    display: 'inline-flex',
    alignItems: 'center',
    gap: 4,
    background: '#F5C518',
    borderRadius: 3,
    padding: '2px 6px 3px',
  },
  imdbLabel: {
    color: '#000000',
    fontSize: 10,
    fontWeight: 900,
    letterSpacing: '0.02em',
    lineHeight: 1,
  },
  imdbScore: {
    color: '#000000',
    fontSize: 13,
    fontWeight: 900,
    lineHeight: 1,
  },
  typeTag: {
    color: 'rgba(255,255,255,0.65)',
    fontSize: 11,
    fontWeight: 700,
    letterSpacing: '0.06em',
  },
  metaLine: {
    color: 'rgb(170, 170, 170)',
    fontSize: '0.875rem',
    margin: '0 0 16px 0',
    fontWeight: 400,
    textShadow: '0 2px 4px rgba(0,0,0,0.8)',
    lineHeight: 1.4,
  },
  genreRow: {
    display: 'flex',
    alignItems: 'center',
    gap: 6,
    flexWrap: 'wrap' as const,
    marginBottom: 18,
  },
  certBadge: {
    display: 'inline-flex',
    alignItems: 'center',
    padding: '1px 5px 2px',
    border: '1px solid rgba(255,255,255,0.60)',
    color: 'rgba(255,255,255,0.60)',
    borderRadius: 2,
    fontSize: '0.72rem',
    fontWeight: 600,
    textTransform: 'uppercase' as const,
    letterSpacing: '0.02em',
    flexShrink: 0,
  },
  genrePill: {
    display: 'inline-flex',
    alignItems: 'center',
    padding: '2px 8px',
    background: 'rgba(255,255,255,0.10)',
    border: '1px solid rgba(255,255,255,0.14)',
    borderRadius: 4,
    fontSize: '0.75rem',
    color: 'rgba(255,255,255,0.82)',
    fontWeight: 500,
    flexShrink: 0,
  },
  description: {
    color: 'rgba(255,255,255,0.82)',
    fontSize: '0.95rem',
    lineHeight: 1.6,
    margin: '0 0 0 0',
    maxWidth: 480,
    display: '-webkit-box',
    WebkitLineClamp: 2,
    WebkitBoxOrient: 'vertical' as const,
    overflow: 'hidden',
    textShadow: '0 1px 3px rgba(0,0,0,0.8)',
  },
  awards: {
    color: '#54D17A',
    fontSize: '0.82rem',
    lineHeight: 1.45,
    margin: '12px 0 0',
    maxWidth: 480,
    fontWeight: 650,
    textShadow: '0 1px 3px rgba(0,0,0,0.8)',
  },
  watchBtn: {
    display: 'inline-flex',
    alignItems: 'center',
    gap: 9,
    background: '#FFFFFF',
    color: '#000000',
    border: '1px solid transparent',
    borderRadius: 7,
    padding: '9px 22px',
    fontSize: '0.925rem',
    fontWeight: 700,
    cursor: 'pointer',
    fontFamily: "'Montserrat', sans-serif",
    transition: 'all 0.25s ease-in-out',
    boxShadow: '0 8px 32px rgba(0,0,0,0.3), 0 4px 16px rgba(0,0,0,0.1)',
  },
  indicators: {
    position: 'absolute',
    bottom: 88,
    left: '50%',
    transform: 'translateX(-50%)',
    display: 'flex',
    gap: 6,
    zIndex: 10,
    padding: '10px 16px',
    opacity: 0.5,
    transition: 'opacity 0.2s',
  },
  indicator: {
    width: 8,
    height: 8,
    borderRadius: 999,
    background: 'rgba(255,255,255,0.40)',
    border: '1px solid rgba(255,255,255,0.20)',
    cursor: 'pointer',
    padding: 0,
    transition: 'all 0.3s ease',
  },
  indicatorActive: {
    background: 'rgba(255,255,255,0.90)',
    width: 24,
  },
};
