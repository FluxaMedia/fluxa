import { useCallback, useEffect, type MutableRefObject, type RefObject } from 'react';
import { castSeek } from '../../core/cast';
import { sendCmd } from './PlayerOverlayPrimitives';

export function usePlayerSeekInteractions({ durRef, lastSeekAtRef, activeCastDeviceIdRef, seekbarRef, seekFillRef, seekBufferRef, seekDotRef, chapterSegmentsRef, segmentFillRefs, segmentBufferRefs, isDraggingRef, dragPosRef, startSeekOverlay, resetActivity }: { durRef: MutableRefObject<number>; lastSeekAtRef: MutableRefObject<number>; activeCastDeviceIdRef: RefObject<string | null>; seekbarRef: RefObject<HTMLDivElement | null>; seekFillRef: RefObject<HTMLDivElement | null>; seekBufferRef: RefObject<HTMLDivElement | null>; seekDotRef: RefObject<HTMLDivElement | null>; chapterSegmentsRef: MutableRefObject<Array<{ start: number; end: number }> | null>; segmentFillRefs: MutableRefObject<(HTMLDivElement | null)[]>; segmentBufferRefs: MutableRefObject<(HTMLDivElement | null)[]>; isDraggingRef: MutableRefObject<boolean>; dragPosRef: MutableRefObject<number>; startSeekOverlay: () => void; resetActivity: () => void }) {
  const applyFills = useCallback((fraction: number, bufferFraction?: number) => {
    const segments = chapterSegmentsRef.current;
    if (segments?.length) {
      for (let index = 0; index < segments.length; index++) {
        const segment = segments[index];
        const segmentLength = segment.end - segment.start;
        const fill = segmentFillRefs.current[index];
        const buffer = segmentBufferRefs.current[index];
        if (fill) fill.style.width = fraction >= segment.end ? '100%' : fraction <= segment.start ? '0%' : `${((fraction - segment.start) / segmentLength * 100).toFixed(3)}%`;
        if (buffer && bufferFraction !== undefined) buffer.style.width = bufferFraction >= segment.end ? '100%' : bufferFraction <= segment.start ? '0%' : `${((bufferFraction - segment.start) / segmentLength * 100).toFixed(3)}%`;
      }
    } else {
      if (seekFillRef.current) seekFillRef.current.style.width = `${(fraction * 100).toFixed(3)}%`;
      if (seekBufferRef.current && bufferFraction !== undefined) seekBufferRef.current.style.width = `${(bufferFraction * 100).toFixed(3)}%`;
    }
    if (seekDotRef.current) seekDotRef.current.style.left = `${(fraction * 100).toFixed(3)}%`;
  }, []);

  const seekToFraction = useCallback((fraction: number) => {
    const time = fraction * durRef.current;
    lastSeekAtRef.current = Date.now();
    startSeekOverlay();
    if (activeCastDeviceIdRef.current) castSeek(time);
    else sendCmd(`set time-pos ${Math.floor(time)}`);
  }, [startSeekOverlay]);

  const onSeekMouseDown = useCallback((event: React.MouseEvent) => {
    if (event.button !== 0) return;
    event.preventDefault();
    resetActivity();
    isDraggingRef.current = true;
    const bar = seekbarRef.current;
    if (!bar) return;
    const rect = bar.getBoundingClientRect();
    const fraction = Math.max(0, Math.min(1, (event.clientX - rect.left) / rect.width));
    dragPosRef.current = fraction;
    applyFills(fraction);
  }, [applyFills, resetActivity]);

  useEffect(() => {
    const onMouseUp = () => {
      if (!isDraggingRef.current) return;
      isDraggingRef.current = false;
      seekToFraction(dragPosRef.current);
    };
    const onMouseMove = (event: MouseEvent) => {
      if (!isDraggingRef.current) return;
      const bar = seekbarRef.current;
      if (!bar) return;
      const rect = bar.getBoundingClientRect();
      const fraction = Math.max(0, Math.min(1, (event.clientX - rect.left) / rect.width));
      dragPosRef.current = fraction;
      applyFills(fraction);
    };
    window.addEventListener('mouseup', onMouseUp);
    window.addEventListener('mousemove', onMouseMove);
    return () => {
      window.removeEventListener('mouseup', onMouseUp);
      window.removeEventListener('mousemove', onMouseMove);
    };
  }, [applyFills, seekToFraction]);

  return { applyFills, onSeekMouseDown };
}
