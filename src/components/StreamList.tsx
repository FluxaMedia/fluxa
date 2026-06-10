import React, { useState } from 'react';
import type { Stream } from '../core/types';
import { t } from '../i18n';

interface Props {
  streams: Stream[];
  isLoading?: boolean;
  onPlay?: (stream: Stream) => void;
}

export function StreamList({ streams, isLoading, onPlay }: Props) {
  if (isLoading) {
    return (
      <div style={styles.container}>
        <p style={styles.sectionTitle}>{t('auto.sources')}</p>
        <div style={styles.loadingBox}>
          <div style={styles.spinner} />
        </div>
      </div>
    );
  }

  if (streams.length === 0) {
    return (
      <div style={styles.container}>
        <p style={styles.sectionTitle}>{t('auto.sources')}</p>
        <p style={styles.empty}>{t('sources.none_available')}</p>
      </div>
    );
  }

  return (
    <div style={styles.container}>
      <p style={styles.sectionTitle}>{t('auto.sources')}</p>
      <div style={styles.list}>
        {streams.map((stream, idx) => (
          <StreamRow key={idx} stream={stream} onPlay={onPlay} />
        ))}
      </div>
    </div>
  );
}

function StreamRow({ stream, onPlay }: { stream: Stream; onPlay?: (s: Stream) => void }) {
  const [hovered, setHovered] = useState(false);
  const title = stream.title ?? stream.name ?? stream.description ?? t('auto.stream');
  const isTorrent = !!stream.infoHash;

  return (
    <div
      style={{
        ...styles.row,
        background: hovered ? 'rgba(255,255,255,0.08)' : 'rgba(255,255,255,0.04)',
        boxShadow: hovered ? '0 0 0 2px rgba(255,255,255,0.25)' : 'none',
      }}
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
      onClick={() => onPlay?.(stream)}
    >
      <div style={styles.playIcon}>
        <svg width="16" height="16" viewBox="0 0 24 24" fill="rgba(255,255,255,0.8)">
          <path d="M8 5v14l11-7z"/>
        </svg>
      </div>
      <div style={styles.rowInfo}>
        <p style={styles.rowTitle}>{title}</p>
        {stream.addonName && <p style={styles.addonName}>{stream.addonName}</p>}
      </div>
      {isTorrent && <span style={styles.torrentTag}>{t('auto.torrent')}</span>}
    </div>
  );
}

const styles: Record<string, React.CSSProperties> = {
  container: {
    paddingTop: 24,
  },
  sectionTitle: {
    color: 'rgba(255,255,255,0.4)',
    fontSize: 13,
    fontWeight: 700,
    letterSpacing: '0.8px',
    margin: '0 0 12px',
    fontFamily: 'sans-serif',
  },
  loadingBox: {
    display: 'flex',
    justifyContent: 'center',
    padding: '32px 0',
  },
  spinner: {
    width: 28,
    height: 28,
    border: '3px solid rgba(255,255,255,0.15)',
    borderTopColor: 'rgba(255,255,255,0.8)',
    borderRadius: '50%',
    animation: 'spin 0.8s linear infinite',
  },
  list: {
    display: 'flex',
    flexDirection: 'column',
    gap: 6,
  },
  row: {
    display: 'flex',
    alignItems: 'center',
    gap: 14,
    padding: '12px 16px',
    borderRadius: 12,
    cursor: 'pointer',
    transition: 'background 0.15s, box-shadow 0.15s',
  },
  playIcon: {
    width: 32,
    height: 32,
    borderRadius: '50%',
    background: 'rgba(255,255,255,0.1)',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    flexShrink: 0,
  },
  rowInfo: {
    flex: 1,
    minWidth: 0,
  },
  rowTitle: {
    color: '#FFFFFF',
    fontSize: 14,
    fontWeight: 600,
    margin: 0,
    overflow: 'hidden',
    textOverflow: 'ellipsis',
    whiteSpace: 'nowrap',
    fontFamily: 'sans-serif',
  },
  addonName: {
    color: 'rgba(255,255,255,0.45)',
    fontSize: 12,
    margin: '2px 0 0',
    fontFamily: 'sans-serif',
  },
  torrentTag: {
    background: 'rgba(255,255,255,0.08)',
    color: 'rgba(255,255,255,0.5)',
    fontSize: 10,
    fontWeight: 700,
    padding: '2px 7px',
    borderRadius: 4,
    letterSpacing: '0.5px',
    fontFamily: 'sans-serif',
    flexShrink: 0,
  },
  empty: {
    color: 'rgba(255,255,255,0.4)',
    fontSize: 14,
    fontFamily: 'sans-serif',
  },
};
