import { storageDelete, storageRead, storageWrite } from './storage';
import * as library from './libraryStorage';
import { coreInvoke, engineCompleteEffect, engineDispatch, engineInit, engineSnapshot } from './wasmEngine';

const companionUrl = import.meta.env.VITE_FLUXA_COMPANION_URL || 'http://127.0.0.1:19876';
const nuvioUrl = import.meta.env.VITE_NUVIO_SUPABASE_URL || '';
const nuvioKey = import.meta.env.VITE_NUVIO_SUPABASE_KEY || '';
const oauthClientIds: Record<string, string> = {
  trakt: import.meta.env.VITE_TRAKT_CLIENT_ID || '',
  simkl: import.meta.env.VITE_SIMKL_CLIENT_ID || '',
  anilist: import.meta.env.VITE_ANILIST_CLIENT_ID || '',
};

export class PlatformUnsupportedError extends Error {
  constructor(command: string) {
    super(`${command} is not available in this platform`);
    this.name = 'PlatformUnsupportedError';
  }
}

async function companion(path: string, init?: RequestInit): Promise<Response> {
  return fetch(`${companionUrl}${path}`, init);
}

async function oauthExchange(service: string, code: string): Promise<string> {
  const response = await companion(`/oauth/${service}/exchange`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ code }),
  });
  if (!response.ok) throw new Error(`${service} OAuth exchange failed (${response.status})`);
  return response.text();
}

async function oauthRefresh(service: string, refreshToken: string): Promise<string> {
  const response = await companion(`/oauth/${service}/refresh`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ refreshToken }),
  });
  if (!response.ok) throw new Error(`${service} OAuth refresh failed (${response.status})`);
  return response.text();
}

async function traktDeviceRequest(path: string, body: Record<string, string>): Promise<string> {
  const clientId = oauthClientIds.trakt;
  if (!clientId) throw new Error('Trakt OAuth client is not configured');
  const response = await fetch(`https://api.trakt.tv${path}`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json', 'trakt-api-key': clientId },
    body: JSON.stringify({ ...body, client_id: clientId }),
  });
  if (!response.ok) throw new Error(`Trakt device request failed (${response.status})`);
  return response.text();
}

export async function webInvoke<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  switch (command) {
    case 'core_invoke':
      return coreInvoke(args?.method as string, args?.argsJson as string) as Promise<T>;
    case 'engine_init':
      return engineInit(args?.initialJson as string) as Promise<T>;
    case 'engine_dispatch': {
      const actionJson = args?.actionJson as string;
      console.debug('[fluxa:web:engine:dispatch:start]', JSON.parse(actionJson).type);
      return engineDispatch(actionJson)
        .then((result) => {
          console.debug('[fluxa:web:engine:dispatch:end]', JSON.parse(actionJson).type, Boolean(result));
          return result as T;
        })
        .catch((error) => {
          console.error('[fluxa:web:engine:dispatch:error]', JSON.parse(actionJson).type, error);
          throw error;
        });
    }
    case 'engine_complete_effect': {
      const resultJson = args?.resultJson as string;
      const effect = JSON.parse(resultJson) as { effectId?: string; status?: string; error?: string };
      console.debug('[fluxa:web:engine:effect:start]', effect.effectId, effect.status, effect.error);
      return engineCompleteEffect(resultJson)
        .then((result) => {
          console.debug('[fluxa:web:engine:effect:end]', effect.effectId, Boolean(result));
          return result as T;
        })
        .catch((error) => {
          console.error('[fluxa:web:engine:effect:error]', effect.effectId, error);
          throw error;
        });
    }
    case 'engine_snapshot':
      return engineSnapshot() as Promise<T>;
    case 'storage_read':
      return storageRead(args?.key as string) as T;
    case 'storage_write':
      return storageWrite(args?.key as string, args?.value as string) as T;
    case 'storage_delete':
      return storageDelete(args?.key as string) as T;
    case 'library_snapshot':
      return library.librarySnapshot(args?.profileKey as string) as T;
    case 'library_progress_read':
      return library.progressRead(args?.profileKey as string, args?.mediaId as string) as T;
    case 'library_progress_list':
      return library.progressList(args?.profileKey as string) as T;
    case 'library_progress_upsert':
      return library.progressUpsert(args?.profileKey as string, args?.mediaId as string, args?.progressJson as string) as T;
    case 'library_progress_upsert_many':
      return library.progressUpsertMany(args?.profileKey as string, args?.updatesJson as string) as T;
    case 'library_progress_delete':
      return library.progressDelete(args?.profileKey as string, args?.mediaId as string) as T;
    case 'library_status_set':
      return library.statusSet(
        args?.profileKey as string,
        args?.mediaId as string,
        (args?.status as string | null) ?? null,
        (args?.itemJson as string | null) ?? null,
      ) as T;
    case 'library_status_list':
      return library.statusList(args?.profileKey as string) as T;
    case 'library_watched_set':
      return library.watchedSet(args?.profileKey as string, args?.videoId as string, args?.watched === true) as T;
    case 'library_watched_list':
      return library.watchedList(args?.profileKey as string) as T;
    case 'library_last_watched_list':
      return library.lastWatchedList(args?.profileKey as string) as T;
    case 'library_last_watched_upsert':
      return library.lastWatchedUpsert(args?.profileKey as string, args?.seriesId as string, args?.entryJson as string) as T;
    case 'library_last_watched_delete':
      return library.lastWatchedDelete(args?.profileKey as string, args?.seriesId as string) as T;
    case 'library_continue_watching_list':
      return library.continueWatchingList(args?.profileKey as string) as T;
    case 'library_continue_watching_upsert':
      return library.continueWatchingUpsert(args?.profileKey as string, args?.mediaId as string, args?.itemJson as string) as T;
    case 'library_continue_watching_delete':
      return library.continueWatchingDelete(args?.profileKey as string, args?.mediaId as string) as T;
    case 'http_fetch_text': {
      const response = await fetch(args?.url as string);
      return { status_code: response.status, body: await response.text() } as T;
    }
    case 'http_execute_text': {
      const response = await fetch(args?.url as string, {
        method: args?.method as string,
        headers: args?.headers as Record<string, string> | undefined,
        body: args?.body == null ? undefined : String(args.body),
      });
      return { status_code: response.status, body: await response.text() } as T;
    }
    case 'debug_log':
      console.debug('[fluxa]', args?.msg);
      return undefined as T;
    case 'get_version':
      return 'web' as T;
    case 'get_oauth_client_id':
      return (oauthClientIds[String(args?.service)] || '') as T;
    case 'trakt_device_start':
      return traktDeviceRequest('/oauth/device/code', {}) as Promise<T>;
    case 'trakt_device_poll':
      return traktDeviceRequest('/oauth/device/token', { code: String(args?.deviceCode) }) as Promise<T>;
    case 'trakt_oauth_exchange':
      return oauthExchange('trakt', String(args?.code)) as Promise<T>;
    case 'anilist_oauth_exchange':
      return oauthExchange('anilist', String(args?.code)) as Promise<T>;
    case 'simkl_oauth_exchange':
      return fetch('https://api.simkl.com/oauth/token', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          code: args?.code,
          client_id: oauthClientIds.simkl,
          code_verifier: args?.codeVerifier,
          redirect_uri: `${window.location.origin}${window.location.pathname}?oauth=simkl`,
          grant_type: 'authorization_code',
        }),
      }).then(async (response) => {
        if (!response.ok) throw new Error(`Simkl OAuth exchange failed (${response.status})`);
        return response.text();
      }) as Promise<T>;
    case 'trakt_oauth_refresh':
      return oauthRefresh('trakt', String(args?.refreshToken)) as Promise<T>;
    case 'nuvio_request': {
      if (!nuvioUrl || !nuvioKey) throw new Error('Nuvio web configuration is missing');
      const path = args?.path as string;
      if (!path.startsWith('/') || path.startsWith('//') || path.startsWith('/\\')) throw new Error('invalid Nuvio path');
      const method = args?.method as string;
      const token = args?.token as string | null | undefined;
      const requestUrl = `${nuvioUrl.replace(/\/$/, '')}${path}`;
      console.debug('[fluxa:web:nuvio:start]', method, path);
      const response = await fetch(requestUrl, {
        method,
        headers: {
          apikey: nuvioKey,
          'Content-Type': 'application/json',
          ...(token ? { Authorization: `Bearer ${token}` } : {}),
        },
        body: args?.body == null ? (method === 'POST' ? '{}' : undefined) : String(args.body),
      });
      const responseText = await response.text();
      console.debug('[fluxa:web:nuvio:end]', response.status, path);
      return [response.status, responseText] as T;
    }
    case 'start_torrent_stream': {
      const response = await companion('/torrent/start', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(args),
      });
      if (!response.ok) throw new Error(`torrent start failed (${response.status})`);
      return ((await response.json()) as { url: string }).url as T;
    }
    case 'stop_torrent_stream': {
      const response = await companion('/torrent/stop', { method: 'POST' });
      return (response.ok ? await response.json() : false) as T;
    }
    case 'register_trailer_proxy_url':
      return args?.url as T;
    default:
      throw new PlatformUnsupportedError(command);
  }
}
