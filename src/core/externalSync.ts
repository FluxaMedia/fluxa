import { invoke } from '@tauri-apps/api/core';
import { coreInvoke } from './engine';
import { dropTraktPlaybackProgress, refreshTraktProfile } from './traktSync';
import { syncTraktNow, pushMarkWatchedTrakt, pushWatchlistTrakt } from './traktExternalSync';
import { syncSimklNow, pushMarkWatchedSimkl, pushWatchlistSimkl } from './simklExternalSync';
import {
  pushStremioPlaybackProgress,
  pushStremioWatched,
  pushStremioWatchlist,
  syncStremioNow,
} from './stremioExternalSync';
import { pushAnimeTrackingExternal } from './animeExternalSync';
import { pushLibraryStatusAniList, pushWatchlistAniList, syncAniListNow } from './anilistExternalSync';
import {
  nuvioDeleteWatchHistory,
  nuvioDeleteWatchProgress,
  nuvioPullLibrary,
  nuvioPushLibrary,
  nuvioPushWatchHistory,
  nuvioPushWatchProgress,
  nuvioRefreshToken,
} from './nuvioApi';
import { saveProfile } from './profiles';
import { loadLibrary, loadPrefs, saveLibrary, buildContinueWatching, persistProgressMerge } from './libraryOps';
import type { UserProfile } from './types';

export { enqueueTraktScrobble } from './traktSync';
export { replaceExternalContinueWatching } from './externalSyncUtils';

export type WatchedEpisodeInfo = {
  contentId: string;
  contentType: string;
  videoId?: string;
  season?: number;
  episode?: number;
  title?: string;
};

export type WatchProgressInfo = {
  contentId: string;
  contentType: string;
  videoId: string;
  positionSeconds: number;
  durationSeconds: number;
  lastWatched: number;
  season?: number;
  episode?: number;
};

async function integrationSettings(): Promise<Record<string, unknown>> {
  const prefs = await loadPrefs();
  return (await coreInvoke<Record<string, unknown>>('integrationSettingsPlan', JSON.stringify({
    settings: {
      librarySource: prefs.integrationLibrarySource,
      watchProgressSource: prefs.watchProgressSource,
      continueWatchingDays: Number(prefs.continueWatchingDays) || 0,
      similarTitlesSource: prefs.similarTitlesSource,
      traktCommentsEnabled: prefs.traktCommentsEnabled,
    },
  }))) ?? {};
}

export async function promoteExternalProgress(
  items: Record<string, unknown>[],
  source: string,
  profile: UserProfile | null,
): Promise<void> {
  if (!profile) return;
  const lib = await loadLibrary();
  const progress = (lib.progress as Record<string, Record<string, unknown>> | undefined) ?? {};
  const progressBefore = { ...progress };
  const plan = await coreInvoke<{
    progress: Record<string, Record<string, unknown>>;
    promotions: Array<{
      item: Record<string, unknown>;
      externalProgress: WatchProgressInfo;
      meta: import('./types').Meta;
      episode: import('./types').Video | null;
      scrobbleTrakt: boolean;
      scrobbleSimkl: boolean;
    }>;
  }>('promoteExternalProgressPlan', JSON.stringify({ progress, items, source }));
  if (!plan) return;
  for (const promotion of plan.promotions) {
    await pushPlaybackProgressExternal(promotion.externalProgress, promotion.item, profile);
    if (promotion.scrobbleTrakt && profile.traktAccessToken) {
      const clientId = await getOAuthClientId('trakt');
      await pushMarkWatchedTrakt([promotion.externalProgress.videoId], true, profile.traktAccessToken, clientId, promotion.externalProgress.lastWatched).catch(() => undefined);
    }
    if (promotion.scrobbleSimkl && profile.simklAccessToken) {
      const clientId = await getOAuthClientId('simkl');
      await pushMarkWatchedSimkl([promotion.externalProgress.videoId], true, promotion.meta as unknown as Record<string, unknown>, profile.simklAccessToken, clientId, promotion.externalProgress.lastWatched).catch(() => undefined);
    }
  }
  if (plan.promotions.length > 0) {
    lib.progress = plan.progress;
    lib.continueWatching = await buildContinueWatching(plan.progress);
    await persistProgressMerge(progressBefore, plan.progress);
    await saveLibrary(lib);
  }
}

const nuvioLibraryMutationQueues = new Map<string, Promise<void>>();

function queueNuvioLibraryMutation(key: string, mutation: () => Promise<void>): Promise<void> {
  const previous = nuvioLibraryMutationQueues.get(key) ?? Promise.resolve();
  const next = previous.catch(() => undefined).then(mutation);
  nuvioLibraryMutationQueues.set(key, next);
  void next.finally(() => {
    if (nuvioLibraryMutationQueues.get(key) === next) nuvioLibraryMutationQueues.delete(key);
  });
  return next;
}

async function getOAuthClientId(service: string): Promise<string> {
  try {
    return await invoke<string>('get_oauth_client_id', { service });
  } catch {
    return '';
  }
}

function isAuthFailure(err: unknown): boolean {
  const message = err instanceof Error ? err.message : String(err);
  return /\b(401|403)\b|JWT|token|expired/i.test(message);
}

async function refreshNuvioProfile(profile: UserProfile): Promise<UserProfile> {
  if (!profile.nuvioRefreshToken) return profile;
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

async function validNuvioProfile(profile: UserProfile): Promise<UserProfile> {
  if (!profile.nuvioAccessToken || !profile.nuvioRefreshToken) return profile;
  const expiresAt = profile.nuvioTokenExpiresAt ?? 0;
  if (expiresAt > Math.floor(Date.now() / 1000) + 60) return profile;
  return refreshNuvioProfile(profile);
}

export async function pushMarkWatchedExternal(
  videoIds: string[],
  watched: boolean,
  meta: Record<string, unknown> | undefined,
  profile: UserProfile | null,
  episodeInfo?: WatchedEpisodeInfo | WatchedEpisodeInfo[],
  progressInfo?: WatchProgressInfo,
): Promise<void> {
  if (!profile) return;
  const activeProfile = await refreshTraktProfile(profile).catch(() => profile);
  const settings = await integrationSettings();
  const tasks: Promise<void>[] = [];
  const plan = await coreInvoke<{
    trakt: boolean; simkl: boolean; anilist: boolean; stremio: boolean; nuvio: boolean;
    animeEpisode?: WatchedEpisodeInfo; animeProgressEpisode?: number;
    episodes: WatchedEpisodeInfo[];
    watchedKeys: Array<{ content_id: string; season?: number; episode?: number }>;
    historyItems: Array<{ content_id: string; content_type: string; title?: string; season?: number; episode?: number; watched_at: number }>;
    progressEntry?: { content_id: string; content_type: string; video_id: string; position: number; duration: number; last_watched: number; season?: number; episode?: number };
  }>('externalProviderActionPlan', JSON.stringify({ kind: 'markWatched', profile: activeProfile, videoIds, watched, meta, episodeInfo, progressInfo, integrationSettings: settings, nowMs: Date.now() }));
  if (!plan) return;

  if (plan.trakt) {
    tasks.push((async () => {
      const clientId = await getOAuthClientId('trakt');
      await pushMarkWatchedTrakt(videoIds, watched, activeProfile.traktAccessToken!, clientId);
    })().catch(() => undefined));
  }

  if (plan.simkl) {
    tasks.push((async () => {
      const clientId = await getOAuthClientId('simkl');
      await pushMarkWatchedSimkl(videoIds, watched, meta, activeProfile.simklAccessToken!, clientId);
    })().catch(() => undefined));
  }

  if (plan.anilist) {
    tasks.push(pushAnimeTrackingExternal({
      meta,
      episode: plan.animeEpisode,
      progressEpisode: plan.animeProgressEpisode,
      watched,
    }, activeProfile).catch(() => undefined));
  }

  if (plan.stremio) {
    tasks.push(pushStremioWatched(meta, watched, plan.episodes, activeProfile).catch(() => undefined));
  }
  if (plan.nuvio) {
    tasks.push((async () => {
      let nuvioProfile = await validNuvioProfile(activeProfile);
      const push = async () => {
        const token = nuvioProfile.nuvioAccessToken!;
        const profileIdx = nuvioProfile.nuvioProfileIndex ?? 1;
        if (!watched) {
          if (plan.watchedKeys.length > 0) await nuvioDeleteWatchHistory(token, profileIdx, plan.watchedKeys);
          return;
        }
        if (plan.episodes.length > 0) {
          await Promise.all(plan.episodes.map((info) =>
            nuvioDeleteWatchProgress(token, profileIdx, info.contentId, info.season, info.episode).catch(() => undefined),
          ));
          await nuvioPushWatchHistory(token, profileIdx, plan.historyItems);
        }
        if (plan.progressEntry) {
          await nuvioPushWatchProgress(token, profileIdx, [plan.progressEntry]);
        }
      };
      try {
        await push();
      } catch (err) {
        if (!isAuthFailure(err)) throw err;
        nuvioProfile = await refreshNuvioProfile(nuvioProfile);
        await push();
      }
    })().catch(() => undefined));
  }

  await Promise.all(tasks);
}

export async function pushWatchlistExternal(
  item: Record<string, unknown>,
  command: 'add' | 'remove',
  profile: UserProfile | null,
): Promise<void> {
  if (!profile) return;
  const activeProfile = await refreshTraktProfile(profile).catch(() => profile);
  const id = String(item.id ?? '');
  const contentType = String(item.type ?? 'movie');
  const tasks: Promise<void>[] = [];
  const plan = await coreInvoke<{ trakt: boolean; simkl: boolean; anilist: boolean; stremio: boolean; nuvio: boolean }>('externalProviderActionPlan', JSON.stringify({ kind: 'watchlist', profile: activeProfile, item, command, nowMs: Date.now() }));
  if (!plan) return;

  if (plan.trakt) {
    tasks.push((async () => {
      const clientId = await getOAuthClientId('trakt');
      await pushWatchlistTrakt(id, contentType, command, activeProfile.traktAccessToken!, clientId);
    })().catch(() => undefined));
  }

  if (plan.simkl) {
    tasks.push((async () => {
      const clientId = await getOAuthClientId('simkl');
      await pushWatchlistSimkl(id, contentType, command, activeProfile.simklAccessToken!, clientId);
    })().catch(() => undefined));
  }

  if (plan.anilist) {
    tasks.push(pushWatchlistAniList(id, command, activeProfile.anilistAccessToken!).catch(() => undefined));
  }

  if (plan.stremio) {
    tasks.push(pushStremioWatchlist(item, command, activeProfile).catch(() => undefined));
  }

  if (plan.nuvio) {
    const queueKey = `${activeProfile.nuvioUserId ?? activeProfile.id}:${activeProfile.nuvioProfileIndex ?? 1}`;
    tasks.push(queueNuvioLibraryMutation(queueKey, async () => {
      let nuvioProfile = await validNuvioProfile(activeProfile);
      const token = nuvioProfile.nuvioAccessToken;
      if (!token) return;
      const profileIdx = nuvioProfile.nuvioProfileIndex ?? 1;
      const remote = await nuvioPullLibrary(token, profileIdx);
      const updated = await coreInvoke<typeof remote>('nuvioLibraryMutationPlan', JSON.stringify({ remote, item, command, nowMs: Date.now() }));
      if (updated) await nuvioPushLibrary(token, profileIdx, updated);
    }).catch(() => undefined));
  }

  await Promise.all(tasks);
}

export async function pushPlaybackProgressExternal(
  progress: WatchProgressInfo,
  meta: Record<string, unknown>,
  profile: UserProfile | null,
): Promise<void> {
  if (!profile) return;
  const plan = await coreInvoke<{
    stremio: boolean; nuvio: boolean;
    progressEntry?: { content_id: string; content_type: string; video_id: string; position: number; duration: number; last_watched: number; season?: number; episode?: number };
  }>('externalProviderActionPlan', JSON.stringify({ kind: 'progress', profile, progress, nowMs: Date.now() }));
  if (!plan) return;
  const tasks: Promise<void>[] = [];
  if (plan.stremio) {
    tasks.push(pushStremioPlaybackProgress(meta, progress, profile).catch(() => undefined));
  }
  if (plan.nuvio && plan.progressEntry) {
    tasks.push((async () => {
      const fresh = await validNuvioProfile(profile);
      if (!fresh.nuvioAccessToken) return;
      await nuvioPushWatchProgress(fresh.nuvioAccessToken, fresh.nuvioProfileIndex ?? 1, [plan.progressEntry!]);
    })().catch(() => undefined));
  }
  await Promise.all(tasks);
}

export async function pushLibraryStatusExternal(
  item: Record<string, unknown>,
  list: string,
  command: 'add' | 'remove',
  profile: UserProfile | null,
): Promise<void> {
  if (!profile) return;
  const plan = await coreInvoke<{ anilist: boolean }>('externalProviderActionPlan', JSON.stringify({ kind: 'status', profile, item, list, command, nowMs: Date.now() }));
  if (!plan?.anilist) return;
  const id = String(item.id ?? '');
  await pushLibraryStatusAniList(id, list, command, profile.anilistAccessToken!).catch(() => undefined);
}

export async function dropExternalPlaybackProgress(item: Record<string, unknown>): Promise<void> {
  const id = String(item.id ?? '');
  if (!id) return;
  const plan = await coreInvoke<{ dropTrakt: boolean }>('externalProviderActionPlan', JSON.stringify({ kind: 'dropProgress', profile: {}, item, nowMs: Date.now() }));
  if (plan?.dropTrakt) {
    await dropTraktPlaybackProgress(id);
  }
}

export async function syncExternalIntegrationNow(payload: Record<string, unknown>): Promise<unknown> {
  const plan = await coreInvoke<{ provider: string; supported: boolean; error?: string }>('externalProviderActionPlan', JSON.stringify({ kind: 'sync', provider: payload.provider }));
  const provider = plan?.provider ?? '';
  if (!plan?.supported) return { synced: false, error: plan?.error };
  if (provider === 'anilist') return syncAniListNow(payload);
  if (provider === 'simkl') return syncSimklNow(payload);
  if (provider === 'trakt') return syncTraktNow(payload);
  if (provider === 'stremio') return syncStremioNow(payload);
  if (provider === 'nuvio') return syncNuvioNow(payload);
  return { synced: false };
}

async function syncNuvioNow(payload: Record<string, unknown>): Promise<unknown> {
  const profile = payload.profile as UserProfile | undefined;
  if (!profile?.nuvioAccessToken) return { synced: false, error: 'Nuvio is not connected' };
  const { importNuvioProfileData } = await import('./nuvioSync');
  const report = await importNuvioProfileData(profile);
  const failures = Object.entries(report.errors);
  if (failures.length > 0) {
    return { synced: false, error: failures.map(([step, msg]) => `${step}: ${msg}`).join('; ') };
  }
  return { synced: true, provider: 'nuvio' };
}
