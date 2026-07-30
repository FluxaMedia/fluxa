import React, { useState } from 'react';
import type { LibraryItem, Meta } from '../core/types';
import { t } from '../i18n';
import { styles } from './libraryScreenStyles';

function itemActivityTime(item: LibraryItem): number {
  const raw = (item as LibraryItem & { savedAt?: string; updatedAt?: string; lastWatchedAt?: string }).savedAt
    ?? (item as LibraryItem & { savedAt?: string; updatedAt?: string; lastWatchedAt?: string }).lastWatchedAt
    ?? item.statusChangedAt
    ?? item.newEpisodeReleasedAt
    ?? item.lastAirDateCheckedAt
    ?? (item as LibraryItem & { updatedAt?: string }).updatedAt;
  const parsed = raw ? Date.parse(raw) : 0;
  return Number.isFinite(parsed) ? parsed : 0;
}

export function HistoryTimeline({ items, onNavigateDetail }: { items: LibraryItem[]; onNavigateDetail: (meta: Meta) => void }) {
  return (
    <div style={styles.historyScroll}>
      {items.map((item) => {
        const at = itemActivityTime(item);
        const progress = (item.timeOffset ?? 0) > 0 && (item.duration ?? 0) > 0
          ? Math.min(100, Math.round(((item.timeOffset ?? 0) / (item.duration ?? 1)) * 100))
          : null;
        const label = item.statusChangedAt
          ? t('library.history_status_changed')
          : item.lastVideoId
            ? t('library.history_watched_episode', item.lastEpisodeSeason ?? 1, item.lastEpisodeNumber ?? '')
            : t('library.history_updated');
        return (
          <button key={`${item.id}:${at}`} style={styles.historyRow} onClick={() => onNavigateDetail(item as unknown as Meta)}>
            <div style={styles.historyDate}>
              <span style={styles.historyDay}>{at ? new Date(at).toLocaleDateString(undefined, { day: '2-digit' }) : '--'}</span>
              <span style={styles.historyMonth}>{at ? new Date(at).toLocaleDateString(undefined, { month: 'short' }) : ''}</span>
            </div>
            {item.poster && <img src={item.poster} alt="" style={styles.historyPoster} />}
            <div style={styles.historyInfo}>
              <p style={styles.historyTitle}>{item.name}</p>
              <p style={styles.historyMeta}>{label}</p>
              {progress != null && progress > 0 && progress < 100 && (
                <div style={styles.historyProgressTrack}>
                  <div style={{ ...styles.historyProgressFill, width: `${progress}%` }} />
                </div>
              )}
            </div>
            <span style={styles.historyTime}>{at ? new Date(at).toLocaleTimeString(undefined, { hour: '2-digit', minute: '2-digit' }) : ''}</span>
          </button>
        );
      })}
    </div>
  );
}

export function CircleBtn({ onClick, size, children }: { onClick: () => void; size: number; children: React.ReactNode }) {
  const [hovered, setHovered] = useState(false);
  return (
    <button
      style={{
        width: size, height: size, minWidth: size, borderRadius: '50%',
        background: hovered ? 'rgba(255,255,255,0.12)' : 'rgba(255,255,255,0.05)',
        border: 'none', cursor: 'pointer', display: 'flex', alignItems: 'center',
        justifyContent: 'center', flexShrink: 0, transition: 'background 0.15s',
      }}
      onClick={onClick}
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
    >
      {children}
    </button>
  );
}

export function TabChip({ active, onClick, children }: { active: boolean; onClick: () => void; children: React.ReactNode }) {
  const [hovered, setHovered] = useState(false);
  return (
    <button
      style={{
        background: active ? '#FFFFFF' : hovered ? 'rgba(255,255,255,0.1)' : 'rgba(255,255,255,0.05)',
        color: active ? '#000000' : '#FFFFFF',
        border: 'none', borderRadius: '1.25rem', padding: '0.5rem 1.25rem',
        fontSize: '0.875rem', fontWeight: 700, cursor: 'pointer',
        transition: 'background 0.15s, color 0.15s',
      }}
      onClick={onClick}
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
    >
      {children}
    </button>
  );
}
