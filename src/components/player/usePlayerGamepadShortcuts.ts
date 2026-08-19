import { useEffect, type Dispatch, type MutableRefObject, type SetStateAction } from 'react';
import { onGamepadAction } from '../../platform/gamepadInput';
import { runPlayerAction, type PlayerActionContext } from './playerActions';
import type { FeedbackFlash } from './PlayerOverlayPrimitives';

type TrackPopover = 'audio' | 'sub' | 'speed' | null;

type Bindings = PlayerActionContext & {
  closePlayer: () => Promise<void>;
  showShortcutsHelp: boolean;
  showEpisodePanel: boolean;
  setShowEpisodePanel: Dispatch<SetStateAction<boolean>>;
  episodePanelOpenRef: MutableRefObject<boolean>;
  trackPopover: TrackPopover;
  setTrackPopover: Dispatch<SetStateAction<TrackPopover>>;
  flashFeedback: (icon: FeedbackFlash['icon'], label: string) => void;
};

export function usePlayerGamepadShortcuts(bindings: Bindings) {
  useEffect(() => {
    return onGamepadAction((action) => {
      const {
        closePlayer,
        showShortcutsHelp,
        showEpisodePanel,
        setShowEpisodePanel,
        episodePanelOpenRef,
        trackPopover,
        setTrackPopover,
        setShowShortcutsHelp,
      } = bindings;

      if (action === 'back') {
        if (showShortcutsHelp) {
          setShowShortcutsHelp(false);
          return;
        }
        if (trackPopover) {
          setTrackPopover(null);
          return;
        }
        if (showEpisodePanel) {
          setShowEpisodePanel(false);
          episodePanelOpenRef.current = false;
          return;
        }
        void closePlayer();
        return;
      }
      if (action === 'left') {
        runPlayerAction('player_seek_back', bindings);
        return;
      }
      if (action === 'right') {
        runPlayerAction('player_seek_forward', bindings);
        return;
      }
      if (action === 'up') {
        runPlayerAction('player_volume_up', bindings);
        return;
      }
      if (action === 'down') {
        runPlayerAction('player_volume_down', bindings);
        return;
      }
      if (action === 'enter') {
        runPlayerAction('player_play_pause', bindings);
        return;
      }
      if (action === 'previous') {
        runPlayerAction('player_seek_big_back', bindings);
        return;
      }
      if (action === 'next') {
        runPlayerAction('player_next_episode', bindings) || runPlayerAction('player_seek_big_forward', bindings);
        return;
      }
      if (action === 'rewind') {
        runPlayerAction('player_frame_step_back', bindings);
        return;
      }
      if (action === 'fastForward') {
        runPlayerAction('player_frame_step_forward', bindings);
        return;
      }
    });
  }, [bindings]);
}
