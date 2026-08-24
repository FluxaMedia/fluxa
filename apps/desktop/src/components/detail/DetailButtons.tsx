import React, { useState } from 'react';
import { Play } from 'lucide-react';
import { t } from '../../i18n';
import { Button, color, fontSize, radius, space, weight } from '../../design';
import { MS } from './detailStyles';

export function ModernPlayButton({
  continueLabel,
  hasProgress,
  onClick,
}: {
  continueLabel: string | null;
  hasProgress: boolean;
  onClick: () => void;
}) {
  const text = continueLabel
    ? hasProgress
      ? t('format.continue_episode', continueLabel)
      : t('format.play_episode', continueLabel)
    : t('common.play');
  return (
    <Button
      variant="primary"
      onClick={onClick}
      icon={<Play size={17} fill="currentColor" strokeWidth={0} />}
      style={{ minWidth: '9.25rem', fontSize: fontSize.md }}
    >
      {text}
    </Button>
  );
}

export function ModernIconBtn({
  title,
  active,
  onClick,
  children,
}: {
  title: string;
  active?: boolean;
  onClick: () => void;
  children: React.ReactNode;
}) {
  const [hovered, setHovered] = useState(false);
  return (
    <button
      title={title}
      aria-label={title}
      onClick={onClick}
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
      style={{
        width: '2.5rem',
        height: '2.5rem',
        borderRadius: radius.circle,
        border: 'none',
        background: hovered ? color.fillHover : 'transparent',
        color: active || hovered ? color.textPrimary : color.textMuted,
        display: 'grid',
        placeItems: 'center',
        padding: 0,
        flexShrink: 0,
        cursor: 'pointer',
        transition: 'background 0.12s ease, color 0.12s ease',
      }}
    >
      {children}
    </button>
  );
}

export function ModernTabBar({
  tabs,
  active,
  onChange,
  trailing,
}: {
  tabs: Array<{ id: string; label: string }>;
  active: string;
  onChange: (id: string) => void;
  trailing?: React.ReactNode;
}) {
  return (
    <div style={MS.tabRow}>
      <div style={MS.tabBar}>
        {tabs.map((tab) => (
          <button
            key={tab.id}
            style={{ ...MS.tabBtn, ...(tab.id === active ? MS.tabBtnActive : {}) }}
            onClick={(e) => {
              onChange(tab.id);
              e.currentTarget.blur();
            }}
          >
            {tab.label}
          </button>
        ))}
      </div>
      {trailing && <div style={MS.tabTrailing}>{trailing}</div>}
    </div>
  );
}

export function AgeBadge({ label }: { label: string }) {
  return (
    <span
      style={{
        border: `1px solid ${color.outline}`,
        borderRadius: radius.xs,
        padding: `0 ${space[1]}`,
        fontSize: fontSize.xs,
        fontWeight: weight.semibold,
        color: color.textStrong,
        flexShrink: 0,
      }}
    >
      {label}
    </span>
  );
}
