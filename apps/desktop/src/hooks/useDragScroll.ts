import { useRef } from 'react';
import type React from 'react';

const DRAG_THRESHOLD = 4;

/** Enables mouse drag panning for a native horizontal scroll container. */
export function useDragScroll(
  scrollRef: React.RefObject<HTMLDivElement | null>,
): Pick<
  React.HTMLAttributes<HTMLDivElement>,
  'onPointerDown' | 'onPointerMove' | 'onPointerUp' | 'onPointerCancel' | 'onClickCapture' | 'onDragStart'
> {
  const dragRef = useRef<{ pointerId: number; startX: number; startScrollLeft: number; moved: boolean; captured: boolean } | null>(null);
  const suppressClickRef = useRef(false);

  const finishDrag = (target: HTMLDivElement, pointerId: number) => {
    const drag = dragRef.current;
    if (!drag || drag.pointerId !== pointerId) return;
    if (drag.captured && target.hasPointerCapture(pointerId)) target.releasePointerCapture(pointerId);
    suppressClickRef.current = drag.moved;
    dragRef.current = null;
  };

  return {
    onPointerDown: (event) => {
      if (event.pointerType !== 'mouse' || event.button !== 0) return;
      const el = scrollRef.current;
      if (!el || el.scrollWidth <= el.clientWidth) return;
      dragRef.current = {
        pointerId: event.pointerId,
        startX: event.clientX,
        startScrollLeft: el.scrollLeft,
        moved: false,
        captured: false,
      };
    },
    onPointerMove: (event) => {
      const drag = dragRef.current;
      const el = scrollRef.current;
      if (!drag || !el || drag.pointerId !== event.pointerId) return;
      const distance = event.clientX - drag.startX;
      if (!drag.moved && Math.abs(distance) >= DRAG_THRESHOLD) {
        drag.moved = true;
        drag.captured = true;
        event.currentTarget.setPointerCapture(event.pointerId);
      }
      if (!drag.moved) return;
      el.scrollLeft = drag.startScrollLeft - distance;
      event.preventDefault();
    },
    onPointerUp: (event) => finishDrag(event.currentTarget, event.pointerId),
    onPointerCancel: (event) => finishDrag(event.currentTarget, event.pointerId),
    onDragStart: (event) => event.preventDefault(),
    onClickCapture: (event) => {
      if (!suppressClickRef.current) return;
      suppressClickRef.current = false;
      event.preventDefault();
      event.stopPropagation();
    },
  };
}
