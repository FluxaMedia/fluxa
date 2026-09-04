import { ChevronRight } from 'lucide-react';
import { t } from '../../i18n';
import type { CSSProperties, ReactNode } from 'react';

export const SUBTITLE_OPACITIES = ['1.0', '0.75', '0.5', '0.25', '0.0'];
export const SUBTITLE_COLORS: { value: string; labelKey: string }[] = [
  { value: '#FFFFFF', labelKey: 'auto.white' },
  { value: '#000000', labelKey: 'auto.black' },
  { value: '#FFE45C', labelKey: 'auto.yellow' },
  { value: '#FF5D5D', labelKey: 'auto.red' },
  { value: '#3F7CFF', labelKey: 'auto.blue' },
  { value: '#54D17A', labelKey: 'auto.green' },
  { value: '#FF8A3D', labelKey: 'auto.orange' },
];

export function ColorOption({ color }: { color: string }) {
  const label = SUBTITLE_COLORS.find((c) => c.value === color)?.labelKey;
  return (
    <>
      <span
        style={{
          width: '0.75rem',
          height: '0.75rem',
          borderRadius: '50%',
          background: color,
          border: '1px solid rgba(255,255,255,0.25)',
          flexShrink: 0,
        }}
      />
      {label ? t(label) : color}
    </>
  );
}

export type SubtitleStylePage =
  | 'delay'
  | 'position'
  | 'textColor'
  | 'textOpacity'
  | 'size'
  | 'font'
  | 'characterEdge'
  | 'outlineColor'
  | 'outlineOpacity'
  | 'backgroundColor'
  | 'backgroundOpacity'
  | 'forceStyle'
  | 'shadow';
export const CHARACTER_EDGES = ['none', 'raised', 'depressed', 'uniform', 'drop-shadow'] as const;

export const styleBtn: CSSProperties = {
  background: 'rgba(255,255,255,0.07)',
  border: '1px solid rgba(255,255,255,0.10)',
  borderRadius: '0.375rem',
  color: 'rgba(255,255,255,0.85)',
  fontSize: '0.75rem',
  padding: '0.3125rem 0.5625rem',
  cursor: 'pointer',
};

export const rowBtn: CSSProperties = {
  display: 'flex',
  alignItems: 'center',
  gap: '0.625rem',
  width: '100%',
  background: 'none',
  border: 'none',
  color: 'rgba(255,255,255,0.7)',
  fontSize: '0.8125rem',
  padding: '0.5rem 0.875rem',
  cursor: 'pointer',
  textAlign: 'left',
};

export function SubtitleStyleNavigationRow({ label, value, onClick }: { label: string; value: ReactNode; onClick: () => void }) {
  return (
    <button
      type="button"
      onClick={onClick}
      style={{
        width: '100%',
        minHeight: '2.25rem',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'space-between',
        gap: '0.75rem',
        background: 'none',
        border: 'none',
        borderBottom: '1px solid rgba(255,255,255,0.06)',
        color: 'rgba(255,255,255,0.78)',
        fontSize: '0.75rem',
        padding: 0,
        cursor: 'pointer',
        textAlign: 'left',
      }}
    >
      <span>{label}</span>
      <span
        style={{
          display: 'flex',
          alignItems: 'center',
          gap: '0.25rem',
          maxWidth: '55%',
          color: 'rgba(255,255,255,0.72)',
          overflow: 'hidden',
          textOverflow: 'ellipsis',
          whiteSpace: 'nowrap',
        }}
      >
        {value}
        <ChevronRight size={14} style={{ flexShrink: 0, color: 'rgba(255,255,255,0.4)' }} />
      </span>
    </button>
  );
}

export type SubtitleCaptureCue = { start: number; end: number; text: string };
export type SubtitleStyleKey =
  | 'subtitleTextOpacity'
  | 'subtitleBackgroundColor'
  | 'subtitleBackgroundOpacity'
  | 'subtitleOutlineColor'
  | 'subtitleOutlineOpacity'
  | 'subtitleForceStyle'
  | 'subtitleCharacterEdge'
  | 'subtitleShadow';
