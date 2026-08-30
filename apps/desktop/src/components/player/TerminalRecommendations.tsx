import { Maximize2, Pause, Play, X } from 'lucide-react';
import { t } from '../../i18n';
import type { Meta } from '../../core/types';
import { HeroSection } from '../HeroSection';
import { prefBool, prefString } from '../../core/appPrefs';

interface Props {
  items: Meta[];
  onPlay: (item: Meta) => void;
  onDismiss: () => void;
  paused?: boolean;
  onTogglePause?: () => void;
  onRestore?: () => void;
  onClose?: () => void;
  onActivity?: () => void;
  prefs?: Record<string, unknown>;
}

export function TerminalRecommendations({ items, onPlay, onDismiss, paused, onTogglePause, onRestore, onClose, onActivity, prefs = {} }: Props) {
  if (items.length === 0) return null;
  return (
    <section
      aria-label={t('player.recommendations')}
      style={{
        position: 'absolute',
        inset: 0,
        zIndex: 8,
        display: 'block',
        width: '100%',
        height: '100vh',
        ['--hero-height' as string]: '100vh',
        background: 'transparent',
        color: '#fff',
      }}
    >
      <button
        type="button"
        onClick={onDismiss}
        aria-label={t('common.close')}
        style={{ position: 'absolute', top: '1.25rem', right: '1.5rem', border: 0, background: 'transparent', color: '#fff', cursor: 'pointer' }}
      >
        <X size={24} />
      </button>
      <HeroSection
        meta={items[0]}
        slides={items.slice(1)}
        onPlay={onPlay}
        autoplayTrailer={prefBool(prefs, 'detailHeroAutoplayTrailer', false)}
        autoplayTrailerDelaySecs={Number(prefString(prefs, 'detailHeroAutoplayTrailerDelaySecs', '2'))}
        autoSlide={false}
        showLeftGradient={false}
        panelBottom="3.5rem"
        panelStyle={{ left: 'auto', right: '7.5rem', maxWidth: '31rem' }}
        bottomGradientHeight="32rem"
        nativeMiniCutout
        preferredSubtitleLanguage={prefString(prefs, 'preferredSubtitleLanguage', 'none')}
        secondarySubtitleLanguage={prefString(prefs, 'secondarySubtitleLanguage', 'none')}
      />
      <div
        onMouseEnter={onActivity}
        onMouseMove={onActivity}
        style={{ position: 'absolute', top: '1.5rem', left: '1.5rem', width: '28rem', height: '15.75rem', zIndex: 12 }}
      >
        {onTogglePause && onRestore && onClose && <div
          style={{
            position: 'absolute',
            left: 0,
            right: 0,
            bottom: 0,
            display: 'flex',
            justifyContent: 'center',
            gap: '0.25rem',
            padding: '0.35rem',
            borderRadius: '0 0 0.45rem 0.45rem',
            background: 'linear-gradient(to top, rgba(0,0,0,0.85), transparent)',
            opacity: 0,
            transition: 'opacity 0.2s ease',
          }}
          className="recommendation-mini-controls"
        >
          <button type="button" onClick={onTogglePause} aria-label={paused ? t('player.play') : t('player.pause')} style={miniButton}>
            {paused ? <Play size={15} fill="currentColor" /> : <Pause size={15} fill="currentColor" />}
          </button>
          <button type="button" onClick={onRestore} aria-label={t('player.restore_window')} style={miniButton}>
            <Maximize2 size={15} />
          </button>
          <button type="button" onClick={onClose} aria-label={t('common.close')} style={miniButton}>
            <X size={15} />
          </button>
        </div>}
      </div>
      <style>{`.recommendation-mini-controls:hover, div:hover > .recommendation-mini-controls { opacity: 1 !important; }`}</style>
    </section>
  );
}

const miniButton = {
  width: '2rem',
  height: '2rem',
  display: 'grid',
  placeItems: 'center',
  border: 0,
  borderRadius: '50%',
  color: '#fff',
  background: 'rgba(0,0,0,0.62)',
  cursor: 'pointer',
};
