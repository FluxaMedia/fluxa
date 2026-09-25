import { Camera, Captions, Gauge, Repeat, RotateCcw, RotateCw, Sparkles, Volume1, Volume2, VolumeOff } from 'lucide-react';
import { t } from '../../i18n';
import type { FeedbackFlash } from './PlayerOverlayPrimitives';

export function PlayerFeedback({ feedback, muted, volumeLevel }: { feedback: FeedbackFlash | null; muted: boolean; volumeLevel: number }) {
  if (!feedback) return null;
  return (
    <div
      style={{
        position: 'absolute',
        top: '5.5rem',
        left: '50%',
        transform: 'translate(-50%, -50%)',
        background: 'rgba(0,0,0,0.92)',
        border: '1px solid rgba(255,255,255,0.14)',
        boxShadow: '0 0.5rem 1.5rem rgba(0,0,0,0.35)',
        borderRadius: '0.75rem',
        padding: '0.625rem 1rem',
        display: 'flex',
        alignItems: 'center',
        gap: '0.5rem',
        color: '#fff',
        fontSize: '0.9375rem',
        fontWeight: 700,
        pointerEvents: 'none',
        zIndex: 20,
      }}
    >
      {feedback.icon === 'seekBack' && <RotateCcw size={20} />}
      {feedback.icon === 'seekFwd' && <RotateCw size={20} />}
      {feedback.icon === 'speed' && <Gauge size={20} />}
      {feedback.icon === 'abLoop' && <Repeat size={20} />}
      {feedback.icon === 'screenshot' && <Camera size={20} />}
      {feedback.icon === 'subDelay' && <Captions size={20} />}
      {feedback.icon === 'anime4k' && <Sparkles size={20} />}
      {feedback.icon === 'volume' && (muted ? <VolumeOff size={20} /> : volumeLevel < 50 ? <Volume1 size={20} /> : <Volume2 size={20} />)}
      {feedback.icon === 'volume' ? (
        <span>{feedback.label || (muted ? t('player.muted') : `${Math.round(volumeLevel)}%`)}</span>
      ) : (
        feedback.label && <span>{feedback.label}</span>
      )}
    </div>
  );
}
