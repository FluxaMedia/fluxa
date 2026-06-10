export type IntroSegmentResult = { startTime: number; endTime: number; type: string };

import {
  coreParseAniskipResults,
  coreParseIntroDbSegments,
  corePlaybackIntroLookupContentId,
  coreUniqueIntroSegments,
} from './engine';
import { loadAddons } from './libraryOps';
import { fetchPlannedResources } from './fetchPlanning';
import { tryFetchJson } from './httpClient';

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
  const useAniSkip = payload.useAniSkip !== false;

  const sources: unknown[][] = [];

  if (useIntroDb && imdbId && season > 0 && episode > 0) {
    const url = `https://api.introdb.app/segments?imdb_id=${encodeURIComponent(imdbId)}&season=${season}&episode=${episode}`;
    const data = await tryFetchJson(url);
    const parsed = await coreParseIntroDbSegments(JSON.stringify(data));
    if (parsed) sources.push(parsed);
  }

  if (useAniSkip && title && episode > 0) {
    const malId = await resolveMalId(title);
    if (malId) {
      const data = await tryFetchJson(`https://api.aniskip.com/v2/skip-times/${malId}/${episode}?types=op,ed,recap`);
      const parsed = await coreParseAniskipResults(JSON.stringify(data));
      if (parsed) sources.push(parsed);
    }
  }

  if (sources.length === 0) return [];
  if (sources.length === 1) return sources[0];
  return (await coreUniqueIntroSegments(JSON.stringify(sources[0]), JSON.stringify(sources[1]))) ?? [];
}

async function resolveMalId(title: string): Promise<number | null> {
  const query = title.replace(/\s+\(\d{4}\)$/, '').trim();
  if (query.length < 2) return null;
  const data = await tryFetchJson(`https://api.jikan.moe/v4/anime?q=${encodeURIComponent(query)}&limit=5`);
  const items = (data as { data?: Array<{ mal_id?: number }> } | null)?.data ?? [];
  return items.find((item) => typeof item.mal_id === 'number' && item.mal_id > 0)?.mal_id ?? null;
}
