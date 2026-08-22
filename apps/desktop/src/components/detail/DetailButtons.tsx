import React from 'react';
import { Play } from 'lucide-react';
import { t } from '../../i18n';
import { Button, IconButton, color, fontSize, radius, space, weight } from '../../design';
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
  return (
    <IconButton title={title} aria-label={title} active={active} size="2.375rem" onClick={onClick}>
      {children}
    </IconButton>
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
