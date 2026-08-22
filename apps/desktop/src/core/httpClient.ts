import { platformFetch as nativeFetch } from '../platform/http';
import { platformInvoke } from '../platform/invoke';

export let _appVersion = '1';
platformInvoke<string>('get_version')
  .then((v) => {
    _appVersion = v;
  })
  .catch(() => {});

const DEFAULT_TIMEOUT_MS = 12_000;

const NO_CORS_HOSTS = new Set(['api.introdb.app', 'api.aniskip.com', 'api.anime-skip.com', 'api.skipdb.tv', 'api.theintrodb.org']);
const NATIVE_FETCH_HOSTS = new Set(['api.trakt.tv', 'media.trakt.tv', 'api.simkl.com', 'data.simkl.in', 'api.mdblist.com']);

export async function platformFetch(url: string, init?: RequestInit): Promise<Response> {
  const signal = init?.signal ?? AbortSignal.timeout(DEFAULT_TIMEOUT_MS);
  const hostname = new URL(url).hostname;
  if (NO_CORS_HOSTS.has(hostname)) {
    return nativeFetch(url, { ...init, signal });
  }
  const { ['User-Agent']: _omitted, ...nativeHeaders } = (init?.headers ?? {}) as Record<string, string>;
  if (NATIVE_FETCH_HOSTS.has(hostname)) {
    return nativeFetch(url, {
      ...init,
      headers: { ...nativeHeaders, 'User-Agent': `Fluxa Desktop/${_appVersion}` },
      signal,
    });
  }
  try {
    return await fetch(url, { ...init, headers: nativeHeaders, signal });
  } catch (error) {
    if (signal.aborted) throw error;
    return nativeFetch(url, { ...init, signal });
  }
}

export async function fetchJson(url: string, init?: RequestInit): Promise<unknown> {
  const res = await platformFetch(url, {
    headers: { 'User-Agent': `Fluxa/${_appVersion}`, ...init?.headers },
    ...init,
  });
  if (!res.ok) throw new Error(`Request failed (${res.status})`);
  return res.json();
}

export async function tryFetchJson(url: string, init?: RequestInit): Promise<unknown | null> {
  try {
    return await fetchJson(url, init);
  } catch (err) {
    console.error(`tryFetchJson failed for ${redactSecrets(url)}`, err);
    return null;
  }
}

export type RequestPlan = {
  url: string;
  method: 'GET' | 'POST';
  headers?: Record<string, string>;
  body?: string | null;
};

export async function executeRequestPlan(plan: RequestPlan): Promise<unknown> {
  return fetchJson(plan.url, { method: plan.method, headers: plan.headers, body: plan.body ?? undefined });
}

export async function tryExecuteRequestPlan(plan: RequestPlan): Promise<unknown | null> {
  try {
    return await executeRequestPlan(plan);
  } catch (err) {
    console.error(`tryExecuteRequestPlan failed for ${redactSecrets(plan.url)}`, err);
    return null;
  }
}

function redactSecrets(url: string): string {
  const parsed = new URL(url);
  for (const key of ['api_key', 'token', 'access_token']) {
    if (parsed.searchParams.has(key)) parsed.searchParams.set(key, '***');
  }
  return parsed.toString();
}
