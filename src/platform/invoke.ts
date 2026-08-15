export function platformInvoke<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  if (import.meta.env.VITE_FLUXA_TARGET === 'web' || import.meta.env.VITE_FLUXA_TARGET === 'webos') {
    return import('./web/invoke').then(({ webInvoke }) => webInvoke<T>(command, args));
  }
  return import('@tauri-apps/api/core').then(({ invoke }) => invoke<T>(command, args));
}
