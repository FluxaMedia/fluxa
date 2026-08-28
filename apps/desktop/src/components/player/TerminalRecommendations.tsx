import { Play, X } from 'lucide-react';
import { t } from '../../i18n';
import type { Meta } from '../../core/types';

interface Props {
  items: Meta[];
  onPlay: (item: Meta) => void;
  onDismiss: () => void;
}

export function TerminalRecommendations({ items, onPlay, onDismiss }: Props) {
  if (items.length === 0) return null;
  return (
    <section
      aria-label={t('player.recommendations')}
      style={{
        position: 'absolute',
        inset: 0,
        zIndex: 8,
        display: 'flex',
        flexDirection: 'column',
        justifyContent: 'center',
        gap: '1rem',
        padding: '2rem 4vw',
        background: 'linear-gradient(90deg, rgba(5,7,12,.98), rgba(5,7,12,.82) 55%, rgba(5,7,12,.94))',
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
      <div style={{ fontSize: '1.5rem', fontWeight: 700 }}>{t('player.recommendations')}</div>
      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fill, minmax(10rem, 1fr))', gap: '1rem', maxWidth: '75rem' }}>
        {items.slice(0, 10).map((item) => (
          <button
            type="button"
            key={item.id}
            onClick={() => onPlay(item)}
            style={{ padding: 0, border: '1px solid rgba(255,255,255,.18)', borderRadius: '.45rem', overflow: 'hidden', background: '#151923', color: '#fff', textAlign: 'left', cursor: 'pointer' }}
          >
            <div style={{ aspectRatio: '2 / 3', background: '#222938' }}>
              {item.poster && <img src={item.poster} alt="" style={{ width: '100%', height: '100%', objectFit: 'cover', display: 'block' }} />}
            </div>
            <div style={{ padding: '.65rem', display: 'flex', alignItems: 'center', gap: '.4rem', minHeight: '3.2rem' }}>
              <Play size={14} fill="currentColor" />
              <span style={{ overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap', fontSize: '.85rem' }}>{item.name}</span>
            </div>
          </button>
        ))}
      </div>
    </section>
  );
}
