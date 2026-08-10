import React, { useEffect, useMemo, useRef, useState } from 'react';
import { Info, Play, Plus, Volume2, VolumeX } from 'lucide-react';
import { seasonPosterUrl } from '../core/seasonPosters';
import type { Meta } from '../core/types';
import { t } from '../i18n';
import { heroKeyframes, heroStyles as styles } from './heroStyles';
import { HeroIconBtn, parseReleaseYear, readOptionalString } from './HeroSectionParts';
import { youtubeVideoId } from './detail/TrailerCarousel';
import { useTrailerPlayback } from '../hooks/useTrailerPlayback';

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
  const pendingRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const activeIndexRef = useRef(activeIndex);
  activeIndexRef.current = activeIndex;
  const indicatorFillRef = useRef<HTMLSpanElement | null>(null);
  const indicatorAnimationRef = useRef<Animation | null>(null);

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
    trailerContainerRef, trailerVideoRef, trailerAudioRef,
    trailerStreamUrl, trailerAudioUrl, trailerReady, trailerActive, trailerPending, trailerMuted, activeTrailerSubtitle,
    handleTrailerPlaying, handleTrailerTimeUpdate, handleTrailerStopped, toggleTrailerMute,
  } = useTrailerPlayback({
    metaId: activeMeta.id,
    trailerVideoIds,
    autoplay: autoplayTrailer,
    autoplayDelaySecs: autoplayTrailerDelaySecs,
    preferredSubtitleLanguage,
    secondarySubtitleLanguage,
    isActive,
  });

  const slideIntervalMs = autoplayTrailer ? Math.max(DEFAULT_SLIDE_INTERVAL_MS, autoplayTrailerDelaySecs * 1000 + 3000) : DEFAULT_SLIDE_INTERVAL_MS;
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

  const genreLine = (Array.isArray(activeMeta.genres) ? activeMeta.genres : [])
    .filter((g): g is string => typeof g === 'string' && g.length > 0)
    .slice(0, 5);

  useEffect(() => {
    setBgError(false);
    setLogoError(false);
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
    const el = indicatorFillRef.current;
    indicatorAnimationRef.current?.cancel();
    if (!el) return;
    const animation = el.animate(
      [{ width: '0%' }, { width: '100%' }],
      { duration: slideIntervalMs, easing: 'linear', fill: 'forwards' },
    );
    if (trailerPending || !isActive) animation.pause();
    indicatorAnimationRef.current = animation;
    return () => animation.cancel();
  }, [activeIndex, slideIntervalMs]);

  useEffect(() => {
    const animation = indicatorAnimationRef.current;
    if (!animation) return;
    if (trailerPending || !isActive) animation.pause();
    else animation.play();
  }, [trailerPending, isActive]);

  useEffect(() => {
    if (!canSlide) return;
    const next = items[(activeIndex + 1) % items.length];
    if (!next) return;
    const nextBg = (preferSeasonPosters ? seasonPosterUrl(next) : undefined) ?? next.background ?? next.poster;
    if (nextBg) { const img = new Image(); img.src = nextBg; }
    if (next.logo) { const img = new Image(); img.src = next.logo; }
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
    if (e.key === 'ArrowLeft') { e.preventDefault(); goTo(activeIndex - 1); }
    else if (e.key === 'ArrowRight') { e.preventDefault(); goTo(activeIndex + 1); }
  };

  const dragRef = useRef<{ startX: number; startY: number } | null>(null);

  const handlePointerDown = (e: React.PointerEvent) => {
    if (!canSlide) return;
    dragRef.current = { startX: e.clientX, startY: e.clientY };
  };
  const handlePointerUp = (e: React.PointerEvent) => {
    const drag = dragRef.current;
    dragRef.current = null;
    if (!drag) return;
    const deltaX = e.clientX - drag.startX;
    const deltaY = e.clientY - drag.startY;
    if (Math.abs(deltaX) < SWIPE_THRESHOLD_PX || Math.abs(deltaX) < Math.abs(deltaY)) return;
    goTo(activeIndex + (deltaX < 0 ? 1 : -1));
  };

  return (
    <div
      style={{ ...styles.hero, cursor: canSlide ? 'grab' : undefined }}
      tabIndex={canSlide ? 0 : -1}
      onKeyDown={handleKeyDown}
      onPointerDown={handlePointerDown}
      onPointerUp={handlePointerUp}
      onPointerLeave={() => { dragRef.current = null; }}
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
      {trailerAudioUrl && (
        <audio ref={trailerAudioRef} key={trailerAudioUrl} src={trailerAudioUrl} preload="auto" />
      )}

      {trailerActive && activeTrailerSubtitle && (
        <div style={styles.trailerSubtitleOverlay}>
          {activeTrailerSubtitle}
        </div>
      )}

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

      <div style={{ ...styles.panel, ...contentStyle }}>
        {logoUrl ? (
          <img
            src={logoUrl}
            alt={activeMeta.name}
            decoding="async"
            style={{
              ...styles.logo,
              ...(trailerActive ? styles.logoTrailerActive : null),
            }}
            onError={() => setLogoError(true)}
          />
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

          {metaParts.length > 0 && (
            <p style={styles.metaLine}>{metaParts.join(' · ')}</p>
          )}

          {(!isNaN(imdbNum) || certification || genreLine.length > 0) && (
            <div style={styles.metaRow}>
              {!isNaN(imdbNum) && (
                <span style={styles.imdbBadge}>
                  <img src="/imdb.svg" alt="IMDb" style={styles.imdbLogo} />
                  <span style={styles.imdbScore}>{imdbNum.toFixed(1)}</span>
                </span>
              )}
              {certification && (
                <span style={styles.certBadge}>{certification}</span>
              )}
              {genreLine.length > 0 && (
                <span style={styles.genreText}>{genreLine.join('  ·  ')}</span>
              )}
            </div>
          )}

          {activeMeta.description && (
            <p style={styles.description}>{activeMeta.description}</p>
          )}

          {awards && <p style={styles.awards}>{awards}</p>}
        </div>

        <div style={styles.actions}>
          <button style={styles.watchBtn} onClick={() => onPlay?.(activeMeta)}>
            <Play size={13} fill="currentColor" />
            {t('common.play')}
          </button>
          <HeroIconBtn onClick={() => onAddToWatchlist?.(activeMeta)} title={t('auto.my_list')} ariaLabel={t('auto.my_list')}>
            <Plus size={20} />
          </HeroIconBtn>
          <HeroIconBtn onClick={() => onDetails?.(activeMeta)} title={t('auto.info')} ariaLabel={t('auto.info')}>
            <Info size={20} />
          </HeroIconBtn>
        </div>
      </div>

      {canSlide && !trailerActive && (
        <div style={styles.indicators}>
          {items.map((item, i) => (
            <button
              key={item.id || item.name}
              aria-label={`Show ${item.name}`}
              style={styles.indicatorTrack}
              onClick={() => goTo(i)}
            >
              <span
                ref={i === activeIndex ? indicatorFillRef : undefined}
                style={{
                  ...styles.indicatorFill,
                  ...(i < activeIndex ? styles.indicatorFillDone : null),
                }}
              />
            </button>
          ))}
        </div>
      )}
    </div>
  );
});
