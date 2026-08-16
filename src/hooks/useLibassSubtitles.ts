import { useEffect, useRef, useState } from 'react';
import { isBrowserTarget } from '../platform/browser';
import type { LibassRenderer } from '../platform/web/libass';
import { loadPrefs } from '../core/libraryOps';
import { subtitleStylePrefsFrom, toAssDocument, type SubtitleStylePrefs } from '../core/webSubtitles';
import type { PlayerSubtitleSource } from '../core/playerUtils';

export type LibassSubtitleStatus = 'idle' | 'loading' | 'ready' | 'unavailable';

export function useLibassSubtitles(
  videoRef: React.RefObject<HTMLVideoElement | null>,
  canvasRef: React.RefObject<HTMLCanvasElement | null>,
  subtitles: PlayerSubtitleSource[],
  selectedIndex: number,
) {
  const [stylePrefs, setStylePrefs] = useState<SubtitleStylePrefs | null>(null);
  const [status, setStatus] = useState<LibassSubtitleStatus>('idle');
  const instanceRef = useRef<LibassRenderer | null>(null);

  useEffect(() => {
    let cancelled = false;
    void loadPrefs()
      .then((prefs) => { if (!cancelled) setStylePrefs(subtitleStylePrefsFrom(prefs)); })
      .catch(() => { if (!cancelled) setStylePrefs(subtitleStylePrefsFrom(null)); });
    return () => { cancelled = true; };
  }, []);

  useEffect(() => {
    const video = videoRef.current;
    const canvas = canvasRef.current;
    const source = selectedIndex >= 0 ? subtitles[selectedIndex] : undefined;

    const teardown = () => {
      const instance = instanceRef.current;
      instanceRef.current = null;
      if (instance) void instance.destroy().catch(() => undefined);
    };

    if (!video || !canvas || !source || !stylePrefs || !isBrowserTarget()) {
      teardown();
      setStatus('idle');
      return undefined;
    }

    let cancelled = false;
    setStatus('loading');

    void (async () => {
      let document: string | null = null;
      try {
        const response = await fetch(source.url);
        if (!response.ok) throw new Error(`subtitle request failed: ${response.status}`);
        document = toAssDocument(await response.text(), stylePrefs);
      } catch (error) {
        console.error('[fluxa:web:subtitles]', source.url, error);
      }
      if (cancelled) return;
      if (!document) {
        teardown();
        setStatus('unavailable');
        return;
      }
      teardown();
      try {
        const { createLibassRenderer } = await import('../platform/web/libass');
        if (cancelled) return;
        instanceRef.current = createLibassRenderer(video, canvas, document);
        setStatus('ready');
      } catch (error) {
        console.error('[fluxa:web:subtitles:renderer]', error);
        setStatus('unavailable');
      }
    })();

    return () => {
      cancelled = true;
      teardown();
    };
  }, [videoRef, canvasRef, subtitles, selectedIndex, stylePrefs]);

  useEffect(() => () => {
    const instance = instanceRef.current;
    instanceRef.current = null;
    if (instance) void instance.destroy().catch(() => undefined);
  }, []);

  return { status };
}
