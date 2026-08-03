import {
  coreTmdbBuiltinCatalogUrl,
  coreTmdbBuiltinManifest,
  coreTmdbBulkMetas,
  coreTmdbEpisodesToVideos,
  coreTmdbFullMetaToMeta,
  coreTmdbPickLogo,
  coreInvoke,
} from './engine';
import { tryFetchJson } from './httpClient';
import { runWithConcurrency } from './fetchPlanning';
import type { AddonDescriptor } from './types';

export const BUILTIN_TMDB_TRANSPORT_URL = 'tmdb://builtin';

export function isBuiltinTmdbAddon(addonOrTransportUrl: AddonDescriptor | string | undefined | null): boolean {
  const transportUrl = typeof addonOrTransportUrl === 'string' ? addonOrTransportUrl : addonOrTransportUrl?.transportUrl;
  return transportUrl === BUILTIN_TMDB_TRANSPORT_URL;
}

export async function builtinTmdbDescriptor(apiKey: string): Promise<AddonDescriptor | null> {
  if (!apiKey.trim()) return null;
  const manifestJson = await coreTmdbBuiltinManifest();
  const manifest = JSON.parse(manifestJson) as AddonDescriptor['manifest'];
  return { transportUrl: BUILTIN_TMDB_TRANSPORT_URL, manifest };
}

export async function withBuiltinTmdbAddon(
  addons: AddonDescriptor[],
  prefs: Record<string, unknown>,
): Promise<AddonDescriptor[]> {
  const apiKey = String(prefs.tmdbApiKey ?? '');
  const descriptor = await builtinTmdbDescriptor(apiKey);
  if (!descriptor) return addons;
  return prefs.tmdbPreferOverAddons ? [descriptor, ...addons] : [...addons, descriptor];
}

export async function fetchBuiltinCatalog(
  type: string,
  extra: Record<string, unknown>,
  apiKey: string,
  language: string,
  signal?: AbortSignal,
): Promise<{ metas: unknown[] }> {
  const url = await coreTmdbBuiltinCatalogUrl(type, extra, apiKey, language);
  if (!url) return { metas: [] };
  const data = (await tryFetchJson(url, { signal })) as { results?: unknown[] } | null;
  const items = data?.results ?? [];
  if (items.length === 0) return { metas: [] };
  const metas = (await coreTmdbBulkMetas(JSON.stringify(items), type, language)) ?? [];
  return { metas };
}

interface TmdbSeasonSummary {
  season_number?: number;
}

interface TmdbMetaUrls {
  tmdbId: string;
  detailsUrl: string;
  creditsUrl: string;
  imagesUrl: string;
  externalIdsUrl: string;
  keywordsUrl?: string;
  alternativeTitlesUrl?: string;
  contentRatingsUrl?: string;
  watchProvidersUrl?: string;
}

async function fetchBuiltinMetaUrls(
  type: string,
  id: string,
  apiKey: string,
  language: string,
  signal?: AbortSignal,
): Promise<TmdbMetaUrls | null> {
  const plan = await coreInvoke<{ findUrl?: string } & Partial<TmdbMetaUrls>>(
    'tmdbBuiltinMetaRequestPlan',
    JSON.stringify({ contentType: type, contentId: id, apiKey, language }),
  );
  if (!plan) return null;
  if (plan.detailsUrl && plan.creditsUrl && plan.imagesUrl && plan.externalIdsUrl && plan.tmdbId) {
    return plan as TmdbMetaUrls;
  }
  if (!plan.findUrl) return null;
  const find = await tryFetchJson(plan.findUrl, { signal });
  if (!find) return null;
  return coreInvoke<TmdbMetaUrls>(
    'tmdbBuiltinMetaUrlsFromFind',
    JSON.stringify({ find, contentType: type, apiKey, language }),
  );
}

export async function fetchBuiltinMeta(
  type: string,
  id: string,
  apiKey: string,
  language: string,
  signal?: AbortSignal,
): Promise<{ meta: unknown } | null> {
  const urls = await fetchBuiltinMetaUrls(type, id, apiKey, language, signal);
  if (!urls) return null;

  const [details, credits, images, externalIds, keywords, alternativeTitles, contentRatings, watchProviders] = await Promise.all([
    tryFetchJson(urls.detailsUrl, { signal }),
    tryFetchJson(urls.creditsUrl, { signal }),
    tryFetchJson(urls.imagesUrl, { signal }),
    tryFetchJson(urls.externalIdsUrl, { signal }),
    urls.keywordsUrl ? tryFetchJson(urls.keywordsUrl, { signal }) : null,
    urls.alternativeTitlesUrl ? tryFetchJson(urls.alternativeTitlesUrl, { signal }) : null,
    urls.contentRatingsUrl ? tryFetchJson(urls.contentRatingsUrl, { signal }) : null,
    urls.watchProvidersUrl ? tryFetchJson(urls.watchProvidersUrl, { signal }) : null,
  ]);
  if (!details) return null;

  const extras = { keywords, alternativeTitles, contentRatings, watchProviders };
  const meta = await coreTmdbFullMetaToMeta(
    JSON.stringify(details), JSON.stringify(credits ?? {}), JSON.stringify(images ?? {}),
    JSON.stringify(externalIds ?? {}), JSON.stringify(extras), type, language,
  ) as Record<string, unknown> | null;
  if (!meta) return null;

  if (type === 'series') {
    const seasons = ((details as { seasons?: TmdbSeasonSummary[] }).seasons ?? [])
      .map((s) => s.season_number)
      .filter((n): n is number => typeof n === 'number' && n > 0);
    const seriesId = String(meta.id ?? id);
    const videoLists = await runWithConcurrency(seasons, 4, (seasonNumber) =>
      fetchBuiltinSeasonVideos(urls.tmdbId, seasonNumber, seriesId, apiKey, language, signal));
    meta.videos = videoLists.flat();
  }

  return { meta };
}

export async function fetchTmdbLogo(
  type: string,
  id: string,
  apiKey: string,
  language: string,
  signal?: AbortSignal,
): Promise<string | null> {
  const urls = await fetchBuiltinMetaUrls(type, id, apiKey, language, signal);
  if (!urls) return null;
  const images = await tryFetchJson(urls.imagesUrl, { signal });
  if (!images) return null;
  const picked = await coreTmdbPickLogo(JSON.stringify(images), language);
  return picked?.logo ?? null;
}

async function fetchBuiltinSeasonVideos(
  tmdbId: string, season: number, seriesId: string, apiKey: string, language: string,
  signal?: AbortSignal,
): Promise<unknown[]> {
  const url = await coreInvoke<string>('tmdbSeasonRequestUrl', JSON.stringify({
    contentId: `tmdb:${tmdbId}`,
    season,
    apiKey,
    language,
  }));
  if (!url) return [];
  const seasonData = await tryFetchJson(url, { signal });
  if (!seasonData) return [];
  return (await coreTmdbEpisodesToVideos(JSON.stringify(seasonData), seriesId)) ?? [];
}

export async function fetchBuiltinSeasonEpisodes(
  seriesId: string, season: number, apiKey: string, language: string,
  signal?: AbortSignal,
): Promise<{ episodes: unknown[] }> {
  const urls = await fetchBuiltinMetaUrls('series', seriesId, apiKey, language, signal);
  if (!urls) return { episodes: [] };
  const videos = await fetchBuiltinSeasonVideos(urls.tmdbId, season, seriesId, apiKey, language, signal);
  return { episodes: videos };
}
