import type { AppState } from './types';

export const DEFAULT_APP_PREFS: Record<string, unknown> = {
  language: 'en',
  startPage: 'home',
  backgroundPlayback: false,
  alertNewEpisodes: true,
  accentColorArgb: '#FFFFFF',
  amoledMode: false,
  animationsEnabled: true,
  navLayout: 'sidebar',
  navBarPosition: 'left',
  navItemsAlign: 'center',
  interfaceDensity: 'medium',
  continueWatchingEnabled: true,
  showHeroSection: true,
  homeSeasonPostersOnHero: true,
  detailSeasonPostersOnHero: true,
  preferredPlayer: 'mpv',
  playbackSpeed: '1.0',
  seekSeconds: '10',
  holdToSpeedEnabled: true,
  holdSpeed: '2.0',
  playerBufferCacheMb: '100',
  playerForwardBufferSeconds: '120',
  playerBackBufferSeconds: '30',
  subtitleSize: '100',
  subtitleColor: '#FFFFFF',
  subtitleTextOpacity: '1.0',
  subtitleBackgroundColor: '#000000',
  subtitleBackgroundOpacity: '0.5',
  subtitleOutlineColor: '#000000',
  subtitleOutlineOpacity: '1.0',
  tmdbApiKey: '',
  tmdbCastImagesEnabled: true,
  tmdbSimilarResultsEnabled: true,
  tmdbTrailersEnabled: true,
  tmdbRecommendationsEnabled: true,
  tmdbCollectionInfoEnabled: true,
  tmdbEpisodeImagesEnabled: true,
  tmdbLogosBackdropsEnabled: true,
  tmdbRatingsEnabled: true,
  autoPlayNextEpisode: true,
  nextEpisodeThresholdPercent: '85',
  watchedThresholdPercent: '90',
};

export function appPrefs(state: AppState): Record<string, unknown> {
  return {
    ...DEFAULT_APP_PREFS,
    ...((state.settings?.values ?? {}) as Record<string, unknown>),
  };
}

export function prefString(prefs: Record<string, unknown>, key: string, fallback = ''): string {
  const value = prefs[key];
  return typeof value === 'string' && value.trim() ? value.trim() : fallback;
}

export function prefBool(prefs: Record<string, unknown>, key: string, fallback = false): boolean {
  const value = prefs[key];
  return typeof value === 'boolean' ? value : fallback;
}
