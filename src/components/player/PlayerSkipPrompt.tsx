import { SkipForward } from 'lucide-react';
import type { CSSProperties, RefObject } from 'react';
import { t } from '../../i18n';
import { sendCmd, type ActiveSkip } from './PlayerOverlayPrimitives';

export function PlayerSkipPrompt({ activeSkip, skipFillRef, onDismiss, onActivity }: { activeSkip: ActiveSkip | null; skipFillRef: RefObject<HTMLDivElement | null>; onDismiss: () => void; onActivity: () => void }) {
  if (!activeSkip) return null;
  return <div style={{ position: 'absolute', bottom: '6.625rem', right: 0, zIndex: 4, display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
    {activeSkip.type === 'outro' && <button onClick={(event) => { event.stopPropagation(); onActivity(); onDismiss(); }} style={watchCreditsBtn}>{t('player.watch_credits')}</button>}
    <button onClick={(event) => { event.stopPropagation(); onActivity(); sendCmd(`set time-pos ${Math.floor(activeSkip.endMs / 1000)}`); }} className="fluxa-skip-btn" style={skipBtn}>
      <div ref={skipFillRef} aria-hidden="true" style={{ position: 'absolute', inset: 0, width: '0%', background: '#fff', transition: 'width 0.15s linear' }} />
      <SkipForward size={18} style={{ position: 'relative', zIndex: 1 }} />
      <span style={{ position: 'relative', zIndex: 1 }}>{activeSkip.label}</span>
    </button>
  </div>;
}

const skipBtn: CSSProperties = { appearance: 'none', background: 'rgba(218,218,218,0.88)', backdropFilter: 'blur(0.5rem)', boxShadow: 'none', outline: 'none', border: 'none', borderRadius: '0.1875rem 0 0 0.1875rem', color: '#090909', fontSize: '0.9375rem', fontWeight: 600, padding: '0.75rem 1.4375rem', cursor: 'pointer', letterSpacing: '0.0125rem', display: 'flex', alignItems: 'center', gap: '0.5rem', position: 'relative', overflow: 'hidden' };
const watchCreditsBtn: CSSProperties = { appearance: 'none', background: 'rgba(218,218,218,0.88)', backdropFilter: 'blur(0.5rem)', border: 'none', borderRadius: '0.1875rem', color: '#090909', fontSize: '0.9375rem', fontWeight: 600, padding: '0.75rem 1.125rem', cursor: 'pointer', letterSpacing: '0.0125rem', transition: 'filter 0.12s, transform 0.12s' };
