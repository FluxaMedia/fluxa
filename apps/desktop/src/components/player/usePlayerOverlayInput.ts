import { useCallback, useEffect, type WheelEvent } from 'react';
import { sendCmd, type FeedbackFlash } from './PlayerOverlayPrimitives';

export function usePlayerOverlayInput({
  resetActivity,
  startSeekOverlay,
  flashFeedback,
}: {
  resetActivity: () => void;
  startSeekOverlay: () => void;
  flashFeedback: (icon: FeedbackFlash['icon'], label: string) => void;
}) {
  useEffect(() => {
    window.addEventListener('mousemove', resetActivity);
    window.addEventListener('pointerdown', resetActivity);
    return () => {
      window.removeEventListener('mousemove', resetActivity);
      window.removeEventListener('pointerdown', resetActivity);
    };
  }, [resetActivity]);

  return useCallback(
    (event: WheelEvent) => {
      if (Math.abs(event.deltaY) < 2) return;
      resetActivity();
      if (event.shiftKey) {
        startSeekOverlay();
        const seconds = event.deltaY < 0 ? 5 : -5;
        flashFeedback(seconds > 0 ? 'seekFwd' : 'seekBack', `${seconds > 0 ? '+' : ''}${seconds}s ${seconds > 0 ? 'Forwarded' : 'Rewound'}`);
        sendCmd(`seek ${seconds} relative`);
        return;
      }
      const delta = event.deltaY < 0 ? 5 : -5;
      flashFeedback('volume', `Volume ${delta > 0 ? '+' : ''}${delta}`);
      sendCmd(`add volume ${delta}`);
    },
    [flashFeedback, resetActivity, startSeekOverlay],
  );
}
