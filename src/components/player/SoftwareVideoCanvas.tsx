import { useEffect, useRef, type RefObject } from 'react';
import { embeddedMpvRenderFrame, type EmbeddedMpvStatus } from '../../core/mpvPlayer';

export function SoftwareVideoCanvas({
  statusRef,
  onFirstFrame,
}: {
  statusRef: RefObject<EmbeddedMpvStatus | null>;
  onFirstFrame?: () => void;
}) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const firstFrameFiredRef = useRef(false);

  useEffect(() => {
    let cancelled = false;
    let timer: ReturnType<typeof setTimeout> | undefined;
    const draw = async () => {
      if (cancelled) return;
      const canvas = canvasRef.current;
      if (!canvas) {
        timer = setTimeout(draw, 100);
        return;
      }
      const viewportW = Math.max(320, window.innerWidth);
      const viewportH = Math.max(180, window.innerHeight);
      const scale = Math.min(Math.min(window.devicePixelRatio || 1, 1.5), 960 / viewportW, 540 / viewportH);
      try {
        const frame = await embeddedMpvRenderFrame(
          Math.max(320, Math.floor(viewportW * scale)),
          Math.max(180, Math.floor(viewportH * scale)),
        );
        if (cancelled) return;
        if (canvas.width !== frame.width || canvas.height !== frame.height) {
          canvas.width = frame.width;
          canvas.height = frame.height;
        }
        const context = canvas.getContext('2d');
        if (context) {
          const binary = atob(frame.pixelsBase64);
          const pixels = new Uint8ClampedArray(binary.length);
          for (let index = 0; index < binary.length; index++) pixels[index] = binary.charCodeAt(index);
          context.putImageData(new ImageData(pixels, frame.width, frame.height), 0, 0);
          const status = statusRef.current;
          const hasRenderedVideo =
            status?.loaded &&
            status.hasVideoTrack &&
            status.voConfigured === 'yes' &&
            status.framesRendered >= 2 &&
            status.pausedForCache !== 'yes' &&
            parseFloat(status.timePos ?? '0') > 0.15;
          if (!firstFrameFiredRef.current && onFirstFrame && hasRenderedVideo) {
            firstFrameFiredRef.current = true;
            onFirstFrame();
          }
        }
        timer = setTimeout(draw, 42);
      } catch {
        timer = setTimeout(draw, 120);
      }
    };
    void draw();
    return () => {
      cancelled = true;
      if (timer) clearTimeout(timer);
    };
  }, [onFirstFrame, statusRef]);

  return (
    <canvas
      ref={canvasRef}
      style={{
        position: 'absolute',
        inset: 0,
        width: '100%',
        height: '100%',
        objectFit: 'contain',
        background: '#000',
        zIndex: 0,
        pointerEvents: 'none',
      }}
    />
  );
}
