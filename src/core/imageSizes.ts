const TMDB_WIDTHS: Record<string, number> = {
  w92: 92,
  w154: 154,
  w185: 185,
  w300: 300,
  w342: 342,
  w500: 500,
  w780: 780,
  w1280: 1280,
};

const ORDERED_TIERS: Array<[string, number]> = Object.entries(TMDB_WIDTHS).sort((a, b) => a[1] - b[1]);

const TMDB_RE = /^(https?:\/\/image\.tmdb\.org\/t\/p\/)([^/]+)(\/.+)$/;

interface CardImageOptions {
  kind?: 'poster' | 'backdrop';
  displayWidth?: number;
  dpr?: number;
}

function pickTier(kind: 'poster' | 'backdrop', displayWidth?: number, dpr = 1): string {
  if (!displayWidth) return kind === 'poster' ? 'w300' : 'w780';
  const needed = displayWidth * Math.min(dpr, 2);
  const fit = ORDERED_TIERS.find(([, w]) => w >= needed);
  return fit ? fit[0] : 'w1280';
}

export function cardImageUrl(url: string | undefined, kindOrOptions: 'poster' | 'backdrop' | CardImageOptions = 'poster'): string | undefined {
  if (!url) return url;
  const options: CardImageOptions = typeof kindOrOptions === 'string' ? { kind: kindOrOptions } : kindOrOptions;
  const kind = options.kind ?? 'poster';
  const match = url.match(TMDB_RE);
  if (!match) return url;
  const target = pickTier(kind, options.displayWidth, options.dpr);
  const current = TMDB_WIDTHS[match[2]];
  if (current !== undefined && current <= TMDB_WIDTHS[target]) return url;
  return `${match[1]}${target}${match[3]}`;
}
