import React from 'react';
import { X } from 'lucide-react';
import { t } from '../i18n';

interface Props {
  serverDown: boolean;
  justRecovered: boolean;
  dismissed: boolean;
  onDismiss: () => void;
}

export function NuvioStatusBanner({ serverDown, justRecovered, dismissed, onDismiss }: Props) {
  const visible = (serverDown && !dismissed) || justRecovered;
  if (!visible) return null;

  const isGreen = justRecovered && !serverDown;

  return (
    <div
      style={{
        position: 'fixed',
        top: 0,
        left: 0,
        right: 0,
        zIndex: 99999,
        height: '2.25rem',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        gap: '0.5rem',
        background: isGreen ? 'rgba(22, 163, 74, 0.94)' : 'rgba(185, 28, 28, 0.94)',
        backdropFilter: 'blur(0.5rem)',
        transition: 'background 0.3s',
        padding: '0 1rem',
      }}
    >
      <span style={{ fontSize: '0.75rem', fontWeight: 500, color: '#fff', letterSpacing: '0.01em' }}>
        {isGreen ? t('nuvio.status.online') : t('nuvio.status.offline')}
      </span>
      {!isGreen && (
        <button
          onClick={onDismiss}
          style={{
            position: 'absolute',
            right: '0.75rem',
            background: 'transparent',
            border: 'none',
            color: 'rgba(255,255,255,0.7)',
            cursor: 'pointer',
            padding: '0.25rem',
            display: 'flex',
            alignItems: 'center',
          }}
          aria-label={t('common.close')}
        >
          <X size={14} />
        </button>
      )}
    </div>
  );
}
