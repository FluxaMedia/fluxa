import { useState } from 'react';
import { Play, X } from 'lucide-react';
import { t } from '../../i18n';

interface NextEpCardProps {
  subtitle: string;
  thumbnail: string | null;
  countdown: number | null;
  countdownTotal: number;
  bottom: number;
  onPlay: () => void;
  onDismiss: () => void;
}

export function NextEpCard({ subtitle, thumbnail, countdown, countdownTotal, bottom, onPlay, onDismiss }: NextEpCardProps) {
  const [hovered, setHovered] = useState(false);
  const [thumbErr, setThumbErr] = useState(false);
  const epCodeMatch = subtitle.match(/^(S\d+:E\d+)\s+(.+)/i);
  const epCode = epCodeMatch ? epCodeMatch[1] : null;
  const epTitle = epCodeMatch ? epCodeMatch[2] : subtitle;
  const progress = countdown !== null && countdownTotal > 0 ? 1 - countdown / countdownTotal : 0;
  const borderColor = hovered ? 'rgba(255,255,255,0.5)' : 'rgba(255,255,255,0.35)';

  return (
    <div
      style={{
        position: 'absolute',
        bottom,
        right: 0,
        zIndex: 4,
        display: 'flex',
        flexDirection: 'column',
        maxWidth: '21.25rem',
        background: hovered ? 'rgba(28,33,44,0.97)' : 'rgba(18,22,30,0.93)',
        backdropFilter: 'blur(0.75rem)',
        border: `1px solid ${borderColor}`,
        borderRadius: '0.5rem 0 0 0.5rem',
        overflow: 'hidden',
        animation: 'fluxa-nextep-in 0.2s cubic-bezier(0.16, 1, 0.3, 1)',
        transition: 'background 0.15s, border-color 0.15s',
      }}
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
    >
      {thumbnail && !thumbErr && (
        <button
          onClick={(e) => {
            e.stopPropagation();
            onPlay();
          }}
          aria-label={t('player.next_label', epTitle)}
          style={{
            display: 'block',
            width: '100%',
            height: '11.875rem',
            border: 'none',
            borderBottom: `1px solid ${borderColor}`,
            padding: 0,
            cursor: 'pointer',
            background: '#0d0f16',
            transition: 'border-color 0.15s',
          }}
        >
          <img
            src={thumbnail}
            alt=""
            onError={() => setThumbErr(true)}
            style={{ width: '100%', height: '100%', objectFit: 'cover', display: 'block' }}
          />
        </button>
      )}

      <button
        onClick={(e) => {
          e.stopPropagation();
          onDismiss();
        }}
        style={{
          position: 'absolute',
          top: '0.5rem',
          right: '0.5rem',
          background: 'rgba(0,0,0,0.48)',
          border: 'none',
          borderRadius: '50%',
          color: 'rgba(255,255,255,0.8)',
          cursor: 'pointer',
          padding: '0.3125rem',
          display: 'flex',
        }}
        aria-label={t('player.dismiss')}
      >
        <X size={15} />
      </button>
      <button
        onClick={(e) => {
          e.stopPropagation();
          onPlay();
        }}
        aria-label={t('player.next_label', epTitle)}
        style={{
          position: 'relative',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          gap: '0.5rem',
          minHeight: '2.75rem',
          border: 'none',
          borderTop: '1px solid rgba(0,0,0,0.12)',
          padding: '0.625rem 1rem',
          overflow: 'hidden',
          background: '#d8d8d8',
          color: '#090909',
          cursor: 'pointer',
          fontSize: '0.875rem',
          fontWeight: 700,
        }}
      >
        <span
          aria-hidden="true"
          style={{
            position: 'absolute',
            inset: 0,
            width: `${(progress * 100).toFixed(2)}%`,
            background: '#fff',
            transition: 'width 1s linear',
          }}
        />
        <Play size={16} fill="currentColor" strokeWidth={0} style={{ position: 'relative', zIndex: 1 }} />
        <span style={{ position: 'relative', zIndex: 1, whiteSpace: 'nowrap', overflow: 'hidden', textOverflow: 'ellipsis' }}>
          {epCode ? `${epCode} · ${epTitle}` : epTitle}
        </span>
      </button>
    </div>
  );
}
