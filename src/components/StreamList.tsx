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
          <StreamRow key={stream.url ?? stream.infoHash ?? idx} stream={stream} onPlay={onPlay} />
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
        boxShadow: hovered ? '0 0 0 0.125rem rgba(255,255,255,0.25)' : 'none',
      }}
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
      onClick={() => onPlay?.(stream)}
    >
      <div style={styles.playIcon}>
        <svg width="16" height="16" viewBox="0 0 24 24" fill="rgba(255,255,255,0.8)">
          <path d="M8 5v14l11-7z" />
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
    paddingTop: '1.5rem',
  },
  sectionTitle: {
    color: 'rgba(255,255,255,0.4)',
    fontSize: '0.8125rem',
    fontWeight: 700,
    letterSpacing: '0.05rem',
    margin: '0 0 0.75rem',
  },
  loadingBox: {
    display: 'flex',
    justifyContent: 'center',
    padding: '2rem 0',
  },
  spinner: {
    width: '1.75rem',
    height: '1.75rem',
    border: '0.1875rem solid rgba(255,255,255,0.15)',
    borderTopColor: 'rgba(255,255,255,0.8)',
    borderRadius: '50%',
    animation: 'spin 0.8s linear infinite',
  },
  list: {
    display: 'flex',
    flexDirection: 'column',
    gap: '0.375rem',
  },
  row: {
    display: 'flex',
    alignItems: 'center',
    gap: '0.875rem',
    padding: '0.75rem 1rem',
    borderRadius: '0.75rem',
    cursor: 'pointer',
    transition: 'background 0.15s, box-shadow 0.15s',
  },
  playIcon: {
    width: '2rem',
    height: '2rem',
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
    fontSize: '0.875rem',
    fontWeight: 600,
    margin: 0,
    overflow: 'hidden',
    textOverflow: 'ellipsis',
    whiteSpace: 'nowrap',
  },
  addonName: {
    color: 'rgba(255,255,255,0.45)',
    fontSize: '0.75rem',
    margin: '0.125rem 0 0',
  },
  torrentTag: {
    background: 'rgba(255,255,255,0.08)',
    color: 'rgba(255,255,255,0.5)',
    fontSize: '0.625rem',
    fontWeight: 700,
    padding: '0.125rem 0.4375rem',
    borderRadius: '0.25rem',
    letterSpacing: '0.0313rem',
    flexShrink: 0,
  },
  empty: {
    color: 'rgba(255,255,255,0.4)',
    fontSize: '0.875rem',
  },
};
