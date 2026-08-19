import React, { useState } from 'react';
import { Play } from 'lucide-react';
import { t } from '../../i18n';
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
  const [hovered, setHovered] = useState(false);
  const text = continueLabel
    ? hasProgress
      ? t('format.continue_episode', continueLabel)
      : t('format.play_episode', continueLabel)
    : t('common.play');
  return (
    <button
      style={{ ...MS.playBtn, background: hovered ? '#e2e2e2' : '#FFFFFF' }}
      onClick={onClick}
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
    >
      <Play size={18} fill="currentColor" strokeWidth={0} style={{ marginRight: '0.625rem', flexShrink: 0 }} />
      <span style={{ overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>{text}</span>
    </button>
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
      style={{
        width: '2.75rem',
        height: '2.75rem',
        borderRadius: '50%',
        border: `0.125rem solid rgba(255,255,255,${hovered || active ? 0.7 : 0.28})`,
        background: active ? 'rgba(255,255,255,0.12)' : 'transparent',
        color: '#fff',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        cursor: 'pointer',
        padding: 0,
        flexShrink: 0,
      }}
      onClick={onClick}
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
    >
      {children}
    </button>
  );
}

export function ModernTabBar({
  tabs,
  active,
  onChange,
}: {
  tabs: Array<{ id: string; label: string }>;
  active: string;
  onChange: (id: string) => void;
}) {
  return (
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
  );
}
