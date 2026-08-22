import { useEffect, useRef } from 'react';
import { hapticTap } from '../platform/haptics';

const EDGE_PX = 28;
const TRIGGER_PX = 90;
const MAX_DRIFT_PX = 70;
const MAX_DURATION_MS = 700;

export function useEdgeSwipeBack(enabled: boolean, onBack: () => void): void {
  const backRef = useRef(onBack);
  backRef.current = onBack;

  useEffect(() => {
    if (!enabled) return undefined;
    let start: { x: number; y: number; at: number } | null = null;

    const onStart = (event: TouchEvent) => {
      const touch = event.touches[0];
      start = touch && touch.clientX <= EDGE_PX ? { x: touch.clientX, y: touch.clientY, at: Date.now() } : null;
    };

    const onEnd = (event: TouchEvent) => {
      const from = start;
      start = null;
      const touch = event.changedTouches[0];
      if (!from || !touch) return;
      if (Date.now() - from.at > MAX_DURATION_MS) return;
      if (Math.abs(touch.clientY - from.y) > MAX_DRIFT_PX) return;
      if (touch.clientX - from.x < TRIGGER_PX) return;
      hapticTap();
      backRef.current();
    };

    window.addEventListener('touchstart', onStart, { passive: true });
    window.addEventListener('touchend', onEnd, { passive: true });
    return () => {
      window.removeEventListener('touchstart', onStart);
      window.removeEventListener('touchend', onEnd);
    };
  }, [enabled]);
}
