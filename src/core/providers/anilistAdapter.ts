import type { UserProfile } from '../types';
import { pushWatchlistAniList } from '../anilistExternalSync';
import { pushAnimeTrackingExternal } from '../animeExternalSync';
import type { ProviderAdapter, PushWatchedArgs } from './types';

export const anilistAdapter: ProviderAdapter = {
  id: 'anilist',

  isConnected(profile) {
    return Boolean(profile?.anilistAccessToken);
  },

  async pushWatchlist(profile, item, command) {
    const id = String(item.id ?? '');
    await pushWatchlistAniList(id, command, profile.anilistAccessToken!);
  },

  async pushWatched(profile, args: PushWatchedArgs) {
    if (!args.watched) return;
    await pushAnimeTrackingExternal({
      meta: args.meta,
      episode: args.animeEpisode,
      progressEpisode: args.animeProgressEpisode,
      watched: args.watched,
    }, profile);
  },
};
