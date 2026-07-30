import { Camera, Captions, Gauge, Pause, Play, Repeat, RotateCcw, RotateCw, Sparkles, Volume1, Volume2, VolumeOff } from 'lucide-react';
import { t } from '../../i18n';
import type { FeedbackFlash } from './PlayerOverlayPrimitives';

export function PlayerFeedback({ feedback, muted, volumeLevel }: { feedback: FeedbackFlash | null; muted: boolean; volumeLevel: number }) {
  if (!feedback) return null;
  return <div style={{ position: 'absolute', top: '50%', left: '50%', transform: 'translate(-50%, -50%)', background: 'rgba(0,0,0,0.6)', backdropFilter: 'blur(0.5rem)', borderRadius: '0.875rem', padding: '0.75rem 1.375rem', display: 'flex', alignItems: 'center', gap: '0.5rem', color: '#fff', fontSize: '1.125rem', fontWeight: 700, pointerEvents: 'none', zIndex: 5 }}>
    {feedback.icon === 'play' && <Play size={20} fill="currentColor" strokeWidth={0} />}
    {feedback.icon === 'pause' && <Pause size={20} fill="currentColor" strokeWidth={0} />}
    {feedback.icon === 'seekBack' && <RotateCcw size={20} />}
    {feedback.icon === 'seekFwd' && <RotateCw size={20} />}
    {feedback.icon === 'speed' && <Gauge size={20} />}
    {feedback.icon === 'abLoop' && <Repeat size={20} />}
    {feedback.icon === 'screenshot' && <Camera size={20} />}
    {feedback.icon === 'subDelay' && <Captions size={20} />}
    {feedback.icon === 'anime4k' && <Sparkles size={20} />}
    {feedback.icon === 'volume' && (muted ? <VolumeOff size={20} /> : volumeLevel < 50 ? <Volume1 size={20} /> : <Volume2 size={20} />)}
    {feedback.icon === 'volume' ? <span>{muted ? t('player.muted') : `${Math.round(volumeLevel)}%`}</span> : feedback.label && <span>{feedback.label}</span>}
  </div>;
}
