import { t } from '../../i18n';
import { platformEmit as emit } from '../../platform/browser';
import { sendCmd, type FeedbackFlash } from './PlayerOverlayPrimitives';
import type { Dispatch, MutableRefObject, SetStateAction } from 'react';

export type PlayerActionContext = {
  flashFeedback: (icon: FeedbackFlash['icon'], label: string) => void;
  nextEpSubtitle: string;
  playbackSpeed: number;
  setPlaybackSpeed: Dispatch<SetStateAction<number>>;
  toggleFullscreen: () => Promise<void>;
  toggleMiniPlayer: () => Promise<void>;
  setShowShortcutsHelp: Dispatch<SetStateAction<boolean>>;
  startSeekOverlay: () => void;
  triggerActiveSkip: () => boolean;
  cycleAbLoopRef: MutableRefObject<() => void>;
  openCastPopoverRef: MutableRefObject<() => Promise<void>>;
  takeScreenshotRef: MutableRefObject<() => Promise<void>>;
  cycleAnime4kModeRef: MutableRefObject<(direction: 1 | -1) => void>;
  pausedRef: MutableRefObject<boolean>;
  setPaused: Dispatch<SetStateAction<boolean>>;
  setShowStats: Dispatch<SetStateAction<boolean>>;
};

export function runPlayerAction(action: string, ctx: PlayerActionContext): boolean {
  const { flashFeedback, nextEpSubtitle, playbackSpeed, setPlaybackSpeed, toggleFullscreen, toggleMiniPlayer, setShowShortcutsHelp, startSeekOverlay, triggerActiveSkip, cycleAbLoopRef, openCastPopoverRef, takeScreenshotRef, cycleAnime4kModeRef, pausedRef, setPaused, setShowStats } = ctx;

  switch (action) {
    case 'player_seek_back':
      startSeekOverlay();
      flashFeedback('seekBack', '-10s');
      sendCmd('seek -10 relative');
      return true;
    case 'player_seek_forward':
      startSeekOverlay();
      flashFeedback('seekFwd', '+10s');
      sendCmd('seek 10 relative');
      return true;
    case 'player_volume_up':
      flashFeedback('volume', '');
      sendCmd('add volume 5');
      return true;
    case 'player_volume_down':
      flashFeedback('volume', '');
      sendCmd('add volume -5');
      return true;
    case 'player_seek_big_back':
      startSeekOverlay();
      flashFeedback('seekBack', t('player.seek_big_back'));
      sendCmd('seek -60 relative');
      return true;
    case 'player_seek_big_forward':
      startSeekOverlay();
      flashFeedback('seekFwd', t('player.seek_big_forward'));
      sendCmd('seek 60 relative');
      return true;
    case 'player_play_pause': {
      const icon = pausedRef.current ? 'play' : 'pause';
      flashFeedback(icon, '');
      setPaused((prev) => !prev);
      sendCmd('cycle pause');
      return true;
    }
    case 'player_speed_decrease': {
      const next = Math.max(0.25, parseFloat((playbackSpeed - 0.25).toFixed(2)));
      sendCmd(`set speed ${next}`);
      setPlaybackSpeed(next);
      flashFeedback('speed', t('player.speed_decrease'));
      return true;
    }
    case 'player_speed_increase': {
      const next = Math.min(4, parseFloat((playbackSpeed + 0.25).toFixed(2)));
      sendCmd(`set speed ${next}`);
      setPlaybackSpeed(next);
      flashFeedback('speed', t('player.speed_increase'));
      return true;
    }
    case 'player_cycle_subtitle':
      sendCmd('cycle sub');
      return true;
    case 'player_cycle_audio':
      sendCmd('cycle audio');
      return true;
    case 'player_toggle_stats':
      setShowStats((s) => !s);
      return true;
    case 'player_frame_step_forward':
      sendCmd('frame-step');
      flashFeedback('seekFwd', t('player.frame_step'));
      return true;
    case 'player_frame_step_back':
      sendCmd('frame-back-step');
      flashFeedback('seekBack', t('player.frame_back_step'));
      return true;
    case 'player_skip_active':
      return triggerActiveSkip();
    case 'player_next_episode':
      if (!nextEpSubtitle) return false;
      void emit('native-player-next-episode', null);
      return true;
    case 'player_mute':
      sendCmd('cycle mute');
      return true;
    case 'player_sub_delay_earlier':
      flashFeedback('subDelay', t('player.subtitle_delay_earlier'));
      sendCmd('add sub-delay -0.100');
      return true;
    case 'player_sub_delay_later':
      flashFeedback('subDelay', t('player.subtitle_delay_later'));
      sendCmd('add sub-delay 0.100');
      return true;
    case 'player_fullscreen':
      void toggleFullscreen();
      return true;
    case 'player_toggle_shortcuts_help':
      setShowShortcutsHelp((s) => !s);
      return true;
    case 'player_toggle_pip':
      void toggleMiniPlayer();
      return true;
    case 'player_open_cast':
      void openCastPopoverRef.current();
      return true;
    case 'player_ab_loop':
      cycleAbLoopRef.current();
      return true;
    case 'player_screenshot':
      void takeScreenshotRef.current();
      return true;
    case 'player_seek_start':
      startSeekOverlay();
      flashFeedback('seekBack', '0%');
      sendCmd('seek 0 absolute');
      return true;
    case 'player_seek_end':
      startSeekOverlay();
      flashFeedback('seekFwd', '100%');
      sendCmd('seek 100 absolute-percent');
      return true;
    case 'player_anime4k_mode_next':
      cycleAnime4kModeRef.current(1);
      return true;
    case 'player_anime4k_mode_prev':
      cycleAnime4kModeRef.current(-1);
      return true;
    default:
      return false;
  }
}
