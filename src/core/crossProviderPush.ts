import { loadLibrary, profileStorageKey } from './libraryOps';
import { pushWatchlistTrakt, pushMarkWatchedTrakt } from './traktExternalSync';
import { pushWatchlistSimkl, pushMarkWatchedSimkl } from './simklExternalSync';
import { pushWatchlistAniList, pushLibraryStatusAniList } from './anilistExternalSync';
import { pushAnimeTrackingExternal } from './animeExternalSync';
import { pushStremioWatchlist, pushStremioWatched, pushStremioPlaybackProgress } from './stremioExternalSync';
import { nuvioPushLibrary, nuvioPushWatchHistory, nuvioPushWatchProgress } from './nuvioApi';
import { traktScrobble, simklScrobble } from './scrobble';
import type { UserProfile, Meta, Video, LibraryItem } from './types';
import type { ImportCategory } from './importCategories';

function episodeVideoFor(item: LibraryItem): Video | null {
  if (!item.lastVideoId) return null;
  return {
    id: item.lastVideoId,
    name: item.lastEpisodeName,
    season: item.lastEpisodeSeason,
    number: item.lastEpisodeNumber,
  };
}

async function pushWatchlist(destination: string, items: LibraryItem[], profile: UserProfile, traktClientId?: string, simklClientId?: string): Promise<void> {
  if (destination === 'trakt') {
    const token = profile.traktAccessToken;
    if (!token) return;
    for (const item of items) await pushWatchlistTrakt(item.id, item.type, 'add', token, traktClientId ?? '');
  } else if (destination === 'simkl') {
    const token = profile.simklAccessToken;
    if (!token) return;
    for (const item of items) await pushWatchlistSimkl(item.id, item.type, 'add', token, simklClientId ?? '');
  } else if (destination === 'anilist') {
    const token = profile.anilistAccessToken;
    if (!token) return;
    for (const item of items) await pushWatchlistAniList(item.id, 'add', token);
  } else if (destination === 'stremio') {
    for (const item of items) await pushStremioWatchlist(item as unknown as Record<string, unknown>, 'add', profile);
  } else if (destination === 'nuvio') {
    const token = profile.nuvioAccessToken;
    if (!token) return;
    const profileIdx = profile.nuvioProfileIndex ?? 1;
    await nuvioPushLibrary(token, profileIdx, items.map((item) => ({
      content_id: item.id,
      content_type: item.type,
      name: item.name,
      poster: item.poster ?? null,
      background: item.background ?? null,
    })));
  }
}

async function pushWatched(destination: string, completed: LibraryItem[], dropped: LibraryItem[], profile: UserProfile, traktClientId?: string, simklClientId?: string): Promise<void> {
  const all = [...completed, ...dropped];
  if (all.length === 0) return;
  if (destination === 'trakt') {
    const token = profile.traktAccessToken;
    if (!token) return;
    const videoIds = all.map((item) => item.lastVideoId ?? item.id);
    await pushMarkWatchedTrakt(videoIds, true, token, traktClientId ?? '');
  } else if (destination === 'simkl') {
    const token = profile.simklAccessToken;
    if (!token) return;
    const videoIds = all.map((item) => item.lastVideoId ?? item.id);
    await pushMarkWatchedSimkl(videoIds, true, undefined, token, simklClientId ?? '');
  } else if (destination === 'anilist') {
    const token = profile.anilistAccessToken;
    if (!token) return;
    for (const item of completed) await pushLibraryStatusAniList(item.id, 'completed', 'add', token);
    for (const item of dropped) await pushLibraryStatusAniList(item.id, 'dropped', 'add', token);
  } else if (destination === 'stremio') {
    for (const item of all) {
      const episode = episodeVideoFor(item);
      const episodes = episode ? [{
        contentId: item.id,
        contentType: item.type,
        videoId: episode.id,
        season: item.lastEpisodeSeason,
        episode: item.lastEpisodeNumber,
        title: item.lastEpisodeName,
      }] : [];
      await pushStremioWatched(item as unknown as Record<string, unknown>, true, episodes, profile);
    }
  } else if (destination === 'nuvio') {
    const token = profile.nuvioAccessToken;
    if (!token) return;
    const profileIdx = profile.nuvioProfileIndex ?? 1;
    await nuvioPushWatchHistory(token, profileIdx, all.map((item) => ({
      content_id: item.id,
      content_type: item.type,
      title: item.name,
      season: item.lastEpisodeSeason,
      episode: item.lastEpisodeNumber,
      watched_at: Math.floor(Date.now() / 1000),
    })));
  }
}

async function pushContinueWatching(destination: string, items: LibraryItem[], profile: UserProfile): Promise<void> {
  if (destination === 'trakt') {
    for (const item of items) {
      if (!item.duration) continue;
      await traktScrobble(profile, item as unknown as Meta, episodeVideoFor(item), item.timeOffset ?? 0, item.duration, 'pause');
    }
  } else if (destination === 'simkl') {
    for (const item of items) {
      if (!item.duration) continue;
      await simklScrobble(profile, item as unknown as Meta, episodeVideoFor(item), item.timeOffset ?? 0, item.duration, 'pause');
    }
  } else if (destination === 'anilist') {
    if (!profile.anilistAccessToken) return;
    for (const item of items) {
      await pushAnimeTrackingExternal({
        meta: item as unknown as Record<string, unknown>,
        episode: episodeVideoFor(item) ?? undefined,
        progressEpisode: item.lastEpisodeNumber,
        watched: true,
      }, profile);
    }
  } else if (destination === 'stremio') {
    for (const item of items) {
      if (!item.duration || !item.lastVideoId) continue;
      await pushStremioPlaybackProgress(item as unknown as Record<string, unknown>, {
        contentId: item.id,
        contentType: item.type,
        videoId: item.lastVideoId,
        positionSeconds: item.timeOffset ?? 0,
        durationSeconds: item.duration,
        lastWatched: Date.now(),
        season: item.lastEpisodeSeason,
        episode: item.lastEpisodeNumber,
      }, profile);
    }
  } else if (destination === 'nuvio') {
    const token = profile.nuvioAccessToken;
    if (!token) return;
    const profileIdx = profile.nuvioProfileIndex ?? 1;
    const entries = items.filter((item) => item.lastVideoId && item.duration).map((item) => ({
      content_id: item.id,
      content_type: item.type,
      video_id: item.lastVideoId!,
      position: item.timeOffset ?? 0,
      duration: item.duration!,
      last_watched: Math.floor(Date.now() / 1000),
      season: item.lastEpisodeSeason,
      episode: item.lastEpisodeNumber,
    }));
    if (entries.length > 0) await nuvioPushWatchProgress(token, profileIdx, entries);
  }
}

export async function pushImportedCategoriesToDestination(params: {
  destination: string;
  categories: ImportCategory[];
  profile: UserProfile;
  traktClientId?: string;
  simklClientId?: string;
}): Promise<{ errors: Partial<Record<ImportCategory, string>> }> {
  const { destination, categories, profile, traktClientId, simklClientId } = params;
  const profileKey = profileStorageKey(profile);
  const lib = await loadLibrary(profileKey);
  const errors: Partial<Record<ImportCategory, string>> = {};

  if (categories.includes('watchlist')) {
    try {
      await pushWatchlist(destination, (lib.watchlist as LibraryItem[] | undefined) ?? [], profile, traktClientId, simklClientId);
    } catch (err) {
      errors.watchlist = err instanceof Error ? err.message : String(err);
    }
  }

  if (categories.includes('watched')) {
    try {
      await pushWatched(destination, (lib.completed as LibraryItem[] | undefined) ?? [], (lib.dropped as LibraryItem[] | undefined) ?? [], profile, traktClientId, simklClientId);
    } catch (err) {
      errors.watched = err instanceof Error ? err.message : String(err);
    }
  }

  if (categories.includes('continueWatching')) {
    try {
      await pushContinueWatching(destination, (lib.continueWatching as LibraryItem[] | undefined) ?? [], profile);
    } catch (err) {
      errors.continueWatching = err instanceof Error ? err.message : String(err);
    }
  }

  return { errors };
}
