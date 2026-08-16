import { storageRead, storageWrite } from './engine';
import { isBrowserTarget } from '../platform/browser';

type WindowGeometry = { width: number; height: number; x: number; y: number };

const MIN_SAVED_WINDOW_WIDTH = 640;
const MIN_SAVED_WINDOW_HEIGHT = 480;
const MIN_VISIBLE_WINDOW_EDGE = 128;

let suppressed = false;

export function setSuppressWindowGeometrySave(value: boolean): void {
  suppressed = value;
}

export async function restoreWindowGeometry(): Promise<void> {
  if (isBrowserTarget()) return;
  const geometry = await storageRead<WindowGeometry>('windowGeometry');
  if (!geometry) return;
  const { availableMonitors, getCurrentWindow } = await import('@tauri-apps/api/window');
  const { PhysicalPosition, PhysicalSize } = await import('@tauri-apps/api/dpi');
  const win = getCurrentWindow();
  try {
    if (await win.isFullscreen()) return;
    if (!Number.isSafeInteger(geometry.width) || !Number.isSafeInteger(geometry.height)
      || !Number.isSafeInteger(geometry.x) || !Number.isSafeInteger(geometry.y)
      || geometry.width < MIN_SAVED_WINDOW_WIDTH || geometry.height < MIN_SAVED_WINDOW_HEIGHT) return;
    const monitors = await availableMonitors();
    const monitor = monitors.find(({ workArea }) => {
      const left = workArea.position.x;
      const top = workArea.position.y;
      const right = left + workArea.size.width;
      const bottom = top + workArea.size.height;
      return geometry.x < right - MIN_VISIBLE_WINDOW_EDGE
        && geometry.x + geometry.width > left + MIN_VISIBLE_WINDOW_EDGE
        && geometry.y < bottom - MIN_VISIBLE_WINDOW_EDGE
        && geometry.y + geometry.height > top + MIN_VISIBLE_WINDOW_EDGE;
    });
    if (!monitor) return;
    const { position, size } = monitor.workArea;
    const width = Math.min(geometry.width, size.width);
    const height = Math.min(geometry.height, size.height);
    const x = Math.min(Math.max(geometry.x, position.x), position.x + size.width - width);
    const y = Math.min(Math.max(geometry.y, position.y), position.y + size.height - height);
    await win.setSize(new PhysicalSize(width, height));
    await win.setPosition(new PhysicalPosition(x, y));
  } catch { /* ignore */ }
}

export async function toggleWindowFullscreen(): Promise<void> {
  if (isBrowserTarget()) {
    if (document.fullscreenElement) await document.exitFullscreen();
    else await document.documentElement.requestFullscreen?.();
    return;
  }
  const { getCurrentWindow } = await import('@tauri-apps/api/window');
  const win = getCurrentWindow();
  try {
    const isFullscreen = await win.isFullscreen();
    await win.setFullscreen(!isFullscreen);
  } catch { /* ignore */ }
}

export function watchWindowGeometry(): () => void {
  if (isBrowserTarget()) return () => {};
  let timer: ReturnType<typeof setTimeout> | null = null;
  let unlistenResize: (() => void) | null = null;
  let unlistenMove: (() => void) | null = null;
  let disposed = false;

  void (async () => {
    const { getCurrentWindow } = await import('@tauri-apps/api/window');
    const win = getCurrentWindow();

    const save = () => {
      if (suppressed) return;
      if (timer) clearTimeout(timer);
      timer = setTimeout(async () => {
        try {
          if (await win.isFullscreen()) return;
          const size = await win.outerSize();
          if (size.width < MIN_SAVED_WINDOW_WIDTH || size.height < MIN_SAVED_WINDOW_HEIGHT) return;
          const pos = await win.outerPosition();
          await storageWrite('windowGeometry', { width: size.width, height: size.height, x: pos.x, y: pos.y });
        } catch { /* ignore */ }
      }, 500);
    };

    const stopResize = await win.onResized(save);
    const stopMove = await win.onMoved(save);
    if (disposed) {
      stopResize();
      stopMove();
      return;
    }
    unlistenResize = stopResize;
    unlistenMove = stopMove;
  })();

  return () => {
    disposed = true;
    if (timer) clearTimeout(timer);
    unlistenResize?.();
    unlistenMove?.();
  };
}
