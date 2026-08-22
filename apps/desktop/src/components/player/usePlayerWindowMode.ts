import { useCallback, useEffect, useRef, useState } from 'react';
import type { PhysicalPosition, PhysicalSize } from '@tauri-apps/api/dpi';
import { setSuppressWindowGeometrySave } from '../../core/windowGeometry';
import { isBrowserTarget } from '../../platform/browser';

export function usePlayerWindowMode(resetActivity: () => void) {
  const [miniPlayerActive, setMiniPlayerActive] = useState(false);
  const miniPlayerActiveRef = useRef(false);
  const preMiniPlayerSizeRef = useRef<PhysicalSize | null>(null);
  const preMiniPlayerPosRef = useRef<PhysicalPosition | null>(null);
  const isFullscreenRef = useRef(false);

  useEffect(() => {
    if (isBrowserTarget()) return undefined;
    let unlisten: (() => void) | undefined;
    let disposed = false;
    void (async () => {
      const { getCurrentWindow } = await import('@tauri-apps/api/window');
      const win = getCurrentWindow();
      void win
        .isFullscreen()
        .then((fullscreen) => {
          isFullscreenRef.current = fullscreen;
        })
        .catch(() => undefined);
      const stop = await win
        .listen('tauri://resize', () => {
          void win
            .isFullscreen()
            .then((fullscreen) => {
              isFullscreenRef.current = fullscreen;
            })
            .catch(() => undefined);
          resetActivity();
        })
        .catch(() => undefined);
      if (!stop) return;
      if (disposed) stop();
      else unlisten = stop;
    })();
    return () => {
      disposed = true;
      unlisten?.();
    };
  }, [resetActivity]);

  const setPlayerFullscreen = useCallback(async (next: boolean) => {
    isFullscreenRef.current = next;
    if (isBrowserTarget()) {
      if (next) await document.documentElement.requestFullscreen?.();
      else if (document.fullscreenElement) await document.exitFullscreen();
      return;
    }
    const { getCurrentWindow } = await import('@tauri-apps/api/window');
    await getCurrentWindow().setFullscreen(next);
  }, []);

  const toggleFullscreen = useCallback(async () => {
    await setPlayerFullscreen(!isFullscreenRef.current);
  }, [setPlayerFullscreen]);

  const toggleMiniPlayer = useCallback(async () => {
    if (isBrowserTarget()) return;
    const { currentMonitor, getCurrentWindow } = await import('@tauri-apps/api/window');
    const { PhysicalPosition, PhysicalSize } = await import('@tauri-apps/api/dpi');
    const win = getCurrentWindow();
    if (!miniPlayerActive) {
      if (isFullscreenRef.current) await setPlayerFullscreen(false);
      try {
        preMiniPlayerSizeRef.current = await win.outerSize();
        preMiniPlayerPosRef.current = await win.outerPosition();
      } catch {}
      setSuppressWindowGeometrySave(true);
      const width = 420;
      const height = 236;
      try {
        const monitor = await currentMonitor();
        if (monitor) {
          const margin = 24;
          await win.setPosition(
            new PhysicalPosition(
              monitor.position.x + monitor.size.width - width - margin,
              monitor.position.y + monitor.size.height - height - margin,
            ),
          );
        }
      } catch {}
      await win.setSize(new PhysicalSize(width, height));
      await win.setAlwaysOnTop(true);
      setMiniPlayerActive(true);
      return;
    }
    await win.setAlwaysOnTop(false);
    try {
      if (preMiniPlayerSizeRef.current) await win.setSize(preMiniPlayerSizeRef.current);
      if (preMiniPlayerPosRef.current) await win.setPosition(preMiniPlayerPosRef.current);
    } catch {}
    setSuppressWindowGeometrySave(false);
    setMiniPlayerActive(false);
  }, [miniPlayerActive, setPlayerFullscreen]);

  useEffect(() => {
    miniPlayerActiveRef.current = miniPlayerActive;
  }, [miniPlayerActive]);

  useEffect(
    () => () => {
      if (isBrowserTarget()) return;
      if (!miniPlayerActiveRef.current) return;
      setSuppressWindowGeometrySave(false);
      void (async () => {
        const { getCurrentWindow } = await import('@tauri-apps/api/window');
        const win = getCurrentWindow();
        void win.setAlwaysOnTop(false).catch(() => undefined);
        if (preMiniPlayerSizeRef.current) void win.setSize(preMiniPlayerSizeRef.current).catch(() => undefined);
        if (preMiniPlayerPosRef.current) void win.setPosition(preMiniPlayerPosRef.current).catch(() => undefined);
      })();
    },
    [],
  );

  return { miniPlayerActive, isFullscreenRef, setPlayerFullscreen, toggleFullscreen, toggleMiniPlayer };
}
