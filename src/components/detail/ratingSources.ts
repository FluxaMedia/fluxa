export interface RatingSourceInfo {
  icon?: string;
  label: string;
  format: (value: number) => string;
  maskColor?: string;
  lucideIcon?: 'popcorn';
  iconColor?: string;
}

const percent = (value: number) => `${Math.round(value)}%`;
const outOfTen = (value: number) => value.toFixed(1);
const bareScore = (value: number) => `${Math.round(value)}`;

export const RATING_SOURCES: Record<string, RatingSourceInfo> = {
  imdb: { icon: '/imdb.svg', label: 'IMDb', format: outOfTen },
  tmdb: { icon: '/tmdb.svg', label: 'TMDB', format: percent },
  trakt: { icon: '/trakt.svg', label: 'Trakt', format: percent },
  letterboxd: { icon: '/letterboxd.svg', label: 'Letterboxd', format: outOfTen },
  tomatoes: { icon: '/rottentomatoes.svg', label: 'Rotten Tomatoes (Critics)', format: percent, maskColor: '#FA320A' },
  popcorn: { label: 'Rotten Tomatoes (Audience)', format: percent, lucideIcon: 'popcorn', iconColor: '#FA320A' },
  metacritic: { icon: '/metacritic.svg', label: 'Metacritic', format: bareScore, maskColor: '#66CC33' },
  metacriticuser: { icon: '/metacritic.svg', label: 'Metacritic Users', format: outOfTen, maskColor: '#FFCC33' },
  myanimelist: { icon: '/mal.svg', label: 'MyAnimeList', format: outOfTen },
};

export const RATING_DISPLAY_ORDER = [
  'imdb',
  'tmdb',
  'trakt',
  'tomatoes',
  'popcorn',
  'metacritic',
  'metacriticuser',
  'letterboxd',
  'myanimelist',
];

export function orderedRatingEntries(
  ratings: Record<string, number> | null | undefined,
): { source: string; value: number }[] {
  if (!ratings) return [];
  return RATING_DISPLAY_ORDER
    .filter((source) => source in RATING_SOURCES && typeof ratings[source] === 'number')
    .map((source) => ({ source, value: ratings[source] }));
}
