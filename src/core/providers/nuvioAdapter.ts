import { coreInvoke } from '../engine';
import {
  nuvioDeleteWatchHistory,
  nuvioDeleteWatchProgress,
  nuvioPullLibrary,
  nuvioPushLibrary,
  nuvioPushWatchHistory,
  nuvioPushWatchProgress,
  nuvioRefreshToken,
} from '../nuvioApi';
import { saveProfile } from '../profiles';
import type { UserProfile } from '../types';
import type { ProviderAdapter, PushWatchedArgs } from './types';

const mutationQueues = new Map<string, Promise<void>>();

function queueMutation(key: string, mutation: () => Promise<void>): Promise<void> {
  const previous = mutationQueues.get(key) ?? Promise.resolve();
  const next = previous.catch(() => undefined).then(mutation);
  mutationQueues.set(key, next);
  void next.finally(() => {
    if (mutationQueues.get(key) === next) mutationQueues.delete(key);
  });
  return next;
}

function isAuthFailure(err: unknown): boolean {
  const message = err instanceof Error ? err.message : String(err);
  return /\b(401|403)\b|JWT|token|expired/i.test(message);
}

async function refreshProfile(profile: UserProfile): Promise<UserProfile> {
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

async function validProfile(profile: UserProfile): Promise<UserProfile> {
  if (!profile.nuvioAccessToken || !profile.nuvioRefreshToken) return profile;
  const expiresAt = profile.nuvioTokenExpiresAt ?? 0;
  if (expiresAt > Math.floor(Date.now() / 1000) + 60) return profile;
  return refreshProfile(profile);
}

export const nuvioAdapter: ProviderAdapter = {
  id: 'nuvio',

  isConnected(profile) {
    return Boolean(profile?.nuvioAccessToken);
  },

  async pushWatchlist(profile, item, command) {
    const queueKey = `${profile.nuvioUserId ?? profile.id}:${profile.nuvioProfileIndex ?? 1}`;
    await queueMutation(queueKey, async () => {
      const nuvioProfile = await validProfile(profile);
      const token = nuvioProfile.nuvioAccessToken;
      if (!token) return;
      const profileIdx = nuvioProfile.nuvioProfileIndex ?? 1;
      const remote = await nuvioPullLibrary(token, profileIdx);
      const updated = await coreInvoke<typeof remote>(
        'nuvioLibraryMutationPlan',
        JSON.stringify({ remote, item, command, nowMs: Date.now() }),
      );
      if (updated) await nuvioPushLibrary(token, profileIdx, updated);
    });
  },

  async pushWatched(profile, args: PushWatchedArgs) {
    let nuvioProfile = await validProfile(profile);
    const push = async () => {
      const token = nuvioProfile.nuvioAccessToken!;
      const profileIdx = nuvioProfile.nuvioProfileIndex ?? 1;
      if (!args.watched) {
        if (args.watchedKeys.length > 0) await nuvioDeleteWatchHistory(token, profileIdx, args.watchedKeys);
        return;
      }
      if (args.episodes.length > 0) {
        await Promise.all(
          args.episodes.map((info) =>
            nuvioDeleteWatchProgress(token, profileIdx, info.contentId, info.season, info.episode).catch(() => undefined),
          ),
        );
        await nuvioPushWatchHistory(token, profileIdx, args.historyItems);
      }
      if (args.progressEntry) {
        await nuvioPushWatchProgress(token, profileIdx, [args.progressEntry]);
      }
    };
    try {
      await push();
    } catch (err) {
      if (!isAuthFailure(err)) throw err;
      nuvioProfile = await refreshProfile(nuvioProfile);
      await push();
    }
  },
};
