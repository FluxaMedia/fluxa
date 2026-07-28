import {
  coreDetailSeriesLookupId,
  coreParseVideoId,
  coreTmdbImageUrl,
  coreTmdbBulkMetas,
  coreTmdbBulkVideosToTrailers,
  coreMdblistMediaInfoUrl,
  coreMdblistMediaRatingsFromResponse,
  dispatchAction,
} from './engine';
import { loadAddons, loadPrefs } from './libraryOps';
import { fetchPlannedResources } from './fetchPlanning';
import { tryFetchJson } from './httpClient';
import { fetchPluginStreams } from './pluginRuntime';
import { resolveTmdbId, tmdbContentType, tmdbUrl } from './tmdbShared';
import { fetchTraktSimilarItems, fetchSimklSimilarItems } from './similarTitles';
import type { AppState, Video } from './types';
import { DEFAULT_APP_PREFS, prefBool, prefString } from './appPrefs';
import { stringValue } from './playerUtils';

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
    if (!rawItems.length) continue;
    const results = (await coreTmdbBulkMetas(JSON.stringify(rawItems), contentType, language)) ?? [];
    if (results.length) return results;
  }
  return [];
}

export async function fetchTmdbTrailers({ contentType, id, language, apiKey }: TmdbRequest): Promise<unknown[]> {
  if (!apiKey) return [];
  const tmdbId = await resolveTmdbId({ contentType, id, language, apiKey });
  if (!tmdbId) return [];
  const response = await tryFetchJson(tmdbUrl(`3/${tmdbContentType(contentType)}/${tmdbId}/videos`, apiKey, language));
  const rawVideos = (response as { results?: TmdbVideoResult[] } | null)?.results ?? [];
  if (!rawVideos.length) return [];
  return (await coreTmdbBulkVideosToTrailers(JSON.stringify(rawVideos))) ?? [];
}

export async function fetchTmdbPosterFallback({
  contentType,
  id,
  language,
  apiKey,
}: TmdbRequest): Promise<{ poster?: string; background?: string } | null> {
  if (!apiKey) return null;
  const tmdbId = await resolveTmdbId({ contentType, id, language, apiKey });
  if (!tmdbId) return null;
  const response = await tryFetchJson(
    tmdbUrl(`3/${tmdbContentType(contentType)}/${tmdbId}`, apiKey, language),
  ) as TmdbMetaResult | null;
  if (!response) return null;
  const poster = await coreTmdbImageUrl(response.poster_path ?? null, 'w500');
  const background = await coreTmdbImageUrl(response.backdrop_path ?? null, 'w1280');
  if (!poster && !background) return null;
  return { poster: poster ?? undefined, background: background ?? undefined };
}

async function resolveImdbId({ contentType, id, language, apiKey }: TmdbRequest): Promise<string | undefined> {
  const parsed = await coreParseVideoId(id);
  if (parsed.imdb) return parsed.imdb;
  if (!apiKey) return undefined;
  const tmdbId = await resolveTmdbId({ contentType, id, language, apiKey });
  if (!tmdbId) return undefined;
  const response = await tryFetchJson(
    tmdbUrl(`3/${tmdbContentType(contentType)}/${tmdbId}/external_ids`, apiKey, language),
  ) as { imdb_id?: string | null } | null;
  return response?.imdb_id ?? undefined;
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

async function fetchPluginStreamsForDetail(
  contentType: string,
  id: string | undefined,
  detail: unknown,
): Promise<Array<Record<string, unknown>>> {
  if (!id) return [];
  try {
    const prefs = { ...DEFAULT_APP_PREFS, ...(await loadPrefs()) };
    const apiKey = prefString(prefs, 'tmdbApiKey');
    const language = prefString(prefs, 'language', 'en');
    const detailRecord = detail && typeof detail === 'object' && !Array.isArray(detail)
      ? detail as Record<string, unknown>
      : {};
    const detailIds = detailRecord.ids && typeof detailRecord.ids === 'object'
      ? detailRecord.ids as Record<string, unknown>
      : {};
    const embeddedTmdbId = [detailRecord.tmdbId, detailRecord.tmdb_id, detailIds.tmdb]
      .map((value) => typeof value === 'number' || typeof value === 'string' ? String(value).trim() : '')
      .find((value) => /^\d+$/.test(value));
    const [parsed, resolvedTmdbId] = await Promise.all([
      coreParseVideoId(id),
      resolveTmdbId({ contentType, id, language, apiKey }),
    ]);
    const pluginContentId = embeddedTmdbId || resolvedTmdbId || parsed.imdb;
    if (!pluginContentId) return [];
    return await fetchPluginStreams(contentType, pluginContentId, parsed.season, parsed.episode);
  } catch {
    return [];
  }
}

export async function fetchDetailStreams(
  payload: Record<string, unknown>,
  onStateUpdate?: (state: Partial<AppState>) => void,
  generation?: number,
): Promise<unknown> {
  const requestIds = (payload.requestIds as string[] | undefined) ?? (typeof payload.id === 'string' ? [payload.id] : []);
  const idField = (typeof payload.id === 'string' ? payload.id : undefined) ?? requestIds[0];
  const addons = await loadAddons();
  const contentType = payload.contentType as string;

  const partialDispatches: Promise<void>[] = [];
  const failedAddonNames = new Set<string>();

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
              generation,
            })).then((result) => {
              if (result?.state) onStateUpdate(result.state);
            }).catch(() => {}),
          );
        }
      : undefined,
    undefined,
    (addonName) => failedAddonNames.add(addonName),
  );

  // Ensure all partial dispatches complete before completeEffect runs
  await Promise.allSettled(partialDispatches);

  const streams = values.flatMap((value) => ((value as { streams?: unknown[] })?.streams ?? []));

  const pluginStreams = await fetchPluginStreamsForDetail(contentType, idField, payload.detail);
  if (pluginStreams.length > 0) streams.push(...pluginStreams);

  const availableAddons = [...new Set(
    (streams as Array<{ addonName?: string }>).map((s) => s.addonName).filter(Boolean),
  )] as string[];

  for (const addonName of availableAddons) failedAddonNames.delete(addonName);

  return {
    streams,
    availableAddons,
    failedAddons: [...failedAddonNames],
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

interface OmdbRatings {
  rottenTomatoes?: string;
  metascore?: string;
}

async function fetchOmdbRatings(id: string, apiKey: string): Promise<OmdbRatings | null> {
  if (!apiKey) return null;
  const imdbId = id.split(':')[0];
  if (!/^tt\d+$/i.test(imdbId)) return null;
  const response = await tryFetchJson(`https://www.omdbapi.com/?i=${encodeURIComponent(imdbId)}&apikey=${apiKey}`) as {
    Ratings?: { Source?: string; Value?: string }[];
    Metascore?: string;
  } | null;
  if (!response) return null;
  const rottenTomatoes = response.Ratings?.find((r) => r.Source === 'Rotten Tomatoes')?.Value;
  const metascore = response.Metascore && response.Metascore !== 'N/A' ? response.Metascore : undefined;
  if (!rottenTomatoes && !metascore) return null;
  return { rottenTomatoes, metascore };
}

async function fetchMdblistRatings(
  contentType: string,
  id: string,
  apiKey: string,
): Promise<Record<string, number> | null> {
  if (!apiKey) return null;
  const baseId = id.split(':')[0];
  let provider: string;
  let mediaId: string;
  if (/^tt\d+$/i.test(baseId)) {
    provider = 'imdb';
    mediaId = baseId;
  } else if (id.startsWith('tmdb:') && /^\d+$/.test(id.split(':')[1] ?? '')) {
    provider = 'tmdb';
    mediaId = id.split(':')[1] ?? '';
  } else {
    return null;
  }
  if (!mediaId) return null;
  const mediaType = contentType === 'series' ? 'show' : 'movie';
  const url = await coreMdblistMediaInfoUrl(provider, mediaType, mediaId, 'ratings');
  if (!url) return null;
  const separator = url.includes('?') ? '&' : '?';
  const response = await tryFetchJson(`${url}${separator}apikey=${encodeURIComponent(apiKey)}`);
  if (!response) return null;
  return coreMdblistMediaRatingsFromResponse(JSON.stringify(response));
}

interface FanartArtwork {
  hdLogo?: string;
  hdBackdrop?: string;
}

async function resolveTvdbId(tmdbId: string, apiKey: string, language: string): Promise<string | null> {
  const response = await tryFetchJson(
    tmdbUrl(`3/tv/${tmdbId}/external_ids`, apiKey, language),
  ) as { tvdb_id?: number | null } | null;
  return response?.tvdb_id != null ? String(response.tvdb_id) : null;
}

async function fetchFanartArtwork(
  { contentType, id, language, apiKey }: TmdbRequest,
  fanartApiKey: string,
): Promise<FanartArtwork | null> {
  if (!fanartApiKey || !apiKey) return null;
  const tmdbId = await resolveTmdbId({ contentType, id, language, apiKey });
  if (!tmdbId) return null;

  if (contentType === 'series') {
    const tvdbId = await resolveTvdbId(tmdbId, apiKey, language);
    if (!tvdbId) return null;
    const response = await tryFetchJson(`https://webservice.fanart.tv/v3/tv/${tvdbId}?api_key=${fanartApiKey}`) as {
      hdtvlogo?: { url?: string }[];
      showbackground?: { url?: string }[];
    } | null;
    if (!response) return null;
    const hdLogo = response.hdtvlogo?.[0]?.url;
    const hdBackdrop = response.showbackground?.[0]?.url;
    if (!hdLogo && !hdBackdrop) return null;
    return { hdLogo, hdBackdrop };
  }

  const response = await tryFetchJson(`https://webservice.fanart.tv/v3/movies/${tmdbId}?api_key=${fanartApiKey}`) as {
    hdmovielogo?: { url?: string }[];
    moviebackground?: { url?: string }[];
  } | null;
  if (!response) return null;
  const hdLogo = response.hdmovielogo?.[0]?.url;
  const hdBackdrop = response.moviebackground?.[0]?.url;
  if (!hdLogo && !hdBackdrop) return null;
  return { hdLogo, hdBackdrop };
}

export async function fetchContentLogo(
  id: string,
  contentType: string,
  language: string,
  apiKey: string,
  fanartApiKey: string,
): Promise<string | undefined> {
  try {
    const meta = await fetchMetaDetail({ id, contentType }) as Record<string, unknown> | null;
    const addonLogo = meta
      ? stringValue(meta.logo) ?? stringValue(meta.logoUrl) ?? stringValue(meta.titleLogo) ?? stringValue(meta.titleLogoUrl)
      : undefined;
    if (addonLogo) return addonLogo;
  } catch {}
  const artwork = await fetchFanartArtwork({ contentType, id, language, apiKey }, fanartApiKey);
  return artwork?.hdLogo;
}

async function fetchSimilarItems({
  contentType,
  id,
  language,
  apiKey,
  source,
  recommendationsEnabled,
  similarEnabled,
}: TmdbRequest & { source: string; recommendationsEnabled: boolean; similarEnabled: boolean }): Promise<unknown[]> {
  const tmdbFallback = () => fetchTmdbSimilarItems({
    contentType,
    id,
    language,
    apiKey,
    recommendationsEnabled,
    similarEnabled,
  });

  if (source === 'tmdb') return tmdbFallback();

  const imdbId = await resolveImdbId({ contentType, id, language, apiKey });
  if (!imdbId) return tmdbFallback();

  if (source === 'trakt') {
    const items = await fetchTraktSimilarItems({ imdbId, contentType });
    return items.length ? items : tmdbFallback();
  }

  if (source === 'simkl') {
    const items = await fetchSimklSimilarItems({ imdbId, contentType });
    return items.length ? items : tmdbFallback();
  }

  const traktItems = await fetchTraktSimilarItems({ imdbId, contentType });
  if (traktItems.length) return traktItems;
  const simklItems = await fetchSimklSimilarItems({ imdbId, contentType });
  if (simklItems.length) return simklItems;
  return tmdbFallback();
}

export async function fetchDetailSecondary(payload: Record<string, unknown>): Promise<unknown> {
  const prefs = { ...DEFAULT_APP_PREFS, ...(await loadPrefs()) };
  const contentType = String(payload.contentType ?? payload.type ?? 'movie');
  const id = String(payload.id ?? '');
  const language = prefString(prefs, 'language', String(payload.language ?? 'en'));
  const apiKey = prefString(prefs, 'tmdbApiKey');
  const omdbApiKey = prefString(prefs, 'omdbApiKey');
  const mdblistApiKey = prefString(prefs, 'mdblistApiKey');
  const fanartApiKey = prefString(prefs, 'fanartApiKey');
  const requestedSource = String(payload.similarTitlesSource ?? '');
  const source = ['auto', 'trakt', 'simkl', 'tmdb'].includes(requestedSource)
    ? requestedSource
    : prefString(prefs, 'similarTitlesSource', 'auto');
  const recommendationsEnabled = prefBool(prefs, 'tmdbRecommendationsEnabled', true);
  const similarEnabled = prefBool(prefs, 'tmdbSimilarResultsEnabled', true);
  const shouldFetchSimilar = source !== 'tmdb' || recommendationsEnabled || similarEnabled;

  const [similarItems, trailers, omdbRatings, mdblistRatings, fanartArtwork] = await Promise.all([
    shouldFetchSimilar
      ? fetchSimilarItems({
          contentType,
          id,
          language,
          apiKey,
          source,
          recommendationsEnabled,
          similarEnabled,
        })
      : Promise.resolve([]),
    prefBool(prefs, 'tmdbTrailersEnabled', true)
      ? fetchTmdbTrailers({ contentType, id, language, apiKey })
      : Promise.resolve([]),
    fetchOmdbRatings(id, omdbApiKey),
    fetchMdblistRatings(contentType, id, mdblistApiKey),
    fetchFanartArtwork({ contentType, id, language, apiKey }, fanartApiKey),
  ]);

  return {
    watchedVideoIds: [],
    similarItems,
    trailers,
    omdbRatings,
    mdblistRatings,
    fanartArtwork,
  };
}

export async function prefetchDetailStreams(payload: Record<string, unknown>): Promise<unknown> {
  return fetchDetailStreams(payload);
}
