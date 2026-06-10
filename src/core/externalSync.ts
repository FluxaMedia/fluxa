import { invoke } from '@tauri-apps/api/core';
import { coreParseVideoId } from './engine';
import { dropTraktPlaybackProgress } from './traktSync';
import { syncTraktNow, pushMarkWatchedTrakt, pushWatchlistTrakt } from './traktExternalSync';
import { syncSimklNow, pushMarkWatchedSimkl, pushWatchlistSimkl } from './simklExternalSync';
import type { UserProfile } from './types';

export { enqueueTraktScrobble } from './traktSync';
export { replaceExternalContinueWatching } from './externalSyncUtils';

async function getOAuthClientId(service: string): Promise<string> {
  try {
    return await invoke<string>('get_oauth_client_id', { service });
  } catch {
    return '';
  }
}

export async function pushMarkWatchedExternal(
  videoIds: string[],
  watched: boolean,
  meta: Record<string, unknown> | undefined,
  profile: UserProfile | null,
): Promise<void> {
  if (!profile) return;
  const tasks: Promise<void>[] = [];

  if (profile.traktAccessToken && !(profile.traktTokenExpiresAt && Date.now() / 1000 > profile.traktTokenExpiresAt)) {
    tasks.push((async () => {
      const clientId = await getOAuthClientId('trakt');
      await pushMarkWatchedTrakt(videoIds, watched, profile.traktAccessToken!, clientId);
    })().catch(() => undefined));
  }

  if (profile.simklAccessToken) {
    tasks.push((async () => {
      const clientId = await getOAuthClientId('simkl');
      await pushMarkWatchedSimkl(videoIds, watched, meta, profile.simklAccessToken!, clientId);
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
  const id = String(item.id ?? '');
  const contentType = String(item.type ?? 'movie');
  const tasks: Promise<void>[] = [];

  if (profile.traktAccessToken && !(profile.traktTokenExpiresAt && Date.now() / 1000 > profile.traktTokenExpiresAt)) {
    tasks.push((async () => {
      const clientId = await getOAuthClientId('trakt');
      await pushWatchlistTrakt(id, contentType, command, profile.traktAccessToken!, clientId);
    })().catch(() => undefined));
  }

  if (profile.simklAccessToken && command === 'add') {
    tasks.push((async () => {
      const clientId = await getOAuthClientId('simkl');
      const parsed = await coreParseVideoId(id);
      await pushWatchlistSimkl(id, contentType, parsed, profile.simklAccessToken!, clientId);
    })().catch(() => undefined));
  }

  await Promise.all(tasks);
}

export async function dropExternalPlaybackProgress(item: Record<string, unknown>): Promise<void> {
  const reason = String(item.reason ?? '').toLowerCase();
  const id = String(item.id ?? '');
  if (!id) return;
  if (reason === 'trakt') {
    await dropTraktPlaybackProgress(id);
  }
  // Simkl: no playback progress API — local removal is sufficient.
}

export async function syncExternalIntegrationNow(payload: Record<string, unknown>): Promise<unknown> {
  const provider = String(payload.provider ?? 'trakt').toLowerCase();
  if (provider === 'simkl') return syncSimklNow(payload);
  if (provider === 'trakt') return syncTraktNow(payload);
  return { synced: false, error: `Unsupported external sync provider: ${provider}` };
}
