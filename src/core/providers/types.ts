import type { UserProfile } from '../types';

export type WatchedEpisodeInfo = {
  contentId: string;
  contentType: string;
  videoId?: string;
  season?: number;
  episode?: number;
  title?: string;
};

export type WatchProgressInfo = {
  contentId: string;
  contentType: string;
  videoId: string;
  positionSeconds: number;
  durationSeconds: number;
  lastWatched: number;
  season?: number;
  episode?: number;
};

export type PushWatchedArgs = {
  videoIds: string[];
  watched: boolean;
  meta?: Record<string, unknown>;
  episodes: WatchedEpisodeInfo[];
  watchedAtMs?: number;
  animeEpisode?: WatchedEpisodeInfo;
  animeProgressEpisode?: number;
  watchedKeys: Array<{ content_id: string; season?: number; episode?: number }>;
  historyItems: Array<{ content_id: string; content_type: string; title?: string; season?: number; episode?: number; watched_at: number }>;
  progressEntry?: {
    content_id: string;
    content_type: string;
    video_id: string;
    position: number;
    duration: number;
    last_watched: number;
    season?: number;
    episode?: number;
  };
};

export interface ProviderAdapter {
  id: 'trakt' | 'simkl' | 'anilist' | 'stremio' | 'nuvio';
  isConnected(profile: UserProfile | null): boolean;
  pushWatchlist(profile: UserProfile, item: Record<string, unknown>, command: 'add' | 'remove'): Promise<void>;
  pushWatched(profile: UserProfile, args: PushWatchedArgs): Promise<void>;
  pushFavorite?(profile: UserProfile, item: Record<string, unknown>, command: 'add' | 'remove'): Promise<void>;
}
