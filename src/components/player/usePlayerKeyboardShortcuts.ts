import { useEffect, type Dispatch, type MutableRefObject, type SetStateAction } from 'react';
import { t } from '../../i18n';
import { comboFromEvent, findActionForCombo, type ShortcutOverrides } from '../../core/shortcuts';
import { sendCmd, type Chapter, type FeedbackFlash } from './PlayerOverlayPrimitives';
import { runPlayerAction } from './playerActions';

type TrackPopover = 'audio' | 'sub' | 'speed' | null;
type Point = { x: number; y: number } | null;

type Bindings = {
  closePlayer: () => Promise<void>;
  contextMenu: Point;
  setContextMenu: Dispatch<SetStateAction<Point>>;
  flashFeedback: (icon: FeedbackFlash['icon'], label: string) => void;
  nextEpSubtitle: string;
  playbackSpeed: number;
  setPlaybackSpeed: Dispatch<SetStateAction<number>>;
  setPlayerFullscreen: (next: boolean) => Promise<void>;
  toggleFullscreen: () => Promise<void>;
  toggleMiniPlayer: () => Promise<void>;
  shortcutOverrides: ShortcutOverrides;
  showEpisodePanel: boolean;
  setShowEpisodePanel: Dispatch<SetStateAction<boolean>>;
  showShortcutsHelp: boolean;
  setShowShortcutsHelp: Dispatch<SetStateAction<boolean>>;
  startSeekOverlay: () => void;
  trackPopover: TrackPopover;
  setTrackPopover: Dispatch<SetStateAction<TrackPopover>>;
  triggerActiveSkip: () => boolean;
  episodePanelOpenRef: MutableRefObject<boolean>;
  isFullscreenRef: MutableRefObject<boolean>;
  holdTimerRef: MutableRefObject<ReturnType<typeof setTimeout> | null>;
  holdActiveRef: MutableRefObject<boolean>;
  preSpeedRef: MutableRefObject<number>;
  pausedRef: MutableRefObject<boolean>;
  posRef: MutableRefObject<number>;
  durRef: MutableRefObject<number>;
  chaptersRef: MutableRefObject<Chapter[]>;
  cycleAbLoopRef: MutableRefObject<() => void>;
  openCastPopoverRef: MutableRefObject<() => Promise<void>>;
  takeScreenshotRef: MutableRefObject<() => Promise<void>>;
  cycleAnime4kModeRef: MutableRefObject<(direction: 1 | -1) => void>;
  setPaused: Dispatch<SetStateAction<boolean>>;
  setShowStats: Dispatch<SetStateAction<boolean>>;
};

export function usePlayerKeyboardShortcuts(bindings: Bindings) {
  const {
    closePlayer,
    contextMenu,
    setContextMenu,
    flashFeedback,
    nextEpSubtitle,
    playbackSpeed,
    setPlaybackSpeed,
    setPlayerFullscreen,
    toggleFullscreen,
    toggleMiniPlayer,
    shortcutOverrides,
    showEpisodePanel,
    setShowEpisodePanel,
    showShortcutsHelp,
    setShowShortcutsHelp,
    startSeekOverlay,
    trackPopover,
    setTrackPopover,
    triggerActiveSkip,
    episodePanelOpenRef,
    isFullscreenRef,
    holdTimerRef,
    holdActiveRef,
    preSpeedRef,
    pausedRef,
    posRef,
    durRef,
    chaptersRef,
    cycleAbLoopRef,
    openCastPopoverRef,
    takeScreenshotRef,
    cycleAnime4kModeRef,
    setPaused,
    setShowStats,
  } = bindings;
  useEffect(() => {
    const onKeyDown = (e: KeyboardEvent) => {
      if (e.code === 'Space') {
        e.preventDefault();
        if (holdTimerRef.current) return;
        holdActiveRef.current = false;
        preSpeedRef.current = playbackSpeed;
        holdTimerRef.current = setTimeout(() => {
          holdActiveRef.current = true;
          sendCmd('set speed 2.00');
          flashFeedback('speed', '2×');
        }, 300);
        return;
      }
      if (e.shiftKey && /^Digit[1-9]$/.test(e.code)) {
        const index = parseInt(e.code.replace('Digit', ''), 10) - 1;
        if (index < chaptersRef.current.length) {
          e.preventDefault();
          sendCmd(`set chapter ${index}`);
          flashFeedback('seekFwd', chaptersRef.current[index].title || `${t('player.chapter')} ${index + 1}`);
        }
        return;
      }
      if (/^Digit[1-9]$/.test(e.code)) {
        e.preventDefault();
        const pct = parseInt(e.code.replace('Digit', ''), 10) * 10;
        startSeekOverlay();
        flashFeedback(pct < (posRef.current / Math.max(1, durRef.current)) * 100 ? 'seekBack' : 'seekFwd', `${pct}%`);
        sendCmd(`seek ${pct} absolute-percent`);
        return;
      }
      if (e.code === 'Digit0') {
        e.preventDefault();
        startSeekOverlay();
        flashFeedback('seekBack', '0%');
        sendCmd('seek 0 absolute');
        return;
      }
      if (e.code === 'Enter') {
        if (triggerActiveSkip()) e.preventDefault();
        return;
      }
      if (e.code === 'F11') {
        e.preventDefault();
        void toggleFullscreen();
        return;
      }
      if (e.code === 'Escape') {
        e.preventDefault();
        if (showShortcutsHelp) {
          setShowShortcutsHelp(false);
          return;
        }
        if (contextMenu) {
          setContextMenu(null);
          return;
        }
        if (showEpisodePanel) {
          setShowEpisodePanel(false);
          episodePanelOpenRef.current = false;
          return;
        }
        if (trackPopover) {
          setTrackPopover(null);
          return;
        }
        if (isFullscreenRef.current) {
          void setPlayerFullscreen(false);
        }
        return;
      }
      if (e.code === 'Backspace') {
        if (contextMenu || showEpisodePanel || trackPopover || showShortcutsHelp) return;
        e.preventDefault();
        void closePlayer();
        return;
      }

      const action = findActionForCombo(comboFromEvent(e), 'player', shortcutOverrides);
      if (
        action &&
        runPlayerAction(action, {
          flashFeedback,
          nextEpSubtitle,
          playbackSpeed,
          setPlaybackSpeed,
          toggleFullscreen,
          toggleMiniPlayer,
          setShowShortcutsHelp,
          startSeekOverlay,
          triggerActiveSkip,
          cycleAbLoopRef,
          openCastPopoverRef,
          takeScreenshotRef,
          cycleAnime4kModeRef,
          pausedRef,
          setPaused,
          setShowStats,
        })
      ) {
        e.preventDefault();
      }
    };

    const onKeyUp = (e: KeyboardEvent) => {
      if (e.code === 'Space') {
        if (holdTimerRef.current) {
          clearTimeout(holdTimerRef.current);
          holdTimerRef.current = null;
        }
        if (holdActiveRef.current) {
          holdActiveRef.current = false;
          sendCmd(`set speed ${preSpeedRef.current.toFixed(2)}`);
        } else {
          const icon = pausedRef.current ? 'play' : 'pause';
          flashFeedback(icon, '');
          setPaused((prev) => !prev);
          sendCmd('cycle pause');
        }
      }
    };

    window.addEventListener('keydown', onKeyDown);
    window.addEventListener('keyup', onKeyUp);
    return () => {
      window.removeEventListener('keydown', onKeyDown);
      window.removeEventListener('keyup', onKeyUp);
    };
  }, [bindings]);
}
