import { coreInvoke, storageDelete, storageRead, storageWrite } from './engine';
import { loadActiveProfile, loadAddons, loadLibrary, loadPrefs, saveAddons, saveLibrary, savePrefs } from './libraryOps';
import { pluginRepositoryUrlsKey, pluginScraperEnabledKey } from './pluginsStorage';
import { loadProfiles, saveProfiles } from './profiles';
import { DEFAULT_APP_PREFS } from './appPrefs';
import {
  fluxaCreateProfile,
  fluxaProfiles,
  fluxaPull,
  fluxaPush,
  fluxaRefresh,
  fluxaSignOut,
  fluxaSnapshot,
  fluxaUpdateProfile,
  type FluxaChange,
  type FluxaDocument,
  type FluxaSession,
} from './fluxaSyncApi';
import type { AddonDescriptor, UserCollection, UserProfile } from './types';

const SESSION_KEY = 'fluxa_sync_session';
const STATUS_LISTS = ['watchlist', 'completed', 'dropped'] as const;

type KnownMap = Record<string, { revision: number; hash: string }>;

interface SyncState {
  cursor: number;
  known: KnownMap;
}

interface LocalState {
  library: Record<string, unknown[]>;
  progress: Record<string, unknown>;
  watched: Record<string, boolean>;
  lastWatched: Record<string, unknown>;
  collections: UserCollection[];
  addons: AddonDescriptor[];
  plugins: { repositoryUrls: string[]; scraperOverrides: Record<string, boolean> };
  settings: Record<string, unknown>;
  profile: Record<string, unknown>;
}

function stateKey(remoteProfileId: string): string {
  return `fluxa_sync_state_${remoteProfileId}`;
}

export async function fluxaSession(): Promise<FluxaSession | null> {
  return storageRead<FluxaSession>(SESSION_KEY);
}

export async function setFluxaSession(session: FluxaSession): Promise<void> {
  await storageWrite(SESSION_KEY, session);
}

export async function signOutFluxa(): Promise<void> {
  const session = await fluxaSession();
  if (session) {
    await fluxaSignOut(session.instanceUrl, session.accessToken, session.refreshToken).catch(() => undefined);
  }
  await storageDelete(SESSION_KEY);
}

async function authorized(): Promise<FluxaSession | null> {
  const session = await fluxaSession();
  if (!session) return null;
  if (session.expiresAt - Date.now() > 60_000) return session;
  const refreshed = await fluxaRefresh(session.instanceUrl, session.refreshToken);
  await setFluxaSession(refreshed);
  return refreshed;
}

function profileIdentity(profile: UserProfile): Record<string, unknown> {
  return {
    name: profile.name ?? null,
    avatarUrl: profile.avatarUrl ?? null,
    color: profile.color ?? null,
    pinHash: profile.pinHash ?? null,
    usesPrimaryAddons: profile.usesPrimaryAddons ?? null,
    usesPrimaryPlugins: profile.usesPrimaryPlugins ?? null,
  };
}

async function readLocalState(profile: UserProfile): Promise<LocalState> {
  const [library, addons, settings, repositoryUrls, scraperOverrides] = await Promise.all([
    loadLibrary(),
    loadAddons(),
    loadPrefs(),
    storageRead<string[]>(await pluginRepositoryUrlsKey()),
    storageRead<Record<string, boolean>>(await pluginScraperEnabledKey()),
  ]);
  const statuses: Record<string, unknown[]> = {};
  for (const list of STATUS_LISTS) {
    const items = library[list];
    if (Array.isArray(items)) statuses[list] = items;
  }
  return {
    library: statuses,
    progress: (library.progress as Record<string, unknown>) ?? {},
    watched: (library.watched as Record<string, boolean>) ?? {},
    lastWatched: (library.lastWatchedEpisodes as Record<string, unknown>) ?? {},
    collections: profile.libraryCollections ?? [],
    addons,
    plugins: { repositoryUrls: repositoryUrls ?? [], scraperOverrides: scraperOverrides ?? {} },
    settings,
    profile: profileIdentity(profile),
  };
}

async function writeLocalState(profile: UserProfile, local: LocalState): Promise<void> {
  const current = await loadLibrary();
  await saveLibrary({
    ...current,
    ...local.library,
    progress: local.progress,
    watched: local.watched,
    lastWatchedEpisodes: local.lastWatched,
  });
  if (Array.isArray(local.addons)) await saveAddons(local.addons);
  if (local.settings) await savePrefs(local.settings);
  if (local.plugins) {
    await storageWrite(await pluginRepositoryUrlsKey(), local.plugins.repositoryUrls ?? []);
    await storageWrite(await pluginScraperEnabledKey(), local.plugins.scraperOverrides ?? {});
  }
  const identity = (local.profile ?? {}) as Partial<UserProfile>;
  const profiles = await loadProfiles();
  await saveProfiles(
    profiles.map((entry) =>
      entry.id === profile.id
        ? { ...entry, ...identity, id: entry.id, libraryCollections: local.collections }
        : entry,
    ),
  );
}

export async function syncFluxaProfiles(): Promise<UserProfile[]> {
  const session = await authorized();
  if (!session) return loadProfiles();
  const [local, remote] = await Promise.all([
    loadProfiles(),
    fluxaProfiles(session.instanceUrl, session.accessToken),
  ]);
  const plan = await coreInvoke<{
    creates: Array<{ localId: string; body: Record<string, unknown> }>;
    updates: Array<{ id: string; body: Record<string, unknown> }>;
    profiles: UserProfile[];
  }>('fluxaSyncProfilePlan', JSON.stringify({ local, remote }));
  if (!plan) return local;

  const linked = [...plan.profiles];
  for (const create of plan.creates) {
    const created = await fluxaCreateProfile(session.instanceUrl, session.accessToken, create.body as never);
    const index = linked.findIndex((entry) => entry.id === create.localId);
    if (index >= 0) linked[index] = { ...linked[index], fluxaProfileId: created.id };
  }
  for (const update of plan.updates) {
    await fluxaUpdateProfile(session.instanceUrl, session.accessToken, update.id, update.body as never);
  }
  await saveProfiles(linked);
  return linked;
}

export async function syncFluxaProfileData(profile: UserProfile): Promise<void> {
  const session = await authorized();
  const remoteId = profile.fluxaProfileId;
  if (!session || !remoteId) return;

  const state = (await storageRead<SyncState>(stateKey(remoteId))) ?? { cursor: 0, known: {} };
  const pulled =
    state.cursor > 0
      ? await fluxaPull(session.instanceUrl, session.accessToken, remoteId, state.cursor)
      : await fluxaSnapshot(session.instanceUrl, session.accessToken, remoteId);
  const incoming = pulled.resetRequired
    ? await fluxaSnapshot(session.instanceUrl, session.accessToken, remoteId)
    : pulled;

  let local = await readLocalState(profile);
  let known = state.known;

  if (incoming.changes.length > 0) {
    const applied = await coreInvoke<{ local: LocalState; known: KnownMap }>(
      'fluxaSyncApplyPull',
      JSON.stringify({ changes: incoming.changes, local, known, settingsDefaults: DEFAULT_APP_PREFS }),
    );
    if (applied) {
      local = applied.local;
      known = applied.known;
      await writeLocalState(profile, local);
    }
  }

  const documents = await coreInvoke<{ documents: FluxaDocument[] }>(
    'fluxaSyncDocuments',
    JSON.stringify({ ...local, settingsDefaults: DEFAULT_APP_PREFS }),
  );
  const plan = await coreInvoke<{ changes: FluxaChange[] }>(
    'fluxaSyncPushPlan',
    JSON.stringify({ documents: documents?.documents ?? [], known }),
  );

  let cursor = incoming.cursor;
  const changes = plan?.changes ?? [];
  if (changes.length > 0) {
    const result = await fluxaPush(session.instanceUrl, session.accessToken, remoteId, changes);
    cursor = result.cursor;
    const merged = await coreInvoke<{ known: KnownMap }>(
      'fluxaSyncApplyPushResult',
      JSON.stringify({ known, changes, applied: result.applied, conflicts: result.conflicts }),
    );
    if (merged) known = merged.known;
  }

  await storageWrite(stateKey(remoteId), { cursor, known });
}

export async function syncFluxaNow(): Promise<void> {
  if (!(await fluxaSession())) return;
  await syncFluxaProfiles();
  const active = await loadActiveProfile();
  if (active) await syncFluxaProfileData(active);
}
