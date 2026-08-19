import React from 'react';
import { ChevronDown } from 'lucide-react';
import { useEscapeKey } from '../hooks/useEscapeKey';
import { Popover } from './ui/Popover';
import type { ImportCategory } from '../core/importCategories';

type ScanResult = { counts: Partial<Record<ImportCategory, number>>; error?: string };

export function ImportDialog({
  title,
  titleIcon,
  items,
  destinations,
  destinationLabel,
  localOnlyLabel,
  scanLabel,
  scanningLabel,
  backLabel,
  continueLabel,
  confirmLabel,
  cancelLabel,
  onScan,
  onConfirm,
  onCancel,
}: {
  title: string;
  titleIcon?: React.ReactNode;
  items: { key: ImportCategory; label: string }[];
  destinations: { key: string; label: string; icon?: React.ReactNode }[];
  destinationLabel: string;
  localOnlyLabel: string;
  scanLabel: string;
  scanningLabel: string;
  backLabel: string;
  continueLabel: string;
  confirmLabel: string;
  cancelLabel: string;
  onScan: (selected: ImportCategory[]) => Promise<ScanResult>;
  onConfirm: (selected: ImportCategory[], destination: string | null) => void;
  onCancel: () => void;
}) {
  useEscapeKey(onCancel);
  const [selected, setSelected] = React.useState<Set<ImportCategory>>(() => new Set(items.map((item) => item.key)));
  const [destination, setDestination] = React.useState<string>('');
  const [phase, setPhase] = React.useState<'select' | 'scanning' | 'preview'>('select');
  const [scanResult, setScanResult] = React.useState<ScanResult | null>(null);
  const [destinationOpen, setDestinationOpen] = React.useState(false);
  const destinationBtnRef = React.useRef<HTMLButtonElement>(null);
  const destinationOptions: { key: string; label: string; icon?: React.ReactNode }[] = [
    { key: '', label: localOnlyLabel },
    ...destinations,
  ];
  const selectedDestination = destinationOptions.find((d) => d.key === destination) ?? destinationOptions[0];

  const toggle = (key: ImportCategory) => {
    setSelected((current) => {
      const next = new Set(current);
      if (next.has(key)) next.delete(key);
      else next.add(key);
      return next;
    });
  };

  const runScan = () => {
    setPhase('scanning');
    void onScan([...selected]).then((result) => {
      setScanResult(result);
      setPhase('preview');
    });
  };

  return (
    <div style={S.overlay} onClick={onCancel}>
      <div style={S.dialog} onClick={(e) => e.stopPropagation()}>
        <div style={S.titleRow}>
          {titleIcon}
          <p style={S.title}>{title}</p>
        </div>

        {phase === 'select' && (
          <>
            <div style={S.list}>
              {items.map((item) => (
                <label key={item.key} style={S.row}>
                  <input type="checkbox" checked={selected.has(item.key)} onChange={() => toggle(item.key)} style={S.checkbox} />
                  <span>{item.label}</span>
                </label>
              ))}
            </div>
            {destinations.length > 0 && (
              <div style={S.destinationBlock}>
                <p style={S.destinationLabel}>{destinationLabel}</p>
                <button ref={destinationBtnRef} type="button" onClick={() => setDestinationOpen((o) => !o)} style={S.destinationTrigger}>
                  <span style={S.destinationTriggerContent}>
                    {selectedDestination.icon}
                    <span style={S.destinationTriggerLabel}>{selectedDestination.label}</span>
                  </span>
                  <ChevronDown
                    size={16}
                    style={{
                      flexShrink: 0,
                      color: 'rgba(255,255,255,0.6)',
                      transform: destinationOpen ? 'rotate(180deg)' : 'none',
                      transition: 'transform 0.15s',
                    }}
                  />
                </button>
                <Popover
                  open={destinationOpen}
                  onClose={() => setDestinationOpen(false)}
                  anchorRef={destinationBtnRef}
                  placement="bottom-start"
                  matchWidth
                  maxHeight="14rem"
                  padding="0.25rem"
                  zIndex={10050}
                >
                  {destinationOptions.map((dest) => (
                    <button
                      key={dest.key || 'local'}
                      type="button"
                      onClick={() => {
                        setDestination(dest.key);
                        setDestinationOpen(false);
                      }}
                      style={{
                        ...S.destinationMenuItem,
                        background: dest.key === destination ? 'rgba(255,255,255,0.12)' : 'transparent',
                        fontWeight: dest.key === destination ? 700 : 500,
                      }}
                    >
                      {dest.icon}
                      <span>{dest.label}</span>
                    </button>
                  ))}
                </Popover>
              </div>
            )}
            <div style={S.actions}>
              <button onClick={onCancel} style={S.cancelBtn}>
                {cancelLabel}
              </button>
              <button onClick={runScan} disabled={selected.size === 0} style={{ ...S.confirmBtn, opacity: selected.size === 0 ? 0.5 : 1 }}>
                {scanLabel}
              </button>
            </div>
          </>
        )}

        {phase === 'scanning' && (
          <div style={S.actions}>
            <button disabled style={{ ...S.confirmBtn, width: '100%', opacity: 0.6 }}>
              {scanningLabel}
            </button>
          </div>
        )}

        {phase === 'preview' && scanResult && (
          <>
            {scanResult.error ? (
              <p style={S.errorText}>{scanResult.error}</p>
            ) : (
              <>
                <div style={S.list}>
                  {items
                    .filter((item) => selected.has(item.key))
                    .map((item) => (
                      <div key={item.key} style={S.countRow}>
                        <span>{item.label}</span>
                        <span style={S.countValue}>{scanResult.counts[item.key] ?? 0}</span>
                      </div>
                    ))}
                </div>
                <div style={S.destinationReminder}>
                  {selectedDestination.icon}
                  <span>
                    {destinationLabel}: <strong style={S.destinationReminderName}>{selectedDestination.label}</strong>
                  </span>
                </div>
              </>
            )}
            <div style={S.actions}>
              <button onClick={() => setPhase('select')} style={S.cancelBtn}>
                {backLabel}
              </button>
              {!scanResult.error && (
                <button onClick={() => onConfirm([...selected], destination || null)} style={S.confirmBtn}>
                  {continueLabel}
                </button>
              )}
            </div>
          </>
        )}
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
    fontFamily: FONT,
  },
  titleRow: { display: 'flex', alignItems: 'center', justifyContent: 'center', gap: '0.5rem', marginBottom: '1rem' },
  title: { margin: 0, color: '#FFFFFF', fontSize: '1rem', fontWeight: 700, textAlign: 'center' },
  list: { display: 'flex', flexDirection: 'column', gap: '0.5rem', marginBottom: '1rem' },
  row: {
    display: 'flex',
    alignItems: 'center',
    gap: '0.625rem',
    color: 'rgba(255,255,255,0.85)',
    fontSize: '0.8125rem',
    cursor: 'pointer',
  },
  checkbox: { width: '1rem', height: '1rem', cursor: 'pointer', accentColor: '#FFFFFF' },
  countRow: {
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'space-between',
    color: 'rgba(255,255,255,0.85)',
    fontSize: '0.8125rem',
    padding: '0.5rem 0.625rem',
    borderRadius: '0.375rem',
    background: 'rgba(255,255,255,0.045)',
  },
  countValue: { color: '#FFFFFF', fontWeight: 700 },
  destinationReminder: {
    display: 'flex',
    alignItems: 'center',
    gap: '0.5rem',
    marginBottom: '1.375rem',
    padding: '0.625rem',
    borderRadius: '0.375rem',
    background: 'rgba(255,255,255,0.045)',
    color: 'rgba(255,255,255,0.65)',
    fontSize: '0.75rem',
  },
  destinationReminderName: { color: '#FFFFFF', fontWeight: 700 },
  errorText: { color: '#FF5D5D', fontSize: '0.8125rem', margin: '0 0 1rem', lineHeight: 1.5 },
  destinationBlock: { marginBottom: '1.375rem' },
  destinationLabel: { margin: '0 0 0.5rem', color: 'rgba(255,255,255,0.55)', fontSize: '0.75rem', fontWeight: 600 },
  destinationTrigger: {
    width: '100%',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'space-between',
    gap: '0.5rem',
    height: '2.75rem',
    borderRadius: '0.5rem',
    background: 'rgba(255,255,255,0.06)',
    border: '1px solid rgba(255,255,255,0.10)',
    color: '#FFFFFF',
    fontSize: '0.8125rem',
    fontFamily: FONT,
    padding: '0 0.75rem',
    cursor: 'pointer',
    outline: 'none',
  },
  destinationTriggerContent: { display: 'flex', alignItems: 'center', gap: '0.5rem', minWidth: 0 },
  destinationTriggerLabel: { overflow: 'hidden', whiteSpace: 'nowrap', textOverflow: 'ellipsis' },
  destinationMenuItem: {
    width: '100%',
    display: 'flex',
    alignItems: 'center',
    gap: '0.5rem',
    textAlign: 'left',
    padding: '0.375rem 0.5rem',
    border: 'none',
    borderRadius: '0.4375rem',
    cursor: 'pointer',
    fontSize: '0.8125rem',
    color: '#FFFFFF',
    fontFamily: FONT,
    transition: 'background 0.12s',
  },
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
    background: '#FFFFFF',
    color: '#000',
    fontSize: '0.8125rem',
    fontWeight: 600,
    fontFamily: FONT,
    cursor: 'pointer',
    outline: 'none',
  },
};
