import React, { useRef, useState } from 'react';
import { ChevronDown } from 'lucide-react';
import { Popover } from './ui/Popover';

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
  const btnRef = useRef<HTMLButtonElement>(null);

  return (
    <div style={{ position: 'relative', flexShrink: 0 }}>
      <button ref={btnRef} style={FD.trigger} onClick={() => setOpen((o) => !o)}>
        <span style={FD.label}>{value}</span>
        <ChevronDown
          size={16}
          style={{
            flexShrink: 0,
            color: 'rgba(255,255,255,0.6)',
            transform: open ? 'rotate(180deg)' : 'none',
            transition: 'transform 0.2s',
          }}
        />
      </button>
      <Popover
        open={open}
        onClose={() => setOpen(false)}
        anchorRef={btnRef}
        placement="bottom-start"
        matchWidth
        minWidth="10rem"
        maxHeight="18.75rem"
        padding="0.25rem"
      >
        {options.map((opt) => (
          <button
            key={opt.value}
            className="ui-popover-row"
            style={{
              ...FD.menuItem,
              background: opt.label === value ? 'rgba(255,255,255,0.12)' : 'transparent',
              fontWeight: opt.label === value ? 700 : 500,
            }}
            onClick={() => {
              onSelect(opt.value);
              setOpen(false);
            }}
          >
            {opt.label}
          </button>
        ))}
      </Popover>
    </div>
  );
}

const FD: Record<string, React.CSSProperties> = {
  trigger: {
    display: 'inline-flex',
    alignItems: 'center',
    gap: '0.375rem',
    height: '2.25rem',
    padding: '0 0.75rem',
    background: 'rgba(255,255,255,0.05)',
    border: '1px solid rgba(255,255,255,0.1)',
    borderRadius: '0.5rem',
    color: '#FFFFFF',
    cursor: 'pointer',
    fontSize: '0.8125rem',
    fontWeight: 700,
    transition: 'background 0.15s',
  },
  label: {
    flex: 1,
    overflow: 'hidden',
    whiteSpace: 'nowrap',
    textOverflow: 'ellipsis',
  },
  menuItem: {
    width: '100%',
    textAlign: 'left',
    padding: '0.5625rem 0.75rem',
    border: 'none',
    borderRadius: '0.4375rem',
    cursor: 'pointer',
    fontSize: '0.8125rem',
    color: '#FFFFFF',
    transition: 'background 0.12s',
  },
};
