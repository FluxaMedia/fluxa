import React, { useState } from 'react';
import { t } from '../../i18n';
import { MS } from './detailStyles';

export function ModernPlayButton({ continueLabel, hasProgress, onClick }: { continueLabel: string | null; hasProgress: boolean; onClick: () => void }) {
  const [hovered, setHovered] = useState(false);
  const text = continueLabel
    ? hasProgress ? t('format.continue_episode', continueLabel) : t('format.play_episode', continueLabel)
    : t('common.play');
  return (
    <button
      style={{ ...MS.playBtn, background: hovered ? '#282a35' : '#1c1d25' }}
      onClick={onClick}
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
    >
      <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor" style={{ marginRight: 10, flexShrink: 0 }}>
        <path d="M8 5v14l11-7z" />
      </svg>
      <span style={{ overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>{text}</span>
    </button>
  );
}

export function ModernIconBtn({ title, active, onClick, children }: { title: string; active?: boolean; onClick: () => void; children: React.ReactNode }) {
  const [hovered, setHovered] = useState(false);
  return (
    <button
      title={title}
      style={{
        width: 44, height: 44, borderRadius: '50%',
        border: `2px solid rgba(255,255,255,${hovered || active ? 0.7 : 0.28})`,
        background: active ? 'rgba(255,255,255,0.12)' : 'transparent',
        color: '#fff', display: 'flex', alignItems: 'center', justifyContent: 'center',
        cursor: 'pointer', padding: 0, flexShrink: 0,
      }}
      onClick={onClick}
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
    >
      {children}
    </button>
  );
}

export function ModernTabBar({ tabs, active, onChange }: { tabs: Array<{ id: string; label: string }>; active: string; onChange: (id: string) => void }) {
  return (
    <div style={MS.tabBar}>
      {tabs.map((tab) => (
        <button key={tab.id} style={{ ...MS.tabBtn, ...(tab.id === active ? MS.tabBtnActive : {}) }} onClick={() => onChange(tab.id)}>
          {tab.label}
        </button>
      ))}
    </div>
  );
}
