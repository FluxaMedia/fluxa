import React, { useEffect, useMemo, useRef, useState } from 'react';
import { Info, Play, Volume2, VolumeX } from 'lucide-react';
import { seasonPosterUrl } from '../core/seasonPosters';
import type { Meta } from '../core/types';
import { t } from '../i18n';
import { heroKeyframes, heroStyles as styles } from './heroStyles';
import { readOptionalString } from './HeroSectionParts';
import { youtubeVideoId } from './detail/youtube';
import { useTrailerPlayback } from '../hooks/useTrailerPlayback';
import { assetUrl } from '../platform/assets';

const SWIPE_THRESHOLD_PX = 60;

interface Props {
  meta: Meta;
  slides?: Meta[];
  onPlay?: (meta: Meta) => void;
  onDetails?: (meta: Meta) => void;
  onAddToWatchlist?: (meta: Meta) => void;
  preferSeasonPosters?: boolean;
  isActive?: boolean;
  autoplayTrailer?: boolean;
  autoplayTrailerDelaySecs?: number;
  preferredSubtitleLanguage?: string;
  secondarySubtitleLanguage?: string;
  pendingLogoIds?: Set<string>;
}

const DEFAULT_SLIDE_INTERVAL_MS = 6500;

const prefersReducedMotion = typeof window !== 'undefined' && window.matchMedia?.('(prefers-reduced-motion: reduce)').matches;

export const HeroSection = React.memo(function HeroSection({
  meta,
  slides,
  onPlay,
  onDetails,
  onAddToWatchlist,
  preferSeasonPosters = false,
  isActive = true,
  autoplayTrailer = false,
  autoplayTrailerDelaySecs = 2,
  preferredSubtitleLanguage,
  secondarySubtitleLanguage,
  pendingLogoIds,
}: Props) {
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
  const [visible, setVisible] = useState(true);
  const [bgError, setBgError] = useState(false);
  const [logoError, setLogoError] = useState(false);
  const [logoLoaded, setLogoLoaded] = useState(false);
  const pendingRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const activeIndexRef = useRef(activeIndex);
  activeIndexRef.current = activeIndex;

  const activeMeta = items[activeIndex] ?? meta;
  const canSlide = items.length > 1;

  const trailerVideoIdsRef = useRef<string[]>([]);
  const trailerVideoIds = useMemo(() => {
    const ids: string[] = [];
    for (const trailer of activeMeta.trailers ?? []) {
      const id = youtubeVideoId(trailer.url);
      if (id && !ids.includes(id)) ids.push(id);
    }
    const previous = trailerVideoIdsRef.current;
    if (previous.length === ids.length && previous.every((id, index) => id === ids[index])) {
      return previous;
    }
    trailerVideoIdsRef.current = ids;
    return ids;
  }, [activeMeta.trailers]);

  const {
    trailerContainerRef,
    trailerVideoRef,
    trailerAudioRef,
    trailerStreamUrl,
    trailerAudioUrl,
    trailerReady,
    trailerActive,
    trailerPending,
    trailerMuted,
    activeTrailerSubtitle,
    handleTrailerPlaying,
    handleTrailerTimeUpdate,
    handleTrailerStopped,
    toggleTrailerMute,
  } = useTrailerPlayback({
    metaId: activeMeta.id,
    trailerVideoIds,
    autoplay: autoplayTrailer,
    autoplayDelaySecs: autoplayTrailerDelaySecs,
    preferredSubtitleLanguage,
    secondarySubtitleLanguage,
    isActive,
  });

  const slideIntervalMs = autoplayTrailer
    ? Math.max(DEFAULT_SLIDE_INTERVAL_MS, autoplayTrailerDelaySecs * 1000 + 3000)
    : DEFAULT_SLIDE_INTERVAL_MS;
  const imageUrl = (preferSeasonPosters ? seasonPosterUrl(activeMeta) : undefined) ?? activeMeta.background ?? activeMeta.poster;
  const bgUrl = !bgError ? imageUrl : null;
  const logoUrl = !logoError ? activeMeta.logo : null;

  const imdbNum = activeMeta.imdbRating != null ? Number(activeMeta.imdbRating) : NaN;
  const tagline = readOptionalString(activeMeta, ['tagline', 'tagLine', 'slogan']);
  const awards = readOptionalString(activeMeta, ['awards']);
  const certification = readOptionalString(activeMeta, ['certification', 'contentRating', 'rating']);

  const genreLine = (Array.isArray(activeMeta.genres) ? activeMeta.genres : [])
    .filter((g): g is string => typeof g === 'string' && g.length > 0)
    .slice(0, 5);

  useEffect(() => {
    setBgError(false);
    setLogoError(false);
    setLogoLoaded(false);
  }, [activeMeta.id, imageUrl, activeMeta.logo]);

  useEffect(() => {
    return () => {
      if (pendingRef.current) clearTimeout(pendingRef.current);
    };
  }, []);

  function slideToIndex(next: number) {
    const clamped = ((next % items.length) + items.length) % items.length;
    if (pendingRef.current) clearTimeout(pendingRef.current);
    setVisible(false);
    pendingRef.current = setTimeout(() => {
      setActiveIndex(clamped);
      setVisible(true);
      pendingRef.current = null;
    }, 220);
  }

  useEffect(() => {
    if (!canSlide || !isActive || trailerPending) return;
    const id = window.setInterval(() => {
      slideToIndex(activeIndexRef.current + 1);
    }, slideIntervalMs);
    return () => window.clearInterval(id);
  }, [canSlide, items.length, isActive, trailerPending, slideIntervalMs]);

  useEffect(() => {
    if (!canSlide) return;
    const next = items[(activeIndex + 1) % items.length];
    if (!next) return;
    const nextBg = (preferSeasonPosters ? seasonPosterUrl(next) : undefined) ?? next.background ?? next.poster;
    if (nextBg) {
      const img = new Image();
      img.src = nextBg;
    }
    if (next.logo) {
      const img = new Image();
      img.src = next.logo;
    }
  }, [canSlide, items, activeIndex, preferSeasonPosters]);

  const goTo = (index: number) => {
    if (!canSlide) return;
    slideToIndex(index);
  };

  const contentStyle: React.CSSProperties = {
    opacity: visible ? 1 : 0,
    transition: 'opacity 0.25s ease',
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (!canSlide) return;
    if (e.key === 'ArrowLeft') {
      e.preventDefault();
      goTo(activeIndex - 1);
    } else if (e.key === 'ArrowRight') {
      e.preventDefault();
      goTo(activeIndex + 1);
    }
  };

  const dragRef = useRef<{ startX: number; startY: number } | null>(null);
  const handlePointerDown = (e: React.PointerEvent) => {
    if (!canSlide) return;
    if ((e.target as HTMLElement).closest('button, a, input')) return;
    dragRef.current = { startX: e.clientX, startY: e.clientY };
    e.currentTarget.setPointerCapture(e.pointerId);
  };
  const handlePointerUp = (e: React.PointerEvent) => {
    const drag = dragRef.current;
    dragRef.current = null;
    if (e.currentTarget.hasPointerCapture(e.pointerId)) e.currentTarget.releasePointerCapture(e.pointerId);
    if (!drag) return;
    const deltaX = e.clientX - drag.startX;
    const deltaY = e.clientY - drag.startY;
    if (Math.abs(deltaX) < SWIPE_THRESHOLD_PX || Math.abs(deltaX) < Math.abs(deltaY)) return;
    if (deltaX < 0) goTo(activeIndex + 1);
    else goTo(activeIndex - 1);
  };

  return (
    <div
      style={{ ...styles.hero, cursor: canSlide ? 'grab' : undefined }}
      tabIndex={canSlide ? 0 : -1}
      onKeyDown={handleKeyDown}
      onPointerDown={handlePointerDown}
      onPointerUp={handlePointerUp}
      onPointerCancel={() => {
        dragRef.current = null;
      }}
    >
      <style>{heroKeyframes}</style>
      {bgUrl && (
        <img
          key={activeMeta.id || activeIndex}
          src={bgUrl}
          alt=""
          decoding="async"
          style={{
            ...styles.backdrop,
            ...contentStyle,
            opacity: visible ? (trailerActive ? 0 : 1) : 0,
            transition: 'opacity 0.6s ease',
            animation: prefersReducedMotion ? 'none' : `heroKenBurns ${slideIntervalMs + 400}ms ease-out forwards`,
            animationPlayState: trailerActive || !isActive ? 'paused' : 'running',
          }}
          onError={() => setBgError(true)}
        />
      )}

      <div ref={trailerContainerRef} style={styles.trailerContainer}>
        {trailerStreamUrl && (
          <video
            ref={trailerVideoRef}
            key={trailerStreamUrl}
            style={{ ...styles.trailerFrame, opacity: trailerReady ? 1 : 0, transition: 'opacity 0.6s ease' }}
            src={trailerStreamUrl}
            autoPlay
            muted={trailerMuted}
            playsInline
            onPlaying={handleTrailerPlaying}
            onTimeUpdate={(e) => handleTrailerTimeUpdate(e.currentTarget)}
            onEnded={handleTrailerStopped}
            onError={handleTrailerStopped}
          />
        )}
        {trailerAudioUrl && <audio ref={trailerAudioRef} key={trailerAudioUrl} src={trailerAudioUrl} preload="auto" />}

        {trailerActive && activeTrailerSubtitle && <div style={styles.trailerSubtitleOverlay}>{activeTrailerSubtitle}</div>}

        {trailerActive && (
          <button
            onClick={toggleTrailerMute}
            style={styles.trailerMuteButton}
            aria-label={trailerMuted ? 'Unmute' : 'Mute'}
            onMouseEnter={(e) => {
              e.currentTarget.style.background = 'rgba(0,0,0,0.6)';
              e.currentTarget.style.borderColor = 'rgba(255,255,255,0.4)';
            }}
            onMouseLeave={(e) => {
              e.currentTarget.style.background = 'rgba(0,0,0,0.4)';
              e.currentTarget.style.borderColor = 'rgba(255,255,255,0.2)';
            }}
          >
            {trailerMuted ? <VolumeX size={20} /> : <Volume2 size={20} />}
          </button>
        )}
      </div>

      <div style={styles.gradientTop} />
      <div style={{ ...styles.gradientLeft, opacity: trailerActive ? 0.45 : 1, transition: 'opacity 0.6s ease' }} />
      <div style={styles.gradientBottom} />

      <div className="hero-panel" style={{ ...styles.panel, ...contentStyle }}>
        {logoUrl ? (
          <img
            className="hero-logo"
            src={logoUrl}
            alt={activeMeta.name}
            decoding="async"
            ref={(el) => {
              if (el?.complete) setLogoLoaded(true);
            }}
            style={{
              ...styles.logo,
              ...(trailerActive ? styles.logoTrailerActive : null),
              opacity: logoLoaded ? 1 : 0,
              transition: `${styles.logo.transition}, opacity 0.4s ease`,
            }}
            onLoad={() => setLogoLoaded(true)}
            onError={() => setLogoError(true)}
          />
        ) : pendingLogoIds?.has(activeMeta.id) ? (
          <div style={{ height: styles.logo.height, marginBottom: styles.logo.marginBottom }} />
        ) : (
          <h1 style={styles.title}>{String(activeMeta.name ?? '')}</h1>
        )}

        <div
          style={{
            maxHeight: trailerActive ? 0 : 600,
            opacity: trailerActive ? 0 : 1,
            overflow: 'hidden',
            transition: 'max-height 0.5s ease, opacity 0.3s ease',
          }}
        >
          {tagline && <p style={styles.tagline}>{tagline}</p>}

          {activeMeta.description && (
            <p className="hero-desc" style={{ ...styles.description, animation: 'heroFadeIn 0.4s ease' }}>
              {activeMeta.description}
            </p>
          )}

          {(!isNaN(imdbNum) || certification || genreLine.length > 0) && (
            <div style={styles.metaRow}>
              {!isNaN(imdbNum) && (
                <span style={styles.imdbBadge}>
                  <img src={assetUrl('imdb.svg')} alt="IMDb" style={styles.imdbLogo} />
                  <span style={styles.imdbScore}>{imdbNum.toFixed(1)}</span>
                </span>
              )}
              {certification && <span style={styles.certBadge}>{certification}</span>}
              {genreLine.length > 0 && <span style={styles.genreText}>{genreLine.join('  ·  ')}</span>}
            </div>
          )}

          {awards && <p style={styles.awards}>{awards}</p>}
        </div>

        <div className="hero-actions" style={styles.actions}>
          <button style={styles.watchBtn} onClick={() => onPlay?.(activeMeta)}>
            <Play size={13} fill="currentColor" />
            {t('common.play')}
          </button>
          <button style={styles.moreInfoBtn} onClick={() => onDetails?.(activeMeta)}>
            <Info size={16} />
            {t('hero.more_info')}
          </button>
        </div>
      </div>

      {canSlide && !trailerActive && (
        <div className="hero-indicators" style={styles.indicators}>
          {items.map((item, i) => (
            <button
              key={item.id || item.name}
              aria-label={`Show ${item.name}`}
              style={{ ...styles.indicatorTrack, ...(i === activeIndex ? styles.indicatorTrackActive : null) }}
              onClick={() => goTo(i)}
            />
          ))}
        </div>
      )}
    </div>
  );
});
