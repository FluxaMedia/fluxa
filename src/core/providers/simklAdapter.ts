import type { UserProfile } from '../types';
import { getOAuthClientId } from '../traktSync';
import { pushMarkWatchedSimkl, pushWatchlistSimkl } from '../simklExternalSync';
import type { ProviderAdapter, PushWatchedArgs } from './types';

export const simklAdapter: ProviderAdapter = {
  id: 'simkl',

  isConnected(profile) {
    return Boolean(profile?.simklAccessToken);
  },

  async pushWatchlist(profile, item, command) {
    if (command !== 'add') return;
    const clientId = await getOAuthClientId('simkl');
    const id = String(item.id ?? '');
    const contentType = String(item.type ?? 'movie');
    await pushWatchlistSimkl(id, contentType, command, profile.simklAccessToken!, clientId);
  },

  async pushWatched(profile, args: PushWatchedArgs) {
    const clientId = await getOAuthClientId('simkl');
    await pushMarkWatchedSimkl(args.videoIds, args.watched, args.meta, profile.simklAccessToken!, clientId, args.watchedAtMs);
  },
};
