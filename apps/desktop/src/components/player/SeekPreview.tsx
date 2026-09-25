import { platformInvoke as invoke } from '../../platform/invoke';
import { useCallback, useEffect, useRef, useState, type MutableRefObject, type RefObject } from 'react';
import { fmtTime, type Chapter } from './PlayerOverlayPrimitives';

const TRACK_HIT_TOLERANCE_PX = 10;

export function SeekPreview({
  barRef,
  durRef,
  chaptersRef,
}: {
  barRef: RefObject<HTMLDivElement | null>;
  durRef: MutableRefObject<number>;
  chaptersRef: MutableRefObject<Chapter[]>;
}) {
  const [preview, setPreview] = useState<{ x: number; time: number; chapter: string | null } | null>(null);
  const [thumbImg, setThumbImg] = useState<string | null>(null);
  const thumbTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const thumbRequestTimeRef = useRef<number | null>(null);
  const thumbRequestInFlightRef = useRef(false);
  const thumbLastRequestedAtRef = useRef(0);
  const previewTimeRef = useRef<number | null>(null);

  const requestThumbnail = useCallback((requestedTime: number) => {
    if (thumbRequestInFlightRef.current || previewTimeRef.current !== requestedTime) return;

    const remaining = Math.max(0, 75 - (Date.now() - thumbLastRequestedAtRef.current));
    if (remaining > 0) {
      if (thumbTimerRef.current) clearTimeout(thumbTimerRef.current);
      thumbTimerRef.current = setTimeout(() => {
        thumbTimerRef.current = null;
        requestThumbnail(requestedTime);
      }, remaining);
      return;
    }

    thumbRequestInFlightRef.current = true;
    thumbLastRequestedAtRef.current = Date.now();
    const startedAt = performance.now();
    console.debug('[fluxa] seek thumbnail request start', { time: requestedTime });
    invoke<string>('player_get_seek_thumbnail', { timePos: requestedTime })
      .then((img) => {
        console.debug('[fluxa] seek thumbnail request response', {
          time: requestedTime,
          ms: Math.round(performance.now() - startedAt),
          bytes: img?.length ?? 0,
        });
        if (img && previewTimeRef.current !== null) setThumbImg(img);
      })
      .catch((error) => {
        console.error('[fluxa] seek thumbnail failed', {
          time: requestedTime,
          ms: Math.round(performance.now() - startedAt),
          error,
        });
      })
      .finally(() => {
        thumbRequestInFlightRef.current = false;
        const latestTime = previewTimeRef.current;
        if (latestTime != null && latestTime !== requestedTime) {
          if (thumbTimerRef.current) clearTimeout(thumbTimerRef.current);
          thumbTimerRef.current = setTimeout(() => {
            thumbTimerRef.current = null;
            requestThumbnail(latestTime);
          }, 100);
        }
      });
  }, []);

  useEffect(() => {
    const bar = barRef.current;
    if (!bar) return;
    const onMove = (e: MouseEvent) => {
      const rect = bar.getBoundingClientRect();
      if (Math.abs(e.clientY - (rect.top + rect.height / 2)) > TRACK_HIT_TOLERANCE_PX) {
        setPreview(null);
        return;
      }
      const frac = Math.max(0, Math.min(1, (e.clientX - rect.left) / rect.width));
      const previewTime = frac * durRef.current;
      const chaps = chaptersRef.current;
      let chapterName: string | null = null;
      if (chaps.length > 0) {
        let found = chaps[0].title;
        for (const ch of chaps) {
          if (ch.startMs / 1000 <= previewTime) found = ch.title;
          else break;
        }
        chapterName = found || null;
      }
      const previewHalfWidth = 80;
      setPreview({
        x: Math.max(previewHalfWidth, Math.min(rect.width - previewHalfWidth, e.clientX - rect.left)),
        time: previewTime,
        chapter: chapterName,
      });
    };
    const onLeave = () => setPreview(null);
    bar.addEventListener('mousemove', onMove);
    bar.addEventListener('mouseleave', onLeave);
    return () => {
      bar.removeEventListener('mousemove', onMove);
      bar.removeEventListener('mouseleave', onLeave);
    };
  }, [barRef, durRef, chaptersRef]);

  useEffect(() => {
    previewTimeRef.current = preview?.time ?? null;
    if (!preview) {
      setThumbImg(null);
      return;
    }
    const requestedTime = preview.time;
    thumbRequestTimeRef.current = requestedTime;
    if (thumbTimerRef.current) clearTimeout(thumbTimerRef.current);

    thumbTimerRef.current = setTimeout(() => {
      thumbTimerRef.current = null;
      requestThumbnail(requestedTime);
    }, 50);
    return () => {
      if (thumbTimerRef.current) clearTimeout(thumbTimerRef.current);
    };
  }, [preview?.time, requestThumbnail]);

  if (!preview) return null;

  return (
    <div
      style={{
        position: 'absolute',
        // Keep the preview above the track even though the seekbar now reserves
        // extra height for chapter labels below it.
        bottom: 'calc(50% + 1.25rem)',
        left: preview.x,
        transform: 'translateX(-50%)',
        pointerEvents: 'none',
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'center',
        gap: '0.25rem',
      }}
    >
      <div
        style={{
          width: '10rem',
          height: '5.625rem',
          borderRadius: '0.25rem',
          overflow: 'hidden',
          boxShadow: '0 0.125rem 0.75rem rgba(0,0,0,0.8)',
          border: '1px solid rgba(255,255,255,0.12)',
          background: 'rgba(255,255,255,0.05)',
          flexShrink: 0,
        }}
      >
        {thumbImg && <img src={thumbImg} alt="" style={{ width: '100%', height: '100%', objectFit: 'cover', display: 'block' }} />}
      </div>
      <div style={{ whiteSpace: 'nowrap', display: 'flex', flexDirection: 'column', alignItems: 'center', gap: '0.125rem' }}>
        {preview.chapter && (
          <span
            style={{
              fontSize: '0.8125rem',
              fontWeight: 600,
              color: 'rgba(255,255,255,0.85)',
              letterSpacing: '0.0125rem',
              textShadow: '0 1px 0.375rem rgba(0,0,0,1), 0 0 0.75rem rgba(0,0,0,0.9)',
            }}
          >
            {preview.chapter}
          </span>
        )}
        <span
          style={{
            fontSize: '0.875rem',
            fontWeight: 700,
            color: '#fff',
            letterSpacing: '0.025rem',
            textShadow: '0 1px 0.375rem rgba(0,0,0,1), 0 0 0.75rem rgba(0,0,0,0.9)',
          }}
        >
          {fmtTime(preview.time)}
        </span>
      </div>
    </div>
  );
}
