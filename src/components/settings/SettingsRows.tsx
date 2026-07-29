import React, { useState, type RefObject } from 'react';
import { t } from '../../i18n';
import { styles, FONT } from './settingsStyles';
import type { SyncMeta } from './settingsTypes';
import { Popover } from '../ui/Popover';
import { timeAgo } from './settingsOptions';

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
        minHeight: '3.625rem',
        borderRadius: 0,
        border: 'none',
        borderBottom: '1px solid rgba(255,255,255,0.055)',
        background: hovered
          ? accent === '#FFFFFF'
            ? 'rgba(255,255,255,0.03)'
            : `${accent}18`
          : 'transparent',
        display: 'flex',
        alignItems: 'center',
        padding: '0.75rem 1rem',
        boxSizing: 'border-box',
        gap: '0.75rem',
        cursor: onClick ? 'pointer' : 'default',
        transition: 'background 0.12s',
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
    <div style={{
      width: '100%',
      minHeight: '3.75rem',
      borderBottom: '1px solid rgba(255,255,255,0.055)',
      display: 'flex',
      alignItems: 'center',
      padding: '0.75rem 1rem',
      boxSizing: 'border-box',
      gap: '0.75rem',
    }}>
      <span style={styles.rowIcon}>{icon}</span>
      <div>
        <p style={styles.rowTitle}>{title}</p>
        <p style={styles.rowSubtitle}>{value}</p>
      </div>
    </div>
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
  expanded,
}: {
  icon: React.ReactNode;
  title: string;
  value: string;
  valueColor?: string;
  onClick?: () => void;
  destructive?: boolean;
  busy?: boolean;
  expanded?: boolean;
}) {
  const [hovered, setHovered] = useState(false);
  return (
    <div
      style={{
        width: '100%',
        minHeight: '3.875rem',
        borderBottom: '1px solid rgba(255,255,255,0.055)',
        background: hovered && onClick ? (destructive ? 'rgba(255,80,80,0.05)' : 'rgba(255,255,255,0.03)') : 'transparent',
        display: 'flex',
        alignItems: 'center',
        padding: '0.75rem 1rem',
        boxSizing: 'border-box',
        gap: '0.75rem',
        cursor: onClick ? 'pointer' : 'default',
        transition: 'background 0.12s',
        opacity: busy ? 0.55 : 1,
      }}
      onClick={busy ? undefined : onClick}
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
    >
      <div style={{ flexShrink: 0 }}>{icon}</div>
      <div style={{ flex: 1, minWidth: 0 }}>
        <p style={{ color: destructive ? '#FF5A5A' : 'rgba(255,255,255,0.90)', fontSize: '0.875rem', fontWeight: 600, margin: 0, fontFamily: FONT, lineHeight: 1.25 }}>
          {title}
        </p>
        {value && (
          <p style={{ color: valueColor ?? 'rgba(255,255,255,0.40)', fontSize: '0.75rem', margin: '0.125rem 0 0', fontFamily: FONT, lineHeight: '0.9375rem', fontWeight: 400 }}>
            {value}
          </p>
        )}
      </div>
      {onClick && (
        <svg
          width="18" height="18" viewBox="0 0 24 24" fill="rgba(255,255,255,0.22)"
          style={expanded === undefined ? undefined : { transform: expanded ? 'rotate(180deg)' : 'none', transition: 'transform 0.14s' }}
        >
          <path d={expanded === undefined ? 'm9 18 6-6-6-6v12z' : 'M7 10l5 5 5-5z'} />
        </svg>
      )}
    </div>
  );
}

export function SyncServicePopover({
  open,
  anchorRef,
  serviceName,
  meta,
  busy,
  statusLabel,
  statusColor,
  syncLabel,
  onSyncNow,
  onDisconnect,
  onClose,
}: {
  open: boolean;
  anchorRef: RefObject<HTMLElement | null>;
  serviceName: string;
  meta: SyncMeta | null;
  busy: boolean;
  statusLabel?: string;
  statusColor?: string;
  syncLabel?: string;
  onSyncNow: () => void;
  onDisconnect: () => void;
  onClose: () => void;
}) {
  const isOutOfSync = !meta || Date.now() - meta.lastSyncAt > 6 * 60 * 60 * 1000;
  const effectiveStatus = statusLabel ?? (
    meta?.error
      ? `${t('settings.sync_error')} · ${meta.error}`
      : `${isOutOfSync ? t('settings.out_of_sync') : t('settings.synced')}${meta ? ` · ${timeAgo(meta.lastSyncAt)}` : ''}`
  );
  const effectiveStatusColor = statusColor ?? (meta?.error ? '#FF5A5A' : (isOutOfSync ? '#FF9500' : '#54D17A'));
  const counts = [
    meta && meta.continueWatchingCount > 0 ? `${meta.continueWatchingCount} ${t('auto.continue_watching')}` : null,
    meta && meta.watchlistCount > 0 ? `${meta.watchlistCount} ${t('settings.watchlist')}` : null,
  ].filter(Boolean).join(' · ');

  return (
    <Popover open={open} onClose={onClose} anchorRef={anchorRef} placement="bottom-start" matchWidth padding="0">
      <div style={{ padding: '0.6875rem 1rem' }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: '0.375rem' }}>
          <span style={{ width: '0.3125rem', height: '0.3125rem', borderRadius: '50%', background: effectiveStatusColor, flexShrink: 0 }} />
          <span style={{ color: effectiveStatusColor, fontSize: '0.75rem', fontWeight: 500, fontFamily: FONT }}>{effectiveStatus}</span>
        </div>
        {counts && <p style={{ ...styles.rowSubtitle, marginTop: '0.25rem' }}>{counts}</p>}
      </div>
      <PopoverActionButton
        label={busy ? '…' : syncLabel ?? t('settings.sync_now')}
        onClick={() => { onSyncNow(); onClose(); }}
        disabled={busy}
      />
      <PopoverActionButton
        label={t('auto.disconnect')}
        onClick={() => { onDisconnect(); onClose(); }}
        color="#FF5A5A"
      />
    </Popover>
  );
}

function PopoverActionButton({
  label,
  onClick,
  disabled = false,
  color = 'rgba(255,255,255,0.85)',
}: {
  label: string;
  onClick: () => void;
  disabled?: boolean;
  color?: string;
}) {
  const [hovered, setHovered] = useState(false);
  return (
    <button
      onClick={onClick}
      disabled={disabled}
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
      style={{
        ...styles.dropdownItem,
        justifyContent: 'flex-start',
        color,
        background: hovered && !disabled ? 'rgba(255,255,255,0.06)' : 'transparent',
        cursor: disabled ? 'default' : 'pointer',
        opacity: disabled ? 0.5 : 1,
      }}
    >
      {label}
    </button>
  );
}
