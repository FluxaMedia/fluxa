import { coreInvoke } from './engine';
import { dropTraktPlaybackProgress, getOAuthClientId, refreshTraktProfile } from './traktSync';
import { syncTraktNow, pushMarkWatchedTrakt } from './traktExternalSync';
import { syncSimklNow, pushMarkWatchedSimkl, dropSimklPlaybackProgress } from './simklExternalSync';
import { pushStremioPlaybackProgress, syncStremioNow } from './stremioExternalSync';
import { pushLibraryStatusAniList, syncAniListNow } from './anilistExternalSync';
import { nuvioPushWatchProgress } from './nuvioApi';
import { simklScrobble, traktScrobble } from './scrobble';
import { loadLibrary, loadPrefs, saveLibrary, buildContinueWatching, persistProgressMerge, profileStorageKey } from './libraryOps';
import { providerAdapters } from './providers';
import type { PushWatchedArgs, WatchedEpisodeInfo, WatchProgressInfo } from './providers';
import type { UserProfile } from './types';
import { platformInvoke as invoke } from '../platform/invoke';

function debugLog(msg: string) {
  void invoke('debug_log', { msg }).catch(() => {});
}

export { enqueueTraktScrobble } from './traktSync';
export { replaceExternalContinueWatching } from './externalSyncUtils';
export type { WatchedEpisodeInfo, WatchProgressInfo } from './providers';

async function validNuvioProfile(profile: UserProfile): Promise<UserProfile> {
  const expiresAt = profile.nuvioTokenExpiresAt ?? 0;
  if (!profile.nuvioAccessToken || !profile.nuvioRefreshToken || expiresAt > Math.floor(Date.now() / 1000) + 60) return profile;
  const { nuvioRefreshToken } = await import('./nuvioApi');
  const session = await nuvioRefreshToken(profile.nuvioRefreshToken);
  const { saveProfile } = await import('./profiles');
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
  const profileKey = profileStorageKey(profile);
  const lib = await loadLibrary(profileKey);
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
    await persistProgressMerge(progressBefore, plan.progress, profileKey);
    await saveLibrary(lib, profileKey);
  }
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
  const plan = await coreInvoke<{ trakt: boolean; simkl: boolean; anilist: boolean; stremio: boolean; nuvio: boolean } & Omit<PushWatchedArgs, 'videoIds' | 'watched' | 'meta'>>(
    'externalProviderActionPlan',
    JSON.stringify({ kind: 'markWatched', profile: activeProfile, videoIds, watched, meta, episodeInfo, progressInfo, integrationSettings: settings, nowMs: Date.now() }),
  );
  if (!plan) return;

  const args: PushWatchedArgs = {
    videoIds, watched, meta,
    episodes: plan.episodes, animeEpisode: plan.animeEpisode, animeProgressEpisode: plan.animeProgressEpisode,
    watchedKeys: plan.watchedKeys, historyItems: plan.historyItems, progressEntry: plan.progressEntry,
  };

  await Promise.all(
    providerAdapters
      .filter((adapter) => plan[adapter.id])
      .map((adapter) => adapter.pushWatched(activeProfile, args).catch(() => undefined)),
  );
}

export async function pushWatchlistExternal(
  item: Record<string, unknown>,
  command: 'add' | 'remove',
  profile: UserProfile | null,
): Promise<void> {
  if (!profile) return;
  const activeProfile = await refreshTraktProfile(profile).catch(() => profile);
  const plan = await coreInvoke<{ trakt: boolean; simkl: boolean; anilist: boolean; stremio: boolean; nuvio: boolean }>(
    'externalProviderActionPlan',
    JSON.stringify({ kind: 'watchlist', profile: activeProfile, item, command, nowMs: Date.now() }),
  );
  if (!plan) return;

  await Promise.all(
    providerAdapters
      .filter((adapter) => plan[adapter.id])
      .map((adapter) => adapter.pushWatchlist(activeProfile, item, command).catch(() => undefined)),
  );
}

export async function pushPlaybackProgressExternal(
  progress: WatchProgressInfo,
  meta: Record<string, unknown>,
  profile: UserProfile | null,
  refreshContinueWatching = false,
): Promise<void> {
  if (!profile) return;
  const plan = await coreInvoke<{
    trakt: boolean; simkl: boolean; stremio: boolean; nuvio: boolean;
    progressEntry?: { content_id: string; content_type: string; video_id: string; position: number; duration: number; last_watched: number; season?: number; episode?: number };
  }>('externalProviderActionPlan', JSON.stringify({ kind: 'progress', profile, progress, nowMs: Date.now() }));
  debugLog(`pushPlaybackProgressExternal: videoId=${progress.videoId} position=${progress.positionSeconds} duration=${progress.durationSeconds} plan=${plan ? JSON.stringify({ trakt: plan.trakt, simkl: plan.simkl, stremio: plan.stremio, nuvio: plan.nuvio }) : 'null'} refreshContinueWatching=${refreshContinueWatching}`);
  if (!plan) return;
  const tasks: Promise<void>[] = [];
  const episode = progress.season != null && progress.episode != null
    ? { id: progress.videoId, season: progress.season, episode: progress.episode, number: progress.episode }
    : null;
  if (plan.trakt) {
    tasks.push(traktScrobble(profile, meta as unknown as import('./types').Meta, episode, progress.positionSeconds, progress.durationSeconds, 'pause').then(() => undefined).catch(() => undefined));
  }
  if (plan.simkl) {
    tasks.push(simklScrobble(profile, meta as unknown as import('./types').Meta, episode, progress.positionSeconds, progress.durationSeconds, 'pause').then(() => undefined).catch(() => undefined));
  }
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
  if (refreshContinueWatching && plan.simkl && profile.simklAccessToken) {
    debugLog('pushPlaybackProgressExternal: refreshing simkl continue-watching after push');
    const clientId = await getOAuthClientId('simkl');
    const result = await syncSimklNow({
      token: profile.simklAccessToken,
      clientId,
      profile,
      categories: ['continueWatching'],
      force: true,
    }).catch((error) => ({ synced: false, error: error instanceof Error ? error.message : String(error) }));
    debugLog(`pushPlaybackProgressExternal: simkl continue-watching refresh result=${JSON.stringify(result)}`);
  } else if (refreshContinueWatching) {
    debugLog(`pushPlaybackProgressExternal: refresh requested but skipped simkl=${plan.simkl} hasToken=${!!profile.simklAccessToken}`);
  }
}

export async function pushFavoriteExternal(
  item: Record<string, unknown>,
  command: 'add' | 'remove',
  profile: UserProfile | null,
): Promise<void> {
  if (!profile) return;
  const plan = await coreInvoke<{ trakt: boolean }>('externalProviderActionPlan', JSON.stringify({ kind: 'favorite', profile, command, nowMs: Date.now() }));
  if (!plan) return;

  await Promise.all(
    providerAdapters
      .filter((adapter) => plan[adapter.id as keyof typeof plan] && adapter.pushFavorite)
      .map((adapter) => adapter.pushFavorite!(profile, item, command).catch(() => undefined)),
  );
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
  const plan = await coreInvoke<{ dropTrakt: boolean; dropSimkl: boolean }>('externalProviderActionPlan', JSON.stringify({ kind: 'dropProgress', profile: {}, item, nowMs: Date.now() }));
  if (plan?.dropTrakt) {
    await dropTraktPlaybackProgress(id);
  }
  if (plan?.dropSimkl) {
    await dropSimklPlaybackProgress(id);
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
  const { nuvioPullAddons, nuvioPullLibrary, nuvioPullWatchProgress } = await import('./nuvioApi');
  const profileId = profile.nuvioProfileIndex ?? 1;
  const [addons, library, progress] = await Promise.all([
    nuvioPullAddons(profile.nuvioAccessToken, profileId),
    nuvioPullLibrary(profile.nuvioAccessToken, profileId),
    nuvioPullWatchProgress(profile.nuvioAccessToken, profileId),
  ]);
  return {
    synced: true,
    provider: 'nuvio',
    watchlistCount: library.length,
    continueWatchingCount: progress.length,
    watchedCount: 0,
    collectionsCount: 0,
    addonCount: addons.length,
  };
}
