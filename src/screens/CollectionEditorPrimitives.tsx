import type React from 'react';
import { useState } from 'react';
import { ChevronUp, ChevronDown, X } from 'lucide-react';
import { effectiveFolderImageUrl, effectiveFolderShape } from '../core/collections';
import type { UserCollectionFolder } from '../core/types';
import { t } from '../i18n';

export function moveItem<T>(items: T[], index: number, delta: -1 | 1): T[] {
  const next = index + delta;
  if (next < 0 || next >= items.length) return items;
  const copy = [...items];
  [copy[index], copy[next]] = [copy[next], copy[index]];
  return copy;
}

export function MoveButtons({
  onUp,
  onDown,
  onRemove,
  upDisabled,
  downDisabled,
}: {
  onUp: () => void;
  onDown: () => void;
  onRemove?: () => void;
  upDisabled?: boolean;
  downDisabled?: boolean;
}) {
  const btnStyle = (disabled?: boolean): React.CSSProperties => ({
    width: '1.5rem',
    height: '1.25rem',
    border: 'none',
    borderRadius: '0.25rem',
    background: 'rgba(255,255,255,0.08)',
    color: disabled ? 'rgba(255,255,255,0.2)' : '#fff',
    cursor: disabled ? 'default' : 'pointer',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    padding: 0,
  });
  return (
    <div style={{ display: 'flex', alignItems: 'center', gap: '0.25rem', flexShrink: 0 }}>
      <div style={{ display: 'flex', flexDirection: 'column', gap: '0.125rem' }}>
        <button onClick={onUp} disabled={upDisabled} style={btnStyle(upDisabled)}>
          <ChevronUp size={12} />
        </button>
        <button onClick={onDown} disabled={downDisabled} style={btnStyle(downDisabled)}>
          <ChevronDown size={12} />
        </button>
      </div>
      {onRemove && (
        <button onClick={onRemove} style={{ ...btnStyle(false), height: '2.625rem', color: '#e85d3f' }}>
          <X size={14} />
        </button>
      )}
    </div>
  );
}

export function hexLuminance(hex: string): number {
  const c = hex.replace('#', '');
  const r = parseInt(c.slice(0, 2), 16) / 255;
  const g = parseInt(c.slice(2, 4), 16) / 255;
  const b = parseInt(c.slice(4, 6), 16) / 255;
  const lin = (x: number) => (x <= 0.03928 ? x / 12.92 : Math.pow((x + 0.055) / 1.055, 2.4));
  return 0.2126 * lin(r) + 0.7152 * lin(g) + 0.0722 * lin(b);
}

export function contrastOn(hex: string): string {
  return hexLuminance(hex) > 0.35 ? '#000000' : '#ffffff';
}

export function uid(): string {
  return typeof crypto !== 'undefined' && crypto.randomUUID ? crypto.randomUUID() : `${Date.now()}_${Math.floor(Math.random() * 1e9)}`;
}

export function cleanUrl(s: string): string | undefined {
  const trimmed = s.trim();
  return trimmed || undefined;
}

export function UtilButton({
  label,
  accent,
  disabled,
  onClick,
  fullWidth,
}: {
  label: string;
  accent: string;
  disabled?: boolean;
  onClick: () => void;
  fullWidth?: boolean;
}) {
  const [hovered, setHovered] = useState(false);
  return (
    <button
      disabled={disabled}
      onClick={onClick}
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
      style={{
        width: fullWidth ? '100%' : undefined,
        height: '2.25rem',
        padding: '0 0.875rem',
        border: `1px solid ${accent}${disabled ? '3f' : 'aa'}`,
        borderRadius: '0.5rem',
        background: hovered && !disabled ? `${accent}22` : `${accent}11`,
        color: disabled ? 'rgba(255,255,255,0.3)' : accent,
        fontSize: '0.75rem',
        fontWeight: 700,
        cursor: disabled ? 'default' : 'pointer',
        transition: 'background 0.15s',
        whiteSpace: 'nowrap',
        overflow: 'hidden',
        textOverflow: 'ellipsis',
      }}
    >
      {label}
    </button>
  );
}

export function SaveButton({
  label,
  accent,
  disabled,
  onClick,
}: {
  label: string;
  accent: string;
  disabled?: boolean;
  onClick: () => void;
}) {
  const [hovered, setHovered] = useState(false);
  return (
    <button
      disabled={disabled}
      onClick={onClick}
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
      style={{
        width: '100%',
        height: '2.875rem',
        border: 'none',
        borderRadius: '0.5rem',
        background: disabled ? 'rgba(255,255,255,0.12)' : hovered ? `${accent}dd` : accent,
        color: disabled ? 'rgba(255,255,255,0.4)' : contrastOn(accent),
        fontSize: '0.875rem',
        fontWeight: 900,
        cursor: disabled ? 'default' : 'pointer',
        transition: 'background 0.15s',
      }}
    >
      {label}
    </button>
  );
}

export function FieldInput({
  value,
  placeholder,
  onChange,
  accent,
}: {
  value: string;
  placeholder: string;
  onChange: (v: string) => void;
  accent: string;
}) {
  return (
    <input
      value={value}
      placeholder={placeholder}
      onChange={(e) => onChange(e.target.value)}
      style={{
        width: '100%',
        height: '2.625rem',
        padding: '0 0.75rem',
        background: 'rgba(255,255,255,0.05)',
        border: `1px solid ${accent}55`,
        borderRadius: '0.5rem',
        color: '#fff',
        fontSize: '0.875rem',
        outline: 'none',
        boxSizing: 'border-box',
      }}
    />
  );
}

export function Toggle({ checked, onChange, accent }: { checked: boolean; onChange: (v: boolean) => void; accent: string }) {
  return (
    <div
      onClick={() => onChange(!checked)}
      style={{
        width: '2.625rem',
        height: '1.5rem',
        borderRadius: '0.75rem',
        background: checked ? accent : 'rgba(255,255,255,0.18)',
        position: 'relative',
        cursor: 'pointer',
        transition: 'background 0.2s',
        flexShrink: 0,
      }}
    >
      <div
        style={{
          position: 'absolute',
          top: '0.125rem',
          left: checked ? 20 : 2,
          width: '1.25rem',
          height: '1.25rem',
          borderRadius: '50%',
          background: checked ? '#000' : '#fff',
          transition: 'left 0.2s, background 0.2s',
        }}
      />
    </div>
  );
}

export function FolderRow({ folder, accent, onClick }: { folder: UserCollectionFolder; accent: string; onClick: () => void }) {
  const imgUrl = effectiveFolderImageUrl(folder);
  const shape = effectiveFolderShape(folder).toUpperCase();
  return (
    <div
      onClick={onClick}
      style={{
        display: 'flex',
        alignItems: 'center',
        gap: '0.75rem',
        padding: '0.5rem 0.625rem',
        background: 'rgba(255,255,255,0.05)',
        borderRadius: '0.5rem',
        cursor: 'pointer',
      }}
    >
      <div
        style={{
          width: '3.125rem',
          height: '3.125rem',
          borderRadius: '0.375rem',
          background: 'rgba(255,255,255,0.08)',
          flexShrink: 0,
          overflow: 'hidden',
        }}
      >
        {imgUrl && <img src={imgUrl} alt="" style={{ width: '100%', height: '100%', objectFit: 'cover' }} />}
        {!imgUrl && folder.coverEmoji && (
          <div
            style={{
              width: '100%',
              height: '100%',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              fontSize: '1.5rem',
            }}
          >
            {folder.coverEmoji}
          </div>
        )}
      </div>
      <div style={{ flex: 1, minWidth: 0 }}>
        <div
          style={{
            color: '#fff',
            fontWeight: 700,
            fontSize: '0.875rem',
            overflow: 'hidden',
            textOverflow: 'ellipsis',
            whiteSpace: 'nowrap',
          }}
        >
          {folder.title || t('library.untitled')}
        </div>
        <div
          style={{
            color: 'rgba(255,255,255,0.5)',
            fontSize: '0.6875rem',
            overflow: 'hidden',
            textOverflow: 'ellipsis',
            whiteSpace: 'nowrap',
          }}
        >
          {folder.catalogTitle ?? ''}
        </div>
      </div>
      <div
        style={{
          color: accent,
          fontSize: '0.625rem',
          fontWeight: 900,
          flexShrink: 0,
        }}
      >
        {shape}
      </div>
    </div>
  );
}

export function SectionLabel({ children }: { children: React.ReactNode }) {
  return (
    <p
      style={{
        color: 'rgba(255,255,255,0.42)',
        fontSize: '0.6875rem',
        fontWeight: 600,
        margin: '0 0 0.5rem 0.25rem',
        textTransform: 'uppercase',
        letterSpacing: '0.07em',
      }}
    >
      {children}
    </p>
  );
}

export function Card({ children }: { children: React.ReactNode }) {
  return (
    <div
      style={{
        width: '100%',
        borderRadius: '0.75rem',
        background: '#1A1A1A',
        border: '1px solid rgba(255,255,255,0.10)',
        boxSizing: 'border-box',
        padding: '0.875rem',
        display: 'flex',
        flexDirection: 'column',
        gap: '0.75rem',
      }}
    >
      {children}
    </div>
  );
}

export function Chip({ label, selected, accent, onClick }: { label: string; selected: boolean; accent: string; onClick: () => void }) {
  const [hovered, setHovered] = useState(false);
  return (
    <button
      onClick={onClick}
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
      style={{
        border: 'none',
        borderRadius: '1rem',
        padding: '0.4375rem 0.875rem',
        background: selected ? accent : hovered ? 'rgba(255,255,255,0.10)' : 'rgba(255,255,255,0.06)',
        color: selected ? contrastOn(accent) : '#fff',
        fontWeight: 700,
        fontSize: '0.75rem',
        cursor: 'pointer',
        whiteSpace: 'nowrap',
        transition: 'background 0.12s',
      }}
    >
      {label}
    </button>
  );
}

export function ImagePreviewField({
  label,
  value,
  onChange,
  accent,
}: {
  label: string;
  value: string;
  onChange: (v: string) => void;
  accent: string;
}) {
  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '0.5rem' }}>
      {value.trim() && (
        <img
          src={value.trim()}
          alt=""
          style={{
            width: '100%',
            height: '8.125rem',
            objectFit: 'cover',
            borderRadius: '0.5rem',
          }}
        />
      )}
      <FieldInput value={value} placeholder={label} onChange={onChange} accent={accent} />
    </div>
  );
}
