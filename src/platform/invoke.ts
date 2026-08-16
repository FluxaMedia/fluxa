import { isBrowserTarget } from './browser';

export function platformInvoke<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  if (isBrowserTarget()) {
    return import('./web/invoke').then(({ webInvoke }) => webInvoke<T>(command, args));
  }
  return import('@tauri-apps/api/core').then(({ invoke }) => invoke<T>(command, args));
}

export function platformConvertFileSrc(path: string): Promise<string> {
  if (isBrowserTarget()) return Promise.resolve(path);
  return import('@tauri-apps/api/core').then(({ convertFileSrc }) => convertFileSrc(path));
}
