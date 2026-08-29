import {
  coreDetailSeriesLookupId,
  coreStreamRequestIds,
  coreParseVideoId,
  coreTmdbImageUrl,
  coreTmdbBulkMetas,
  coreTmdbBulkVideosToTrailers,
  coreTmdbMergeEnrichment,
  coreTerminalRecommendationPlan,
  coreMdblistMediaInfoUrl,
  coreMdblistMediaRatingsFromResponse,
  coreInvoke,
  getSnapshot,
  storageRead,
  storageWrite,
} from './engine';
import { loadActiveProfile, loadEnabledAddons, loadLibrary, loadPrefs } from './libraryOps';
import { fetchPlannedResources } from './fetchPlanning';
import { fetchBuiltinMeta, fetchTmdbLogo } from './tmdbAddon';
import { tryFetchJson } from './httpClient';
import { fetchPluginStreams } from './pluginRuntime';
import { fetchTraktSimilarItems, fetchSimklSimilarItems } from './similarTitles';
import type { AppState, Meta, Video } from './types';
import { DEFAULT_APP_PREFS, prefBool, prefString } from './appPrefs';
import { stringValue } from './playerUtils';
import { startPerfSpan } from './performance';

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

interface TmdbDetailRequestPlan {
  tmdbId: string;
  urls: Record<string, string>;
}

async function tmdbDetailRequests(
  { contentType, id, language, apiKey }: TmdbRequest,
  endpoints: string[],
  signal?: AbortSignal,
): Promise<TmdbDetailRequestPlan | null> {
  const plan = await coreInvoke<Partial<TmdbDetailRequestPlan> & { findUrl?: string }>(
    'tmdbDetailRequestPlan',
    JSON.stringify({ contentType, contentId: id, language, apiKey, endpoints }),
  );
  if (!plan) return null;
  if (plan.tmdbId && plan.urls) return plan as TmdbDetailRequestPlan;
  if (!plan.findUrl) return null;
  const find = await tryFetchJson(plan.findUrl, { signal });
  if (!find) return null;
  return coreInvoke<TmdbDetailRequestPlan>(
    'tmdbDetailRequestUrlsFromFind',
    JSON.stringify({ find, contentType, language, apiKey, endpoints }),
  );
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
  const calls = [recommendationsEnabled ? `recommendations` : null, similarEnabled ? `similar` : null].filter(Boolean) as string[];
  const plan = await tmdbDetailRequests({ contentType, id, language, apiKey }, calls);
  if (!plan) return [];

  const responses = await Promise.all(calls.map((path) => tryFetchJson(plan.urls[path])));
  for (const response of responses) {
    const rawItems = (response as { results?: TmdbMetaResult[] } | null)?.results ?? [];
    if (!rawItems.length) continue;
    const results = (await coreTmdbBulkMetas(JSON.stringify(rawItems), contentType, language)) ?? [];
    if (results.length) return results;
  }
  return [];
}

export async function fetchTmdbTrailers({ contentType, id, language, apiKey }: TmdbRequest): Promise<unknown[]> {
  if (!apiKey) return [];
  const plan = await tmdbDetailRequests({ contentType, id, language, apiKey }, ['videos']);
  if (!plan) return [];
  const response = await tryFetchJson(plan.urls.videos);
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
  const plan = await tmdbDetailRequests({ contentType, id, language, apiKey }, ['details']);
  if (!plan) return null;
  const response = (await tryFetchJson(plan.urls.details)) as TmdbMetaResult | null;
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
  const plan = await tmdbDetailRequests({ contentType, id, language, apiKey }, ['external_ids']);
  if (!plan) return undefined;
  const response = (await tryFetchJson(plan.urls.external_ids)) as { imdb_id?: string | null } | null;
  return response?.imdb_id ?? undefined;
}

const TMDB_ENRICHMENT_FLAG_KEYS = {
  artwork: 'tmdbEnrichArtworkEnabled',
  description: 'tmdbEnrichDescriptionEnabled',
  genresKeywords: 'tmdbEnrichGenresKeywordsEnabled',
  castCrew: 'tmdbEnrichCastCrewEnabled',
  network: 'tmdbEnrichNetworkEnabled',
  ratings: 'tmdbRatingsEnabled',
  collection: 'tmdbCollectionInfoEnabled',
  statusSchedule: 'tmdbEnrichStatusScheduleEnabled',
  originTitles: 'tmdbEnrichOriginTitlesEnabled',
  watchProviders: 'tmdbEnrichWatchProvidersEnabled',
  episodeStills: 'tmdbEpisodeImagesEnabled',
} as const;

async function enrichMetaWithTmdb(meta: unknown, contentType: string, id: string): Promise<unknown> {
  const prefs = { ...DEFAULT_APP_PREFS, ...(await loadPrefs()) };
  const apiKey = prefString(prefs, 'tmdbApiKey');
  if (!apiKey) return meta;

  const flags = Object.fromEntries(
    Object.entries(TMDB_ENRICHMENT_FLAG_KEYS).map(([field, prefKey]) => [field, prefBool(prefs, prefKey, true)]),
  );
  if (!Object.values(flags).some(Boolean)) return meta;

  const language = prefString(prefs, 'language', 'en');
  const tmdbResult = await fetchBuiltinMeta(contentType, id, apiKey, language);
  if (!tmdbResult?.meta) return meta;

  const merged = await coreTmdbMergeEnrichment(JSON.stringify(meta), JSON.stringify(tmdbResult.meta), JSON.stringify(flags));
  return merged ?? meta;
}

export function fetchMetaDetail(payload: Record<string, unknown>): Promise<unknown> {
  return fetchMetaDetailUncached(payload);
}

const metaDetailInFlight = new Map<string, Promise<unknown>>();

export function fetchMetaDetailDeduped(payload: Record<string, unknown>): Promise<unknown> {
  const id = typeof payload.id === 'string' ? payload.id : '';
  const contentType = typeof payload.contentType === 'string' ? payload.contentType : '';
  const transportUrl = typeof payload.sourceAddonTransportUrl === 'string' ? payload.sourceAddonTransportUrl : '';
  const key = `${contentType}:${id}:${transportUrl}`;
  const existing = metaDetailInFlight.get(key);
  if (existing) return existing;
  const promise = fetchMetaDetailUncached(payload).finally(() => metaDetailInFlight.delete(key));
  metaDetailInFlight.set(key, promise);
  return promise;
}

async function fetchMetaDetailUncached(payload: Record<string, unknown>): Promise<unknown> {
  const perf = startPerfSpan('detail.meta', { id: payload.id, contentType: payload.contentType, purpose: payload.purpose ?? 'unknown' });
  const startedAt = performance.now();
  const id = payload.id as string;
  const contentType = payload.contentType as string;
  const purpose = typeof payload.purpose === 'string' ? payload.purpose : 'unknown';
  console.debug('[fluxa:detail:meta:start]', JSON.stringify({ id, contentType, purpose }));
  const transportUrl = typeof payload.sourceAddonTransportUrl === 'string' ? payload.sourceAddonTransportUrl : undefined;
  const addons = await loadEnabledAddons();
  console.debug('[fluxa:detail:meta:addons-ready]', JSON.stringify({ id, purpose, count: addons.length, ms: Math.round(performance.now() - startedAt) }));
  const values = await fetchPlannedResources({ kind: 'metaDetail', addons, contentType, id, transportUrl, traceId: perf.traceId });
  console.debug('[fluxa:detail:meta:resources-ready]', JSON.stringify({ id, purpose, values: values.length, ms: Math.round(performance.now() - startedAt) }));
  const winner = values.find((value) => (value as { meta?: unknown }).meta) as { meta?: unknown; __tmdbSourced?: boolean } | undefined;
  if (!winner?.meta) {
    perf.end({ result: 'empty', values: values.length });
    console.debug('[fluxa:detail:meta:end]', JSON.stringify({ id, purpose, meta: false, ms: Math.round(performance.now() - startedAt) }));
    return null;
  }
  if (winner.__tmdbSourced) {
    perf.end({ result: 'tmdb', values: values.length });
    console.debug('[fluxa:detail:meta:end]', JSON.stringify({ id, purpose, meta: true, tmdb: true, ms: Math.round(performance.now() - startedAt) }));
    return winner.meta;
  }
  const result = await enrichMetaWithTmdb(winner.meta, contentType, id);
  perf.end({ result: 'success', values: values.length });
  console.debug('[fluxa:detail:meta:end]', JSON.stringify({ id, purpose, meta: true, tmdb: false, ms: Math.round(performance.now() - startedAt) }));
  return result;
}

export async function fetchMetaVideos(id: string, contentType: string): Promise<Video[]> {
  try {
    const meta = (await fetchMetaDetail({ id, contentType, purpose: 'player-videos' })) as { videos?: Video[] } | null;
    return meta?.videos ?? [];
  } catch {
    return [];
  }
}

async function fetchPluginStreamsForDetail(
  contentType: string,
  id: string | undefined,
  detail: unknown,
  signal?: AbortSignal,
  onScraperStreams?: (streams: Array<Record<string, unknown>>) => void,
): Promise<Array<Record<string, unknown>>> {
  if (!id) return [];
  try {
    // Do not perform the TMDB/id-resolution work until we know that at least
    // one compatible scraper is installed. Addon stream discovery must stay
    // on the critical path; plugin enrichment is optional and can arrive late.
    const snapshot = (await getSnapshot()) as { plugins?: { scrapers?: Array<{ enabled?: boolean; supportedTypes?: string[] }> } } | null;
    const installedScrapers = snapshot?.plugins?.scrapers ?? [];
    const scraperMediaType = contentType === 'series' || contentType === 'show' ? 'tv' : contentType;
    if (!installedScrapers.some((scraper) => scraper.enabled !== false && (!scraper.supportedTypes || scraper.supportedTypes.includes(scraperMediaType)))) {
      return [];
    }
    const prefs = { ...DEFAULT_APP_PREFS, ...(await loadPrefs()) };
    const apiKey = prefString(prefs, 'tmdbApiKey');
    const language = prefString(prefs, 'language', 'en');
    const detailRecord = detail && typeof detail === 'object' && !Array.isArray(detail) ? (detail as Record<string, unknown>) : {};
    const detailIds = detailRecord.ids && typeof detailRecord.ids === 'object' ? (detailRecord.ids as Record<string, unknown>) : {};
    const embeddedTmdbId = [detailRecord.tmdbId, detailRecord.tmdb_id, detailIds.tmdb]
      .map((value) => (typeof value === 'number' || typeof value === 'string' ? String(value).trim() : ''))
      .find((value) => /^\d+$/.test(value));
    const [parsed, tmdbPlan] = await Promise.all([
      coreParseVideoId(id),
      tmdbDetailRequests({ contentType, id, language, apiKey }, [], signal),
    ]);
    const pluginContentId = embeddedTmdbId || tmdbPlan?.tmdbId || parsed.imdb;
    if (!pluginContentId) return [];
    return await fetchPluginStreams(contentType, pluginContentId, parsed.season, parsed.episode, signal, onScraperStreams);
  } catch {
    return [];
  }
}

export async function fetchDetailStreams(
  payload: Record<string, unknown>,
  onStateUpdate?: (state: Partial<AppState>) => void,
  generation?: number,
  signal?: AbortSignal,
): Promise<unknown> {
  const perf = startPerfSpan('detail.streams', { requestIds: payload.requestIds, contentType: payload.contentType });
  const startedAt = performance.now();
  const requestedIds = (payload.requestIds as string[] | undefined) ?? (typeof payload.id === 'string' ? [payload.id] : []);
  const idField = (typeof payload.id === 'string' ? payload.id : undefined) ?? requestedIds[0];
  const detailRecord = payload.detail && typeof payload.detail === 'object' && !Array.isArray(payload.detail)
    ? payload.detail as Record<string, unknown>
    : {};
  const detailId = typeof detailRecord.id === 'string' ? detailRecord.id : undefined;
  const currentSeriesLookupId = detailId && payload.contentType === 'series'
    ? await coreDetailSeriesLookupId(detailId)
    : undefined;
  const requestIds = [...new Set((await Promise.all(requestedIds.map((id) => coreStreamRequestIds({
    contentType: String(payload.contentType ?? ''),
    id,
    detailId,
    currentSeriesLookupId,
  })))).flat())];
  console.debug('[fluxa:streams:effect:start]', JSON.stringify({ requestedIds, requestIds, contentType: payload.contentType }));
  const addons = await loadEnabledAddons();
  console.debug('[fluxa:streams:addons:ready]', JSON.stringify({ requestIds, addons: addons.map((addon) => addon.name), ms: Math.round(performance.now() - startedAt) }));
  const contentType = payload.contentType as string;

  let publishedStreams: unknown[] = [];
  let publishedAddons: string[] = [];
  const failedAddonNames = new Set<string>();

  const append = (incoming: unknown[]) => {
    if (!onStateUpdate || incoming.length === 0) return;
    publishedStreams = [...publishedStreams, ...incoming];
    publishedAddons = [
      ...new Set(
        (publishedStreams as Array<{ addonName?: string }>)
          .map((stream) => stream.addonName)
          .filter((name): name is string => Boolean(name)),
      ),
    ];
    console.debug('[fluxa:streams:partial]', JSON.stringify({
      requestIds,
      incoming: incoming.length,
      total: publishedStreams.length,
      ms: Math.round(performance.now() - startedAt),
    }));
    // The stream list is a high-frequency UI snapshot. Do not dispatch every
    // addon response through the JSON core boundary; completion below still
    // commits the authoritative list to FluxaCore.
    onStateUpdate({
      detail: {
        streams: publishedStreams as AppState['detail']['streams'],
        visibleStreams: publishedStreams as AppState['detail']['visibleStreams'],
        availableAddons: publishedAddons,
      },
    });
  };

  const pluginStreamsPromise = fetchPluginStreamsForDetail(contentType, idField, payload.detail, signal, append);
  const values = await fetchPlannedResources(
    { kind: 'streams', addons, contentType, requestIds, traceId: perf.traceId },
    (partialValue) => append((partialValue as { streams?: unknown[] })?.streams ?? []),
    signal,
    (addonName) => failedAddonNames.add(addonName),
  );
  console.debug('[fluxa:streams:addons:complete]', JSON.stringify({ requestIds, values: values.length, ms: Math.round(performance.now() - startedAt) }));

  const streams = values.flatMap((value) => (value as { streams?: unknown[] })?.streams ?? []);
  if (streams.length > 0) {
    // Addon results are sufficient to finish the critical path. Plugin
    // scrapers may append compatible streams later using the same generation.
    void pluginStreamsPromise.then((pluginStreams) => {
      if (pluginStreams.length > 0) append(pluginStreams);
    }).catch(() => {});
  } else {
    // If addons returned nothing, give optional scrapers a chance before
    // declaring the source panel empty.
    const pluginStreams = await pluginStreamsPromise;
    if (pluginStreams.length > 0) streams.push(...pluginStreams);
  }

  const availableAddons = [...new Set((streams as Array<{ addonName?: string }>).map((s) => s.addonName).filter(Boolean))] as string[];

  for (const addonName of availableAddons) failedAddonNames.delete(addonName);

  console.debug('[fluxa:streams:effect:end]', JSON.stringify({ requestIds, streams: streams.length, ms: Math.round(performance.now() - startedAt) }));
  perf.end({ streams: streams.length, providers: availableAddons.length });
  return {
    streams,
    availableAddons,
    // A healthy addon result is enough for playback. Do not surface unrelated
    // optional-addon failures as a global error when the source panel already
    // has usable streams.
    failedAddons: streams.length > 0 ? [] : [...failedAddonNames],
    hasStreamProviders: streams.length > 0,
  };
}

export async function fetchSeasonEpisodes(payload: Record<string, unknown>): Promise<unknown> {
  const perf = startPerfSpan('detail.episodes', { seriesId: payload.seriesId, season: payload.season });
  const addons = await loadEnabledAddons();
  const seriesId = await coreDetailSeriesLookupId(payload.seriesId as string);
  const season = payload.season as number;
  const values = await fetchPlannedResources({ kind: 'seasonEpisodes', addons, id: seriesId, season, traceId: perf.traceId });
  const result = values.find((value) => (value as { episodes?: unknown[] })?.episodes?.length) ?? { episodes: [] };
  perf.end({ episodes: Array.isArray((result as { episodes?: unknown[] }).episodes) ? (result as { episodes: unknown[] }).episodes.length : 0 });
  return result;
}

interface OmdbRatings {
  rottenTomatoes?: string;
  metascore?: string;
}

async function fetchOmdbRatings(id: string, apiKey: string): Promise<OmdbRatings | null> {
  if (!apiKey) return null;
  const imdbId = id.split(':')[0];
  if (!/^tt\d+$/i.test(imdbId)) return null;
  const response = (await tryFetchJson(`https://www.omdbapi.com/?i=${encodeURIComponent(imdbId)}&apikey=${apiKey}`)) as {
    Ratings?: { Source?: string; Value?: string }[];
    Metascore?: string;
  } | null;
  if (!response) return null;
  const rottenTomatoes = response.Ratings?.find((r) => r.Source === 'Rotten Tomatoes')?.Value;
  const metascore = response.Metascore && response.Metascore !== 'N/A' ? response.Metascore : undefined;
  if (!rottenTomatoes && !metascore) return null;
  return { rottenTomatoes, metascore };
}

async function fetchMdblistRatings(contentType: string, id: string, apiKey: string): Promise<Record<string, number> | null> {
  if (!apiKey) return null;
  try {
    const tmdbBaseId = id.replace(/^tmdb:/i, '').split(':')[0] ?? '';
    const imdbBaseId = id.split(':')[0] ?? '';
    let provider: string;
    let mediaId: string;
    if (/^\d+$/.test(tmdbBaseId)) {
      provider = 'tmdb';
      mediaId = tmdbBaseId;
    } else if (/^tt\d+$/i.test(imdbBaseId)) {
      provider = 'imdb';
      mediaId = imdbBaseId;
    } else {
      return null;
    }
    const mediaType = contentType === 'series' ? 'show' : 'movie';
    const url = await coreMdblistMediaInfoUrl(provider, mediaType, mediaId, 'ratings');
    if (!url) return null;
    const separator = url.includes('?') ? '&' : '?';
    const response = await tryFetchJson(`${url}${separator}apikey=${encodeURIComponent(apiKey)}`);
    if (!response) return null;
    const ratings = await coreMdblistMediaRatingsFromResponse(JSON.stringify(response));
    return ratings;
  } catch (err) {
    console.error('fetchMdblistRatings failed', err);
    return null;
  }
}

interface FanartArtwork {
  hdLogo?: string;
  hdBackdrop?: string;
}

async function resolveTvdbId(tmdbId: string, apiKey: string, language: string): Promise<string | null> {
  const plan = await tmdbDetailRequests(
    {
      contentType: 'series',
      id: `tmdb:${tmdbId}`,
      apiKey,
      language,
    },
    ['external_ids'],
  );
  if (!plan) return null;
  const response = (await tryFetchJson(plan.urls.external_ids)) as { tvdb_id?: number | null } | null;
  return response?.tvdb_id != null ? String(response.tvdb_id) : null;
}

async function fetchFanartArtwork({ contentType, id, language, apiKey }: TmdbRequest, fanartApiKey: string): Promise<FanartArtwork | null> {
  if (!fanartApiKey || !apiKey) return null;
  const plan = await tmdbDetailRequests({ contentType, id, language, apiKey }, []);
  if (!plan) return null;
  const tmdbId = plan.tmdbId;

  if (contentType === 'series') {
    const tvdbId = await resolveTvdbId(tmdbId, apiKey, language);
    if (!tvdbId) return null;
    const response = (await tryFetchJson(`https://webservice.fanart.tv/v3/tv/${tvdbId}?api_key=${fanartApiKey}`)) as {
      hdtvlogo?: { url?: string }[];
      showbackground?: { url?: string }[];
    } | null;
    if (!response) return null;
    const hdLogo = response.hdtvlogo?.[0]?.url;
    const hdBackdrop = response.showbackground?.[0]?.url;
    if (!hdLogo && !hdBackdrop) return null;
    return { hdLogo, hdBackdrop };
  }

  const response = (await tryFetchJson(`https://webservice.fanart.tv/v3/movies/${tmdbId}?api_key=${fanartApiKey}`)) as {
    hdmovielogo?: { url?: string }[];
    moviebackground?: { url?: string }[];
  } | null;
  if (!response) return null;
  const hdLogo = response.hdmovielogo?.[0]?.url;
  const hdBackdrop = response.moviebackground?.[0]?.url;
  if (!hdLogo && !hdBackdrop) return null;
  return { hdLogo, hdBackdrop };
}

const CONTENT_LOGO_CACHE_KEY = 'content_logo_cache';
const CONTENT_LOGO_CACHE_TTL_MS = 24 * 60 * 60 * 1000;
const CONTENT_LOGO_CACHE_NEGATIVE_TTL_MS = 10 * 60 * 1000;
const CONTENT_LOGO_CACHE_MAX_AGE_MS = 30 * 24 * 60 * 60 * 1000;

type ContentLogoCache = Record<string, { fetchedAt: number; logo: string | null }>;

let contentLogoCachePromise: Promise<ContentLogoCache> | null = null;
function loadContentLogoCache(): Promise<ContentLogoCache> {
  if (!contentLogoCachePromise) {
    contentLogoCachePromise = storageRead<ContentLogoCache>(CONTENT_LOGO_CACHE_KEY).then((cache) => cache ?? {});
  }
  return contentLogoCachePromise;
}
void loadContentLogoCache();

const contentLogoInFlight = new Map<string, Promise<string | undefined>>();

export function fetchContentLogo(
  id: string,
  contentType: string,
  language: string,
  apiKey: string,
  fanartApiKey: string,
): Promise<string | undefined> {
  const cacheKey = `${contentType}:${id}`;
  const inFlight = contentLogoInFlight.get(cacheKey);
  if (inFlight) return inFlight;

  const promise = fetchContentLogoUncached(cacheKey, id, contentType, language, apiKey, fanartApiKey).finally(() =>
    contentLogoInFlight.delete(cacheKey),
  );
  contentLogoInFlight.set(cacheKey, promise);
  return promise;
}

async function fetchContentLogoUncached(
  cacheKey: string,
  id: string,
  contentType: string,
  language: string,
  apiKey: string,
  fanartApiKey: string,
): Promise<string | undefined> {
  const now = Date.now();
  const cache = await loadContentLogoCache();
  const cached = cache[cacheKey];
  if (cached) {
    const ttl = cached.logo ? CONTENT_LOGO_CACHE_TTL_MS : CONTENT_LOGO_CACHE_NEGATIVE_TTL_MS;
    if (now - cached.fetchedAt < ttl) return cached.logo ?? undefined;
  }

  let logo: string | undefined;
  try {
    const meta = (await fetchMetaDetail({ id, contentType, purpose: 'content-logo' })) as Record<string, unknown> | null;
    const addonLogo = meta
      ? (stringValue(meta.logo) ?? stringValue(meta.logoUrl) ?? stringValue(meta.titleLogo) ?? stringValue(meta.titleLogoUrl))
      : undefined;
    if (addonLogo) logo = addonLogo;
  } catch {}
  if (!logo && apiKey) {
    try {
      logo = (await fetchTmdbLogo(contentType, id, apiKey, language)) ?? undefined;
    } catch {}
  }
  if (!logo) {
    const artwork = await fetchFanartArtwork({ contentType, id, language, apiKey }, fanartApiKey);
    logo = artwork?.hdLogo;
  }

  cache[cacheKey] = { fetchedAt: now, logo: logo ?? null };
  for (const [key, entry] of Object.entries(cache)) {
    if (now - entry.fetchedAt > CONTENT_LOGO_CACHE_MAX_AGE_MS) delete cache[key];
  }
  void storageWrite(CONTENT_LOGO_CACHE_KEY, cache);

  return logo;
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
  const tmdbFallback = () =>
    fetchTmdbSimilarItems({
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

  const profile = await loadActiveProfile();
  const racers: Promise<unknown[]>[] = [];
  if (profile?.traktAccessToken) racers.push(fetchTraktSimilarItems({ imdbId, contentType }));
  if (profile?.simklAccessToken) racers.push(fetchSimklSimilarItems({ imdbId, contentType }));
  if (!racers.length) return tmdbFallback();

  return Promise.race(racers);
}

/**
 * Fetches candidates for the terminal-player surface. The core owns the
 * eligibility and watched filtering; this shell function only performs the
 * provider I/O and supplies the local watched-id snapshot.
 */
export async function fetchTerminalRecommendations(payload: {
  contentType: string;
  id: string;
  hasNextEpisode: boolean;
  similarTitlesSource?: string;
  limit?: number;
}): Promise<Meta[]> {
  if (payload.hasNextEpisode || !payload.id) return [];
  const prefs = { ...DEFAULT_APP_PREFS, ...(await loadPrefs()) };
  const requestedSource = String(payload.similarTitlesSource ?? '');
  const source = ['auto', 'trakt', 'simkl', 'tmdb'].includes(requestedSource)
    ? requestedSource
    : prefString(prefs, 'similarTitlesSource', 'auto');
  const candidates = await fetchSimilarItems({
    contentType: payload.contentType,
    id: payload.id,
    language: prefString(prefs, 'language', 'en'),
    apiKey: prefString(prefs, 'tmdbApiKey'),
    source,
    recommendationsEnabled: prefBool(prefs, 'tmdbRecommendationsEnabled', true),
    similarEnabled: prefBool(prefs, 'tmdbSimilarResultsEnabled', true),
  });
  if (!candidates.length) return [];

  const library = await loadLibrary();
  const watched = Object.entries((library.watched as Record<string, unknown> | undefined) ?? {})
    .filter(([, value]) => value === true)
    .map(([id]) => id);
  const plan = await coreTerminalRecommendationPlan({
    current: { id: payload.id, type: payload.contentType },
    candidates,
    hasNextEpisode: false,
    watchedIds: watched,
    limit: payload.limit,
  });
  return plan.showRecommendations ? (plan.items as Meta[]) : [];
}

export async function fetchDetailSecondary(payload: Record<string, unknown>): Promise<unknown> {
  const prefs = { ...DEFAULT_APP_PREFS, ...(await loadPrefs()) };
  const contentType = String(payload.contentType ?? payload.type ?? 'movie');
  const id = String(payload.id ?? '');
  const language = prefString(prefs, 'language', String(payload.language ?? 'en'));
  const apiKey = prefString(prefs, 'tmdbApiKey');
  const omdbApiKey = prefString(prefs, 'omdbApiKey');
  const fanartApiKey = prefString(prefs, 'fanartApiKey');
  const requestedSource = String(payload.similarTitlesSource ?? '');
  const source = ['auto', 'trakt', 'simkl', 'tmdb'].includes(requestedSource)
    ? requestedSource
    : prefString(prefs, 'similarTitlesSource', 'auto');
  const recommendationsEnabled = prefBool(prefs, 'tmdbRecommendationsEnabled', true);
  const similarEnabled = prefBool(prefs, 'tmdbSimilarResultsEnabled', true);
  const shouldFetchSimilar = source !== 'tmdb' || recommendationsEnabled || similarEnabled;

  const [similarItems, trailers, omdbRatings, fanartArtwork] = await Promise.all([
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
    prefBool(prefs, 'tmdbTrailersEnabled', true) ? fetchTmdbTrailers({ contentType, id, language, apiKey }) : Promise.resolve([]),
    fetchOmdbRatings(id, omdbApiKey),
    fetchFanartArtwork({ contentType, id, language, apiKey }, fanartApiKey),
  ]);

  return {
    watchedVideoIds: [],
    similarItems,
    trailers,
    omdbRatings,
    fanartArtwork,
  };
}

export async function fetchMdblistRatingsForDetail(payload: Record<string, unknown>): Promise<Record<string, number> | null> {
  const contentType = String(payload.contentType ?? payload.type ?? 'movie');
  const id = String(payload.id ?? '');
  const prefs = { ...DEFAULT_APP_PREFS, ...(await loadPrefs()) };
  const mdblistApiKey = prefString(prefs, 'mdblistApiKey');
  return fetchMdblistRatings(contentType, id, mdblistApiKey);
}

export async function prefetchDetailStreams(payload: Record<string, unknown>, signal?: AbortSignal): Promise<unknown> {
  return fetchDetailStreams(payload, undefined, undefined, signal);
}
