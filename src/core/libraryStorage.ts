import { storageRead, storageWrite, coreNormalizeLibraryDocument, libraryProgressRead, libraryProgressUpsert } from './engine';
import { effectRunnerLibraryKey } from './libraryOps';
import type { Meta, Stream } from './types';

export async function libraryStorageKey(): Promise<string> {
  return effectRunnerLibraryKey();
}

export async function readProfileLibrary(key: string): Promise<Record<string, unknown>> {
  const profileLibrary = await storageRead<Record<string, unknown>>(key);
  if (profileLibrary) return coreNormalizeLibraryDocument(JSON.stringify(profileLibrary));
  const legacyLibrary = await storageRead<Record<string, unknown>>('library');
  if (legacyLibrary) {
    const migrated = await coreNormalizeLibraryDocument(JSON.stringify({ ...legacyLibrary, migratedFrom: 'library' }));
    await storageWrite(key, migrated);
    return migrated;
  }
  return coreNormalizeLibraryDocument('{}');
}

export async function persistLastPlaybackSource(meta: Meta, stream: Stream | null): Promise<void> {
  if (!stream?.url && !stream?.playableUrl && !stream?.infoHash) return;
  const key = await libraryStorageKey();
  const existing = { ...((await libraryProgressRead<Record<string, unknown>>(key, meta.id)) ?? {}) };
  const lastStreamUrl = stream.playableUrl ?? stream.url ?? null;
  const lastStreamTitle = stream.title ?? stream.name ?? existing.lastStreamTitle ?? null;

  await libraryProgressUpsert(key, meta.id, { ...existing, lastStream: stream, lastStreamUrl, lastStreamTitle });
}

export async function readStoredPlaybackSource(metaId: string): Promise<Stream | null> {
  const entry = await libraryProgressRead<Record<string, unknown>>(await libraryStorageKey(), metaId);
  const stream = entry?.lastStream as Stream | undefined;
  if (stream?.url || stream?.playableUrl || stream?.infoHash) return stream;
  return null;
}
