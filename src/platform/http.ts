function redactUrl(rawUrl: string): string {
  try {
    const url = new URL(rawUrl);
    for (const key of ['token', 'access_token', 'api_key', 'apikey', 'code']) {
      if (url.searchParams.has(key)) url.searchParams.set(key, '[redacted]');
    }
    return url.toString();
  } catch {
    return rawUrl;
  }
}

export function platformFetch(url: string, init?: RequestInit): Promise<Response> {
  if (import.meta.env.VITE_FLUXA_TARGET === 'web' || import.meta.env.VITE_FLUXA_TARGET === 'webos') {
    const startedAt = performance.now();
    console.debug('[fluxa:web:http:start]', init?.method ?? 'GET', redactUrl(url));
    return fetch(url, init).then((response) => {
      console.debug('[fluxa:web:http:end]', response.status, Math.round(performance.now() - startedAt), redactUrl(url));
      return response;
    }).catch((error) => {
      console.error('[fluxa:web:http:error]', Math.round(performance.now() - startedAt), redactUrl(url), error);
      throw error;
    });
  }
  return import('@tauri-apps/plugin-http').then(({ fetch: tauriFetch }) => tauriFetch(url, init));
}
