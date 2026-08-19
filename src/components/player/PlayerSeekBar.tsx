import { useState, type MutableRefObject, type RefObject } from 'react';
import { SeekPreview } from './SeekPreview';
import type { Chapter } from './PlayerOverlayPrimitives';

interface PlayerSeekBarProps {
  barRef: RefObject<HTMLDivElement | null>;
  fillRef: RefObject<HTMLDivElement | null>;
  bufferRef: RefObject<HTMLDivElement | null>;
  dotRef: RefObject<HTMLDivElement | null>;
  segmentFillRefs: MutableRefObject<(HTMLDivElement | null)[]>;
  segmentBufferRefs: MutableRefObject<(HTMLDivElement | null)[]>;
  durationRef: MutableRefObject<number>;
  chaptersRef: MutableRefObject<Chapter[]>;
  chapterSegments: Array<{ start: number; end: number }> | null;
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
  const trackHeight = hovered ? '0.3125rem' : '0.1875rem';

  return (
    <div
      ref={barRef}
      className="fluxa-seekbar"
      style={{
        position: 'relative',
        width: '100%',
        height: '2.25rem',
        cursor: 'pointer',
        overflow: 'visible',
        display: 'flex',
        alignItems: 'center',
        touchAction: 'none',
      }}
      onPointerDown={onSeekStart}
      onMouseEnter={() => setHovered(true)}
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
              background: 'color-mix(in srgb, var(--primary-accent-color) 20%, transparent)',
              borderRadius: '0.1875rem',
              pointerEvents: 'none',
            }}
          />
        ))}
      {chapterSegments ? (
        chapterSegments.map((segment, index) => (
          <div
            key={index}
            className="fluxa-seek-track"
            style={{
              position: 'absolute',
              left: `calc(${segment.start * 100}% + 0.125rem)`,
              width: `calc(${(segment.end - segment.start) * 100}% - 0.25rem)`,
              top: '50%',
              height: trackHeight,
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
