import {
  coreBuildContinueWatchingFromProgress,
  coreInvoke,
  coreNormalizeLibraryDocument,
  libraryContinueWatchingDelete,
  libraryContinueWatchingList,
  libraryContinueWatchingUpsert,
  libraryLastWatchedDelete,
  libraryLastWatchedList,
  libraryLastWatchedUpsert,
  libraryProgressDelete,
  libraryProgressList,
  libraryProgressUpsert,
  libraryStatusList,
  libraryStatusSet,
  libraryWatchedList,
  libraryWatchedSet,
  storageRead,
  storageWrite,
} from './engine';
import { normalizeAddonDescriptor } from './addons';
import type { AddonDescriptor, UserProfile } from './types';

let _cachedLibraryKey: string | null = null;

export function invalidateLibraryKeyCache(): void {
  _cachedLibraryKey = null;
}

async function activeProfileStorageSuffix(): Promise<string> {
  const profileId = (await storageRead<string>('active_profile_id'))?.trim();
  return profileId ? profileId.replace(/[^a-zA-Z0-9_-]/g, '_') : 'guest';
}

export function profileStorageKey(profile: UserProfile): string {
  return `library_${profile.id.replace(/[^a-zA-Z0-9_-]/g, '_')}`;
}

export async function effectRunnerLibraryKey(): Promise<string> {
  if (_cachedLibraryKey) return _cachedLibraryKey;
  _cachedLibraryKey = `library_${await activeProfileStorageSuffix()}`;
  return _cachedLibraryKey;
}

export async function addonsStorageKey(): Promise<string> {
  return `addons_${await activeProfileStorageSuffix()}`;
}

export async function loadAddons(): Promise<AddonDescriptor[]> {
  return Promise.all(((await storageRead<AddonDescriptor[]>(await addonsStorageKey())) ?? []).map(normalizeAddonDescriptor));
}

export async function saveAddons(addons: AddonDescriptor[]): Promise<void> {
  await storageWrite(await addonsStorageKey(), addons);
}

export async function normalizeLibraryDoc(lib: Record<string, unknown>): Promise<Record<string, unknown>> {
  return coreNormalizeLibraryDocument(JSON.stringify(lib));
}

async function readStructuredLibraryDomains(key: string): Promise<{
  progress: Record<string, unknown>;
  statuses: Record<string, unknown[]>;
  watched: Record<string, boolean>;
  lastWatchedEpisodes: Record<string, unknown>;
  externalContinueWatching: unknown[];
}> {
  const [progress, statuses, watched, lastWatchedEpisodes, externalContinueWatching] = await Promise.all([
    libraryProgressList<unknown>(key),
    libraryStatusList(key),
    libraryWatchedList(key),
    libraryLastWatchedList<unknown>(key),
    libraryContinueWatchingList(key),
  ]);
  return { progress, statuses, watched, lastWatchedEpisodes, externalContinueWatching };
}

export async function loadLibrary(profileKey?: string): Promise<Record<string, unknown>> {
  const key = profileKey ?? await effectRunnerLibraryKey();
  const profileLibrary = await storageRead<Record<string, unknown>>(key);
  if (profileLibrary) {
    const { progress, statuses, watched, lastWatchedEpisodes, externalContinueWatching } = await readStructuredLibraryDomains(key);
    return normalizeLibraryDoc({ ...profileLibrary, ...statuses, progress, watched, lastWatchedEpisodes, externalContinueWatching });
  }
  const legacyLibrary = await storageRead<Record<string, unknown>>('library');
  if (legacyLibrary) {
    const migrated = await normalizeLibraryDoc({ ...legacyLibrary, migratedFrom: 'library' });
    await storageWrite(key, migrated);
    const { progress, statuses, watched, lastWatchedEpisodes, externalContinueWatching } = await readStructuredLibraryDomains(key);
    return normalizeLibraryDoc({ ...migrated, ...statuses, progress, watched, lastWatchedEpisodes, externalContinueWatching });
  }
  const { progress, statuses, watched, lastWatchedEpisodes, externalContinueWatching } = await readStructuredLibraryDomains(key);
  return normalizeLibraryDoc({ ...statuses, progress, watched, lastWatchedEpisodes, externalContinueWatching });
}

export async function saveLibrary(lib: Record<string, unknown>, profileKey?: string): Promise<void> {
  await storageWrite(profileKey ?? await effectRunnerLibraryKey(), await normalizeLibraryDoc(lib));
}

export async function loadPrefs(): Promise<Record<string, unknown>> {
  return (await storageRead<Record<string, unknown>>('prefs')) ?? {};
}

export async function loadActiveProfile(): Promise<UserProfile | null> {
  const profileId = await storageRead<string>('active_profile_id');
  if (!profileId) return null;
  const profiles = (await storageRead<UserProfile[]>('profiles')) ?? [];
  return profiles.find((profile) => profile.id === profileId) ?? null;
}

export async function buildContinueWatching(progressMap: Record<string, unknown>): Promise<unknown[]> {
  return (await coreBuildContinueWatchingFromProgress(JSON.stringify(progressMap))) ?? [];
}

async function diffPlan<T>(method: 'watchedMapDiff' | 'valueMapDiff' | 'itemListDiff' | 'itemListNewEntries', before: unknown, after: unknown): Promise<T | null> {
  return coreInvoke<T>(method, JSON.stringify({
    beforeJson: JSON.stringify(before),
    afterJson: JSON.stringify(after),
  }));
}

export async function persistStatusListMerge(
  before: Record<string, unknown>[],
  after: Record<string, unknown>[],
  list: 'watchlist' | 'completed' | 'dropped',
  profileKey?: string,
): Promise<void> {
  const key = profileKey ?? await effectRunnerLibraryKey();
  const newEntries = (await diffPlan<Record<string, unknown>[]>('itemListNewEntries', before, after)) ?? [];
  for (const item of newEntries) {
    const id = item.id as string | undefined;
    if (id) await libraryStatusSet(key, id, list, item);
  }
}

export async function persistWatchedMerge(
  before: Record<string, boolean>,
  after: Record<string, boolean>,
  profileKey?: string,
): Promise<void> {
  const key = profileKey ?? await effectRunnerLibraryKey();
  const changed = (await diffPlan<Array<{ id: string; value: boolean }>>('watchedMapDiff', before, after)) ?? [];
  for (const { id, value } of changed) {
    await libraryWatchedSet(key, id, value);
  }
}

export async function persistLastWatchedEpisode(seriesId: string, entry: unknown | null): Promise<void> {
  const key = await effectRunnerLibraryKey();
  if (entry === null) await libraryLastWatchedDelete(key, seriesId);
  else await libraryLastWatchedUpsert(key, seriesId, entry);
}

export async function persistContinueWatchingMerge(
  before: Record<string, unknown>[],
  after: Record<string, unknown>[],
  profileKey?: string,
): Promise<void> {
  const key = profileKey ?? await effectRunnerLibraryKey();
  const plan = await diffPlan<{ upserts: Record<string, unknown>[]; deletes: string[] }>('itemListDiff', before, after);
  for (const item of plan?.upserts ?? []) {
    await libraryContinueWatchingUpsert(key, item.id as string, item);
  }
  for (const id of plan?.deletes ?? []) {
    await libraryContinueWatchingDelete(key, id);
  }
}

export async function persistProgressMerge(
  before: Record<string, unknown>,
  after: Record<string, unknown>,
  profileKey?: string,
): Promise<void> {
  const key = profileKey ?? await effectRunnerLibraryKey();
  const plan = await diffPlan<{ upserts: Array<{ id: string; value: unknown }>; deletes: string[] }>('valueMapDiff', before, after);
  for (const { id, value } of plan?.upserts ?? []) {
    await libraryProgressUpsert(key, id, value);
  }
  for (const id of plan?.deletes ?? []) {
    await libraryProgressDelete(key, id);
  }
}
