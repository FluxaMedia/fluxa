import { loadLibrary, profileStorageKey } from './libraryOps';
import { corePushPlan } from './engineExternalSync';
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

  const watchlist = (lib.watchlist as LibraryItem[] | undefined) ?? [];
  const completed = (lib.completed as LibraryItem[] | undefined) ?? [];
  const dropped = (lib.dropped as LibraryItem[] | undefined) ?? [];
  const continueWatching = (lib.continueWatching as LibraryItem[] | undefined) ?? [];
  const byId = new Map<string, LibraryItem>();
  for (const item of [...watchlist, ...completed, ...dropped, ...continueWatching]) byId.set(item.id, item);

  const plan = await corePushPlan({
    destination,
    categories,
    watchlist,
    completed,
    dropped,
    continueWatching,
    nowSec: Math.floor(Date.now() / 1000),
  });

  if (categories.includes('watchlist')) {
    try {
      if (plan.watchlistItems) {
        if (destination === 'trakt') {
          const token = profile.traktAccessToken;
          if (token)
            for (const { id, contentType } of plan.watchlistItems)
              await pushWatchlistTrakt(id, contentType, 'add', token, traktClientId ?? '');
        } else if (destination === 'simkl') {
          const token = profile.simklAccessToken;
          if (token)
            for (const { id, contentType } of plan.watchlistItems)
              await pushWatchlistSimkl(id, contentType, 'add', token, simklClientId ?? '');
        } else if (destination === 'anilist') {
          const token = profile.anilistAccessToken;
          if (token) for (const { id } of plan.watchlistItems) await pushWatchlistAniList(id, 'add', token);
        } else if (destination === 'stremio') {
          for (const { id } of plan.watchlistItems) {
            const item = byId.get(id);
            if (item) await pushStremioWatchlist(item as unknown as Record<string, unknown>, 'add', profile);
          }
        }
      } else if (plan.watchlistNuvioItems && destination === 'nuvio') {
        const token = profile.nuvioAccessToken;
        if (token)
          await nuvioPushLibrary(
            token,
            profile.nuvioProfileIndex ?? 1,
            plan.watchlistNuvioItems.map((item) => ({
              content_id: item.contentId,
              content_type: item.contentType,
              name: item.name ?? undefined,
              poster: item.poster ?? null,
              background: item.background ?? null,
            })),
          );
      }
    } catch (err) {
      errors.watchlist = err instanceof Error ? err.message : String(err);
    }
  }

  if (categories.includes('watched')) {
    try {
      if (plan.watchedVideoIds) {
        if (destination === 'trakt') {
          const token = profile.traktAccessToken;
          if (token && plan.watchedVideoIds.length > 0) await pushMarkWatchedTrakt(plan.watchedVideoIds, true, token, traktClientId ?? '');
        } else if (destination === 'simkl') {
          const token = profile.simklAccessToken;
          if (token && plan.watchedVideoIds.length > 0)
            await pushMarkWatchedSimkl(plan.watchedVideoIds, true, undefined, token, simklClientId ?? '');
        }
      } else if (plan.watchedStatusItems && destination === 'anilist') {
        const token = profile.anilistAccessToken;
        if (token) for (const { id, status } of plan.watchedStatusItems) await pushLibraryStatusAniList(id, status, 'add', token);
      } else if (plan.watchedItemIds && destination === 'stremio') {
        for (const id of plan.watchedItemIds) {
          const item = byId.get(id);
          if (!item) continue;
          const episode = episodeVideoFor(item);
          const episodes = episode
            ? [
                {
                  contentId: item.id,
                  contentType: item.type,
                  videoId: episode.id,
                  season: item.lastEpisodeSeason,
                  episode: item.lastEpisodeNumber,
                  title: item.lastEpisodeName,
                },
              ]
            : [];
          await pushStremioWatched(item as unknown as Record<string, unknown>, true, episodes, profile);
        }
      } else if (plan.watchedNuvioItems && destination === 'nuvio') {
        const token = profile.nuvioAccessToken;
        if (token)
          await nuvioPushWatchHistory(
            token,
            profile.nuvioProfileIndex ?? 1,
            plan.watchedNuvioItems.map((item) => ({
              content_id: item.contentId,
              content_type: item.contentType,
              title: item.title ?? undefined,
              season: item.season ?? undefined,
              episode: item.episode ?? undefined,
              watched_at: item.watchedAt,
            })),
          );
      }
    } catch (err) {
      errors.watched = err instanceof Error ? err.message : String(err);
    }
  }

  if (categories.includes('continueWatching')) {
    try {
      if (plan.progressItemIds) {
        for (const id of plan.progressItemIds) {
          const item = byId.get(id);
          if (!item || !item.duration) continue;
          if (destination === 'trakt') {
            await traktScrobble(profile, item as unknown as Meta, episodeVideoFor(item), item.timeOffset ?? 0, item.duration, 'pause');
          } else if (destination === 'simkl') {
            await simklScrobble(profile, item as unknown as Meta, episodeVideoFor(item), item.timeOffset ?? 0, item.duration, 'pause');
          } else if (destination === 'anilist') {
            if (!profile.anilistAccessToken) continue;
            await pushAnimeTrackingExternal(
              {
                meta: item as unknown as Record<string, unknown>,
                episode: episodeVideoFor(item) ?? undefined,
                progressEpisode: item.lastEpisodeNumber,
                watched: true,
              },
              profile,
            );
          } else if (destination === 'stremio') {
            if (!item.lastVideoId) continue;
            await pushStremioPlaybackProgress(
              item as unknown as Record<string, unknown>,
              {
                contentId: item.id,
                contentType: item.type,
                videoId: item.lastVideoId,
                positionSeconds: item.timeOffset ?? 0,
                durationSeconds: item.duration,
                lastWatched: Date.now(),
                season: item.lastEpisodeSeason,
                episode: item.lastEpisodeNumber,
              },
              profile,
            );
          }
        }
      } else if (plan.progressNuvioEntries && destination === 'nuvio') {
        const token = profile.nuvioAccessToken;
        if (token && plan.progressNuvioEntries.length > 0) {
          await nuvioPushWatchProgress(
            token,
            profile.nuvioProfileIndex ?? 1,
            plan.progressNuvioEntries.map((entry) => ({
              content_id: entry.contentId,
              content_type: entry.contentType,
              video_id: entry.videoId,
              position: entry.position,
              duration: entry.duration,
              last_watched: entry.lastWatched,
              season: entry.season ?? undefined,
              episode: entry.episode ?? undefined,
            })),
          );
        }
      }
    } catch (err) {
      errors.continueWatching = err instanceof Error ? err.message : String(err);
    }
  }

  return { errors };
}
