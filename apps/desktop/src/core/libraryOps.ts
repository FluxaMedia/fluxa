import {
  coreBuildContinueWatchingFromProgress,
  coreInvoke,
  coreNormalizeLibraryDocument,
  libraryContinueWatchingDelete,
  libraryContinueWatchingUpsert,
  libraryLastWatchedDelete,
  libraryLastWatchedUpsert,
  libraryProgressDelete,
  libraryProgressUpsertMany,
  librarySnapshot,
  libraryStatusSet,
  libraryWatchedSet,
  storageRead,
  storageWrite,
} from './engine';
import { normalizeAddonDescriptor } from './addons';
import { coreFilterEnabledAddons } from './engineCoreLibrary';
import type { AddonDescriptor, UserProfile } from './types';

let _cachedLibraryKey: string | null = null;
let _cachedAddonsOwnerId: string | null = null;
let _cachedPrefsOwnerId: string | null = null;
let _enabledAddonsSnapshot: Promise<AddonDescriptor[]> | null = null;
let _profileContextPromise: Promise<{ activeId: string; profiles: UserProfile[] }> | null = null;

export function invalidateLibraryKeyCache(): void {
  _cachedLibraryKey = null;
  invalidateAddonSnapshot();
}

export function invalidateAddonSnapshot(): void {
  _cachedAddonsOwnerId = null;
  _cachedPrefsOwnerId = null;
  _enabledAddonsSnapshot = null;
  _profileContextPromise = null;
}

function profileContext(): Promise<{ activeId: string; profiles: UserProfile[] }> {
  if (!_profileContextPromise) {
    _profileContextPromise = Promise.all([
      storageRead<string>('active_profile_id'),
      storageRead<UserProfile[]>('profiles'),
    ]).then(([activeId, profiles]) => ({
      activeId: activeId?.trim() ?? '',
      profiles: profiles ?? [],
    }));
  }
  return _profileContextPromise;
}

async function activeProfileStorageSuffix(): Promise<string> {
  const { activeId } = await profileContext();
  return activeId ? activeId.replace(/[^a-zA-Z0-9_-]/g, '_') : 'guest';
}

export function profileStorageKey(profile: UserProfile): string {
  return `library_${profile.id.replace(/[^a-zA-Z0-9_-]/g, '_')}`;
}

export async function effectRunnerLibraryKey(): Promise<string> {
  if (_cachedLibraryKey) return _cachedLibraryKey;
  _cachedLibraryKey = `library_${await activeProfileStorageSuffix()}`;
  return _cachedLibraryKey;
}

export async function effectiveAddonsOwnerId(): Promise<string> {
  if (_cachedAddonsOwnerId) return _cachedAddonsOwnerId;
  const { activeId, profiles } = await profileContext();
  const ownerId = await coreInvoke<string>('effectiveAddonsOwnerId', JSON.stringify({ profiles, activeProfileId: activeId })).catch(
    () => null,
  );
  _cachedAddonsOwnerId = (ownerId || activeId || 'guest').replace(/[^a-zA-Z0-9_-]/g, '_');
  return _cachedAddonsOwnerId;
}

export async function addonsStorageKey(): Promise<string> {
  return `addons_${await effectiveAddonsOwnerId()}`;
}

export async function loadAddons(): Promise<AddonDescriptor[]> {
  const startedAt = performance.now();
  const keyStartedAt = performance.now();
  const key = await addonsStorageKey();
  console.debug('[fluxa:addons:key-ready]', JSON.stringify({ ms: Math.round(performance.now() - keyStartedAt) }));
  const stored = (await storageRead<unknown[]>(key)) ?? [];
  console.debug('[fluxa:addons:storage-ready]', JSON.stringify({ count: stored.length, ms: Math.round(performance.now() - startedAt) }));
  const result = await Promise.all(
    stored.map((addon) => {
      // Addons saved by the current schema are already core-normalized. Avoid
      // a Rust IPC round-trip per addon on every detail/search request. Keep
      // the normalizer for legacy records and older storage migrations.
      if (
        addon &&
        typeof addon === 'object' &&
        typeof (addon as { transportUrl?: unknown }).transportUrl === 'string' &&
        (addon as { manifest?: unknown }).manifest &&
        typeof (addon as { manifest: { id?: unknown } }).manifest.id === 'string'
      ) {
        return addon as AddonDescriptor;
      }
      return normalizeAddonDescriptor(addon as Parameters<typeof normalizeAddonDescriptor>[0]);
    }),
  );
  console.debug('[fluxa:addons:normalized]', JSON.stringify({ count: result.length, ms: Math.round(performance.now() - startedAt) }));
  return result;
}

export async function saveAddons(addons: AddonDescriptor[]): Promise<void> {
  await storageWrite(await addonsStorageKey(), addons);
  invalidateAddonSnapshot();
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
  const snapshot = await librarySnapshot<{
    progress: Record<string, unknown>;
    statuses: Record<string, unknown[]>;
    watched: Record<string, boolean>;
    lastWatchedEpisodes: Record<string, unknown>;
    externalContinueWatching: unknown[];
  }>(key);
  return snapshot ?? { progress: {}, statuses: {}, watched: {}, lastWatchedEpisodes: {}, externalContinueWatching: [] };
}

export async function loadLibrary(profileKey?: string): Promise<Record<string, unknown>> {
  const key = profileKey ?? (await effectRunnerLibraryKey());
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
  await storageWrite(profileKey ?? (await effectRunnerLibraryKey()), await normalizeLibraryDoc(lib));
}

export async function prefsOwnerId(): Promise<string> {
  if (_cachedPrefsOwnerId) return _cachedPrefsOwnerId;
  const { activeId, profiles } = await profileContext();
  if (activeId) {
    _cachedPrefsOwnerId = activeId.replace(/[^a-zA-Z0-9_-]/g, '_');
    return _cachedPrefsOwnerId;
  }
  if (profiles.length === 0) {
    _cachedPrefsOwnerId = 'guest';
    return _cachedPrefsOwnerId;
  }
  const primaryId = await coreInvoke<string>('primaryProfileId', JSON.stringify(profiles)).catch(() => null);
  _cachedPrefsOwnerId = (primaryId || profiles[0].id).replace(/[^a-zA-Z0-9_-]/g, '_');
  return _cachedPrefsOwnerId;
}

export async function prefsStorageKey(): Promise<string> {
  return `prefs_${await prefsOwnerId()}`;
}

export async function loadPrefs(): Promise<Record<string, unknown>> {
  return (await storageRead<Record<string, unknown>>(await prefsStorageKey())) ?? {};
}

export async function savePrefs(value: Record<string, unknown>): Promise<void> {
  await storageWrite(await prefsStorageKey(), value);
}

export async function loadActiveProfile(): Promise<UserProfile | null> {
  const { activeId, profiles } = await profileContext();
  if (!activeId) return null;
  return profiles.find((profile) => profile.id === activeId) ?? null;
}

export async function loadEnabledAddons(profileOverride?: UserProfile | null): Promise<AddonDescriptor[]> {
  if (profileOverride === undefined && _enabledAddonsSnapshot) return _enabledAddonsSnapshot;
  if (profileOverride === undefined) {
    _enabledAddonsSnapshot = loadEnabledAddonsUncached();
    return _enabledAddonsSnapshot;
  }
  return loadEnabledAddonsUncached(profileOverride);
}

async function loadEnabledAddonsUncached(profileOverride?: UserProfile | null): Promise<AddonDescriptor[]> {
  const startedAt = performance.now();
  const [addons, loadedProfile] =
    profileOverride === undefined ? await Promise.all([loadAddons(), loadActiveProfile()]) : [await loadAddons(), profileOverride];
  const profile = loadedProfile;
  console.debug('[fluxa:addons:enabled-inputs]', JSON.stringify({ count: addons.length, hasProfile: Boolean(profile), ms: Math.round(performance.now() - startedAt) }));
  const disabledAddonKeys = profile?.addonSettings?.disabledLocalAddons ?? profile?.disabledLocalAddons ?? [];
  if (!disabledAddonKeys.length) return addons;
  return (await coreFilterEnabledAddons(addons, disabledAddonKeys)) ?? addons;
}

export async function buildContinueWatching(progressMap: Record<string, unknown>): Promise<unknown[]> {
  return (await coreBuildContinueWatchingFromProgress(JSON.stringify(progressMap))) ?? [];
}

async function diffPlan<T>(
  method: 'watchedMapDiff' | 'valueMapDiff' | 'itemListDiff' | 'itemListNewEntries',
  before: unknown,
  after: unknown,
): Promise<T | null> {
  return coreInvoke<T>(
    method,
    JSON.stringify({
      beforeJson: JSON.stringify(before),
      afterJson: JSON.stringify(after),
    }),
  );
}

export async function persistStatusListMerge(
  before: Record<string, unknown>[],
  after: Record<string, unknown>[],
  list: 'watchlist' | 'completed' | 'dropped',
  profileKey?: string,
): Promise<void> {
  const key = profileKey ?? (await effectRunnerLibraryKey());
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
  const key = profileKey ?? (await effectRunnerLibraryKey());
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
  const key = profileKey ?? (await effectRunnerLibraryKey());
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
  const key = profileKey ?? (await effectRunnerLibraryKey());
  const plan = await diffPlan<{ upserts: Array<{ id: string; value: unknown }>; deletes: string[] }>('valueMapDiff', before, after);
  if (plan?.upserts?.length) {
    await libraryProgressUpsertMany(
      key,
      plan.upserts.map(({ id, value }) => ({ mediaId: id, value })),
    );
  }
  for (const id of plan?.deletes ?? []) {
    await libraryProgressDelete(key, id);
  }
}
