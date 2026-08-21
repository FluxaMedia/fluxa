import React, { useEffect, useState } from 'react';
import type { Stream, StreamBadge, StreamBadgeRules } from '../core/types';
import { t } from '../i18n';
import { coreMatchStreamBadges } from '../core/engine';
import { useStreamBadgeRules, type StreamBadgePosition } from '../hooks/useStreamBadgeRules';
import { StreamBadgeChips } from './StreamBadgeChips';

interface Props {
  streams: Stream[];
  isLoading?: boolean;
  onPlay?: (stream: Stream) => void;
}

export function StreamList({ streams, isLoading, onPlay }: Props) {
  const { rules, loaded, badgePosition } = useStreamBadgeRules();

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
          <StreamRow
            key={stream.url ?? stream.infoHash ?? idx}
            stream={stream}
            onPlay={onPlay}
            badgeRules={loaded ? rules : null}
            badgePosition={badgePosition}
          />
        ))}
      </div>
    </div>
  );
}

function StreamRow({
  stream,
  onPlay,
  badgeRules,
  badgePosition,
}: {
  stream: Stream;
  onPlay?: (s: Stream) => void;
  badgeRules: StreamBadgeRules | null;
  badgePosition: StreamBadgePosition;
}) {
  const [hovered, setHovered] = useState(false);
  const [badges, setBadges] = useState<StreamBadge[]>([]);
  const title = stream.title ?? stream.name ?? stream.description ?? t('auto.stream');
  const isTorrent = !!stream.infoHash;

  useEffect(() => {
    let cancelled = false;
    if (!badgeRules || badgeRules.imports.length === 0) {
      setBadges([]);
      return;
    }
    void coreMatchStreamBadges(stream, badgeRules).then((matched) => {
      if (!cancelled) setBadges(matched);
    });
    return () => {
      cancelled = true;
    };
  }, [stream, badgeRules]);

  return (
    <div
      style={{
        ...styles.row,
        background: hovered ? 'var(--fluxa-fill-strong)' : 'var(--fluxa-background-elevated)',
        boxShadow: hovered ? '0 0 0 0.125rem var(--fluxa-fill-active)' : 'none',
      }}
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
      onClick={() => onPlay?.(stream)}
    >
      <div style={styles.playIcon}>
        <svg width="16" height="16" viewBox="0 0 24 24" fill="var(--fluxa-text-secondary)">
          <path d="M8 5v14l11-7z" />
        </svg>
      </div>
      <div style={styles.rowInfo}>
        {badgePosition === 'top' && <StreamBadgeChips badges={badges} />}
        <p style={styles.rowTitle}>{title}</p>
        {stream.addonName && <p style={styles.addonName}>{stream.addonName}</p>}
        {badgePosition === 'bottom' && <StreamBadgeChips badges={badges} />}
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
    color: 'var(--fluxa-text-muted)',
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
    border: '0.1875rem solid var(--fluxa-border)',
    borderTopColor: 'var(--fluxa-text-secondary)',
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
    background: 'var(--fluxa-fill)',
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
    color: 'var(--fluxa-text-primary)',
    fontSize: '0.875rem',
    fontWeight: 600,
    margin: 0,
    overflow: 'hidden',
    textOverflow: 'ellipsis',
    whiteSpace: 'nowrap',
  },
  addonName: {
    color: 'var(--fluxa-text-muted)',
    fontSize: '0.75rem',
    margin: '0.125rem 0 0',
  },
  torrentTag: {
    background: 'var(--fluxa-fill)',
    color: 'var(--fluxa-text-muted)',
    fontSize: '0.625rem',
    fontWeight: 700,
    padding: '0.125rem 0.4375rem',
    borderRadius: '0.25rem',
    letterSpacing: '0.0313rem',
    flexShrink: 0,
  },
  empty: {
    color: 'var(--fluxa-text-muted)',
    fontSize: '0.875rem',
  },
};
