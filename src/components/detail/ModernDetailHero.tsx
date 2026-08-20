import React from 'react';
import { color, fade } from '../../design';
import { Maximize2, Volume2, VolumeX, X } from 'lucide-react';
import { t } from '../../i18n';
import { MS } from './detailStyles';
import type { useTrailerPlayback } from '../../hooks/useTrailerPlayback';

export function ModernDetailHero({
  bgUrl,
  bgLayers,
  bgLoadedKeys,
  onBgLayerLoad,
  onBgError,
  trailer,
  heroInView,
  onBack,
  heroLogo,
  displayMetaName,
  progressPercent,
  remainingLabel,
}: {
  bgUrl: string | null | undefined;
  bgLayers: { url: string; key: number }[];
  bgLoadedKeys: Set<number>;
  onBgLayerLoad: (key: number) => void;
  onBgError: () => void;
  trailer: ReturnType<typeof useTrailerPlayback>;
  heroInView: boolean;
  onBack: () => void;
  heroLogo?: string;
  displayMetaName: string;
  progressPercent?: number;
  remainingLabel?: string | null;
}) {
  const {
    trailerContainerRef,
    trailerVideoRef,
    trailerAudioRef,
    trailerStreamUrl,
    trailerAudioUrl,
    trailerReady,
    trailerActive,
    trailerProgressElRef,
    trailerMuted,
    activeTrailerSubtitle,
    handleTrailerPlaying,
    handleTrailerTimeUpdate,
    handleTrailerStopped,
    toggleTrailerMute,
    fullscreenTrailer,
  } = trailer;

  return (
    <>
      {bgUrl ? (
        <div style={MS.pageBgWrap}>
          {bgLayers.map((layer, index) => (
            <img
              key={layer.key}
              src={layer.url}
              alt=""
              style={{
                ...MS.pageBgImg,
                position: 'absolute',
                inset: 0,
                opacity: index === 0 || bgLoadedKeys.has(layer.key) ? 1 : 0,
              }}
              onLoad={() => onBgLayerLoad(layer.key)}
              onError={onBgError}
            />
          ))}

          <div ref={trailerContainerRef} style={MS.heroTrailerContainer}>
            {trailerStreamUrl && (
              <video
                ref={trailerVideoRef}
                key={trailerStreamUrl}
                style={{ ...MS.heroTrailerFrame, opacity: trailerReady ? 1 : 0, transition: 'opacity 0.6s ease' }}
                src={trailerStreamUrl}
                autoPlay
                playsInline
                onPlaying={handleTrailerPlaying}
                onTimeUpdate={(e) => handleTrailerTimeUpdate(e.currentTarget)}
                onEnded={handleTrailerStopped}
                onError={handleTrailerStopped}
              />
            )}
            {trailerAudioUrl && <audio ref={trailerAudioRef} key={trailerAudioUrl} src={trailerAudioUrl} preload="auto" />}
          </div>

          <div style={MS.pageBgGradLeft} />
          <div style={MS.pageBgGradBottom} />
        </div>
      ) : (
        <div style={MS.heroPlaceholder} />
      )}

      {trailerActive && heroInView && (
        <div style={MS.trailerOverlayWrap}>
          {activeTrailerSubtitle && <div style={MS.heroTrailerSubtitleOverlay}>{activeTrailerSubtitle}</div>}

          <button
            style={{ ...MS.heroTrailerFullscreenButton, pointerEvents: 'auto' }}
            onClick={fullscreenTrailer}
            aria-label="Fullscreen trailer"
            title="Fullscreen trailer"
          >
            <Maximize2 size={16} />
          </button>

          <button
            style={{ ...MS.heroTrailerMuteButton, pointerEvents: 'auto' }}
            onClick={toggleTrailerMute}
            aria-label={trailerMuted ? 'Unmute' : 'Mute'}
          >
            {trailerMuted ? <VolumeX size={18} /> : <Volume2 size={18} />}
          </button>
          <div style={MS.heroTrailerProgressTrack}>
            <span ref={trailerProgressElRef} style={{ ...MS.heroTrailerProgressFill, width: '0%' }} />
          </div>
        </div>
      )}

      <div className="detail-hero" style={MS.heroWrap}>
        <button className="detail-back" style={MS.backBtn} onClick={onBack} aria-label={t('common.close')} title={t('common.close')}>
          <X size={18} color={color.textStrong} />
        </button>

        <div className="detail-logo" style={{ ...MS.logoWrap, opacity: trailerActive ? 0 : 1, transition: 'opacity 0.4s ease' }}>
          {heroLogo ? (
            <img
              src={heroLogo}
              alt={displayMetaName}
              style={MS.logo}
              onError={(e) => {
                (e.currentTarget as HTMLImageElement).style.display = 'none';
              }}
            />
          ) : (
            <h1 style={MS.titleHero}>{displayMetaName}</h1>
          )}

          {progressPercent != null && progressPercent > 0 && (
            <div style={MS.heroProgressRow}>
              <span style={MS.heroProgressTrack}>
                <span style={{ ...MS.heroProgressFill, width: `${Math.min(100, progressPercent)}%` }} />
              </span>
              {remainingLabel && <span style={MS.heroProgressLabel}>{remainingLabel}</span>}
            </div>
          )}
        </div>
      </div>
    </>
  );
}
