import {
  coreDetailSeriesLookupId,
  coreTmdbMetaToMeta,
  coreTmdbVideoToTrailer,
  dispatchAction,
} from './engine';
import { loadAddons, loadPrefs } from './libraryOps';
import { fetchPlannedResources } from './fetchPlanning';
import { tryFetchJson } from './httpClient';
import type { AppState, Video } from './types';
import { DEFAULT_APP_PREFS, prefBool, prefString } from './appPrefs';

interface TmdbRequest {
  contentType: string;
  id: string;
  language: string;
  apiKey: string;
}

interface TmdbMetaResult {
  id?: number;
  title?: string;
  name?: string;
  original_name?: string;
  release_date?: string;
  first_air_date?: string;
  media_type?: string;
  poster_path?: string | null;
  backdrop_path?: string | null;
}

interface TmdbVideoResult {
  id?: string;
  key?: string;
  name?: string;
  site?: string;
  type?: string;
}

async function fetchTmdbSimilarItems({
  contentType,
  id,
  language,
  apiKey,
  recommendationsEnabled,
  similarEnabled,
}: TmdbRequest & { recommendationsEnabled: boolean; similarEnabled: boolean }): Promise<unknown[]> {
  if (!apiKey || (!recommendationsEnabled && !similarEnabled)) return [];
  const tmdbId = await resolveTmdbId({ contentType, id, language, apiKey });
  if (!tmdbId) return [];
  const tmdbType = tmdbContentType(contentType);
  const calls = [
    recommendationsEnabled ? `recommendations` : null,
    similarEnabled ? `similar` : null,
  ].filter(Boolean) as string[];

  for (const path of calls) {
    const response = await tryFetchJson(tmdbUrl(`3/${tmdbType}/${tmdbId}/${path}`, apiKey, language));
    const rawItems = (response as { results?: TmdbMetaResult[] } | null)?.results ?? [];
    const results = (await Promise.all(rawItems.map((item) =>
      coreTmdbMetaToMeta(JSON.stringify(item), contentType, language)
    ))).filter(Boolean);
    if (results.length) return results;
  }
  return [];
}

async function fetchTmdbTrailers({ contentType, id, language, apiKey }: TmdbRequest): Promise<unknown[]> {
  if (!apiKey) return [];
  const tmdbId = await resolveTmdbId({ contentType, id, language, apiKey });
  if (!tmdbId) return [];
  const response = await tryFetchJson(tmdbUrl(`3/${tmdbContentType(contentType)}/${tmdbId}/videos`, apiKey, language));
  const rawVideos = (response as { results?: TmdbVideoResult[] } | null)?.results ?? [];
  return (await Promise.all(rawVideos.map((v) => coreTmdbVideoToTrailer(JSON.stringify(v))))).filter(Boolean);
}

async function resolveTmdbId({ contentType, id, language, apiKey }: TmdbRequest): Promise<string | null> {
  const baseId = id.replace(/^tmdb:/i, '').split(':')[0] ?? '';
  if (/^\d+$/.test(baseId)) return baseId;
  const imdbId = id.split(':')[0];
  if (!/^tt\d+$/i.test(imdbId)) return null;
  const response = await tryFetchJson(tmdbUrl(`3/find/${encodeURIComponent(imdbId)}`, apiKey, language, {
    external_source: 'imdb_id',
  }));
  const key = contentType === 'series' ? 'tv_results' : 'movie_results';
  const result = ((response as Record<string, TmdbMetaResult[]> | null)?.[key] ?? [])[0];
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
  if (!language || language === 'en') return 'en-US';
  if (language === 'tr') return 'tr-TR';
  return language.includes('-') ? language : `${language}-${language.toUpperCase()}`;
}

export async function fetchMetaDetail(payload: Record<string, unknown>): Promise<unknown> {
  const id = payload.id as string;
  const contentType = payload.contentType as string;
  const addons = await loadAddons();
  const values = await fetchPlannedResources({ kind: 'metaDetail', addons, contentType, id });
  return (values.find((value) => (value as { meta?: unknown }).meta) as { meta?: unknown } | undefined)?.meta ?? null;
}

export async function fetchMetaVideos(id: string, contentType: string): Promise<Video[]> {
  try {
    const meta = await fetchMetaDetail({ id, contentType }) as { videos?: Video[] } | null;
    return meta?.videos ?? [];
  } catch {
    return [];
  }
}

export async function fetchDetailStreams(
  payload: Record<string, unknown>,
  onStateUpdate?: (state: AppState) => void,
): Promise<unknown> {
  const idField = payload.id as string | undefined;
  const requestIds = (payload.requestIds as string[] | undefined) ?? (idField ? [idField] : []);
  const addons = await loadAddons();
  const contentType = payload.contentType as string;

  const partialDispatches: Promise<void>[] = [];

  const values = await fetchPlannedResources(
    { kind: 'streams', addons, contentType, requestIds },
    onStateUpdate
      ? (partialValue) => {
          const partialStreams = ((partialValue as { streams?: unknown[] })?.streams ?? []);
          if (partialStreams.length === 0) return;
          const partialAddons = [...new Set(
            (partialStreams as Array<{ addonName?: string }>)
              .map((s) => s.addonName)
              .filter(Boolean),
          )] as string[];
          partialDispatches.push(
            dispatchAction(JSON.stringify({
              type: 'detailStreamsAppended',
              streams: partialStreams,
              availableAddons: partialAddons,
            })).then((result) => {
              if (result?.state) onStateUpdate(result.state as AppState);
            }).catch(() => {}),
          );
        }
      : undefined,
  );

  // Ensure all partial dispatches complete before completeEffect runs
  await Promise.allSettled(partialDispatches);

  const streams = values.flatMap((value) => ((value as { streams?: unknown[] })?.streams ?? []));

  const availableAddons = [...new Set(
    (streams as Array<{ addonName?: string }>).map((s) => s.addonName).filter(Boolean),
  )] as string[];

  return {
    streams,
    availableAddons,
    hasStreamProviders: streams.length > 0,
  };
}

export async function fetchSeasonEpisodes(payload: Record<string, unknown>): Promise<unknown> {
  const addons = await loadAddons();
  const seriesId = await coreDetailSeriesLookupId(payload.seriesId as string);
  const season = payload.season as number;
  const values = await fetchPlannedResources({ kind: 'seasonEpisodes', addons, id: seriesId, season });
  return values.find((value) => (value as { episodes?: unknown[] })?.episodes?.length) ?? { episodes: [] };
}

export async function fetchDetailSecondary(payload: Record<string, unknown>): Promise<unknown> {
  const prefs = { ...DEFAULT_APP_PREFS, ...(await loadPrefs()) };
  const contentType = String(payload.contentType ?? payload.type ?? 'movie');
  const id = String(payload.id ?? '');
  const language = prefString(prefs, 'language', String(payload.language ?? 'en'));
  const apiKey = prefString(prefs, 'tmdbApiKey');

  const [similarItems, trailers] = await Promise.all([
    fetchTmdbSimilarItems({
      contentType,
      id,
      language,
      apiKey,
      recommendationsEnabled: prefBool(prefs, 'tmdbRecommendationsEnabled', true),
      similarEnabled: prefBool(prefs, 'tmdbSimilarResultsEnabled', true),
    }),
    prefBool(prefs, 'tmdbTrailersEnabled', true)
      ? fetchTmdbTrailers({ contentType, id, language, apiKey })
      : Promise.resolve([]),
  ]);

  return {
    watchedVideoIds: [],
    similarItems,
    trailers,
  };
}

export async function prefetchDetailStreams(payload: Record<string, unknown>): Promise<unknown> {
  return fetchDetailStreams(payload);
}
