export type IntroSegmentResult = { startTime: number; endTime: number; type: string };

import {
  coreMergeIntroSegments,
  coreAniListMalId,
  coreAniListId,
  coreAnilistMediaIdPlan,
  coreAniskipSegmentsPlan,
  coreAnimeSkipFindShowPlan,
  coreAnimeSkipShowId,
  coreAnimeSkipFindEpisodesPlan,
  coreAnimeSkipFindTimestampsPlan,
  coreParseAnimeSkipResults,
  coreParseAniskipResults,
  coreIntroDbSegmentsPlan,
  coreIntroDbSubmitPlan,
  coreParseIntroDbSegments,
  coreSkipdbSegmentsPlan,
  coreSkipdbSubmitPlan,
  coreParseSkipdbSegments,
  coreTheIntroDbMediaPlan,
  coreTheIntroDbSubmitPlan,
  coreParseTheIntroDbSegments,
  matchAnimeSkipEpisodeId,
  corePlaybackIntroLookupContentId,
} from './engine';
import { loadAddons } from './libraryOps';
import { fetchPlannedResources } from './fetchPlanning';
import { tryExecuteRequestPlan, executeRequestPlan, type RequestPlan } from './httpClient';

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
  const tmdbId = typeof payload.tmdbId === 'number' && payload.tmdbId > 0 ? payload.tmdbId : undefined;
  const useIntroDb = payload.useIntroDb !== false;
  const useSkipDb = payload.useSkipDb !== false;
  const useTheIntroDb = payload.useTheIntroDb !== false;
  const useAniSkip = payload.useAniSkip !== false;
  const useAnimeSkip = payload.useAnimeSkip === true;
  const animeSkipClientId = typeof payload.animeSkipClientId === 'string' ? payload.animeSkipClientId : '';

  const [introDbSegments, skipDbSegments, theIntroDbSegments, aniSkipSegments, animeSkipSegments] = await Promise.all([
    useIntroDb && imdbId && season > 0 && episode > 0 ? fetchIntroDbSegments(imdbId, season, episode) : Promise.resolve(null),
    useSkipDb && imdbId && season > 0 && episode > 0 ? fetchSkipDbSegments(imdbId, season, episode) : Promise.resolve(null),
    useTheIntroDb && (tmdbId || imdbId) && episode > 0 ? fetchTheIntroDbSegments(tmdbId, imdbId, season, episode) : Promise.resolve(null),
    useAniSkip && title && episode > 0 ? fetchAniSkipSegments(title, episode) : Promise.resolve(null),
    useAnimeSkip && animeSkipClientId && title && episode > 0
      ? fetchAnimeSkipSegments(animeSkipClientId, title, season, episode)
      : Promise.resolve(null),
  ]);
  const sources = [introDbSegments, skipDbSegments, theIntroDbSegments, aniSkipSegments, animeSkipSegments].filter(
    (segments): segments is unknown[] => Array.isArray(segments),
  );

  if (sources.length === 0) return [];
  if (sources.length === 1) return sources[0];
  return (await coreMergeIntroSegments(JSON.stringify(sources))) ?? [];
}

async function fetchIntroDbSegments(imdbId: string, season: number, episode: number): Promise<unknown[] | null> {
  const plan = await coreIntroDbSegmentsPlan({ imdbId, season, episode });
  if (!plan) return null;
  const data = await tryExecuteRequestPlan(plan);
  return coreParseIntroDbSegments(JSON.stringify(data));
}

async function fetchSkipDbSegments(imdbId: string, season: number, episode: number): Promise<unknown[] | null> {
  const plan = await coreSkipdbSegmentsPlan({ imdbId, season, episode });
  if (!plan) return null;
  const data = await tryExecuteRequestPlan(plan);
  return coreParseSkipdbSegments(JSON.stringify(data));
}

async function fetchTheIntroDbSegments(
  tmdbId: number | undefined,
  imdbId: string,
  season: number,
  episode: number,
): Promise<unknown[] | null> {
  const plan = await coreTheIntroDbMediaPlan({
    tmdbId,
    imdbId: tmdbId ? undefined : imdbId,
    season,
    episode,
  });
  if (!plan) return null;
  const data = await tryExecuteRequestPlan(plan);
  if (!data) return null;
  return coreParseTheIntroDbSegments({ responseJson: JSON.stringify(data) });
}

async function fetchAniSkipSegments(title: string, episode: number): Promise<unknown[] | null> {
  const malId = await resolveMalId(title);
  if (!malId) return null;
  const plan = await coreAniskipSegmentsPlan({ malId, episode });
  if (!plan) return null;
  const data = await tryExecuteRequestPlan(plan);
  return coreParseAniskipResults(JSON.stringify(data));
}

async function fetchAnimeSkipSegments(
  clientId: string,
  title: string,
  season: number,
  episode: number,
): Promise<unknown[] | null> {
  const anilistId = await resolveAnilistId(title);
  if (!anilistId) return null;

  const showPlan = await coreAnimeSkipFindShowPlan({ clientId, anilistId });
  if (!showPlan) return null;
  const showData = await tryExecuteRequestPlan(showPlan);
  const showId = showData ? await coreAnimeSkipShowId(JSON.stringify(showData)) : null;
  if (!showId) return null;

  const episodesPlan = await coreAnimeSkipFindEpisodesPlan({ clientId, showId });
  if (!episodesPlan) return null;
  const episodesData = await tryExecuteRequestPlan(episodesPlan);
  if (!episodesData) return null;
  const matchedId = await matchAnimeSkipEpisodeId(JSON.stringify(episodesData), season, episode);
  if (!matchedId) return null;

  const timestampsPlan = await coreAnimeSkipFindTimestampsPlan({ clientId, episodeId: matchedId });
  if (!timestampsPlan) return null;
  const timestampsData = await tryExecuteRequestPlan(timestampsPlan);
  if (!timestampsData) return null;
  return coreParseAnimeSkipResults(JSON.stringify(timestampsData));
}

async function resolveAnilistId(title: string): Promise<number | null> {
  const plan = await coreAnilistMediaIdPlan({ title, field: 'id' });
  if (!plan) return null;
  const data = await tryExecuteRequestPlan(plan);
  return data ? coreAniListId(JSON.stringify(data)) : null;
}

async function resolveMalId(title: string): Promise<number | null> {
  const plan = await coreAnilistMediaIdPlan({ title, field: 'idMal' });
  if (!plan) return null;
  const data = await tryExecuteRequestPlan(plan);
  return data ? coreAniListMalId(JSON.stringify(data)) : null;
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
  const plans = await Promise.all(segments.map((segment) => coreIntroDbSubmitPlan({
    apiKey,
    imdbId,
    season,
    episode,
    segmentType: segment.type,
    startSec: segment.startTime / 1000,
    endSec: segment.endTime / 1000,
  })));
  await Promise.all(plans.filter((plan): plan is RequestPlan => !!plan).map((plan) => executeRequestPlan(plan)));
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
  const plans = await Promise.all(segments.map((segment) => coreSkipdbSubmitPlan({
    apiKey,
    imdbId,
    season,
    episode,
    segmentType: segment.type,
    startMs: segment.startTime,
    endMs: segment.endTime,
  })));
  await Promise.all(plans.filter((plan): plan is RequestPlan => !!plan).map((plan) => executeRequestPlan(plan)));
}

export async function submitTheIntroDbSegments(payload: {
  apiKey: string;
  tmdbId: number;
  mediaType: 'movie' | 'tv';
  imdbId?: string;
  season: number;
  episode: number;
  segments: IntroSegmentResult[];
}): Promise<void> {
  const { apiKey, tmdbId, mediaType, imdbId, season, episode, segments } = payload;
  if (!apiKey || !tmdbId || (mediaType === 'tv' && (season <= 0 || episode <= 0)) || segments.length === 0) {
    throw new Error('invalid_submission');
  }
  const plans = await Promise.all(segments.map((segment) => coreTheIntroDbSubmitPlan({
    apiKey,
    tmdbId,
    mediaType,
    imdbId,
    season: mediaType === 'tv' ? season : undefined,
    episode: mediaType === 'tv' ? episode : undefined,
    segment: segment.type,
    startSec: segment.startTime / 1000,
    endSec: segment.endTime / 1000,
  })));
  await Promise.all(plans.filter((plan): plan is RequestPlan => !!plan).map((plan) => executeRequestPlan(plan)));
}
