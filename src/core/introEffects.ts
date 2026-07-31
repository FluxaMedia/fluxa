export type IntroSegmentResult = { startTime: number; endTime: number; type: string };

import {
  coreInvoke,
  coreMergeIntroSegments,
  coreAniListMalId,
  coreParseAnimeSkipResults,
  coreParseAniskipResults,
  coreParseIntroDbSegments,
  coreParseSkipdbSegments,
  corePlaybackIntroLookupContentId,
} from './engine';
import { loadAddons } from './libraryOps';
import { fetchPlannedResources } from './fetchPlanning';
import { fetchJson, tryFetchJson } from './httpClient';

export async function fetchSubtitles(payload: Record<string, unknown>): Promise<unknown> {
  const addons = await loadAddons();
  const values = await fetchPlannedResources({
    kind: 'subtitles',
    addons,
    contentType: payload.contentType,
    id: payload.id,
    extraRaw: payload.extraArgs,
  });
  const subtitles = values.flatMap((value) => ((value as { subtitles?: unknown[] })?.subtitles ?? []));
  return { subtitles };
}

export async function resolveIntroImdbId(payload: Record<string, unknown>): Promise<unknown> {
  const meta = payload.meta as { id?: string } | undefined;
  const videoId = typeof payload.videoId === 'string' ? payload.videoId : undefined;
  const id = videoId || meta?.id;
  if (!id) return null;
  return corePlaybackIntroLookupContentId(id);
}

export async function fetchIntroSegments(payload: Record<string, unknown>): Promise<unknown> {
  const imdbId = typeof payload.imdbId === 'string' ? payload.imdbId : '';
  const season = Number(payload.season ?? 0);
  const episode = Number(payload.episode ?? 0);
  const title = typeof payload.title === 'string' ? payload.title : '';
  const useIntroDb = payload.useIntroDb !== false;
  const useSkipDb = payload.useSkipDb !== false;
  const useAniSkip = payload.useAniSkip !== false;
  const useAnimeSkip = payload.useAnimeSkip === true;
  const animeSkipClientId = typeof payload.animeSkipClientId === 'string' ? payload.animeSkipClientId : '';

  const [introDbSegments, skipDbSegments, aniSkipSegments, animeSkipSegments] = await Promise.all([
    useIntroDb && imdbId && season > 0 && episode > 0
      ? (async () => {
          const url = `https://api.introdb.app/segments?imdb_id=${encodeURIComponent(imdbId)}&season=${season}&episode=${episode}`;
          const data = await tryFetchJson(url);
          return coreParseIntroDbSegments(JSON.stringify(data));
        })()
      : Promise.resolve(null),
    useSkipDb && imdbId && season > 0 && episode > 0
      ? (async () => {
          const url = `https://api.skipdb.tv/api/segments?imdb_id=${encodeURIComponent(imdbId)}&season=${season}&episode=${episode}`;
          const data = await tryFetchJson(url);
          return coreParseSkipdbSegments(JSON.stringify(data));
        })()
      : Promise.resolve(null),
    useAniSkip && title && episode > 0
      ? (async () => {
          const malId = await resolveMalId(title);
          if (!malId) return null;
          const params = new URLSearchParams({ episodeLength: '0' });
          for (const type of ['op', 'ed', 'recap']) params.append('types', type);
          const data = await tryFetchJson(`https://api.aniskip.com/v2/skip-times/${malId}/${episode}?${params}`);
          return coreParseAniskipResults(JSON.stringify(data));
        })()
      : Promise.resolve(null),
    useAnimeSkip && animeSkipClientId && title && episode > 0
      ? fetchAnimeSkipSegments(animeSkipClientId, title, season, episode)
      : Promise.resolve(null),
  ]);
  const sources = [introDbSegments, skipDbSegments, aniSkipSegments, animeSkipSegments].filter(
    (segments): segments is unknown[] => Array.isArray(segments),
  );

  if (sources.length === 0) return [];
  if (sources.length === 1) return sources[0];
  return (await coreMergeIntroSegments(JSON.stringify(sources))) ?? [];
}

async function fetchAnimeSkipSegments(
  clientId: string,
  title: string,
  season: number,
  episode: number,
): Promise<unknown[] | null> {
  const anilistId = await resolveAnilistId(title);
  const show = anilistId
    ? await animeSkipGraphql<{ findShowsByExternalId?: Array<{ id?: string }> }>(
        clientId,
        `query ($service: String!, $serviceId: String!) {
          findShowsByExternalId(service: $service, serviceId: $serviceId) { id }
        }`,
        { service: 'anilist.co', serviceId: String(anilistId) },
      )
    : null;
  const showId = show?.findShowsByExternalId?.[0]?.id;
  if (!showId) return null;

  const episodesData = await animeSkipGraphql<{
    findEpisodesByShowId?: Array<{ id?: string; season?: string | null; number?: string | null; absoluteNumber?: string | null }>;
  }>(
    clientId,
    `query ($showId: ID!) {
      findEpisodesByShowId(showId: $showId) { id season number absoluteNumber }
    }`,
    { showId },
  );
  const episodes = episodesData?.findEpisodesByShowId ?? [];
  const matchedId = await coreInvoke<string | null>('matchAnimeSkipEpisodeId', JSON.stringify({
    episodesJson: JSON.stringify(episodes),
    season,
    episode,
  }));
  if (!matchedId) return null;

  const timestampsData = await animeSkipGraphql<{
    findTimestampsByEpisodeId?: Array<{ at?: number; type?: { name?: string } }>;
  }>(
    clientId,
    `query ($episodeId: ID!) {
      findTimestampsByEpisodeId(episodeId: $episodeId) { at type { name } }
    }`,
    { episodeId: matchedId },
  );
  const timestamps = timestampsData?.findTimestampsByEpisodeId ?? [];
  if (timestamps.length === 0) return null;
  return coreParseAnimeSkipResults(JSON.stringify(timestamps));
}

async function animeSkipGraphql<T>(
  clientId: string,
  query: string,
  variables: Record<string, unknown>,
): Promise<T | null> {
  const data = await tryFetchJson('https://api.anime-skip.com/graphql', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json', 'X-Client-ID': clientId },
    body: JSON.stringify({ query, variables }),
  });
  return (data as { data?: T } | null)?.data ?? null;
}

async function resolveAnilistId(title: string): Promise<number | null> {
  const query = title.replace(/\s+\(\d{4}\)$/, '').trim();
  if (query.length < 2) return null;
  const data = await tryFetchJson('https://graphql.anilist.co', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      query: `query ($search: String) { Media(search: $search, type: ANIME) { id } }`,
      variables: { search: query },
    }),
  });
  const id = (data as { data?: { Media?: { id?: number } } } | null)?.data?.Media?.id;
  return typeof id === 'number' && id > 0 ? id : null;
}

export async function submitIntroDbSegments(payload: {
  apiKey: string;
  imdbId: string;
  season: number;
  episode: number;
  segments: IntroSegmentResult[];
}): Promise<void> {
  const { apiKey, imdbId, season, episode, segments } = payload;
  if (!apiKey || !imdbId || season <= 0 || episode <= 0 || segments.length === 0) {
    throw new Error('invalid_submission');
  }
  await Promise.all(segments.map((segment) => fetchJson('https://api.introdb.app/submit', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      'X-API-Key': apiKey,
    },
    body: JSON.stringify({
      imdb_id: imdbId,
      season,
      episode,
      segment_type: segment.type,
      start_sec: segment.startTime / 1000,
      end_sec: segment.endTime / 1000,
    }),
  })));
}

export async function submitSkipDbSegments(payload: {
  apiKey: string;
  imdbId: string;
  season: number;
  episode: number;
  segments: IntroSegmentResult[];
}): Promise<void> {
  const { apiKey, imdbId, season, episode, segments } = payload;
  if (!apiKey || !imdbId || season <= 0 || episode <= 0 || segments.length === 0) {
    throw new Error('invalid_submission');
  }
  await Promise.all(segments.map((segment) => fetchJson('https://api.skipdb.tv/api/segments', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      'X-API-Key': apiKey,
    },
    body: JSON.stringify({
      imdb_id: imdbId,
      season,
      episode,
      segment_type: segment.type,
      start_ms: segment.startTime,
      end_ms: segment.endTime,
    }),
  })));
}

async function resolveMalId(title: string): Promise<number | null> {
  const query = title.replace(/\s+\(\d{4}\)$/, '').trim();
  if (query.length < 2) return null;
  const data = await tryFetchJson('https://graphql.anilist.co', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      query: `query ($search: String) { Media(search: $search, type: ANIME) { idMal } }`,
      variables: { search: query },
    }),
  });
  return data ? coreAniListMalId(JSON.stringify(data)) : null;
}
