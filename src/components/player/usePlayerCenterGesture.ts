import { useCallback, useRef, type MutableRefObject } from 'react';
import { sendCmd, type FeedbackFlash } from './PlayerOverlayPrimitives';

export function usePlayerCenterGesture({
  playbackSpeed,
  preSpeedRef,
  pausedRef,
  episodePanelOpenRef,
  showEpisodePanel,
  setShowEpisodePanel,
  trackPopover,
  setTrackPopover,
  setPaused,
  resetActivity,
  flashFeedback,
  toggleFullscreen,
  holdTimerRef,
  holdActiveRef,
}: {
  playbackSpeed: number;
  preSpeedRef: MutableRefObject<number>;
  pausedRef: MutableRefObject<boolean>;
  episodePanelOpenRef: MutableRefObject<boolean>;
  showEpisodePanel: boolean;
  setShowEpisodePanel: (visible: boolean) => void;
  trackPopover: string | null;
  setTrackPopover: (value: null) => void;
  setPaused: (updater: (paused: boolean) => boolean) => void;
  resetActivity: () => void;
  flashFeedback: (icon: FeedbackFlash['icon'], label: string) => void;
  toggleFullscreen: () => Promise<void>;
  holdTimerRef: MutableRefObject<ReturnType<typeof setTimeout> | null>;
  holdActiveRef: MutableRefObject<boolean>;
}) {
  const clickTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const holdJustEndedRef = useRef(false);

  const onPointerDown = useCallback(
    (event: React.PointerEvent) => {
      if (event.button !== 0 && event.pointerType === 'mouse') return;
      resetActivity();
      preSpeedRef.current = playbackSpeed;
      holdTimerRef.current = setTimeout(() => {
        holdActiveRef.current = true;
        sendCmd('set speed 2.00');
        flashFeedback('speed', '2×');
      }, 300);
    },
    [flashFeedback, playbackSpeed, preSpeedRef, resetActivity],
  );

  const releaseHold = useCallback(() => {
    if (holdTimerRef.current) {
      clearTimeout(holdTimerRef.current);
      holdTimerRef.current = null;
    }
    if (!holdActiveRef.current) return;
    holdActiveRef.current = false;
    holdJustEndedRef.current = true;
    sendCmd(`set speed ${preSpeedRef.current.toFixed(2)}`);
  }, [preSpeedRef]);

  const onClick = useCallback(() => {
    if (holdJustEndedRef.current) {
      holdJustEndedRef.current = false;
      return;
    }
    resetActivity();
    if (showEpisodePanel) {
      setShowEpisodePanel(false);
      episodePanelOpenRef.current = false;
      return;
    }
    if (trackPopover) {
      setTrackPopover(null);
      return;
    }
    if (clickTimerRef.current) {
      clearTimeout(clickTimerRef.current);
      clickTimerRef.current = null;
      void toggleFullscreen();
      return;
    }
    clickTimerRef.current = setTimeout(() => {
      clickTimerRef.current = null;
      flashFeedback(pausedRef.current ? 'play' : 'pause', '');
      setPaused((paused) => !paused);
      sendCmd('cycle pause');
    }, 250);
  }, [
    episodePanelOpenRef,
    flashFeedback,
    pausedRef,
    resetActivity,
    setPaused,
    setShowEpisodePanel,
    setTrackPopover,
    showEpisodePanel,
    toggleFullscreen,
    trackPopover,
  ]);

  return { onCenterPointerDown: onPointerDown, releaseCenterHold: releaseHold, onCenterClick: onClick };
}
