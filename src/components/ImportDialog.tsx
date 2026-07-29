import React from 'react';
import { useEscapeKey } from '../hooks/useEscapeKey';
import type { ImportCategory } from '../core/importCategories';

export function ImportDialog({ title, items, confirmLabel, cancelLabel, onConfirm, onCancel }: {
  title: string;
  items: { key: ImportCategory; label: string }[];
  confirmLabel: string;
  cancelLabel: string;
  onConfirm: (selected: ImportCategory[]) => void;
  onCancel: () => void;
}) {
  useEscapeKey(onCancel);
  const [selected, setSelected] = React.useState<Set<ImportCategory>>(() => new Set(items.map((item) => item.key)));

  const toggle = (key: ImportCategory) => {
    setSelected((current) => {
      const next = new Set(current);
      if (next.has(key)) next.delete(key);
      else next.add(key);
      return next;
    });
  };

  return (
    <div style={S.overlay} onClick={onCancel}>
      <div style={S.dialog} onClick={(e) => e.stopPropagation()}>
        <p style={S.title}>{title}</p>
        <div style={S.list}>
          {items.map((item) => (
            <label key={item.key} style={S.row}>
              <input type="checkbox" checked={selected.has(item.key)} onChange={() => toggle(item.key)} style={S.checkbox} />
              <span>{item.label}</span>
            </label>
          ))}
        </div>
        <div style={S.actions}>
          <button onClick={onCancel} style={S.cancelBtn}>{cancelLabel}</button>
          <button
            onClick={() => onConfirm([...selected])}
            disabled={selected.size === 0}
            style={{ ...S.confirmBtn, opacity: selected.size === 0 ? 0.5 : 1 }}
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
  overlay: { position: 'fixed', inset: 0, zIndex: 10000, background: 'rgba(0,0,0,0.7)', display: 'flex', alignItems: 'center', justifyContent: 'center' },
  dialog: { width: '20rem', borderRadius: '0.75rem', background: '#141414', border: '1px solid rgba(255,255,255,0.10)', padding: '1.5rem', fontFamily: FONT },
  title: { margin: '0 0 1rem', color: '#FFFFFF', fontSize: '1rem', fontWeight: 700, textAlign: 'center' },
  list: { display: 'flex', flexDirection: 'column', gap: '0.5rem', marginBottom: '1.375rem' },
  row: { display: 'flex', alignItems: 'center', gap: '0.625rem', color: 'rgba(255,255,255,0.85)', fontSize: '0.8125rem', cursor: 'pointer' },
  checkbox: { width: '1rem', height: '1rem', cursor: 'pointer', accentColor: '#FFFFFF' },
  actions: { display: 'flex', gap: '0.5rem' },
  cancelBtn: { flex: 1, height: '2.75rem', borderRadius: '0.5rem', background: 'rgba(255,255,255,0.06)', border: '1px solid rgba(255,255,255,0.10)', color: 'rgba(255,255,255,0.55)', fontSize: '0.8125rem', fontWeight: 500, fontFamily: FONT, cursor: 'pointer', outline: 'none' },
  confirmBtn: { flex: 1, height: '2.75rem', borderRadius: '0.5rem', border: 'none', background: '#FFFFFF', color: '#000', fontSize: '0.8125rem', fontWeight: 600, fontFamily: FONT, cursor: 'pointer', outline: 'none' },
};
