import { traktAdapter } from './traktAdapter';
import { simklAdapter } from './simklAdapter';
import { anilistAdapter } from './anilistAdapter';
import { stremioAdapter } from './stremioAdapter';
import { nuvioAdapter } from './nuvioAdapter';
import type { ProviderAdapter } from './types';

export const providerAdapters: ProviderAdapter[] = [traktAdapter, simklAdapter, anilistAdapter, stremioAdapter, nuvioAdapter];

export type { ProviderAdapter, PushWatchedArgs, WatchedEpisodeInfo, WatchProgressInfo } from './types';
