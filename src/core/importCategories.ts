export type ImportCategory = 'watchlist' | 'continueWatching' | 'watched' | 'collections' | 'addons';

export const PROVIDER_IMPORT_CATEGORIES: Record<string, ImportCategory[]> = {
  trakt: ['watchlist', 'continueWatching', 'watched'],
  simkl: ['watchlist', 'continueWatching', 'watched'],
  anilist: ['watchlist', 'continueWatching', 'watched'],
  stremio: ['watchlist', 'continueWatching', 'watched', 'addons'],
  nuvio: ['watchlist', 'continueWatching', 'watched', 'collections', 'addons'],
};
