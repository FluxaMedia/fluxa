import type { Meta } from './types';

type UnknownRecord = Record<string, unknown>;

export function seasonPosterUrl(meta: Meta | null | undefined, season?: number): string | undefined {
  if (!meta || meta.type !== 'series') return undefined;
  const record = meta as unknown as UnknownRecord;
  const seasonCollections = [
    record.seasonPosters,
    record.season_posters,
    record.seasonImages,
    record.season_images,
    record.seasons,
  ];

  for (const collection of seasonCollections) {
    const url = urlFromSeasonCollection(collection, season);
    if (url) return url;
  }

  return firstString(record.seasonPoster, record.season_poster);
}

function urlFromSeasonCollection(collection: unknown, season?: number): string | undefined {
  if (Array.isArray(collection)) {
    return urlFromSeasonArray(collection, season);
  }
  if (collection && typeof collection === 'object') {
    return urlFromSeasonMap(collection as UnknownRecord, season);
  }
  return undefined;
}

function urlFromSeasonArray(items: unknown[], season?: number): string | undefined {
  const records = items.filter(isRecord);
  if (season != null) {
    const match = records.find((item) => seasonNumber(item) === season);
    const url = match ? imageUrl(match) : undefined;
    if (url) return url;
  }
  const firstSeason = records
    .filter((item) => seasonNumber(item) !== 0)
    .sort((a, b) => (seasonNumber(a) ?? Number.MAX_SAFE_INTEGER) - (seasonNumber(b) ?? Number.MAX_SAFE_INTEGER))
    .find((item) => imageUrl(item));
  return firstSeason ? imageUrl(firstSeason) : undefined;
}

function urlFromSeasonMap(map: UnknownRecord, season?: number): string | undefined {
  const seasonKeys = season == null
    ? ['1', 'S1', 's1', 'season1', 'season_1']
    : [String(season), `S${season}`, `s${season}`, `season${season}`, `season_${season}`];

  for (const key of seasonKeys) {
    const url = imageUrlValue(map[key]);
    if (url) return url;
  }

  if (season != null) return undefined;
  for (const value of Object.values(map)) {
    const url = imageUrlValue(value);
    if (url) return url;
  }
  return undefined;
}

function imageUrlValue(value: unknown): string | undefined {
  if (typeof value === 'string') return stringValue(value);
  return isRecord(value) ? imageUrl(value) : undefined;
}

function imageUrl(record: UnknownRecord): string | undefined {
  return firstString(
    record.poster,
    record.posterUrl,
    record.poster_url,
    record.background,
    record.backgroundUrl,
    record.background_url,
    record.backdrop,
    record.backdropUrl,
    record.backdrop_url,
    record.url,
    record.image,
    record.img,
  );
}

function seasonNumber(record: UnknownRecord): number | undefined {
  const raw = record.season ?? record.seasonNumber ?? record.season_number ?? record.number;
  return typeof raw === 'number' && Number.isFinite(raw) ? raw : undefined;
}

function firstString(...values: unknown[]): string | undefined {
  for (const value of values) {
    const text = stringValue(value);
    if (text) return text;
  }
  return undefined;
}

function stringValue(value: unknown): string | undefined {
  return typeof value === 'string' && value.trim() ? value.trim() : undefined;
}

function isRecord(value: unknown): value is UnknownRecord {
  return !!value && typeof value === 'object' && !Array.isArray(value);
}
