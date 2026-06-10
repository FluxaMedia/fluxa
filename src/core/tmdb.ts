import type { Meta, MetaLink } from './types';

interface TmdbFindResponse {
  movie_results?: Array<{ id?: number }>;
  tv_results?: Array<{ id?: number }>;
}

interface TmdbCredit {
  name?: string;
  profile_path?: string | null;
  job?: string;
}

interface TmdbCreditsResponse {
  cast?: TmdbCredit[];
  crew?: TmdbCredit[];
}

export async function fetchTmdbPeopleImages({
  meta,
  links,
  apiKey,
  language,
}: {
  meta: Meta;
  links: MetaLink[];
  apiKey: string;
  language: string;
}): Promise<Record<string, string>> {
  const trimmedKey = apiKey.trim();
  if (!trimmedKey || links.length === 0) return {};

  const tmdbId = await resolveTmdbId(meta, trimmedKey, language);
  if (!tmdbId) return {};

  const type = tmdbContentType(meta.type);
  const credits = await tryFetchJson(tmdbUrl(`3/${type}/${tmdbId}/credits`, trimmedKey, language)) as TmdbCreditsResponse | null;
  if (!credits) return {};

  const wantedNames = new Map(links.map((link) => [normalizePersonName(link.name), link.name]));
  const images: Record<string, string> = {};

  for (const person of [...(credits.cast ?? []), ...(credits.crew ?? [])]) {
    const canonicalName = wantedNames.get(normalizePersonName(person.name ?? ''));
    const image = tmdbImage(person.profile_path, 'w185');
    if (canonicalName && image && !images[canonicalName]) {
      images[canonicalName] = image;
    }
  }

  return images;
}

async function resolveTmdbId(meta: Meta, apiKey: string, language: string): Promise<string | null> {
  const baseId = meta.id.replace(/^tmdb:/i, '').split(':')[0] ?? '';
  if (/^\d+$/.test(baseId)) return baseId;

  const imdbId = meta.id.split(':')[0];
  if (!/^tt\d+$/i.test(imdbId)) return null;

  const response = await tryFetchJson(tmdbUrl(`3/find/${encodeURIComponent(imdbId)}`, apiKey, language, {
    external_source: 'imdb_id',
  })) as TmdbFindResponse | null;
  const key = meta.type === 'series' ? 'tv_results' : 'movie_results';
  const result = response?.[key]?.[0];
  return result?.id != null ? String(result.id) : null;
}

function tmdbUrl(path: string, apiKey: string, language: string, extra: Record<string, string> = {}): string {
  const params = new URLSearchParams({
    api_key: apiKey,
    language: tmdbLanguage(language),
    ...extra,
  });
  return `https://api.themoviedb.org/${path}?${params.toString()}`;
}

function tmdbContentType(contentType: string): string {
  return contentType === 'series' ? 'tv' : 'movie';
}

function tmdbLanguage(language: string): string {
  if (!language || language === 'en' || language === 'english_us') return 'en-US';
  if (language === 'tr' || language === 'tr_tr') return 'tr-TR';
  return language.includes('-') ? language : `${language}-${language.toUpperCase()}`;
}

function tmdbImage(path: string | null | undefined, size: string): string | undefined {
  return path ? `https://image.tmdb.org/t/p/${size}${path}` : undefined;
}

function normalizePersonName(name: string): string {
  return name.trim().toLowerCase().replace(/\s+/g, ' ');
}

async function tryFetchJson(url: string): Promise<unknown | null> {
  try {
    const response = await fetch(url);
    if (!response.ok) return null;
    return response.json();
  } catch {
    return null;
  }
}
