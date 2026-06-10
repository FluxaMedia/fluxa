import React, { useEffect, useRef, useState } from 'react';

export function FilterDropdown({
  value,
  options,
  onSelect,
}: {
  value: string;
  options: { value: string; label: string }[];
  onSelect: (v: string) => void;
}) {
  const [open, setOpen] = useState(false);
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!open) return;
    const handler = (e: MouseEvent) => {
      if (ref.current && !ref.current.contains(e.target as Node)) setOpen(false);
    };
    document.addEventListener('mousedown', handler);
    return () => document.removeEventListener('mousedown', handler);
  }, [open]);

  return (
    <div ref={ref} style={{ position: 'relative', flexShrink: 0 }}>
      <button style={FD.trigger} onClick={() => setOpen((o) => !o)}>
        <span style={FD.label}>{value}</span>
        <svg
          width="16"
          height="16"
          viewBox="0 0 24 24"
          fill="rgba(255,255,255,0.6)"
          style={{ flexShrink: 0, transform: open ? 'rotate(180deg)' : 'none', transition: 'transform 0.2s' }}
        >
          <path d="M7 10l5 5 5-5z" />
        </svg>
      </button>
      {open && (
        <div style={FD.menu}>
          {options.map((opt) => (
            <button
              key={opt.value}
              style={{
                ...FD.menuItem,
                background: opt.label === value ? 'rgba(255,255,255,0.12)' : 'transparent',
                fontWeight: opt.label === value ? 700 : 500,
              }}
              onClick={() => { onSelect(opt.value); setOpen(false); }}
            >
              {opt.label}
            </button>
          ))}
        </div>
      )}
    </div>
  );
}

const FD: Record<string, React.CSSProperties> = {
  trigger: {
    display: 'inline-flex',
    alignItems: 'center',
    gap: 6,
    height: 36,
    padding: '0 12px',
    background: 'rgba(255,255,255,0.05)',
    border: '1px solid rgba(255,255,255,0.1)',
    borderRadius: 8,
    color: '#FFFFFF',
    cursor: 'pointer',
    fontSize: 13,
    fontWeight: 700,
    fontFamily: 'sans-serif',
    transition: 'background 0.15s',
  },
  label: {
    flex: 1,
    overflow: 'hidden',
    whiteSpace: 'nowrap',
    textOverflow: 'ellipsis',
  },
  menu: {
    position: 'absolute',
    top: 'calc(100% + 4px)',
    left: 0,
    minWidth: 160,
    background: '#15161E',
    border: '1px solid rgba(255,255,255,0.1)',
    borderRadius: 10,
    padding: '4px',
    display: 'flex',
    flexDirection: 'column',
    gap: 2,
    zIndex: 200,
    boxShadow: '0 8px 32px rgba(0,0,0,0.6)',
    maxHeight: 300,
    overflowY: 'auto',
    scrollbarWidth: 'thin',
    scrollbarColor: 'rgba(255,255,255,0.1) transparent',
  },
  menuItem: {
    width: '100%',
    textAlign: 'left',
    padding: '9px 12px',
    border: 'none',
    borderRadius: 7,
    cursor: 'pointer',
    fontSize: 13,
    fontFamily: 'sans-serif',
    color: '#FFFFFF',
    transition: 'background 0.12s',
  },
};
