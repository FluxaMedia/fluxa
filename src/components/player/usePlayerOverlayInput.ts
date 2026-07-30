import { useCallback, useEffect, type WheelEvent } from 'react';
import { sendCmd, type FeedbackFlash } from './PlayerOverlayPrimitives';

export function usePlayerOverlayInput({ resetActivity, startSeekOverlay, flashFeedback }: { resetActivity: () => void; startSeekOverlay: () => void; flashFeedback: (icon: FeedbackFlash['icon'], label: string) => void }) {
  useEffect(() => {
    window.addEventListener('mousemove', resetActivity);
    return () => window.removeEventListener('mousemove', resetActivity);
  }, [resetActivity]);

  return useCallback((event: WheelEvent) => {
    if (Math.abs(event.deltaY) < 2) return;
    resetActivity();
    if (event.shiftKey) {
      startSeekOverlay();
      const seconds = event.deltaY < 0 ? 5 : -5;
      flashFeedback(seconds > 0 ? 'seekFwd' : 'seekBack', `${seconds > 0 ? '+' : ''}${seconds}s`);
      sendCmd(`seek ${seconds} relative`);
      return;
    }
    flashFeedback('volume', '');
    sendCmd(`add volume ${event.deltaY < 0 ? 5 : -5}`);
  }, [flashFeedback, resetActivity, startSeekOverlay]);
}
