import {
  nuvioRefreshToken,
  nuvioPullAddons,
  nuvioPullCollections,
  nuvioPullLibrary,
  nuvioPullProfiles,
  nuvioPullWatchHistory,
  nuvioPullWatchProgress,
  nuvioPullWatchProgressDelta,
  nuvioGetWatchProgressDeltaCursor,
  nuvioPullWatchHistoryDelta,
  nuvioGetWatchHistoryDeltaCursor,
  nuvioListAvatars,
  type NuvioAddon,
  type NuvioAvatar,
  type NuvioProfile,
  type NuvioWatchedItem,
  type NuvioWatchProgress,
  type NuvioWatchProgressDeltaEvent,
  type NuvioWatchedItemDeltaEvent,
} from './nuvioApi';
import { platformFetch } from './httpClient';
import { buildContinueWatching, loadLibrary, saveLibrary, persistProgressMerge, persistWatchedMerge, persistContinueWatchingMerge } from './libraryOps';
import {
  coreNuvioBuildLocalProfiles,
  coreNuvioImportMergePlan,
  coreNuvioLibraryToWatchlist,
  coreNuvioMapCollections,
  coreNuvioProgressMetaNeeds,
  coreNuvioSortAddonsByPriority,
  storageRead,
  storageWrite,
} from './engine';
import type { UserProfile } from './types';
import { loadProfiles, saveProfile, saveProfiles } from './profiles';
import { fetchPlannedResources } from './fetchPlanning';
import { saveProviderLibrary } from './providerLibraries';

export type NuvioImportStep = 'addons' | 'library' | 'progress' | 'history' | 'collections';

export interface NuvioImportReport {
  errors: Partial<Record<NuvioImportStep, string>>;
  changed: boolean;
}

export interface NuvioSyncMeta {
  lastSyncAt: number;
  continueWatchingCount: number;
  watchlistCount: number;
  error?: string;
}

export async function recordNuvioSyncMeta(report: NuvioImportReport | { errors: Partial<Record<NuvioImportStep, string>> }): Promise<void> {
  const failures = Object.entries(report.errors);
  const error = failures.length > 0 ? failures.map(([step, msg]) => `${step}: ${msg}`).join('; ') : undefined;
  const meta: NuvioSyncMeta = { lastSyncAt: Date.now(), continueWatchingCount: 0, watchlistCount: 0, error };
  await storageWrite('nuvio_sync_meta', meta);
}

export async function freshNuvioProfile(profile: UserProfile): Promise<UserProfile> {
  if (!profile.nuvioRefreshToken) return profile;
  const expiresAt = profile.nuvioTokenExpiresAt ?? 0;
  if (profile.nuvioAccessToken && expiresAt > Math.floor(Date.now() / 1000) + 60) return profile;
  const session = await nuvioRefreshToken(profile.nuvioRefreshToken);
  const updated: UserProfile = {
    ...profile,
    nuvioAccessToken: session.access_token,
    nuvioRefreshToken: session.refresh_token ?? profile.nuvioRefreshToken,
    nuvioTokenExpiresAt: Math.floor(Date.now() / 1000) + (session.expires_in ?? 3600),
    nuvioUserId: session.user?.id ?? profile.nuvioUserId,
  };
  await saveProfile(updated);
  return updated;
}

function profileStorageSuffix(profile: UserProfile): string {
  return profile.id.replace(/[^a-zA-Z0-9_-]/g, '_');
}

function deltaCursorKey(profile: UserProfile, resource: 'progress' | 'history'): string {
  return `nuvio_${resource}_event_cursor_${profileStorageSuffix(profile)}`;
}

function deltaCacheKey(profile: UserProfile, resource: 'progress' | 'history'): string {
  return `nuvio_${resource}_remote_cache_${profileStorageSuffix(profile)}`;
}

function watchedItemKey(item: Pick<NuvioWatchedItem, 'content_id' | 'season' | 'episode'>): string {
  return `${item.content_id}:${item.season ?? ''}:${item.episode ?? ''}`;
}

function applyProgressDelta(items: NuvioWatchProgress[], events: NuvioWatchProgressDeltaEvent[]): NuvioWatchProgress[] {
  const byKey = new Map(items.map((item) => [item.progress_key, item]));
  for (const event of [...events].sort((a, b) => a.event_id - b.event_id)) {
    if (event.operation === 'delete') byKey.delete(event.progress_key);
    else byKey.set(event.progress_key, event);
  }
  return [...byKey.values()];
}

function applyHistoryDelta(items: NuvioWatchedItem[], events: NuvioWatchedItemDeltaEvent[]): NuvioWatchedItem[] {
  const byKey = new Map(items.map((item) => [watchedItemKey(item), item]));
  for (const event of [...events].sort((a, b) => a.event_id - b.event_id)) {
    const key = watchedItemKey(event);
    if (event.operation === 'delete') byKey.delete(key);
    else byKey.set(key, event);
  }
  return [...byKey.values()];
}

async function pullAllProgressDelta(token: string, profileId: number, cursor: number): Promise<NuvioWatchProgressDeltaEvent[]> {
  const events: NuvioWatchProgressDeltaEvent[] = [];
  let nextCursor = cursor;
  for (;;) {
    const batch = await nuvioPullWatchProgressDelta(token, profileId, nextCursor, 1_000);
    if (batch.length === 0) return events;
    events.push(...batch);
    const batchCursor = Math.max(nextCursor, ...batch.map((event) => event.event_id));
    if (batchCursor === nextCursor || batch.length < 1_000) return events;
    nextCursor = batchCursor;
  }
}

async function pullAllHistoryDelta(token: string, profileId: number, cursor: number): Promise<NuvioWatchedItemDeltaEvent[]> {
  const events: NuvioWatchedItemDeltaEvent[] = [];
  let nextCursor = cursor;
  for (;;) {
    const batch = await nuvioPullWatchHistoryDelta(token, profileId, nextCursor, 1_000);
    if (batch.length === 0) return events;
    events.push(...batch);
    const batchCursor = Math.max(nextCursor, ...batch.map((event) => event.event_id));
    if (batchCursor === nextCursor || batch.length < 1_000) return events;
    nextCursor = batchCursor;
  }
}

async function pullAllNuvioLibrary(token: string, profileId: number): Promise<Awaited<ReturnType<typeof nuvioPullLibrary>>> {
  const items: Awaited<ReturnType<typeof nuvioPullLibrary>> = [];
  const limit = 500;
  for (let offset = 0; ; offset += limit) {
    const page = await nuvioPullLibrary(token, profileId, limit, offset);
    items.push(...page);
    if (page.length < limit) return items;
  }
}

async function pullAllNuvioWatchHistory(token: string, profileId: number): Promise<NuvioWatchedItem[]> {
  const items: NuvioWatchedItem[] = [];
  const pageSize = 500;
  for (let page = 1; ; page += 1) {
    const batch = await nuvioPullWatchHistory(token, profileId, pageSize, page);
    items.push(...batch);
    if (batch.length < pageSize) return items;
  }
}

export async function buildLocalNuvioProfiles(
  sessionProfile: UserProfile,
  nuvioProfiles: NuvioProfile[],
  avatarCatalog: NuvioAvatar[],
  existingProfiles: UserProfile[],
): Promise<UserProfile[]> {
  const result = await coreNuvioBuildLocalProfiles(sessionProfile, nuvioProfiles, avatarCatalog, existingProfiles);
  return (result as UserProfile[] | null) ?? existingProfiles;
}

export async function refreshNuvioProfiles(profile: UserProfile): Promise<UserProfile> {
  const freshProfile = await freshNuvioProfile(profile);
  const token = freshProfile.nuvioAccessToken;
  if (!token) return freshProfile;
  const [nuvioProfiles, avatarCatalog, existingProfiles] = await Promise.all([
    nuvioPullProfiles(token),
    nuvioListAvatars(),
    loadProfiles(),
  ]);
  const importedProfiles = await buildLocalNuvioProfiles(freshProfile, nuvioProfiles, avatarCatalog, existingProfiles);
  await saveProfiles(importedProfiles);
  return importedProfiles.find((candidate) => candidate.id === freshProfile.id)
    ?? importedProfiles.find((candidate) => candidate.nuvioUserId === freshProfile.nuvioUserId && candidate.nuvioProfileIndex === freshProfile.nuvioProfileIndex)
    ?? freshProfile;
}

async function fetchAddonManifests(addons: NuvioAddon[]): Promise<{
  addonList: NuvioAddon[];
  manifestIdByUrl: Map<string, string>;
  descriptors: Array<Record<string, unknown>>;
}> {
  const sorted = (await coreNuvioSortAddonsByPriority(addons)) ?? addons;
  const enabled = sorted.filter((a) => a.enabled);
  const manifestIdByUrl = new Map<string, string>();
  const manifests = await Promise.allSettled(
    enabled.map(async (a) => {
      const res = await platformFetch(a.url);
      if (!res.ok) return null;
      return res.json() as Promise<Record<string, unknown>>;
    })
  );

  const manifestByUrl = new Map<string, Record<string, unknown> | null>();
  enabled.forEach((a, i) => {
    const mResult = manifests[i];
    const m = mResult.status === 'fulfilled' && mResult.value ? mResult.value : null;
    if (m?.id) manifestIdByUrl.set(a.url, String(m.id));
    manifestByUrl.set(a.url, m);
  });

  const descriptors = sorted.map((a) => {
    const m = manifestByUrl.get(a.url) ?? null;
    return {
      transportUrl: a.url,
      manifest: m ?? { id: a.url, name: a.name ?? a.url, version: '0.0.1', resources: [], types: [], catalogs: [] },
    };
  });

  return { addonList: addons, manifestIdByUrl, descriptors };
}

async function fetchAddonMetas(
  needs: Array<{ contentId: string; contentType: string }>,
  addonDescriptors: Array<Record<string, unknown>>,
): Promise<Record<string, unknown>> {
  const metas: Record<string, unknown> = {};
  if (needs.length === 0 || addonDescriptors.length === 0) return metas;
  await Promise.allSettled(
    needs.map(async (need) => {
      try {
        const values = await fetchPlannedResources({
          kind: 'metaDetail',
          addons: addonDescriptors,
          contentType: need.contentType,
          id: need.contentId,
        });
        const meta = (values.find((value) => (value as { meta?: unknown }).meta) as { meta?: { name?: string } } | undefined)?.meta;
        if (meta?.name) metas[need.contentId] = meta;
      } catch {}
    })
  );
  return metas;
}

export async function importNuvioProfileData(
  profile: UserProfile,
  onStep?: (step: NuvioImportStep, ok: boolean, error?: string) => void,
): Promise<NuvioImportReport> {
  const freshProfile = await freshNuvioProfile(profile).catch(() => profile);
  const token = freshProfile.nuvioAccessToken;
  const profileIdx = freshProfile.nuvioProfileIndex ?? 1;
  if (!token) return { errors: { library: 'Missing Nuvio token' }, changed: false };

  const suffix = profileStorageSuffix(profile);
  const profileKey = `library_${suffix}`;
  void import('@tauri-apps/api/core').then(({ invoke }) =>
    invoke('debug_log', { msg: `nuvio-import-debug: importNuvioProfileData start profile.id=${profile.id} profileIdx=${profileIdx} profileKey=${profileKey}` }),
  );
  const existingLib = await loadLibrary(profileKey);
  const libDoc: Record<string, unknown> = {
    schemaVersion: 2,
    ...existingLib,
    progress: { ...((existingLib.progress as Record<string, unknown> | undefined) ?? {}) },
    continueWatching: Array.isArray(existingLib.continueWatching) ? existingLib.continueWatching : [],
    watchlist: Array.isArray(existingLib.watchlist) ? existingLib.watchlist : [],
    watched: { ...((existingLib.watched as Record<string, boolean> | undefined) ?? {}) },
  };
  const progressBefore = { ...(libDoc.progress as Record<string, unknown>) };
  const watchedBefore = { ...(libDoc.watched as Record<string, boolean>) };
  const continueWatchingBefore = [...(libDoc.continueWatching as Record<string, unknown>[])];
  const errors: Partial<Record<NuvioImportStep, string>> = {};

  let addonDescriptors: Array<Record<string, unknown>> = [];
  try {
    const addons = await nuvioPullAddons(token, profileIdx);
    const fetched = await fetchAddonManifests(addons);
    addonDescriptors = fetched.descriptors;
    await storageWrite(`addons_${suffix}`, fetched.descriptors);
    onStep?.('addons', true);
  } catch (err) {
    errors.addons = err instanceof Error ? err.message : String(err);
    onStep?.('addons', false, errors.addons);
  }

  let library: unknown[] = [];
  try {
    library = await pullAllNuvioLibrary(token, profileIdx);
    libDoc.watchlist = (await coreNuvioLibraryToWatchlist(library)) ?? libDoc.watchlist;
    onStep?.('library', true);
  } catch (err) {
    errors.library = err instanceof Error ? err.message : String(err);
    onStep?.('library', false, errors.library);
  }

  let watchProgress: NuvioWatchProgress[] | null = null;
  let progressCursor: number | null = null;
  try {
    const cursor = await storageRead<number>(deltaCursorKey(profile, 'progress'));
    const cached = await storageRead<NuvioWatchProgress[]>(deltaCacheKey(profile, 'progress'));
    if (typeof cursor === 'number' && Array.isArray(cached)) {
      const events = await pullAllProgressDelta(token, profileIdx, cursor);
      watchProgress = applyProgressDelta(cached, events);
      if (events.length > 0) {
        progressCursor = Math.max(cursor, ...events.map((event) => event.event_id));
      }
    } else {
      watchProgress = await nuvioPullWatchProgress(token, profileIdx, 1_000);
      progressCursor = await nuvioGetWatchProgressDeltaCursor(token, profileIdx);
    }
  } catch (err) {
    errors.progress = err instanceof Error ? err.message : String(err);
    onStep?.('progress', false, errors.progress);
  }

  let addonMetas: Record<string, unknown> = {};
  if (watchProgress) {
    const needs = (await coreNuvioProgressMetaNeeds(watchProgress, library)) ?? [];
    addonMetas = await fetchAddonMetas(needs, addonDescriptors);
  }

  let watchHistory: NuvioWatchedItem[] | null = null;
  let historyCursor: number | null = null;
  try {
    const cursor = await storageRead<number>(deltaCursorKey(profile, 'history'));
    const cached = await storageRead<NuvioWatchedItem[]>(deltaCacheKey(profile, 'history'));
    if (typeof cursor === 'number' && Array.isArray(cached)) {
      const events = await pullAllHistoryDelta(token, profileIdx, cursor);
      watchHistory = applyHistoryDelta(cached, events);
      if (events.length > 0) {
        historyCursor = Math.max(cursor, ...events.map((event) => event.event_id));
      }
    } else {
      watchHistory = await pullAllNuvioWatchHistory(token, profileIdx);
      historyCursor = await nuvioGetWatchHistoryDeltaCursor(token, profileIdx);
    }
  } catch (err) {
    errors.history = err instanceof Error ? err.message : String(err);
    onStep?.('history', false, errors.history);
  }

  const plan = await coreNuvioImportMergePlan({
    progress: progressBefore,
    watched: watchedBefore,
    library,
    addonMetas,
    watchProgress,
    watchHistory,
  });
  let appliedRemoteWatchState = false;
  if (plan) {
    libDoc.progress = plan.progress;
    libDoc.watched = plan.watched;
    libDoc.continueWatching = await buildContinueWatching(plan.progress);
    await saveProviderLibrary('nuvio', {
      watchlist: (libDoc.watchlist as Record<string, unknown>[]) ?? [],
      watching: libDoc.continueWatching as Record<string, unknown>[],
      completed: [],
      dropped: [],
    }, profileKey);
    appliedRemoteWatchState = true;
  }
  if (watchProgress) onStep?.('progress', true);
  if (watchHistory) onStep?.('history', true);

  await persistProgressMerge(progressBefore, libDoc.progress as Record<string, unknown>, profileKey);
  await persistWatchedMerge(watchedBefore, libDoc.watched as Record<string, boolean>, profileKey);
  await persistContinueWatchingMerge(continueWatchingBefore, libDoc.continueWatching as Record<string, unknown>[], profileKey);
  await saveLibrary(libDoc, profileKey);
  if (appliedRemoteWatchState && watchProgress) {
    await storageWrite(deltaCacheKey(profile, 'progress'), watchProgress);
    if (progressCursor != null) await storageWrite(deltaCursorKey(profile, 'progress'), progressCursor);
  }
  if (appliedRemoteWatchState && watchHistory) {
    await storageWrite(deltaCacheKey(profile, 'history'), watchHistory);
    if (historyCursor != null) await storageWrite(deltaCursorKey(profile, 'history'), historyCursor);
  }

  try {
    const collections = await nuvioPullCollections(token, profileIdx);
    if (collections.length > 0) {
      const raw = (collections[0]?.collections_json ?? []) as unknown[];
      const mapped = (await coreNuvioMapCollections(raw)) ?? [];
      const profiles = (await storageRead<UserProfile[]>('profiles')) ?? [];
      await storageWrite('profiles', profiles.map((p) => p.id === profile.id ? { ...p, libraryCollections: mapped as UserProfile['libraryCollections'] } : p));
    }
    onStep?.('collections', true);
  } catch (err) {
    errors.collections = err instanceof Error ? err.message : String(err);
    onStep?.('collections', false, errors.collections);
  }

  const changed = JSON.stringify(continueWatchingBefore) !== JSON.stringify(libDoc.continueWatching)
    || JSON.stringify(progressBefore) !== JSON.stringify(libDoc.progress)
    || JSON.stringify(watchedBefore) !== JSON.stringify(libDoc.watched);

  return { errors, changed };
}
