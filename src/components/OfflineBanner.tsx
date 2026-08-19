import React, { useEffect, useState } from 'react';
import { X } from 'lucide-react';
import { t } from '../i18n';

interface Props {
  online: boolean;
}

export function OfflineBanner({ online }: Props) {
  const [dismissed, setDismissed] = useState(false);

  useEffect(() => {
    if (!online) setDismissed(false);
  }, [online]);

  if (online || dismissed) return null;

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
        background: 'rgba(64, 64, 64, 0.94)',
        backdropFilter: 'blur(0.5rem)',
        padding: '0 1rem',
      }}
    >
      <span style={{ fontSize: '0.75rem', fontWeight: 500, color: '#fff', letterSpacing: '0.01em' }}>{t('app.status.noInternet')}</span>
      <button
        onClick={() => setDismissed(true)}
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
    </div>
  );
}
