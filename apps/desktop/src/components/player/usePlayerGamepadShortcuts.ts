import { useEffect, type Dispatch, type MutableRefObject, type SetStateAction } from 'react';
import { onGamepadAction } from '../../platform/gamepadInput';
import { PLAYER_ACTION_FOR_GAMEPAD } from '../../core/gamepadBindings';
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
      if (action === 'next') {
        runPlayerAction('player_next_episode', bindings) || runPlayerAction('player_seek_big_forward', bindings);
        return;
      }
      const mapped = PLAYER_ACTION_FOR_GAMEPAD[action];
      if (mapped) runPlayerAction(mapped, bindings);
    });
  }, [bindings]);
}
