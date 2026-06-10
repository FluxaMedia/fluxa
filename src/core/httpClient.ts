import { fetch as tauriFetch } from '@tauri-apps/plugin-http';
import { getVersion } from '@tauri-apps/api/app';
import { httpFetchText } from './engine';

export let _appVersion = '1';
getVersion().then((v) => { _appVersion = v; }).catch(() => {});

export async function platformFetch(url: string, init?: RequestInit): Promise<Response> {
  // Use httpFetchText only for headerless GET requests (addon/catalog fetches).
  // Authenticated API calls always go through tauriFetch so headers are sent.
  const hasHeaders = init?.headers && Object.keys(init.headers).length > 0;
  if (!hasHeaders && !init?.body && (!init?.method || init.method.toUpperCase() === 'GET')) {
    const response = await httpFetchText(url);
    return new Response(response.body, { status: response.statusCode });
  }
  return tauriFetch(url, init);
}

export async function fetchJson(url: string, init?: RequestInit): Promise<unknown> {
  const res = await platformFetch(url, {
    headers: { 'User-Agent': `Fluxa/${_appVersion}`, ...init?.headers },
    ...init,
  });
  if (!res.ok) throw new Error(`Request failed (${res.status})`);
  return res.json();
}

export async function tryFetchJson(url: string): Promise<unknown | null> {
  try {
    return await fetchJson(url);
  } catch {
    return null;
  }
}
