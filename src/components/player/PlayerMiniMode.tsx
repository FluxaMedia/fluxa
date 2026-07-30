import { ChevronLeft, Minimize2, Pause, Play } from 'lucide-react';
import type { CSSProperties, RefObject } from 'react';
import { t } from '../../i18n';

export function PlayerMiniMode({ overlayRef, miniProgressRef, opacityStyle, paused, onTogglePause, onRestore, onClose, onActivity }: { overlayRef: RefObject<HTMLDivElement | null>; miniProgressRef: RefObject<HTMLDivElement | null>; opacityStyle: CSSProperties; paused: boolean; onTogglePause: () => void; onRestore: () => void; onClose: () => void; onActivity: () => void }) {
  return <div ref={overlayRef} onMouseMove={onActivity} style={{ position: 'fixed', inset: 0, zIndex: 9998, background: 'transparent' }}>
    <div style={{ position: 'absolute', bottom: 0, left: 0, right: 0, height: '0.125rem', background: 'rgba(255,255,255,0.15)' }}><div ref={miniProgressRef} style={{ height: '100%', width: '0%', background: 'var(--primary-accent-color)' }} /></div>
    <div style={{ ...opacityStyle, position: 'absolute', bottom: '0.125rem', left: 0, right: 0, display: 'flex', alignItems: 'center', justifyContent: 'center', gap: '0.5rem', padding: '0.5rem 0.625rem', background: 'linear-gradient(to top, rgba(0,0,0,0.85), transparent)' }}>
      <button onClick={(event) => { event.stopPropagation(); onTogglePause(); }} className="fluxa-ibtn" style={iconBtn} title={paused ? t('player.play') : t('player.pause')}>{paused ? <Play size={16} fill="currentColor" strokeWidth={0} /> : <Pause size={16} fill="currentColor" strokeWidth={0} />}</button>
      <button onClick={(event) => { event.stopPropagation(); onRestore(); }} className="fluxa-ibtn" style={iconBtn} title={t('player.restore_window')}><Minimize2 size={16} /></button>
      <button onClick={(event) => { event.stopPropagation(); onClose(); }} className="fluxa-ibtn" style={iconBtn} title={t('player.back')}><ChevronLeft size={16} /></button>
    </div>
  </div>;
}

const iconBtn: CSSProperties = { background: 'none', border: 'none', color: '#fff', cursor: 'pointer', display: 'flex', alignItems: 'center', justifyContent: 'center', width: '2.75rem', height: '2.75rem', borderRadius: '0.5rem', padding: 0, flexShrink: 0 };
