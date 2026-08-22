import type { UserProfile } from '../types';
import { getOAuthClientId } from '../traktSync';
import { pushFavoriteTrakt, pushMarkWatchedTrakt, pushWatchlistTrakt } from '../traktExternalSync';
import type { ProviderAdapter, PushWatchedArgs } from './types';

export const traktAdapter: ProviderAdapter = {
  id: 'trakt',

  isConnected(profile) {
    return Boolean(profile?.traktAccessToken);
  },

  async pushWatchlist(profile, item, command) {
    const clientId = await getOAuthClientId('trakt');
    const id = String(item.id ?? '');
    const contentType = String(item.type ?? 'movie');
    await pushWatchlistTrakt(id, contentType, command, profile.traktAccessToken!, clientId);
  },

  async pushWatched(profile, args: PushWatchedArgs) {
    const clientId = await getOAuthClientId('trakt');
    await pushMarkWatchedTrakt(args.videoIds, args.watched, profile.traktAccessToken!, clientId, args.watchedAtMs);
  },

  async pushFavorite(profile, item, command) {
    const clientId = await getOAuthClientId('trakt');
    const id = String(item.id ?? '');
    const contentType = String(item.type ?? 'movie');
    await pushFavoriteTrakt(id, contentType, command, profile.traktAccessToken!, clientId);
  },
};
