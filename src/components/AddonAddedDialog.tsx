import React from 'react';
import { t } from '../i18n';
import { useEscapeKey } from '../hooks/useEscapeKey';

const FONT = "'Montserrat', -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif";

interface Props {
  addonName: string;
  onConfirm: () => void;
}

export function AddonAddedDialog({ addonName, onConfirm }: Props) {
  useEscapeKey(onConfirm);
  return (
    <div style={S.backdrop} onClick={onConfirm}>
      <div style={S.dialog} onClick={(e) => e.stopPropagation()}>
        <p style={S.title}>{t('addons.installed_dialog_title')}</p>
        <p style={S.body}>{t('addons.installed_dialog_body', addonName)}</p>
        <div style={S.actions}>
          <button style={S.confirmBtn} onClick={onConfirm}>
            {t('common.ok')}
          </button>
        </div>
      </div>
    </div>
  );
}

const S: Record<string, React.CSSProperties> = {
  backdrop: {
    position: 'fixed',
    inset: 0,
    zIndex: 99998,
    background: 'var(--fluxa-scrim)',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    backdropFilter: 'blur(0.25rem)',
  },
  dialog: {
    background: 'var(--fluxa-surface-raised)',
    border: '1px solid var(--fluxa-border)',
    borderRadius: '0.75rem',
    padding: '1.75rem 1.75rem 1.5rem',
    maxWidth: '25rem',
    width: '90%',
    fontFamily: FONT,
  },
  title: {
    margin: '0 0 0.75rem',
    fontSize: '1rem',
    fontWeight: 600,
    color: 'var(--fluxa-text-primary)',
  },
  body: {
    margin: '0 0 1.5rem',
    fontSize: '0.8125rem',
    color: 'var(--fluxa-text-secondary)',
    lineHeight: 1.6,
  },
  actions: {
    display: 'flex',
    gap: '0.625rem',
    justifyContent: 'flex-end',
  },
  confirmBtn: {
    padding: '0.5625rem 1.125rem',
    borderRadius: '0.5rem',
    border: 'none',
    background: 'var(--fluxa-accent)',
    color: 'var(--fluxa-accent-foreground)',
    fontSize: '0.8125rem',
    fontWeight: 600,
    fontFamily: FONT,
    cursor: 'pointer',
  },
};
