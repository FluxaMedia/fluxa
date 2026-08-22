import { AudioLines, Camera, Clock, Gauge, Info, Repeat } from 'lucide-react';
import { t } from '../../i18n';
import { Popover } from '../ui/Popover';

interface Props {
  point: { x: number; y: number };
  abLoopStage: 'none' | 'a' | 'ab';
  showStats: boolean;
  onClose: () => void;
  onCycleAbLoop: () => void;
  onCopyTimestamp: () => void;
  onToggleStats: () => void;
  onToggleShortcuts: () => void;
  onOpenAudioTracks: () => void;
  onScreenshot: () => void;
}

const rowStyle = (active = false): React.CSSProperties => ({
  display: 'flex',
  alignItems: 'center',
  gap: '0.625rem',
  width: '100%',
  background: 'none',
  border: 'none',
  color: active ? 'var(--primary-accent-color)' : 'rgba(255,255,255,0.85)',
  fontSize: '0.8125rem',
  padding: '0.5rem 0.875rem',
  cursor: 'pointer',
  textAlign: 'left',
});

export function PlayerContextMenu({
  point,
  abLoopStage,
  showStats,
  onClose,
  onCycleAbLoop,
  onCopyTimestamp,
  onToggleStats,
  onToggleShortcuts,
  onOpenAudioTracks,
  onScreenshot,
}: Props) {
  const select = (action: () => void) => () => {
    action();
    onClose();
  };
  return (
    <Popover open onClose={onClose} point={point} width="11.25rem">
      <button className="ui-popover-row" onClick={select(onCycleAbLoop)} style={rowStyle(abLoopStage !== 'none')}>
        <Repeat size={15} />
        {abLoopStage === 'none' ? t('player.ab_loop') : abLoopStage === 'a' ? t('player.ab_loop_a_set') : t('player.ab_loop_active')}
      </button>
      <button className="ui-popover-row" onClick={select(onCopyTimestamp)} style={rowStyle()}>
        <Clock size={15} />
        {t('player.copy_timestamp')}
      </button>
      <button className="ui-popover-row" onClick={select(onToggleStats)} style={rowStyle(showStats)}>
        <Info size={15} />
        {t('player.stats')}
      </button>
      <button className="ui-popover-row" onClick={select(onToggleShortcuts)} style={rowStyle()}>
        <Gauge size={15} />
        {t('player.shortcuts_help')}
      </button>
      <button className="ui-popover-row" onClick={select(onOpenAudioTracks)} style={rowStyle()}>
        <AudioLines size={15} />
        {t('player.track_info')}
      </button>
      <button className="ui-popover-row" onClick={select(onScreenshot)} style={rowStyle()}>
        <Camera size={15} />
        {t('player.screenshot')}
      </button>
    </Popover>
  );
}
