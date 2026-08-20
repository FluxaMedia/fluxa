import { assetUrl } from '../../platform/assets';
import { color, fade } from '../../design';
export interface RatingSourceInfo {
  icon?: string;
  label: string;
  format: (value: number) => string;
  maskColor?: string;
  colorForValue?: (value: number) => string;
  iconForValue?: (value: number) => string;
}

const percent = (value: number) => `${Math.round(value)}%`;
const outOfTen = (value: number) => value.toFixed(1);
const bareScore = (value: number) => `${Math.round(value)}`;

const METACRITIC_GREEN = color.ratingHigh;
const METACRITIC_YELLOW = color.ratingMid;
const METACRITIC_RED = color.ratingLow;

const metacriticCriticColor = (value: number) => {
  if (value >= 61) return METACRITIC_GREEN;
  if (value >= 40) return METACRITIC_YELLOW;
  return METACRITIC_RED;
};

const metacriticUserColor = (value: number) => {
  if (value >= 7.5) return METACRITIC_GREEN;
  if (value >= 5.0) return METACRITIC_YELLOW;
  return METACRITIC_RED;
};

const tomatoIcon = (value: number) => (value >= 60 ? assetUrl('rt-tomato-fresh.svg') : assetUrl('rt-tomato-rotten.svg'));
const popcornIcon = (value: number) => (value >= 60 ? assetUrl('rt-popcorn-full.svg') : assetUrl('rt-popcorn-spilled.svg'));

export const RATING_SOURCES: Record<string, RatingSourceInfo> = {
  imdb: { icon: assetUrl('imdb.svg'), label: 'IMDb', format: outOfTen },
  tmdb: { icon: assetUrl('tmdb.svg'), label: 'TMDB', format: percent },
  trakt: { icon: assetUrl('trakt.svg'), label: 'Trakt', format: percent },
  letterboxd: { icon: assetUrl('letterboxd.svg'), label: 'Letterboxd', format: outOfTen },
  tomatoes: { icon: assetUrl('rt-tomato-fresh.svg'), label: 'Rotten Tomatoes (Critics)', format: percent, iconForValue: tomatoIcon },
  popcorn: { icon: assetUrl('rt-popcorn-full.svg'), label: 'Rotten Tomatoes (Audience)', format: percent, iconForValue: popcornIcon },
  metacritic: {
    icon: assetUrl('metacritic.svg'),
    label: 'Metacritic',
    format: bareScore,
    maskColor: METACRITIC_GREEN,
    colorForValue: metacriticCriticColor,
  },
  metacriticuser: {
    icon: assetUrl('metacritic.svg'),
    label: 'Metacritic Users',
    format: outOfTen,
    maskColor: METACRITIC_GREEN,
    colorForValue: metacriticUserColor,
  },
  myanimelist: { icon: assetUrl('mal.svg'), label: 'MyAnimeList', format: outOfTen },
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

export function orderedRatingEntries(ratings: Record<string, number> | null | undefined): { source: string; value: number }[] {
  if (!ratings) return [];
  return RATING_DISPLAY_ORDER.filter((source) => source in RATING_SOURCES && typeof ratings[source] === 'number').map((source) => ({
    source,
    value: ratings[source],
  }));
}
