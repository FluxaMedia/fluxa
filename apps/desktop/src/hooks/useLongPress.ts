import { useCallback, useEffect, useRef } from 'react';

const HOLD_MS = 480;
const MOVE_TOLERANCE_PX = 12;

export function useLongPress(onTrigger: (point: { x: number; y: number }) => void) {
  const timer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const origin = useRef<{ x: number; y: number } | null>(null);
  const fired = useRef(false);

  const clear = useCallback(() => {
    if (timer.current !== null) clearTimeout(timer.current);
    timer.current = null;
    origin.current = null;
  }, []);

  useEffect(() => clear, [clear]);

  return {
    onPointerDown: (event: React.PointerEvent) => {
      if (event.pointerType === 'mouse') return;
      fired.current = false;
      origin.current = { x: event.clientX, y: event.clientY };
      timer.current = setTimeout(() => {
        fired.current = true;
        onTrigger({ x: event.clientX, y: event.clientY });
      }, HOLD_MS);
    },
    onPointerMove: (event: React.PointerEvent) => {
      const start = origin.current;
      if (!start) return;
      if (Math.abs(event.clientX - start.x) > MOVE_TOLERANCE_PX || Math.abs(event.clientY - start.y) > MOVE_TOLERANCE_PX) {
        clear();
      }
    },
    onPointerUp: clear,
    onPointerCancel: clear,
    onPointerLeave: clear,
    onClick: (event: React.MouseEvent) => {
      if (!fired.current) return;
      fired.current = false;
      event.preventDefault();
      event.stopPropagation();
    },
  };
}
