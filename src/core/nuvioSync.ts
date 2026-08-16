import {
  nuvioRefreshToken,
  nuvioPullAddons,
  nuvioPullCollections,
  nuvioPullLibrary,
  nuvioPullLibraryDelta,
  nuvioGetLibraryDeltaCursor,
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
  type NuvioLibraryItem,
  type NuvioLibraryDeltaEvent,
  type NuvioWatchedItemDeltaEvent,
} from './nuvioApi';
import { platformInvoke } from '../platform/invoke';
import { platformFetch } from './httpClient';
import { buildContinueWatching } from './libraryOps';
import {
  coreNuvioBuildLocalProfiles,
  coreNuvioImportMergePlan,
  coreNuvioLibraryToWatchlist,
  coreNuvioMapCollections,
  coreNuvioProgressMetaNeeds,
  coreNuvioSortAddonsByPriority,
  coreInvoke,
  storageRead,
  storageWrite,
} from './engine';
import type { UserProfile } from './types';
import { loadProfiles, saveProfile, saveProfiles } from './profiles';
import { fetchPlannedResources } from './fetchPlanning';
import { loadProviderLibraries, saveProviderLibrary } from './providerLibraries';
import type { ImportCategory } from './importCategories';

export type NuvioImportStep = 'addons' | 'library' | 'progress' | 'history' | 'collections';

export interface NuvioImportReport {
  errors: Partial<Record<NuvioImportStep, string>>;
  changed: boolean;
  counts?: { watchlist: number; continueWatching: number; watched: number; collections: number; addons: number };
}

const activeImports = new Map<string, Promise<NuvioImportReport>>();

export interface NuvioSyncMeta {
  lastSyncAt: number;
  continueWatchingCount: number;
  watchlistCount: number;
  watchedCount?: number;
  error?: string;
}

export async function recordNuvioSyncMeta(report: NuvioImportReport | { errors: Partial<Record<NuvioImportStep, string>> }): Promise<void> {
  const failures = Object.entries(report.errors);
  const error = failures.length > 0 ? failures.map(([step, msg]) => `${step}: ${msg}`).join('; ') : undefined;
  const counts = 'counts' in report ? report.counts : undefined;
  const previous = counts ? undefined : await storageRead<NuvioSyncMeta>('nuvio_sync_meta');
  const meta: NuvioSyncMeta = {
    lastSyncAt: Date.now(),
    continueWatchingCount: counts?.continueWatching ?? previous?.continueWatchingCount ?? 0,
    watchlistCount: counts?.watchlist ?? previous?.watchlistCount ?? 0,
    watchedCount: counts?.watched ?? previous?.watchedCount ?? 0,
    error,
  };
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

function nuvioSyncScope(profile: UserProfile): string {
  const identity = profile.nuvioUserId?.trim() || profile.nuvioEmail?.trim() || profile.id;
  return `${identity}#${profile.nuvioProfileIndex ?? 1}`.replace(/[^a-zA-Z0-9_-]/g, '_');
}

function deltaCursorKey(profile: UserProfile, resource: 'library' | 'progress' | 'history'): string {
  return `nuvio_${resource}_event_cursor_${nuvioSyncScope(profile)}`;
}

function deltaCacheKey(profile: UserProfile, resource: 'library' | 'progress' | 'history'): string {
  return `nuvio_${resource}_remote_cache_${nuvioSyncScope(profile)}`;
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

async function pullAllLibraryDelta(token: string, profileId: number, cursor: number): Promise<NuvioLibraryDeltaEvent[]> {
  const events: NuvioLibraryDeltaEvent[] = [];
  let nextCursor = cursor;
  for (;;) {
    const batch = await nuvioPullLibraryDelta(token, profileId, nextCursor, 1_000);
    if (batch.length === 0) return events;
    events.push(...batch);
    const batchCursor = Math.max(nextCursor, ...batch.map((event) => event.event_id));
    if (batchCursor === nextCursor || batch.length < 1_000) return events;
    nextCursor = batchCursor;
  }
}

async function pullAllNuvioLibrary(
  token: string,
  profileId: number,
  onItemProgress?: (index: number, total: number | null, title: string) => void,
): Promise<Awaited<ReturnType<typeof nuvioPullLibrary>>> {
  const items: Awaited<ReturnType<typeof nuvioPullLibrary>> = [];
  const limit = 500;
  for (let offset = 0; ; offset += limit) {
    const page = await nuvioPullLibrary(token, profileId, limit, offset);
    for (const item of page) {
      items.push(item);
      onItemProgress?.(items.length, null, item.name);
    }
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

async function fetchAddonManifests(
  addons: NuvioAddon[],
  onItemProgress?: (index: number, total: number | null, title: string) => void,
): Promise<{
  addonList: NuvioAddon[];
  manifestIdByUrl: Map<string, string>;
  descriptors: Array<Record<string, unknown>>;
}> {
  const sorted = (await coreNuvioSortAddonsByPriority(addons)) ?? addons;
  const enabled = sorted.filter((a) => a.enabled);
  const manifestIdByUrl = new Map<string, string>();
  let completed = 0;
  const manifests = await Promise.allSettled(
    enabled.map(async (a) => {
      try {
        const res = await platformFetch(a.url);
        if (!res.ok) return null;
        const manifest = await (res.json() as Promise<Record<string, unknown>>);
        onItemProgress?.(++completed, enabled.length, typeof manifest.name === 'string' ? manifest.name : a.name ?? a.url);
        return manifest;
      } catch {
        return null;
      }
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
  onItemProgress?: (index: number, total: number | null, title: string) => void,
): Promise<Record<string, unknown>> {
  const metas: Record<string, unknown> = {};
  if (needs.length === 0 || addonDescriptors.length === 0) return metas;
  let completed = 0;
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
        if (meta?.name) {
          metas[need.contentId] = meta;
          onItemProgress?.(++completed, needs.length, meta.name);
        }
      } catch {}
    })
  );
  return metas;
}

async function importNuvioProfileDataInner(
  profile: UserProfile,
  onStep?: (step: NuvioImportStep, ok: boolean, error?: string) => void,
  onItemProgress?: (index: number, total: number | null, title: string) => void,
  categories?: ImportCategory[],
  dryRun?: boolean,
): Promise<NuvioImportReport> {
  const wants = (category: ImportCategory) => !categories || categories.includes(category);
  const freshProfile = await freshNuvioProfile(profile).catch(() => profile);
  const token = freshProfile.nuvioAccessToken;
  const profileIdx = freshProfile.nuvioProfileIndex ?? 1;
  if (!token) return { errors: { library: 'Missing Nuvio token' }, changed: false };

  const suffix = profileStorageSuffix(profile);
  const profileKey = `library_${suffix}`;
  void platformInvoke('debug_log', { msg: `nuvio-import-debug: importNuvioProfileData start profile.id=${profile.id} profileIdx=${profileIdx} profileKey=${profileKey}` });
  const progressBefore: Record<string, unknown> = {};
  const watchedBefore: Record<string, boolean> = {};
  let mappedWatchlist: unknown[] = [];
  const errors: Partial<Record<NuvioImportStep, string>> = {};

  let addonDescriptors: Array<Record<string, unknown>> = [];
  let addonCount = 0;
  if (wants('addons')) {
    try {
      const addons = await nuvioPullAddons(token, profileIdx);
      const fetched = await fetchAddonManifests(addons, onItemProgress);
      addonDescriptors = fetched.descriptors;
      addonCount = fetched.descriptors.length;
      if (!dryRun) await storageWrite(`addons_${suffix}`, fetched.descriptors);
      onStep?.('addons', true);
    } catch (err) {
      errors.addons = err instanceof Error ? err.message : String(err);
      onStep?.('addons', false, errors.addons);
    }
  } else {
    addonDescriptors = (await storageRead<Array<Record<string, unknown>>>(`addons_${suffix}`)) ?? [];
  }

  let library: NuvioLibraryItem[] = [];
  let libraryCursor: number | null = null;
  let watchlistCount = 0;
  if (wants('watchlist')) {
    try {
      const cursor = await storageRead<number>(deltaCursorKey(profile, 'library'));
      const cached = await storageRead<NuvioLibraryItem[]>(deltaCacheKey(profile, 'library'));
      const state = { initialized: typeof cursor === 'number' && Array.isArray(cached), cursor: cursor ?? 0, items: cached ?? [] };
      const request = await coreInvoke<{ mode: 'delta' | 'bootstrap'; cursor: number }>('nuvioDeltaSyncRequestPlan', JSON.stringify({ state }));
      let snapshot: NuvioLibraryItem[] = [];
      let snapshotCursor: number | undefined;
      let events: NuvioLibraryDeltaEvent[] = [];
      if (request?.mode === 'delta') {
        events = await pullAllLibraryDelta(token, profileIdx, request.cursor);
      } else {
        snapshotCursor = await nuvioGetLibraryDeltaCursor(token, profileIdx);
        snapshot = await pullAllNuvioLibrary(token, profileIdx, onItemProgress);
        events = await pullAllLibraryDelta(token, profileIdx, snapshotCursor);
      }
      const applied = await coreInvoke<{ cursor: number; items: NuvioLibraryItem[] }>('nuvioApplyDeltaSync', JSON.stringify({ resource: 'library', state, snapshot, snapshotCursor, events }));
      library = applied?.items ?? await pullAllNuvioLibrary(token, profileIdx, onItemProgress);
      libraryCursor = applied?.cursor ?? await nuvioGetLibraryDeltaCursor(token, profileIdx);
      mappedWatchlist = ((await coreNuvioLibraryToWatchlist(library)) ?? []) as unknown[];
      watchlistCount = mappedWatchlist.length;
      onStep?.('library', true);
    } catch (err) {
      errors.library = err instanceof Error ? err.message : String(err);
      onStep?.('library', false, errors.library);
    }
  }

  let watchProgress: NuvioWatchProgress[] | null = null;
  let progressCursor: number | null = null;
  if (wants('continueWatching')) {
    try {
      const cursor = await storageRead<number>(deltaCursorKey(profile, 'progress'));
      const cached = await storageRead<NuvioWatchProgress[]>(deltaCacheKey(profile, 'progress'));
      const state = {
        initialized: typeof cursor === 'number' && Array.isArray(cached),
        cursor: cursor ?? 0,
        items: cached ?? [],
      };
      const request = await coreInvoke<{ mode: 'delta' | 'bootstrap'; cursor: number }>(
        'nuvioProgressSyncRequestPlan',
        JSON.stringify({ state }),
      );
      let snapshot: NuvioWatchProgress[] = [];
      let snapshotCursor: number | undefined;
      let events: NuvioWatchProgressDeltaEvent[] = [];
      if (request?.mode === 'delta') {
        events = await pullAllProgressDelta(token, profileIdx, request.cursor);
      } else {
        snapshotCursor = await nuvioGetWatchProgressDeltaCursor(token, profileIdx);
        snapshot = await nuvioPullWatchProgress(token, profileIdx, 1_000);
        events = await pullAllProgressDelta(token, profileIdx, snapshotCursor);
      }
      const applied = await coreInvoke<{ cursor: number; items: NuvioWatchProgress[] }>(
        'nuvioApplyProgressSync',
        JSON.stringify({ state, snapshot, snapshotCursor, events }),
      );
      if (applied) {
        watchProgress = applied.items;
        progressCursor = applied.cursor;
      }
      if (!applied) {
        watchProgress = await nuvioPullWatchProgress(token, profileIdx, 1_000);
        progressCursor = await nuvioGetWatchProgressDeltaCursor(token, profileIdx);
      }
    } catch (err) {
      errors.progress = err instanceof Error ? err.message : String(err);
      onStep?.('progress', false, errors.progress);
    }
  }

  let addonMetas: Record<string, unknown> = {};
  if (watchProgress) {
    const needs = (await coreNuvioProgressMetaNeeds(watchProgress, library)) ?? [];
    addonMetas = await fetchAddonMetas(needs, addonDescriptors, onItemProgress);
  }

  let watchHistory: NuvioWatchedItem[] | null = null;
  let historyCursor: number | null = null;
  if (wants('watched')) {
    try {
      const cursor = await storageRead<number>(deltaCursorKey(profile, 'history'));
      const cached = await storageRead<NuvioWatchedItem[]>(deltaCacheKey(profile, 'history'));
      const state = {
        initialized: typeof cursor === 'number' && Array.isArray(cached),
        cursor: cursor ?? 0,
        items: cached ?? [],
      };
      const request = await coreInvoke<{ mode: 'delta' | 'bootstrap'; cursor: number }>(
        'nuvioDeltaSyncRequestPlan',
        JSON.stringify({ state }),
      );
      let snapshot: NuvioWatchedItem[] = [];
      let snapshotCursor: number | undefined;
      let events: NuvioWatchedItemDeltaEvent[] = [];
      if (request?.mode === 'delta') {
        events = await pullAllHistoryDelta(token, profileIdx, request.cursor);
      } else {
        snapshotCursor = await nuvioGetWatchHistoryDeltaCursor(token, profileIdx);
        snapshot = await pullAllNuvioWatchHistory(token, profileIdx);
        events = await pullAllHistoryDelta(token, profileIdx, snapshotCursor);
      }
      const applied = await coreInvoke<{ cursor: number; items: NuvioWatchedItem[] }>(
        'nuvioApplyDeltaSync',
        JSON.stringify({ resource: 'history', state, snapshot, snapshotCursor, events }),
      );
      if (applied) {
        watchHistory = applied.items;
        historyCursor = applied.cursor;
      } else {
        watchHistory = await pullAllNuvioWatchHistory(token, profileIdx);
        historyCursor = await nuvioGetWatchHistoryDeltaCursor(token, profileIdx);
      }
    } catch (err) {
      errors.history = err instanceof Error ? err.message : String(err);
      onStep?.('history', false, errors.history);
    }
  }

  const plan = await coreNuvioImportMergePlan({
    progress: progressBefore,
    watched: watchedBefore,
    library,
    addonMetas,
    watchProgress,
    watchHistory,
    categories,
    dryRun,
  });
  let nuvioContinueWatching: Record<string, unknown>[] = [];
  if (!dryRun) {
    if (plan?.progress != null) {
      nuvioContinueWatching = (await buildContinueWatching(plan.progress)) as Record<string, unknown>[];
    }
    if (wants('watchlist') || wants('continueWatching')) {
      const existing = (await loadProviderLibraries(profileKey)).nuvio;
      await saveProviderLibrary('nuvio', {
        watchlist: wants('watchlist') ? mappedWatchlist as Record<string, unknown>[] : existing?.watchlist ?? [],
        watching: wants('continueWatching') ? nuvioContinueWatching : existing?.watching ?? [],
        completed: existing?.completed ?? [],
        dropped: existing?.dropped ?? [],
        favorites: existing?.favorites ?? [],
      }, profileKey);
    }
    if (watchProgress) {
      await storageWrite(deltaCacheKey(profile, 'progress'), watchProgress);
      if (progressCursor != null) await storageWrite(deltaCursorKey(profile, 'progress'), progressCursor);
    }
    if (wants('watchlist') && libraryCursor != null) {
      await storageWrite(deltaCacheKey(profile, 'library'), library);
      await storageWrite(deltaCursorKey(profile, 'library'), libraryCursor);
    }
    if (watchHistory) {
      await storageWrite(deltaCacheKey(profile, 'history'), watchHistory);
      if (historyCursor != null) await storageWrite(deltaCursorKey(profile, 'history'), historyCursor);
    }
  }
  if (watchProgress) onStep?.('progress', true);
  if (watchHistory) onStep?.('history', true);

  let collectionsCount = 0;
  if (wants('collections')) {
    try {
      const collections = await nuvioPullCollections(token, profileIdx);
      if (collections.length > 0) {
        const raw = (collections[0]?.collections_json ?? []) as unknown[];
        const mapped = (await coreNuvioMapCollections(raw)) ?? [];
        collectionsCount = mapped.length;
        if (!dryRun) {
          const profiles = (await storageRead<UserProfile[]>('profiles')) ?? [];
          await storageWrite('profiles', profiles.map((p) => p.id === profile.id ? { ...p, libraryCollections: mapped as UserProfile['libraryCollections'] } : p));
        }
      }
      onStep?.('collections', true);
    } catch (err) {
      errors.collections = err instanceof Error ? err.message : String(err);
      onStep?.('collections', false, errors.collections);
    }
  }

  const counts = {
    watchlist: watchlistCount,
    continueWatching: plan?.progressCount ?? 0,
    watched: plan?.watchedCount ?? 0,
    collections: collectionsCount,
    addons: addonCount,
  };

  const changed = wants('watchlist') || wants('continueWatching')
    || Boolean(plan?.watched && Object.keys(plan.watched).length > 0);

  return { errors, changed, counts };
}

export function importNuvioProfileData(
  profile: UserProfile,
  onStep?: (step: NuvioImportStep, ok: boolean, error?: string) => void,
  onItemProgress?: (index: number, total: number | null, title: string) => void,
  categories?: ImportCategory[],
  dryRun?: boolean,
): Promise<NuvioImportReport> {
  const key = `${profile.id}:${profile.nuvioProfileIndex ?? 1}`;
  const current = activeImports.get(key);
  if (current) return current;
  const task = importNuvioProfileDataInner(profile, onStep, onItemProgress, categories, dryRun);
  activeImports.set(key, task);
  void task.finally(() => activeImports.delete(key));
  return task;
}
