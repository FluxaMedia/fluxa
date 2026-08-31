import { platformFetch } from './httpClient';
import type { AddonDescriptor } from './types';

const STREMIO_API = 'https://api.strem.io';

export class StremioApiError extends Error {
  code?: number;

  constructor(message: string, code?: number) {
    super(message);
    this.name = 'StremioApiError';
    this.code = code;
  }
}

async function stremioPost<T>(path: string, body: Record<string, unknown>): Promise<T> {
  const res = await platformFetch(`${STREMIO_API}${path}`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json', Accept: 'application/json' },
    body: JSON.stringify(body),
  });
  if (!res.ok) {
    throw new StremioApiError(`Stremio API ${res.status}`, res.status);
  }
  const data = (await res.json()) as { result?: T; error?: { message?: string; code?: number } };
  if (data.error) {
    throw new StremioApiError(data.error.message ?? 'Stremio request failed', data.error.code);
  }
  if (data.result == null) {
    throw new StremioApiError('Stremio request returned no result');
  }
  return data.result;
}

export async function stremioPullLibrary(authKey: string): Promise<Record<string, unknown>[]> {
  const result = await stremioPost<Record<string, unknown>[]>('/api/datastoreGet', {
    authKey,
    collection: 'libraryItem',
    ids: [],
    all: true,
  });
  return Array.isArray(result) ? result : [];
}

export async function stremioPushLibrary(authKey: string, changes: Record<string, unknown>[]): Promise<void> {
  if (changes.length === 0) return;
  await stremioPost('/api/datastorePut', {
    authKey,
    collection: 'libraryItem',
    changes,
  });
}

export async function stremioPullAddons(authKey: string): Promise<AddonDescriptor[]> {
  const result = await stremioPost<{ addons?: AddonDescriptor[] }>('/api/addonCollectionGet', {
    authKey,
    update: true,
  });
  return Array.isArray(result.addons) ? result.addons : [];
}

export async function stremioReplaceAddons(authKey: string, addons: AddonDescriptor[]): Promise<void> {
  await stremioPost('/api/addonCollectionSet', { authKey, addons });
}
