import { invoke } from '@tauri-apps/api/core';
import { fetch as tauriFetch } from '@tauri-apps/plugin-http';
import type { UserProfile, Meta, Video } from './types';

export function traktScrobbleOnClose(
  profile: UserProfile | null,
  meta: Meta | null,
  episode: Video | null,
  timePosSec: number,
  durationSec: number,
): void {
  if (!profile?.traktAccessToken || !meta) return;
  if (profile.traktTokenExpiresAt && Date.now() / 1000 > profile.traktTokenExpiresAt) return;

  const progress = Math.min(100, (timePosSec / durationSec) * 100);
  const isEpisode = meta.type === 'series' && episode;
  const baseId = meta.id.split(':')[0] ?? meta.id;
  const isTmdb = meta.id.startsWith('tmdb:');
  const ids = isTmdb
    ? { tmdb: parseInt(meta.id.replace('tmdb:', '').split(':')[0] ?? '0', 10) }
    : /^tt\d+/.test(baseId) ? { imdb: baseId } : null;
  if (!ids) return;

  const action = progress >= 80 ? 'stop' : 'pause';
  const body = isEpisode
    ? { show: { ids }, episode: { season: episode.season ?? 1, number: episode.episode ?? episode.number ?? 1 }, progress }
    : { movie: { ids }, progress };

  void invoke<string>('get_oauth_client_id', { service: 'trakt' })
    .then((clientId) => tauriFetch(`https://api.trakt.tv/scrobble/${action}`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        Authorization: `Bearer ${profile.traktAccessToken}`,
        'trakt-api-version': '2',
        'trakt-api-key': clientId,
      },
      body: JSON.stringify(body),
    }))
    .catch(() => undefined);
}

export function simklScrobbleOnClose(
  profile: UserProfile | null,
  meta: Meta | null,
  episode: Video | null,
  timePosSec: number,
  durationSec: number,
): void {
  if (!profile?.simklAccessToken || !meta) return;

  const progress = Math.min(100, (timePosSec / durationSec) * 100);
  const isEpisode = meta.type === 'series' && episode;
  const baseId = meta.id.split(':')[0] ?? meta.id;
  if (!/^tt\d+/.test(baseId)) return;

  const token = profile.simklAccessToken;

  void invoke<string>('get_oauth_client_id', { service: 'simkl' })
    .then(async (clientId) => {
      const simklQuery = `client_id=${encodeURIComponent(clientId)}&app-name=fluxa&app-version=1.0`;
      const authHeaders = {
        'Content-Type': 'application/json',
        'Authorization': `Bearer ${token}`,
      };

      const lookupRes = await tauriFetch(
        `https://api.simkl.com/search/id?imdb=${encodeURIComponent(baseId)}&${simklQuery}`,
        { headers: authHeaders },
      );
      const lookupJson = lookupRes.ok
        ? (await lookupRes.json() as Array<{ type?: string; ids?: Record<string, unknown> }>)
        : [];
      const wantType = isEpisode ? 'tv' : 'movie';
      const found = lookupJson.find((item) => item.type === wantType);
      const simklId = typeof found?.ids?.simkl === 'number' ? found.ids.simkl : null;
      const ids: Record<string, unknown> = simklId != null ? { simkl: simklId } : { imdb: baseId };

      let scrobbleSeason = isEpisode ? (episode.season ?? 1) : 1;
      let scrobbleNumber = isEpisode ? (episode.episode ?? episode.number ?? 1) : 1;

      if (isEpisode && simklId != null) {
        const epRes = await tauriFetch(
          `https://api.simkl.com/tv/${simklId}/episodes?${simklQuery}`,
          { headers: authHeaders },
        );
        if (epRes.ok) {
          const epList = await epRes.json() as Array<{ season?: number; episode?: number; date?: string; title?: string }>;
          const releaseDate = episode.released?.slice(0, 10);
          const epName = (episode.name ?? episode.title ?? '').toLowerCase().trim();
          const list = Array.isArray(epList) ? epList : [];
          let matched = releaseDate ? list.find((e) => e.date?.startsWith(releaseDate)) : undefined;
          if (!matched && epName) matched = list.find((e) => (e.title ?? '').toLowerCase().trim() === epName);
          if (matched?.season != null && matched?.episode != null) {
            scrobbleSeason = matched.season;
            scrobbleNumber = matched.episode;
          }
        }
      }

      const body = isEpisode
        ? { show: { ids }, episode: { season: scrobbleSeason, number: scrobbleNumber }, progress }
        : { movie: { ids }, progress };

      await tauriFetch(`https://api.simkl.com/scrobble/stop?${simklQuery}`, {
        method: 'POST',
        headers: authHeaders,
        body: JSON.stringify(body),
      });
    })
    .catch(() => undefined);
}
