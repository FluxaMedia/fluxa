export function isBrowserTarget(): boolean {
  return import.meta.env.VITE_FLUXA_TARGET === 'web' || import.meta.env.VITE_FLUXA_TARGET === 'webos';
}

export type UnlistenFn = () => void;

export async function platformOpenExternal(url: string): Promise<void> {
  if (isBrowserTarget()) {
    window.open(url, '_blank', 'noopener,noreferrer');
    return;
  }
  const { open } = await import('@tauri-apps/plugin-shell');
  await open(url);
}

export async function platformOpenDialog(_options?: Record<string, unknown>): Promise<string | string[] | null> {
  if (isBrowserTarget()) return null;
  const { open } = await import('@tauri-apps/plugin-dialog');
  return open(_options as never) as Promise<string | string[] | null>;
}

export async function platformSaveDialog(_options?: Record<string, unknown>): Promise<string | null> {
  if (isBrowserTarget()) return null;
  const { save } = await import('@tauri-apps/plugin-dialog');
  return save(_options as never) as Promise<string | null>;
}

export async function platformListen<T>(event: string, handler: (event: { payload: T }) => void): Promise<() => void> {
  if (isBrowserTarget()) return () => {};
  const { listen } = await import('@tauri-apps/api/event');
  return listen<T>(event, handler);
}

export async function platformEmit<T>(event: string, payload?: T): Promise<void> {
  if (isBrowserTarget()) return;
  const { emit } = await import('@tauri-apps/api/event');
  await emit(event, payload);
}

export async function platformSetCursorVisible(visible: boolean): Promise<void> {
  if (isBrowserTarget()) return;
  await (await import('@tauri-apps/api/window')).getCurrentWindow().setCursorVisible(visible);
}
