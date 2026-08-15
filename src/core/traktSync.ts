import { _appVersion, platformFetch } from './httpClient';
import { loadActiveProfile } from './libraryOps';
import { platformInvoke } from '../platform/invoke';
import { coreInvoke } from './engine';
import { saveProfile } from './profiles';
import type { UserProfile } from './types';

export async function refreshTraktProfile(profile: UserProfile, force = false): Promise<UserProfile> {
  const expiresAt = profile.traktTokenExpiresAt ?? 0;
  if (!profile.traktAccessToken || !profile.traktRefreshToken || (!force && expiresAt > Math.floor(Date.now() / 1000) + 60)) return profile;
  const tokenJson = await platformInvoke<string>('trakt_oauth_refresh', { refreshToken: profile.traktRefreshToken });
  const tokens = JSON.parse(tokenJson) as { access_token: string; refresh_token?: string; created_at?: number; expires_in?: number };
  const updated: UserProfile = {
    ...profile,
    traktAccessToken: tokens.access_token,
    traktRefreshToken: tokens.refresh_token ?? profile.traktRefreshToken,
    traktTokenExpiresAt: (tokens.created_at ?? Math.floor(Date.now() / 1000)) + (tokens.expires_in ?? 0),
  };
  await saveProfile(updated);
  return updated;
}

export async function getOAuthClientId(service: string): Promise<string> {
  try {
    return await platformInvoke<string>('get_oauth_client_id', { service });
  } catch {
    return '';
  }
}

export function traktHeaders(token: string, clientId: string): HeadersInit {
  return {
    'Content-Type': 'application/json',
    'User-Agent': `Fluxa Desktop/${_appVersion}`,
    Authorization: `Bearer ${token}`,
    'trakt-api-version': '2',
    'trakt-api-key': clientId,
  };
}

export async function dropTraktPlaybackProgress(showId: string): Promise<void> {
  const storedProfile = await loadActiveProfile();
  const profile = storedProfile ? await refreshTraktProfile(storedProfile).catch(() => storedProfile) : null;
  const token = profile?.traktAccessToken;
  if (!token) return;
  if (profile?.traktTokenExpiresAt && Date.now() / 1000 > profile.traktTokenExpiresAt) return;

  const clientId = await getOAuthClientId('trakt');
  const headers = traktHeaders(token, clientId);

  try {
    const responses = await Promise.all([
      platformFetch('https://api.trakt.tv/sync/playback/movies', { headers }),
      platformFetch('https://api.trakt.tv/sync/playback/episodes', { headers }),
    ]);
    if (responses.some((response) => !response.ok)) return;
    const pages = await Promise.all(responses.map((response) => response.json().catch(() => [])));
    const playbackItems = pages.flatMap((page) => Array.isArray(page) ? page : []);

    const deleteIds = await coreInvoke<number[]>('traktPlaybackDeleteIds', JSON.stringify({ contentId: showId, items: playbackItems })) ?? [];
    const deletePromises = deleteIds.map((id) =>
      platformFetch(`https://api.trakt.tv/sync/playback/${id}`, {
        method: 'DELETE',
        headers,
      }).then(() => undefined).catch(() => undefined),
    );

    await Promise.all(deletePromises);
  } catch {
  }
}

export async function enqueueTraktScrobble(payload: Record<string, unknown>): Promise<unknown> {
  const url = payload.url as string | undefined;
  const body = payload.body as unknown;
  const token = payload.token as string | undefined;
  if (!url || !token) return {};
  try {
    await fetch(url, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'User-Agent': `Fluxa Desktop/${_appVersion}`,
        Authorization: `Bearer ${token}`,
        'trakt-api-version': '2',
        'trakt-api-key': payload.clientId as string,
      },
      body: JSON.stringify(body),
    });
  } catch {
  }
  return {};
}
