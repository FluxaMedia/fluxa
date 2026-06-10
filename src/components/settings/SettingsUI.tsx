import React, { useEffect, useRef, useState } from 'react';
import { getVersion } from '@tauri-apps/api/app';
import { t } from '../../i18n';
import { styles } from './settingsStyles';
import type { SyncMeta } from './settingsTypes';

export function AccountIcon() {
  return <svg width="22" height="22" viewBox="0 0 24 24" fill="currentColor"><path d="M12 12c2.21 0 4-1.79 4-4s-1.79-4-4-4-4 1.79-4 4 1.79 4 4 4zm0 2c-2.67 0-8 1.34-8 4v2h16v-2c0-2.66-5.33-4-8-4z"/></svg>;
}
export function PaletteIcon() {
  return <svg width="22" height="22" viewBox="0 0 24 24" fill="currentColor"><path d="M12 3c-4.97 0-9 4.03-9 9s4.03 9 9 9c.83 0 1.5-.67 1.5-1.5 0-.39-.15-.74-.39-1.01-.23-.26-.38-.61-.38-.99 0-.83.67-1.5 1.5-1.5H16c2.76 0 5-2.24 5-5 0-4.42-4.03-8-9-8zm-5.5 9c-.83 0-1.5-.67-1.5-1.5S5.67 9 6.5 9 8 9.67 8 10.5 7.33 12 6.5 12zm3-4C8.67 8 8 7.33 8 6.5S8.67 5 9.5 5s1.5.67 1.5 1.5S10.33 8 9.5 8zm5 0c-.83 0-1.5-.67-1.5-1.5S13.67 5 14.5 5s1.5.67 1.5 1.5S15.33 8 14.5 8zm3 4c-.83 0-1.5-.67-1.5-1.5S16.67 9 17.5 9s1.5.67 1.5 1.5-.67 1.5-1.5 1.5z"/></svg>;
}
export function PlayCircleIcon() {
  return <svg width="22" height="22" viewBox="0 0 24 24" fill="currentColor"><path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-2 14.5v-9l6 4.5-6 4.5z"/></svg>;
}
export function ExtensionIcon() {
  return <svg width="22" height="22" viewBox="0 0 24 24" fill="currentColor"><path d="M20.5 11H19V7c0-1.1-.9-2-2-2h-4V3.5C13 2.12 11.88 1 10.5 1S8 2.12 8 3.5V5H4c-1.1 0-1.99.9-1.99 2v3.8H3.5c1.49 0 2.7 1.21 2.7 2.7s-1.21 2.7-2.7 2.7H2V20c0 1.1.9 2 2 2h3.8v-1.5c0-1.49 1.21-2.7 2.7-2.7s2.7 1.21 2.7 2.7V22H17c1.1 0 2-.9 2-2v-4h1.5c1.38 0 2.5-1.12 2.5-2.5S21.88 11 20.5 11z"/></svg>;
}
export function SettingsIcon() {
  return <svg width="22" height="22" viewBox="0 0 24 24" fill="currentColor"><path d="M19.14 12.94c.04-.3.06-.61.06-.94 0-.32-.02-.64-.07-.94l2.03-1.58c.18-.14.23-.41.12-.61l-1.92-3.32c-.12-.22-.37-.29-.59-.22l-2.39.96c-.5-.38-1.03-.7-1.62-.94l-.36-2.54c-.04-.24-.24-.41-.48-.41h-3.84c-.24 0-.43.17-.47.41l-.36 2.54c-.59.24-1.13.57-1.62.94l-2.39-.96c-.22-.08-.47 0-.59.22L2.74 8.87c-.12.21-.08.47.12.61l2.03 1.58c-.05.3-.09.63-.09.94s.02.64.07.94l-2.03 1.58c-.18.14-.23.41-.12.61l1.92 3.32c.12.22.37.29.59.22l2.39-.96c.5.38 1.03.7 1.62.94l.36 2.54c.05.24.24.41.48.41h3.84c.24 0 .44-.17.47-.41l.36-2.54c.59-.24 1.13-.57 1.62-.94l2.39.96c.22.08.47 0 .59-.22l1.92-3.32c.12-.22.07-.47-.12-.61l-2.01-1.58zM12 15.6c-1.98 0-3.6-1.62-3.6-3.6s1.62-3.6 3.6-3.6 3.6 1.62 3.6 3.6-1.62 3.6-3.6 3.6z"/></svg>;
}
export function ArrowBackIcon() {
  return <svg width="22" height="22" viewBox="0 0 24 24" fill="currentColor"><path d="M20 11H7.83l5.59-5.59L12 4l-8 8 8 8 1.41-1.41L7.83 13H20v-2z"/></svg>;
}
export function StorageIcon() {
  return <svg width="22" height="22" viewBox="0 0 24 24" fill="currentColor"><path d="M2 20h20v-4H2v4zm2-3h2v2H4v-2zM2 4v4h20V4H2zm4 3H4V5h2v2zm-4 7h20v-4H2v4zm2-3h2v2H4v-2z"/></svg>;
}
export function DownloadIcon() {
  return <svg width="22" height="22" viewBox="0 0 24 24" fill="currentColor"><path d="M5 20h14v-2H5v2zM19 9h-4V3H9v6H5l7 7 7-7z"/></svg>;
}
export function RefreshIcon() {
  return <svg width="22" height="22" viewBox="0 0 24 24" fill="currentColor"><path d="M17.65 6.35C16.2 4.9 14.21 4 12 4c-4.42 0-7.99 3.58-7.99 8s3.57 8 7.99 8c3.73 0 6.84-2.55 7.73-6h-2.08c-.82 2.33-3.04 4-5.65 4-3.31 0-6-2.69-6-6s2.69-6 6-6c1.66 0 3.14.69 4.22 1.78L13 11h7V4l-2.35 2.35z"/></svg>;
}

export function timeAgo(ts: number): string {
  const s = Math.floor((Date.now() - ts) / 1000);
  if (s < 60) return `${s}s ago`;
  const m = Math.floor(s / 60);
  if (m < 60) return `${m} minute${m !== 1 ? 's' : ''} ago`;
  const h = Math.floor(m / 60);
  if (h < 24) return `${h} hour${h !== 1 ? 's' : ''} ago`;
  const d = Math.floor(h / 24);
  if (d < 7) return `${d} day${d !== 1 ? 's' : ''} ago`;
  return `${Math.floor(d / 7)} week${Math.floor(d / 7) !== 1 ? 's' : ''} ago`;
}

export function langOptions() {
  return [
    { value: 'none', label: t('settings.none') },
    { value: 'tr', label: t('language.turkish') },
    { value: 'en', label: t('language.english') },
    { value: 'ja', label: t('language.japanese') },
    { value: 'ko', label: t('language.korean') },
    { value: 'zh', label: t('language.chinese') },
    { value: 'de', label: t('language.german') },
    { value: 'fr', label: t('language.french') },
    { value: 'es', label: t('language.spanish') },
    { value: 'it', label: t('language.italian') },
    { value: 'pt', label: t('language.portuguese') },
    { value: 'ru', label: t('language.russian') },
    { value: 'ar', label: t('language.arabic') },
    { value: 'hi', label: t('language.hindi') },
  ];
}

export function streamSourceOptions() {
  return [
    { value: 'first', label: t('settings.stream_source_first_available') },
    { value: 'manual', label: t('settings.stream_source_manual') },
    { value: 'regex', label: t('settings.stream_source_regex_short') },
  ];
}

export function isFeedEnabled(selected: string[], key: string): boolean {
  return selected.length === 0 || selected.includes(key);
}

export function SidebarItem({
  label,
  subtitle,
  icon,
  selected,
  onClick,
}: {
  label: string;
  subtitle?: string;
  icon: React.ReactNode;
  selected: boolean;
  onClick: () => void;
}) {
  const [hovered, setHovered] = useState(false);
  return (
    <button
      style={{
        width: '100%',
        minHeight: 72,
        background: hovered || selected ? 'rgba(255,255,255,0.055)' : 'transparent',
        color: '#FFFFFF',
        border: 'none',
        borderRadius: 0,
        cursor: 'pointer',
        display: 'flex',
        alignItems: 'center',
        gap: 12,
        padding: '13px 18px',
        fontSize: 15,
        fontWeight: 800,
        fontFamily: 'sans-serif',
        transition: 'background 0.15s',
        textAlign: 'left',
        flexShrink: 0,
      }}
      onClick={onClick}
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
    >
      <span style={{ ...styles.categoryIcon, color: selected ? '#FFFFFF' : 'var(--primary-accent-color)' }}>{icon}</span>
      <span style={{ flex: 1, minWidth: 0 }}>
        <span style={styles.categoryLabel}>{label}</span>
        {subtitle && <span style={styles.categorySubtitle}>{subtitle}</span>}
      </span>
      <span style={styles.chevronIcon}>
        <svg width="22" height="22" viewBox="0 0 24 24" fill="currentColor">
          <path d="m9 18 6-6-6-6v12z" />
        </svg>
      </span>
    </button>
  );
}

export function SettingsDetailHeader({ title }: { title: string }) {
  return (
    <div style={styles.detailHeader}>
      <p style={styles.detailTitle}>{title}</p>
    </div>
  );
}

export function SettingsSection({
  title,
  subtitle,
  children,
}: {
  title: string;
  subtitle: string;
  children: React.ReactNode;
}) {
  return (
    <div style={styles.settingsGroup}>
      <div style={styles.groupHeading}>
        <p style={styles.groupTitle}>{title}</p>
        {subtitle && <p style={styles.groupSubtitle}>{subtitle}</p>}
      </div>
      <div style={styles.settingsCard}>{children}</div>
    </div>
  );
}

export function SettingsPanel({ children }: { children: React.ReactNode }) {
  return (
    <div
      style={{
        width: '100%',
        boxSizing: 'border-box',
        borderRadius: 0,
        background: 'transparent',
        borderBottom: '1px solid rgba(255,255,255,0.055)',
        padding: '14px 16px',
      }}
    >
      <div style={{ display: 'flex', flexDirection: 'column', gap: 7 }}>{children}</div>
    </div>
  );
}

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
        borderBottom: '1px solid rgba(255,255,255,0.055)',
        padding: '14px 16px',
        boxSizing: 'border-box',
      }}
    >
      <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: 10 }}>
        <div style={{ flex: 1, paddingRight: 12, minWidth: 0 }}>
          <p style={styles.rowTitle}>{title}</p>
          <p style={styles.rowSubtitle}>{subtitle}</p>
        </div>
        <span style={{ color: 'var(--primary-accent-color)', fontSize: 15, fontWeight: 800, fontFamily: 'sans-serif', flexShrink: 0, minWidth: 42, textAlign: 'right' }}>
          {label}
        </span>
      </div>
      <div style={{ position: 'relative', height: 20, display: 'flex', alignItems: 'center' }}>
        <div style={{ position: 'absolute', left: 0, right: 0, height: 4, borderRadius: 2, background: 'rgba(255,255,255,0.12)' }} />
        <div style={{ position: 'absolute', left: 0, width: `${pct}%`, height: 4, borderRadius: 2, background: 'var(--primary-accent-color)', transition: 'width 0.05s' }} />
        <input
          type="range"
          min={min}
          max={max}
          step={step}
          value={value}
          onChange={(e) => onChange(Number(e.target.value))}
          style={{ position: 'absolute', left: 0, right: 0, width: '100%', margin: 0, opacity: 0, cursor: 'pointer', height: 20 }}
        />
        <div
          style={{
            position: 'absolute',
            left: `calc(${pct}% - 9px)`,
            width: 18,
            height: 18,
            borderRadius: '50%',
            background: 'var(--primary-accent-color)',
            boxShadow: '0 1px 6px rgba(0,0,0,0.5)',
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
        minHeight: 64,
        borderRadius: 0,
        background: hovered ? 'rgba(255,255,255,0.045)' : 'transparent',
        border: 'none',
        borderBottom: '1px solid rgba(255,255,255,0.055)',
        display: 'flex',
        alignItems: 'center',
        padding: '13px 16px',
        boxSizing: 'border-box',
        justifyContent: 'space-between',
        cursor: 'pointer',
        transition: 'background 0.15s',
      }}
      onClick={() => onToggle(!checked)}
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
    >
      <div style={{ flex: 1, paddingRight: 12 }}>
        <p style={styles.rowTitle}>{title}</p>
        <p style={styles.rowSubtitle}>{subtitle}</p>
      </div>
      <div
        onClick={(e) => { e.stopPropagation(); onToggle(!checked); }}
        style={{ flexShrink: 0, width: 48, height: 28, borderRadius: 999, background: checked ? 'var(--primary-accent-color)' : 'rgba(255,255,255,0.16)', position: 'relative', transition: 'background 0.2s', cursor: 'pointer', padding: 3, boxSizing: 'border-box' }}
      >
        <div style={{ position: 'absolute', top: 3, left: checked ? 23 : 3, width: 22, height: 22, borderRadius: '50%', background: checked ? '#000000' : 'rgba(255,255,255,0.82)', transition: 'left 0.2s, background 0.2s' }} />
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
}: {
  title: string;
  subtitle: string;
  options: { value: string; label: string }[];
  selected: string;
  onSelect: (v: string) => void;
}) {
  const selectedLabel = options.find((opt) => opt.value === selected)?.label ?? selected;
  return (
    <SettingsPanel>
      <div style={{ display: 'flex', alignItems: 'center', gap: 16 }}>
        <div style={{ flex: 1, minWidth: 0 }}>
          <p style={styles.rowTitle}>{title}</p>
          <p style={styles.rowSubtitle}>{subtitle}</p>
        </div>
        <Dropdown
          ariaLabel={`${title}: ${selectedLabel}`}
          options={options}
          selected={selected}
          onSelect={onSelect}
        />
      </div>
    </SettingsPanel>
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
  const ref = useRef<HTMLDivElement>(null);
  const selectedLabel = options.find((opt) => opt.value === selected)?.label ?? selected;

  useEffect(() => {
    if (!open) return;
    const handlePointer = (event: MouseEvent) => {
      if (ref.current && !ref.current.contains(event.target as Node)) setOpen(false);
    };
    document.addEventListener('mousedown', handlePointer);
    return () => document.removeEventListener('mousedown', handlePointer);
  }, [open]);

  return (
    <div ref={ref} style={{ ...styles.dropdownWrap, zIndex: open ? 1000 : 2 }}>
      <button
        type="button"
        aria-label={ariaLabel}
        aria-expanded={open}
        style={{
          ...styles.dropdownButton,
          borderColor: open ? 'rgba(255,255,255,0.2)' : 'rgba(255,255,255,0.1)',
          background: open ? '#191E28' : '#151922',
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
      {open && (
        <div style={{ ...styles.dropdownMenu, zIndex: 1001 }}>
          {options.map((option) => {
            const active = option.value === selected;
            return (
              <button
                key={option.value}
                type="button"
                style={{
                  ...styles.dropdownItem,
                  background: active ? 'rgba(255,255,255,0.1)' : 'transparent',
                  color: active ? '#FFFFFF' : 'rgba(255,255,255,0.72)',
                }}
                onClick={() => { onSelect(option.value); setOpen(false); }}
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
        </div>
      )}
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
}: {
  title: string;
  subtitle: string;
  value: string;
  placeholder?: string;
  multiline?: boolean;
  onChange: (v: string) => void;
}) {
  const inputStyle: React.CSSProperties = {
    width: '100%',
    boxSizing: 'border-box',
    background: 'rgba(255,255,255,0.055)',
    border: '1px solid rgba(255,255,255,0.12)',
    borderRadius: 10,
    color: '#FFFFFF',
    fontSize: 13,
    fontFamily: 'sans-serif',
    padding: '10px 12px',
    outline: 'none',
    resize: 'vertical',
  };
  return (
    <SettingsPanel>
      <p style={styles.rowTitle}>{title}</p>
      <p style={{ ...styles.rowSubtitle, marginBottom: 8 }}>{subtitle}</p>
      {multiline ? (
        <textarea value={value} placeholder={placeholder} rows={5} onChange={(e) => onChange(e.target.value)} style={inputStyle} />
      ) : (
        <input value={value} placeholder={placeholder} onChange={(e) => onChange(e.target.value)} style={inputStyle} />
      )}
    </SettingsPanel>
  );
}

export function ActionTile({
  title,
  subtitle,
  icon,
  onClick,
  accent = '#FFFFFF',
}: {
  title: string;
  subtitle: string;
  icon: React.ReactNode;
  onClick?: () => void;
  accent?: string;
}) {
  const [hovered, setHovered] = useState(false);
  return (
    <div
      style={{
        width: '100%',
        minHeight: 62,
        borderRadius: 0,
        border: 'none',
        borderBottom: '1px solid rgba(255,255,255,0.055)',
        background: hovered
          ? accent === '#FFFFFF'
            ? 'rgba(255,255,255,0.035)'
            : `${accent}30`
          : 'transparent',
        display: 'flex',
        alignItems: 'center',
        padding: '13px 16px',
        boxSizing: 'border-box',
        gap: 14,
        cursor: onClick ? 'pointer' : 'default',
        transition: 'background 0.15s',
      }}
      onClick={onClick}
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
    >
      <span style={{ ...styles.rowIcon, color: accent }}>{icon}</span>
      <div>
        <p style={{ ...styles.rowTitle, color: accent }}>{title}</p>
        <p style={styles.rowSubtitle}>{subtitle}</p>
      </div>
    </div>
  );
}

export function InfoTile({ title, value, icon }: { title: string; value: string; icon: React.ReactNode }) {
  return (
    <SettingsPanel>
      <div style={{ display: 'flex', alignItems: 'center', gap: 14 }}>
        <span style={styles.rowIcon}>{icon}</span>
        <div>
          <p style={styles.rowTitle}>{title}</p>
          <p style={styles.rowSubtitle}>{value}</p>
        </div>
      </div>
    </SettingsPanel>
  );
}

export function SyncServiceRow({
  icon,
  title,
  value,
  valueColor,
  onClick,
  destructive = false,
  busy = false,
}: {
  icon: React.ReactNode;
  title: string;
  value: string;
  valueColor?: string;
  onClick?: () => void;
  destructive?: boolean;
  busy?: boolean;
}) {
  const [hovered, setHovered] = useState(false);
  return (
    <div
      style={{
        width: '100%',
        minHeight: 66,
        borderBottom: '1px solid rgba(255,255,255,0.055)',
        background: hovered && onClick ? (destructive ? 'rgba(255,80,80,0.06)' : 'rgba(255,255,255,0.035)') : 'transparent',
        display: 'flex',
        alignItems: 'center',
        padding: '13px 18px',
        boxSizing: 'border-box',
        gap: 14,
        cursor: onClick ? 'pointer' : 'default',
        transition: 'background 0.15s',
        opacity: busy ? 0.6 : 1,
      }}
      onClick={busy ? undefined : onClick}
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
    >
      <div style={{ flexShrink: 0 }}>{icon}</div>
      <div style={{ flex: 1, minWidth: 0 }}>
        <p style={{ color: destructive ? '#FF5A5A' : 'rgba(255,255,255,0.90)', fontSize: 15, fontWeight: 800, margin: 0, fontFamily: 'sans-serif', lineHeight: 1.25 }}>
          {title}
        </p>
        {value && (
          <p style={{ color: valueColor ?? 'rgba(255,255,255,0.45)', fontSize: 12, margin: '2px 0 0', fontFamily: 'sans-serif', lineHeight: '15px' }}>
            {value}
          </p>
        )}
      </div>
      {onClick && (
        <svg width="20" height="20" viewBox="0 0 24 24" fill="rgba(255,255,255,0.28)">
          <path d="m9 18 6-6-6-6v12z" />
        </svg>
      )}
    </div>
  );
}

export function SyncServicePopover({
  logoSrc,
  serviceName,
  meta,
  busy,
  onSyncNow,
  onDisconnect,
  onClose,
}: {
  logoSrc: string;
  serviceName: string;
  meta: SyncMeta | null;
  busy: boolean;
  onSyncNow: () => void;
  onDisconnect: () => void;
  onClose: () => void;
}) {
  useEffect(() => {
    const handler = (e: KeyboardEvent) => { if (e.key === 'Escape') onClose(); };
    window.addEventListener('keydown', handler);
    return () => window.removeEventListener('keydown', handler);
  }, [onClose]);

  const isOutOfSync = !meta || Date.now() - meta.lastSyncAt > 6 * 60 * 60 * 1000;
  return (
    <>
      <div style={{ position: 'fixed', inset: 0, zIndex: 199 }} onClick={onClose} />
      <div style={{
        position: 'absolute', left: 0, right: 0, top: '100%', zIndex: 200,
        background: '#1C1C1E', border: '1px solid rgba(255,255,255,0.12)',
        borderRadius: 16, boxShadow: '0 20px 60px rgba(0,0,0,0.7)',
        overflow: 'hidden',
      }}>
        <div style={{ padding: '14px 16px 12px', borderBottom: '1px solid rgba(255,255,255,0.07)', display: 'flex', alignItems: 'center', gap: 12 }}>
          <div style={{ width: 36, height: 36, borderRadius: 10, background: 'rgba(255,255,255,0.07)', display: 'flex', alignItems: 'center', justifyContent: 'center', flexShrink: 0 }}>
            <img src={logoSrc} alt={serviceName} style={{ width: 26, height: 26, objectFit: 'contain' }} />
          </div>
          <div style={{ flex: 1 }}>
            <p style={{ color: '#fff', fontSize: 15, fontWeight: 800, margin: 0, fontFamily: 'sans-serif' }}>{serviceName}</p>
            <div style={{ display: 'flex', alignItems: 'center', gap: 5, marginTop: 2 }}>
              <span style={{ width: 6, height: 6, borderRadius: '50%', background: isOutOfSync ? '#FF9500' : '#54D17A', flexShrink: 0 }} />
              <span style={{ color: isOutOfSync ? '#FF9500' : '#54D17A', fontSize: 11, fontFamily: 'sans-serif', fontWeight: 700 }}>
                {isOutOfSync ? 'Out of Sync' : 'Synced'}
                {meta ? ` · ${timeAgo(meta.lastSyncAt)}` : ''}
              </span>
            </div>
          </div>
        </div>
        {meta && (meta.continueWatchingCount > 0 || meta.watchlistCount > 0) && (
          <div style={{ padding: '10px 16px', borderBottom: '1px solid rgba(255,255,255,0.07)', display: 'flex', gap: 20 }}>
            {meta.continueWatchingCount > 0 && (
              <div>
                <p style={{ color: '#fff', fontSize: 18, fontWeight: 900, margin: 0, fontFamily: 'sans-serif' }}>{meta.continueWatchingCount}</p>
                <p style={{ color: 'rgba(255,255,255,0.45)', fontSize: 11, margin: '2px 0 0', fontFamily: 'sans-serif' }}>Continue Watching</p>
              </div>
            )}
            {meta.watchlistCount > 0 && (
              <div>
                <p style={{ color: '#fff', fontSize: 18, fontWeight: 900, margin: 0, fontFamily: 'sans-serif' }}>{meta.watchlistCount}</p>
                <p style={{ color: 'rgba(255,255,255,0.45)', fontSize: 11, margin: '2px 0 0', fontFamily: 'sans-serif' }}>Watchlist</p>
              </div>
            )}
          </div>
        )}
        <div style={{ padding: '10px 12px', display: 'flex', gap: 8 }}>
          <button
            onClick={() => { onSyncNow(); onClose(); }}
            disabled={busy}
            style={{ flex: 1, height: 38, borderRadius: 10, background: 'rgba(255,255,255,0.08)', border: '1px solid rgba(255,255,255,0.12)', color: '#fff', fontSize: 13, fontWeight: 800, fontFamily: 'sans-serif', cursor: busy ? 'default' : 'pointer', opacity: busy ? 0.5 : 1 }}
          >
            {busy ? '…' : 'Sync Now'}
          </button>
          <button
            onClick={() => { onDisconnect(); onClose(); }}
            style={{ flex: 1, height: 38, borderRadius: 10, background: 'rgba(255,80,80,0.08)', border: '1px solid rgba(255,80,80,0.2)', color: '#FF5A5A', fontSize: 13, fontWeight: 800, fontFamily: 'sans-serif', cursor: 'pointer' }}
          >
            Disconnect
          </button>
        </div>
      </div>
    </>
  );
}

export function VersionFooter() {
  const [version, setVersion] = useState('');
  useEffect(() => { getVersion().then((v) => setVersion(v)).catch(() => {}); }, []);
  return <p style={styles.versionFooter}>{version ? `v${version}` : ''}</p>;
}
