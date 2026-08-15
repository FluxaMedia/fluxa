export function platformFetch(url: string, init?: RequestInit): Promise<Response> {
  if (import.meta.env.VITE_FLUXA_TARGET === 'web' || import.meta.env.VITE_FLUXA_TARGET === 'webos') {
    return fetch(url, init);
  }
  return import('@tauri-apps/plugin-http').then(({ fetch: tauriFetch }) => tauriFetch(url, init));
}
