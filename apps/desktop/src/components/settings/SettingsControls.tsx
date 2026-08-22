import React, { useRef, useState } from 'react';
import { styles, FONT } from './settingsStyles';
import { color, fade, fontSize, radius } from '../../design/tokens';
import { Popover } from '../ui/Popover';

export function SliderTile({
  title,
  subtitle,
  value,
  min,
  max,
  step,
  format,
  onChange,
}: {
  title: string;
  subtitle: string;
  value: number;
  min: number;
  max: number;
  step: number;
  format?: (v: number) => string;
  onChange: (v: number) => void;
}) {
  const pct = ((value - min) / (max - min)) * 100;
  const label = format ? format(value) : `${value}%`;
  return (
    <div
      style={{
        width: '100%',
        borderBottom: `1px solid ${color.line}`,
        padding: '1.125rem 1.125rem',
        boxSizing: 'border-box',
      }}
    >
      <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: '0.75rem' }}>
        <div style={{ flex: 1, paddingRight: '0.75rem', minWidth: 0 }}>
          <p style={styles.rowTitle}>{title}</p>
          <p style={styles.rowSubtitle}>{subtitle}</p>
        </div>
        <span
          style={{
            color: 'var(--primary-accent-color)',
            fontSize: fontSize.base,
            fontWeight: 600,
            fontFamily: FONT,
            flexShrink: 0,
            minWidth: '2.375rem',
            textAlign: 'right',
          }}
        >
          {label}
        </span>
      </div>
      <div style={{ position: 'relative', height: '1.25rem', display: 'flex', alignItems: 'center' }}>
        <div
          style={{
            position: 'absolute',
            left: 0,
            right: 0,
            height: '0.1875rem',
            borderRadius: radius.xs,
            background: color.lineStrong,
          }}
        />
        <div
          style={{
            position: 'absolute',
            left: 0,
            width: `${pct}%`,
            height: '0.1875rem',
            borderRadius: '0.125rem',
            background: 'var(--primary-accent-color)',
            transition: 'width 0.05s',
          }}
        />
        <input
          type="range"
          min={min}
          max={max}
          step={step}
          value={value}
          onChange={(e) => onChange(Number(e.target.value))}
          style={{ position: 'absolute', left: 0, right: 0, width: '100%', margin: 0, opacity: 0, cursor: 'pointer', height: '1.25rem' }}
        />
        <div
          style={{
            position: 'absolute',
            left: `calc(${pct}% - 0.5rem)`,
            width: '1rem',
            height: '1rem',
            borderRadius: radius.circle,
            background: 'var(--primary-accent-color)',
            boxShadow: `0 1px 0.25rem ${fade.shade(0.5)}`,
            transition: 'left 0.05s',
            pointerEvents: 'none',
          }}
        />
      </div>
    </div>
  );
}

export function ToggleTile({
  title,
  subtitle,
  checked,
  onToggle,
}: {
  title: string;
  subtitle: string;
  checked: boolean;
  onToggle: (v: boolean) => void;
}) {
  const [hovered, setHovered] = useState(false);
  return (
    <div
      style={{
        width: '100%',
        minHeight: '4.25rem',
        borderRadius: 0,
        background: hovered ? fade.tint(0.03) : 'transparent',
        border: 'none',
        borderBottom: `1px solid ${color.line}`,
        display: 'flex',
        alignItems: 'center',
        padding: '1rem 1.125rem',
        boxSizing: 'border-box',
        justifyContent: 'space-between',
        cursor: 'pointer',
        transition: 'background 0.12s',
      }}
      onClick={() => onToggle(!checked)}
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
    >
      <div style={{ flex: 1, paddingRight: '1rem' }}>
        <p style={styles.rowTitle}>{title}</p>
        <p style={styles.rowSubtitle}>{subtitle}</p>
      </div>
      <div
        onClick={(e) => {
          e.stopPropagation();
          onToggle(!checked);
        }}
        style={{
          flexShrink: 0,
          width: '2.75rem',
          height: '1.625rem',
          borderRadius: radius.pill,
          background: checked ? 'var(--primary-accent-color)' : fade.tint(0.14),
          position: 'relative',
          transition: 'background 0.18s',
          cursor: 'pointer',
          boxSizing: 'border-box',
        }}
      >
        <div
          style={{
            position: 'absolute',
            top: '0.1875rem',
            left: checked ? 21 : 3,
            width: '1.25rem',
            height: '1.25rem',
            borderRadius: radius.circle,
            background: checked ? color.black : fade.tint(0.8),
            transition: 'left 0.18s',
            boxShadow: `0 1px 0.1875rem ${fade.shade(0.4)}`,
          }}
        />
      </div>
    </div>
  );
}

export function ChoiceTile({
  title,
  subtitle,
  options,
  selected,
  onSelect,
  disabled,
}: {
  title: string;
  subtitle: string;
  options: { value: string; label: string }[];
  selected: string;
  onSelect: (v: string) => void;
  disabled?: boolean;
}) {
  const selectedLabel = options.find((opt) => opt.value === selected)?.label ?? selected;
  return (
    <div
      style={{
        width: '100%',
        minHeight: '4.25rem',
        borderBottom: `1px solid ${color.line}`,
        display: 'flex',
        alignItems: 'center',
        padding: '1rem 1.125rem',
        boxSizing: 'border-box',
        gap: '1rem',
      }}
    >
      <div style={{ flex: 1, minWidth: 0 }}>
        <p style={styles.rowTitle}>{title}</p>
        <p style={styles.rowSubtitle}>{subtitle}</p>
      </div>
      <div style={disabled ? { pointerEvents: 'none', opacity: 0.45 } : undefined}>
        <Dropdown
          ariaLabel={`${title}: ${selectedLabel}`}
          options={options}
          selected={selected}
          onSelect={disabled ? () => {} : onSelect}
        />
      </div>
    </div>
  );
}

export function Dropdown({
  ariaLabel,
  options,
  selected,
  onSelect,
}: {
  ariaLabel: string;
  options: { value: string; label: string }[];
  selected: string;
  onSelect: (value: string) => void;
}) {
  const [open, setOpen] = useState(false);
  const btnRef = useRef<HTMLButtonElement>(null);
  const selectedLabel = options.find((opt) => opt.value === selected)?.label ?? selected;

  return (
    <div style={styles.dropdownWrap}>
      <button
        ref={btnRef}
        type="button"
        aria-label={ariaLabel}
        aria-expanded={open}
        style={{
          ...styles.dropdownButton,
          borderColor: open ? color.lineStrong : color.line,
          background: color.surfaceRaised,
        }}
        onClick={() => setOpen((value) => !value)}
      >
        <span style={styles.dropdownValue}>{selectedLabel}</span>
        <span style={{ ...styles.dropdownIcon, transform: open ? 'rotate(180deg)' : 'none' }}>
          <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor">
            <path d="M7 10l5 5 5-5z" />
          </svg>
        </span>
      </button>
      <Popover
        open={open}
        onClose={() => setOpen(false)}
        anchorRef={btnRef}
        placement="bottom-start"
        matchWidth
        maxHeight="15rem"
        padding="0.25rem"
      >
        {options.map((option) => {
          const active = option.value === selected;
          return (
            <button
              key={option.value}
              type="button"
              className="ui-popover-row"
              style={{
                ...styles.dropdownItem,
                background: active ? color.fillHover : 'transparent',
                color: active ? 'var(--primary-accent-color)' : color.textBody,
              }}
              onClick={() => {
                onSelect(option.value);
                setOpen(false);
              }}
            >
              <span style={styles.dropdownItemLabel}>{option.label}</span>
              {active && (
                <svg width="15" height="15" viewBox="0 0 24 24" fill="currentColor" style={{ flexShrink: 0 }}>
                  <path d="M9 16.17 4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41z" />
                </svg>
              )}
            </button>
          );
        })}
      </Popover>
    </div>
  );
}

export function InputTile({
  title,
  subtitle,
  value,
  placeholder,
  multiline,
  onChange,
  status,
}: {
  title: string;
  subtitle: string;
  value: string;
  placeholder?: string;
  multiline?: boolean;
  onChange: (v: string) => void;
  status?: React.ReactNode;
}) {
  const inputStyle: React.CSSProperties = {
    width: '100%',
    boxSizing: 'border-box',
    background: fade.tint(0.045),
    border: `1px solid ${color.line}`,
    borderRadius: radius.md,
    color: color.textPrimary,
    fontSize: fontSize.base,
    fontFamily: FONT,
    padding: '0.625rem 0.75rem',
    outline: 'none',
    resize: 'vertical',
    lineHeight: '1.5',
  };
  return (
    <div
      style={{
        width: '100%',
        borderBottom: `1px solid ${color.line}`,
        padding: '1.125rem 1.125rem',
        boxSizing: 'border-box',
      }}
    >
      <p style={styles.rowTitle}>{title}</p>
      <p style={{ ...styles.rowSubtitle, marginBottom: '0.625rem' }}>{subtitle}</p>
      {multiline ? (
        <textarea value={value} placeholder={placeholder} rows={5} onChange={(e) => onChange(e.target.value)} style={inputStyle} />
      ) : (
        <input value={value} placeholder={placeholder} onChange={(e) => onChange(e.target.value)} style={inputStyle} />
      )}
      {status}
    </div>
  );
}
