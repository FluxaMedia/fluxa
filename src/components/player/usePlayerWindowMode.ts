import { useCallback, useEffect, useRef, useState } from 'react';
import { currentMonitor, getCurrentWindow } from '@tauri-apps/api/window';
import { PhysicalPosition, PhysicalSize } from '@tauri-apps/api/dpi';
import { setSuppressWindowGeometrySave } from '../../core/windowGeometry';

export function usePlayerWindowMode(resetActivity: () => void) {
  const [miniPlayerActive, setMiniPlayerActive] = useState(false);
  const miniPlayerActiveRef = useRef(false);
  const preMiniPlayerSizeRef = useRef<PhysicalSize | null>(null);
  const preMiniPlayerPosRef = useRef<PhysicalPosition | null>(null);
  const isFullscreenRef = useRef(false);

  useEffect(() => {
    const win = getCurrentWindow();
    void win.isFullscreen().then((fullscreen) => { isFullscreenRef.current = fullscreen; }).catch(() => undefined);
    let unlisten: (() => void) | undefined;
    void win.listen('tauri://resize', () => {
      void win.isFullscreen().then((fullscreen) => { isFullscreenRef.current = fullscreen; }).catch(() => undefined);
      resetActivity();
    }).then((stop) => { unlisten = stop; }).catch(() => undefined);
    return () => unlisten?.();
  }, [resetActivity]);

  const setPlayerFullscreen = useCallback(async (next: boolean) => {
    isFullscreenRef.current = next;
    await getCurrentWindow().setFullscreen(next);
  }, []);

  const toggleFullscreen = useCallback(async () => {
    await setPlayerFullscreen(!isFullscreenRef.current);
  }, [setPlayerFullscreen]);

  const toggleMiniPlayer = useCallback(async () => {
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
          await win.setPosition(new PhysicalPosition(
            monitor.position.x + monitor.size.width - width - margin,
            monitor.position.y + monitor.size.height - height - margin,
          ));
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

  useEffect(() => () => {
    if (!miniPlayerActiveRef.current) return;
    const win = getCurrentWindow();
    setSuppressWindowGeometrySave(false);
    void win.setAlwaysOnTop(false).catch(() => undefined);
    if (preMiniPlayerSizeRef.current) void win.setSize(preMiniPlayerSizeRef.current).catch(() => undefined);
    if (preMiniPlayerPosRef.current) void win.setPosition(preMiniPlayerPosRef.current).catch(() => undefined);
  }, []);

  return { miniPlayerActive, isFullscreenRef, setPlayerFullscreen, toggleFullscreen, toggleMiniPlayer };
}
