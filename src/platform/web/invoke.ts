import { storageDelete, storageRead, storageWrite } from './storage';
import { coreInvoke, engineCompleteEffect, engineDispatch, engineInit, engineSnapshot } from './wasmEngine';

const companionUrl = import.meta.env.VITE_FLUXA_COMPANION_URL || 'http://127.0.0.1:19876';

export class PlatformUnsupportedError extends Error {
  constructor(command: string) {
    super(`${command} is not available in this platform`);
    this.name = 'PlatformUnsupportedError';
  }
}

async function companion(path: string, init?: RequestInit): Promise<Response> {
  return fetch(`${companionUrl}${path}`, init);
}

export async function webInvoke<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  switch (command) {
    case 'core_invoke':
      return coreInvoke(args?.method as string, args?.argsJson as string) as Promise<T>;
    case 'engine_init':
      return engineInit(args?.initialJson as string) as Promise<T>;
    case 'engine_dispatch':
      return engineDispatch(args?.actionJson as string) as Promise<T>;
    case 'engine_complete_effect':
      return engineCompleteEffect(args?.resultJson as string) as Promise<T>;
    case 'engine_snapshot':
      return engineSnapshot() as Promise<T>;
    case 'storage_read':
      return storageRead(args?.key as string) as T;
    case 'storage_write':
      return storageWrite(args?.key as string, args?.value as string) as T;
    case 'storage_delete':
      return storageDelete(args?.key as string) as T;
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
    case 'start_torrent_stream': {
      const response = await companion('/torrent/start', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(args),
      });
      if (!response.ok) throw new Error(`torrent start failed (${response.status})`);
      return (await response.json() as { url: string }).url as T;
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
