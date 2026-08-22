import React, { useEffect, useState } from 'react';
import { platformInvoke } from '../../platform/invoke';
import { styles, FONT } from './settingsStyles';
import { color, fade, fontSize, radius } from '../../design/tokens';

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
        minHeight: '2.5rem',
        background: hovered ? fade.tint(0.04) : 'transparent',
        color: color.textPrimary,
        border: 'none',
        borderRadius: radius.md,
        cursor: 'pointer',
        display: 'flex',
        alignItems: 'center',
        gap: '0.625rem',
        padding: '0.5625rem 0.75rem',
        fontFamily: FONT,
        transition: 'background 0.12s',
        textAlign: 'left',
        flexShrink: 0,
        position: 'relative',
        outline: 'none',
      }}
      onClick={onClick}
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
    >
      {selected && (
        <div
          style={{
            position: 'absolute',
            left: 0,
            top: '50%',
            transform: 'translateY(-50%)',
            width: '0.1875rem',
            height: '1.125rem',
            borderRadius: '0 0.125rem 0.125rem 0',
            background: 'var(--primary-accent-color)',
          }}
        />
      )}
      <span
        style={{
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          flexShrink: 0,
          width: '1.375rem',
          height: '1.375rem',
          color: selected ? 'var(--primary-accent-color)' : color.textDim,
          transition: 'color 0.12s',
        }}
      >
        {icon}
      </span>
      <span style={{ flex: 1, minWidth: 0 }}>
        <span
          style={{
            display: 'block',
            fontSize: fontSize.md,
            fontWeight: selected ? 600 : 500,
            color: selected ? color.textPrimary : color.textBody,
            overflow: 'hidden',
            textOverflow: 'ellipsis',
            whiteSpace: 'nowrap',
            transition: 'color 0.12s',
          }}
        >
          {label}
        </span>
        {subtitle && (
          <span
            style={{
              display: 'block',
              fontSize: fontSize.xs,
              color: color.textFaint,
              marginTop: 1,
              overflow: 'hidden',
              textOverflow: 'ellipsis',
              whiteSpace: 'nowrap',
            }}
          >
            {subtitle}
          </span>
        )}
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

export function SidebarDivider() {
  return <div style={{ height: 1, background: color.line, margin: '0.625rem 0' }} />;
}

export function SettingsSection({ title, subtitle, children }: { title: string; subtitle: string; children: React.ReactNode }) {
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
        borderBottom: `1px solid ${color.line}`,
        padding: '1.125rem 1.125rem',
      }}
    >
      <div style={{ display: 'flex', flexDirection: 'column', gap: '0.4375rem' }}>{children}</div>
    </div>
  );
}

export function VersionFooter() {
  const [version, setVersion] = useState('');
  useEffect(() => {
    platformInvoke<string>('get_version')
      .then((v) => setVersion(v))
      .catch(() => {});
  }, []);
  return <p style={styles.versionFooter}>{version ? `v${version}` : ''}</p>;
}
