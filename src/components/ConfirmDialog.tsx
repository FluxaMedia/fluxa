import React from 'react';
import { useEscapeKey } from '../hooks/useEscapeKey';

export function ConfirmDialog({
  title,
  body,
  confirmLabel,
  cancelLabel,
  destructive,
  onConfirm,
  onCancel,
}: {
  title: string;
  body: string;
  confirmLabel: string;
  cancelLabel: string;
  destructive?: boolean;
  onConfirm: () => void;
  onCancel: () => void;
}) {
  useEscapeKey(onCancel);

  return (
    <div style={S.overlay} onClick={onCancel}>
      <div style={S.dialog} onClick={(e) => e.stopPropagation()}>
        <p style={S.title}>{title}</p>
        <p style={S.body}>{body}</p>
        <div style={S.actions}>
          <button onClick={onCancel} style={S.cancelBtn}>
            {cancelLabel}
          </button>
          <button
            onClick={onConfirm}
            style={{ ...S.confirmBtn, background: destructive ? '#c0392b' : '#FFFFFF', color: destructive ? '#fff' : '#000' }}
          >
            {confirmLabel}
          </button>
        </div>
      </div>
    </div>
  );
}

const FONT = "'Montserrat', -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif";

const S: Record<string, React.CSSProperties> = {
  overlay: {
    position: 'fixed',
    inset: 0,
    zIndex: 10000,
    background: 'rgba(0,0,0,0.7)',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
  },
  dialog: {
    width: '20rem',
    borderRadius: '0.75rem',
    background: '#141414',
    border: '1px solid rgba(255,255,255,0.10)',
    padding: '1.5rem',
    textAlign: 'center',
    fontFamily: FONT,
  },
  title: { margin: 0, color: '#FFFFFF', fontSize: '1rem', fontWeight: 700 },
  body: { margin: '0.625rem 0 1.375rem', color: 'rgba(255,255,255,0.55)', fontSize: '0.8125rem', lineHeight: 1.5 },
  actions: { display: 'flex', gap: '0.5rem' },
  cancelBtn: {
    flex: 1,
    height: '2.75rem',
    borderRadius: '0.5rem',
    background: 'rgba(255,255,255,0.06)',
    border: '1px solid rgba(255,255,255,0.10)',
    color: 'rgba(255,255,255,0.55)',
    fontSize: '0.8125rem',
    fontWeight: 500,
    fontFamily: FONT,
    cursor: 'pointer',
    outline: 'none',
  },
  confirmBtn: {
    flex: 1,
    height: '2.75rem',
    borderRadius: '0.5rem',
    border: 'none',
    fontSize: '0.8125rem',
    fontWeight: 600,
    fontFamily: FONT,
    cursor: 'pointer',
    outline: 'none',
  },
};
