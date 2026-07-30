import type { ComponentProps, ReactNode } from 'react';
import { CastPopover } from './CastPopover';
import { TorrentStatsPopover } from './TorrentStatsPopover';
import { SegmentMarkerPanel } from './SegmentMarkerPanel';
import { PlayerSettingsPopover } from './PlayerSettingsPopover';
import { PlayerShortcutsDialog } from './PlayerShortcutsDialog';
import { PlayerStatsOverlay } from './PlayerStatsOverlay';
import { PlayerContextMenu } from './PlayerContextMenu';

export function PlayerSupplementalPanels({ cast, torrent, marker, settings, shortcuts, stats, context }: { cast?: ComponentProps<typeof CastPopover>; torrent?: ComponentProps<typeof TorrentStatsPopover>; marker?: ComponentProps<typeof SegmentMarkerPanel>; settings?: ComponentProps<typeof PlayerSettingsPopover>; shortcuts?: ComponentProps<typeof PlayerShortcutsDialog>; stats?: ComponentProps<typeof PlayerStatsOverlay>; context?: ComponentProps<typeof PlayerContextMenu> }) {
  return <>{cast && <CastPopover {...cast} />}{torrent && <TorrentStatsPopover {...torrent} />}{marker && <SegmentMarkerPanel {...marker} />}{settings && <PlayerSettingsPopover {...settings} />}{shortcuts && <PlayerShortcutsDialog {...shortcuts} />}{stats && <PlayerStatsOverlay {...stats} />}{context && <PlayerContextMenu {...context} />}</>;
}
