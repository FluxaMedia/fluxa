import { useState } from 'react';
import { effectiveFolderImageUrl, effectiveFolderShape } from '../core/collections';
import type { UserCollectionFolder } from '../core/types';
import { t } from '../i18n';

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
  return typeof crypto !== 'undefined' && crypto.randomUUID
    ? crypto.randomUUID()
    : `${Date.now()}_${Math.floor(Math.random() * 1e9)}`;
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
        height: 36,
        padding: '0 14px',
        border: `1px solid ${accent}${disabled ? '3f' : 'aa'}`,
        borderRadius: 8,
        background: hovered && !disabled ? `${accent}22` : `${accent}11`,
        color: disabled ? 'rgba(255,255,255,0.3)' : accent,
        fontSize: 12,
        fontWeight: 700,
        fontFamily: 'sans-serif',
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
        height: 46,
        border: 'none',
        borderRadius: 8,
        background: disabled ? 'rgba(255,255,255,0.12)' : hovered ? `${accent}dd` : accent,
        color: disabled ? 'rgba(255,255,255,0.4)' : contrastOn(accent),
        fontSize: 14,
        fontWeight: 900,
        fontFamily: 'sans-serif',
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
        height: 42,
        padding: '0 12px',
        background: 'rgba(255,255,255,0.05)',
        border: `1px solid ${accent}55`,
        borderRadius: 8,
        color: '#fff',
        fontSize: 14,
        fontFamily: 'sans-serif',
        outline: 'none',
        boxSizing: 'border-box',
      }}
    />
  );
}

export function Toggle({
  checked,
  onChange,
  accent,
}: {
  checked: boolean;
  onChange: (v: boolean) => void;
  accent: string;
}) {
  return (
    <div
      onClick={() => onChange(!checked)}
      style={{
        width: 42,
        height: 24,
        borderRadius: 12,
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
          top: 2,
          left: checked ? 20 : 2,
          width: 20,
          height: 20,
          borderRadius: '50%',
          background: '#fff',
          transition: 'left 0.2s',
        }}
      />
    </div>
  );
}

export function FolderRow({
  folder,
  accent,
  onClick,
}: {
  folder: UserCollectionFolder;
  accent: string;
  onClick: () => void;
}) {
  const imgUrl = effectiveFolderImageUrl(folder);
  const shape = effectiveFolderShape(folder).toUpperCase();
  return (
    <div
      onClick={onClick}
      style={{
        display: 'flex',
        alignItems: 'center',
        gap: 12,
        padding: '8px 10px',
        background: 'rgba(255,255,255,0.05)',
        borderRadius: 8,
        cursor: 'pointer',
      }}
    >
      <div
        style={{
          width: 50,
          height: 50,
          borderRadius: 6,
          background: 'rgba(255,255,255,0.08)',
          flexShrink: 0,
          overflow: 'hidden',
        }}
      >
        {imgUrl && (
          <img
            src={imgUrl}
            alt=""
            style={{ width: '100%', height: '100%', objectFit: 'cover' }}
          />
        )}
        {!imgUrl && folder.coverEmoji && (
          <div
            style={{
              width: '100%',
              height: '100%',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              fontSize: 24,
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
            fontSize: 14,
            fontFamily: 'sans-serif',
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
            fontSize: 11,
            fontFamily: 'sans-serif',
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
          fontSize: 10,
          fontWeight: 900,
          fontFamily: 'sans-serif',
          flexShrink: 0,
        }}
      >
        {shape}
      </div>
    </div>
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
    <div style={{ display: 'flex', flexDirection: 'column', gap: 8 }}>
      {value.trim() && (
        <img
          src={value.trim()}
          alt=""
          style={{
            width: '100%',
            height: 130,
            objectFit: 'cover',
            borderRadius: 8,
          }}
        />
      )}
      <FieldInput value={value} placeholder={label} onChange={onChange} accent={accent} />
    </div>
  );
}
