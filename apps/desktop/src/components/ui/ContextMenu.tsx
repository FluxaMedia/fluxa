import { useEffect, useRef, useState, type CSSProperties, type KeyboardEvent, type ReactNode } from 'react';
import { Popover } from './Popover';

export interface ContextMenuItem {
  icon: ReactNode;
  label: string;
  onSelect: () => void;
  danger?: boolean;
}

export function ContextMenu({
  point,
  onClose,
  items,
  width = '13.5rem',
}: {
  point: { x: number; y: number } | null;
  onClose: () => void;
  items: ContextMenuItem[];
  width?: number | string;
}) {
  const [activeIndex, setActiveIndex] = useState(0);
  const rowRefs = useRef<(HTMLButtonElement | null)[]>([]);

  useEffect(() => {
    if (point === null) return;
    setActiveIndex(0);
    rowRefs.current[0]?.focus();
  }, [point]);

  const moveFocus = (next: number) => {
    const clamped = (next + items.length) % items.length;
    setActiveIndex(clamped);
    rowRefs.current[clamped]?.focus();
  };

  const onKeyDown = (e: KeyboardEvent) => {
    if (e.key === 'ArrowDown') {
      e.preventDefault();
      moveFocus(activeIndex + 1);
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      moveFocus(activeIndex - 1);
    } else if (e.key === 'Home') {
      e.preventDefault();
      moveFocus(0);
    } else if (e.key === 'End') {
      e.preventDefault();
      moveFocus(items.length - 1);
    }
  };

  return (
    <Popover open={point !== null} onClose={onClose} point={point} width={width}>
      <div onKeyDown={onKeyDown}>
        {items.map((item, i) => (
          <button
            key={i}
            ref={(el) => {
              rowRefs.current[i] = el;
            }}
            type="button"
            className="ui-popover-row"
            onClick={() => {
              item.onSelect();
              onClose();
            }}
            onMouseEnter={() => setActiveIndex(i)}
            style={item.danger ? rowStyleDanger : rowStyle}
          >
            {item.icon}
            {item.label}
          </button>
        ))}
      </div>
    </Popover>
  );
}

const rowStyle: CSSProperties = {
  display: 'flex',
  alignItems: 'center',
  gap: '0.625rem',
  width: '100%',
  background: 'none',
  border: 'none',
  color: 'rgba(255,255,255,0.85)',
  fontSize: '0.8125rem',
  padding: '0.5rem 0.875rem',
  cursor: 'pointer',
  textAlign: 'left',
};

const rowStyleDanger: CSSProperties = { ...rowStyle, color: '#E85D3F' };
