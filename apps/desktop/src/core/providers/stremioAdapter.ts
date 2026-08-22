import type { UserProfile } from '../types';
import { pushStremioWatched, pushStremioWatchlist } from '../stremioExternalSync';
import type { ProviderAdapter, PushWatchedArgs } from './types';

export const stremioAdapter: ProviderAdapter = {
  id: 'stremio',

  isConnected(profile) {
    return Boolean(profile?.stremioAuthKey);
  },

  async pushWatchlist(profile, item, command) {
    await pushStremioWatchlist(item, command, profile);
  },

  async pushWatched(profile, args: PushWatchedArgs) {
    await pushStremioWatched(args.meta, args.watched, args.episodes, profile);
  },
};
