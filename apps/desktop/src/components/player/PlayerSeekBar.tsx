import { useCallback, useEffect, useState, type MutableRefObject, type RefObject } from 'react';
import { SeekPreview } from './SeekPreview';
import type { Chapter } from './PlayerOverlayPrimitives';

const TRACK_HIT_TOLERANCE_PX = 10;

interface PlayerSeekBarProps {
  barRef: RefObject<HTMLDivElement | null>;
  fillRef: RefObject<HTMLDivElement | null>;
  bufferRef: RefObject<HTMLDivElement | null>;
  dotRef: RefObject<HTMLDivElement | null>;
  segmentFillRefs: MutableRefObject<(HTMLDivElement | null)[]>;
  segmentBufferRefs: MutableRefObject<(HTMLDivElement | null)[]>;
  durationRef: MutableRefObject<number>;
  chaptersRef: MutableRefObject<Chapter[]>;
  chapterSegments: Array<{ start: number; end: number; title?: string }> | null;
  skipMarkers: Array<{ start: number; end: number }>;
  onSeekStart: (event: React.PointerEvent<HTMLDivElement>) => void;
}

export function PlayerSeekBar({
  barRef,
  fillRef,
  bufferRef,
  dotRef,
  segmentFillRefs,
  segmentBufferRefs,
  durationRef,
  chaptersRef,
  chapterSegments,
  skipMarkers,
  onSeekStart,
}: PlayerSeekBarProps) {
  const [hovered, setHovered] = useState(false);
  const [dragging, setDragging] = useState(false);
  const [dragFraction, setDragFraction] = useState(0);
  const trackHeight = dragging ? '0.375rem' : hovered ? '0.3125rem' : '0.25rem';
  const activeChapterIndex =
    dragging && chapterSegments?.length
      ? Math.min(
          chapterSegments.length - 1,
          Math.max(
            0,
            chapterSegments.findIndex((segment) => dragFraction >= segment.start && dragFraction < segment.end),
          ),
        )
      : -1;

  const updateDragFraction = useCallback((clientX: number) => {
    const bar = barRef.current;
    if (!bar) return;
    const rect = bar.getBoundingClientRect();
    setDragFraction(Math.max(0, Math.min(1, (clientX - rect.left) / rect.width)));
  }, [barRef]);

  const isTrackHit = useCallback(
    (clientY: number) => {
      const bar = barRef.current;
      if (!bar) return false;
      const rect = bar.getBoundingClientRect();
      return Math.abs(clientY - (rect.top + rect.height / 2)) <= TRACK_HIT_TOLERANCE_PX;
    },
    [barRef],
  );

  useEffect(() => {
    if (!dragging) return;
    const onPointerMove = (event: PointerEvent) => updateDragFraction(event.clientX);
    const onPointerUp = () => setDragging(false);
    window.addEventListener('pointermove', onPointerMove);
    window.addEventListener('pointerup', onPointerUp);
    return () => {
      window.removeEventListener('pointermove', onPointerMove);
      window.removeEventListener('pointerup', onPointerUp);
    };
  }, [dragging, updateDragFraction]);

  return (
    <div
      ref={barRef}
      className="fluxa-seekbar"
      style={{
        position: 'relative',
        width: 'calc(100% - 2rem)',
        margin: '0 1rem',
        height: '3.25rem',
        cursor: 'pointer',
        overflow: 'visible',
        display: 'flex',
        alignItems: 'center',
        touchAction: 'none',
      }}
      onPointerDown={(event) => {
        if (!isTrackHit(event.clientY)) return;
        setDragging(true);
        updateDragFraction(event.clientX);
        onSeekStart(event);
      }}
      onMouseMove={(event) => setHovered(isTrackHit(event.clientY))}
      onMouseLeave={() => setHovered(false)}
    >
      <div
        className="fluxa-seek-track"
        style={{
          position: 'absolute',
          left: 0,
          right: 0,
          top: '50%',
          height: trackHeight,
          transform: 'translateY(-50%)',
          background: 'rgba(255,255,255,0.22)',
          borderRadius: '0.1875rem',
        }}
      />
      {!chapterSegments &&
        skipMarkers.map((segment, index) => (
          <div
            key={`${segment.start}-${segment.end}-${index}`}
            className="fluxa-seek-track"
            style={{
              position: 'absolute',
              left: `${segment.start * 100}%`,
              width: `${(segment.end - segment.start) * 100}%`,
              top: '50%',
              height: trackHeight,
              transform: 'translateY(-50%)',
              background: 'var(--fluxa-accent-soft)',
              borderRadius: '0.1875rem',
              pointerEvents: 'none',
            }}
          />
        ))}
      {chapterSegments ? (
        chapterSegments.map((segment, index) => (
          <div
            key={index}
            style={{
              position: 'absolute',
              left: `calc(${segment.start * 100}% + 0.125rem)`,
              width: `calc(${(segment.end - segment.start) * 100}% - 0.25rem)`,
              top: '50%',
              height: '0px',
              transform: 'translateY(-50%)',
              overflow: 'visible',
            }}
          >
            <div
              className="fluxa-seek-track"
              style={{
                position: 'absolute',
                left: 0,
                right: 0,
                top: 0,
                height: activeChapterIndex === index ? '0.5rem' : trackHeight,
                transform: 'translateY(-50%)',
                overflow: 'hidden',
                background: 'rgba(255,255,255,0.18)',
                borderRadius: '0.125rem',
              }}
            >
              <div
                ref={(element) => {
                  segmentBufferRefs.current[index] = element;
                }}
                style={{ position: 'absolute', left: 0, top: 0, width: '0%', height: '100%', background: 'rgba(255,255,255,0.3)' }}
              />
              <div
                ref={(element) => {
                  segmentFillRefs.current[index] = element;
                }}
                style={{ position: 'absolute', left: 0, top: 0, width: '0%', height: '100%', background: 'var(--primary-accent-color)' }}
              />
            </div>
            <span
              style={{
                position: 'absolute',
                left: '50%',
                top: '0.7rem',
                transform: 'translateX(-50%)',
                maxWidth: '100%',
                overflow: 'hidden',
                textOverflow: 'ellipsis',
                whiteSpace: 'nowrap',
                color: activeChapterIndex === index ? '#fff' : 'rgba(255,255,255,0.62)',
                fontSize: '0.6875rem',
                fontWeight: activeChapterIndex === index ? 700 : 500,
                lineHeight: 1,
                textAlign: 'center',
                pointerEvents: 'none',
              }}
            >
              {segment.title || `Chapter ${index + 1}`}
            </span>
          </div>
        ))
      ) : (
        <>
          <div
            ref={bufferRef}
            className="fluxa-seek-track"
            style={{
              position: 'absolute',
              left: 0,
              top: '50%',
              height: trackHeight,
              transform: 'translateY(-50%)',
              width: '0%',
              background: 'rgba(255,255,255,0.3)',
              borderRadius: '0.1875rem',
            }}
          />
          <div
            ref={fillRef}
            className="fluxa-seek-track"
            style={{
              position: 'absolute',
              left: 0,
              top: '50%',
              height: trackHeight,
              transform: 'translateY(-50%)',
              width: '0%',
              background: 'var(--primary-accent-color)',
              borderRadius: '0.1875rem',
            }}
          />
        </>
      )}
      <div
        ref={dotRef}
        className="fluxa-seek-dot"
        style={{
          position: 'absolute',
          left: '0%',
          top: '50%',
          width: hovered ? '0.875rem' : '0.6875rem',
          height: hovered ? '0.875rem' : '0.6875rem',
          borderRadius: '50%',
          background: 'var(--primary-accent-color)',
          transform: 'translate(-50%, -50%)',
          boxShadow: '0 1px 0.375rem rgba(0,0,0,0.7)',
          pointerEvents: 'none',
        }}
      />
      <SeekPreview barRef={barRef} durRef={durationRef} chaptersRef={chaptersRef} />
    </div>
  );
}
