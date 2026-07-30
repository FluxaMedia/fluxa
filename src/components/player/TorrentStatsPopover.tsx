import { type RefObject } from 'react';
import { t } from '../../i18n';
import type { TorrentStats } from '../../core/mpvPlayer';
import { Popover } from '../ui/Popover';

interface TorrentStatsPopoverProps {
  stats: TorrentStats | null;
  anchorRef: RefObject<HTMLElement | null>;
  onClose: () => void;
}

function formatSpeed(bytesPerSec: number): string {
  if (bytesPerSec >= 1024 * 1024) return `${(bytesPerSec / (1024 * 1024)).toFixed(2)} MB/s`;
  return `${(bytesPerSec / 1024).toFixed(0)} KB/s`;
}

function Row({ label, value }: { label: string; value: string }) {
  return (
    <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', gap: '1.25rem', fontSize: '0.8125rem' }}>
      <span style={{ color: 'rgba(255,255,255,0.55)' }}>{label}</span>
      <span style={{ color: '#fff', fontWeight: 600, fontVariantNumeric: 'tabular-nums' }}>{value}</span>
    </div>
  );
}

export function TorrentStatsPopover({ stats, anchorRef, onClose }: TorrentStatsPopoverProps) {
  return (
    <Popover open onClose={onClose} anchorRef={anchorRef} placement="top" width="15rem" padding="0.75rem 1rem">
      <div style={{ color: 'rgba(255,255,255,0.45)', fontSize: '0.6875rem', fontWeight: 700, letterSpacing: '0.05rem', marginBottom: '0.625rem', textTransform: 'uppercase' }}>
        {t('player.torrent_stats_title')}
      </div>
      {!stats ? (
        <div style={{ color: 'rgba(255,255,255,0.4)', fontSize: '0.8125rem' }}>{t('player.status_starting_torrent')}</div>
      ) : (
        <div style={{ display: 'flex', flexDirection: 'column', gap: '0.5rem' }}>
          <Row label={t('player.torrent_seeders')} value={`${stats.active_peers} / ${stats.total_peers}`} />
          <Row label={t('player.torrent_download_speed')} value={formatSpeed(stats.download_speed)} />
          <Row label={t('player.torrent_progress')} value={`${stats.progress.toFixed(1)}%`} />
        </div>
      )}
    </Popover>
  );
}
