import { storageRead, storageWrite } from './engine';
import type { Meta, Stream } from './types';

export async function libraryStorageKey(): Promise<string> {
  const profileId = (await storageRead<string>('active_profile_id'))?.trim();
  const suffix = profileId ? profileId.replace(/[^a-zA-Z0-9_-]/g, '_') : 'guest';
  return `library_${suffix}`;
}

export async function readProfileLibrary(key: string): Promise<Record<string, unknown>> {
  const profileLibrary = await storageRead<Record<string, unknown>>(key);
  if (profileLibrary) return normalizeLibraryDocument(profileLibrary);
  const legacyLibrary = await storageRead<Record<string, unknown>>('library');
  if (legacyLibrary) {
    const migrated = normalizeLibraryDocument({ ...legacyLibrary, migratedFrom: 'library' });
    await storageWrite(key, migrated);
    return migrated;
  }
  return normalizeLibraryDocument({});
}

export function normalizeLibraryDocument(library: Record<string, unknown>): Record<string, unknown> {
  return {
    schemaVersion: 2,
    ...library,
    progress: library.progress && typeof library.progress === 'object' && !Array.isArray(library.progress)
      ? library.progress
      : {},
    continueWatching: Array.isArray(library.continueWatching) ? library.continueWatching : [],
  };
}

export async function persistLastPlaybackSource(meta: Meta, stream: Stream | null): Promise<void> {
  if (!stream?.url && !stream?.playableUrl && !stream?.infoHash) return;
  const key = await libraryStorageKey();
  const library = await readProfileLibrary(key);
  const progress = { ...((library.progress as Record<string, unknown> | undefined) ?? {}) };
  const existing = { ...((progress[meta.id] as Record<string, unknown> | undefined) ?? {}) };
  const lastStreamUrl = stream.playableUrl ?? stream.url ?? null;
  const lastStreamTitle = stream.title ?? stream.name ?? existing.lastStreamTitle ?? null;

  progress[meta.id] = { ...existing, lastStream: stream, lastStreamUrl, lastStreamTitle };

  const continueWatching = Array.isArray(library.continueWatching)
    ? library.continueWatching.map((raw) => {
        const item = raw as Record<string, unknown>;
        return item.id === meta.id
          ? { ...item, lastStream: stream, lastStreamUrl, lastStreamTitle }
          : item;
      })
    : library.continueWatching;

  await storageWrite(key, { ...library, progress, continueWatching });
}

export async function readStoredPlaybackSource(metaId: string): Promise<Stream | null> {
  const library = await readProfileLibrary(await libraryStorageKey());
  const progress = (library.progress as Record<string, unknown> | undefined) ?? {};
  const entry = progress[metaId] as Record<string, unknown> | undefined;
  const stream = entry?.lastStream as Stream | undefined;
  if (stream?.url || stream?.playableUrl || stream?.infoHash) return stream;
  return null;
}
