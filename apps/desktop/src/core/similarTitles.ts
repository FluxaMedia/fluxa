import { platformInvoke } from '../platform/invoke';
import { coreInvoke, httpExecuteText } from './engine';
import { _appVersion, tryFetchJson } from './httpClient';
import { enrichWithAddonMeta } from './externalSyncUtils';
import type { Meta } from './types';

type TraktLookupItem = {
  movie?: { ids?: { slug?: string } };
  show?: { ids?: { slug?: string } };
};

type TraktRelatedItem = {
  ids?: { imdb?: string; tmdb?: number };
  title?: string;
  year?: number;
};

async function tryFetchJsonWithHeaders(url: string, headers: Record<string, string>): Promise<unknown | null> {
  try {
    const response = await httpExecuteText(url, 'GET', headers);
    if (response.statusCode < 200 || response.statusCode >= 300) return null;
    return JSON.parse(response.body) as unknown;
  } catch {
    return null;
  }
}

async function tryCoreMapper<T>(method: 'traktRelatedLookupSlug' | 'traktRelatedItemsToMetas', args: string): Promise<T | null> {
  try {
    return await coreInvoke<T>(method, args);
  } catch {
    return null;
  }
}

// Trakt's docs list the :id path param as the resource's slug, and imdb ids are
// known to 404 on some shows even though they work for most — so we resolve the
// slug via /search first rather than passing the imdb id straight through.
export async function fetchTraktSimilarItems({ imdbId, contentType }: { imdbId: string; contentType: string }): Promise<Meta[]> {
  const clientId = await platformInvoke<string>('get_oauth_client_id', { service: 'trakt' }).catch(() => '');
  if (!clientId) return [];
  const headers = {
    'Content-Type': 'application/json',
    'User-Agent': `Fluxa Desktop/${_appVersion}`,
    'trakt-api-version': '2',
    'trakt-api-key': clientId,
  };

  const wantType = contentType === 'series' ? 'show' : 'movie';
  const lookup = await tryFetchJsonWithHeaders(`https://api.trakt.tv/search/imdb/${encodeURIComponent(imdbId)}?type=${wantType}`, headers);
  const lookupItems = Array.isArray(lookup) ? (lookup as TraktLookupItem[]) : [];
  const slugFromCore = await tryCoreMapper<string>(
    'traktRelatedLookupSlug',
    JSON.stringify({
      lookupJson: JSON.stringify(lookup ?? []),
      wantType,
    }),
  );
  const slug = slugFromCore ?? lookupItems.find((item) => item[wantType]?.ids?.slug)?.[wantType]?.ids?.slug;
  if (!slug) return [];

  const resource = wantType === 'show' ? 'shows' : 'movies';
  const data = await tryFetchJsonWithHeaders(`https://api.trakt.tv/${resource}/${encodeURIComponent(slug)}/related?limit=20`, headers);
  const relatedItems = Array.isArray(data) ? (data as TraktRelatedItem[]) : [];
  const partialFromCore = await tryCoreMapper<Record<string, unknown>[]>(
    'traktRelatedItemsToMetas',
    JSON.stringify({
      relatedJson: JSON.stringify(Array.isArray(data) ? data : []),
      contentType,
    }),
  );
  const fallbackPartial = relatedItems.flatMap((item) => {
    const id = item.ids?.imdb || (typeof item.ids?.tmdb === 'number' ? `tmdb:${item.ids.tmdb}` : '');
    if (!id || !item.title) return [];
    return [
      {
        id,
        type: contentType,
        name: item.title,
        ...(typeof item.year === 'number' ? { releaseInfo: String(item.year) } : {}),
      },
    ];
  });
  const partial = partialFromCore?.length ? partialFromCore : fallbackPartial;
  if (!partial?.length) return [];
  // Trakt's own images must not be hotlinked ("must be cached... direct linking
  // will be blocked" per their docs), so posters/backgrounds still come from the
  // addon/TMDB enrichment pipeline rather than Trakt's images field.
  return (await enrichWithAddonMeta(partial)) as unknown as Meta[];
}

// Simkl has no dedicated "similar" endpoint - related titles come embedded in the
// movie/show detail response as `users_recommendations`, each carrying only a
// simkl id (no imdb/tmdb), so each item needs one more detail lookup to resolve
// a navigable imdb id.
export async function fetchSimklSimilarItems({ imdbId, contentType }: { imdbId: string; contentType: string }): Promise<Meta[]> {
  const clientId = await platformInvoke<string>('get_oauth_client_id', { service: 'simkl' }).catch(() => '');
  if (!clientId) return [];

  const simklQuery = `client_id=${encodeURIComponent(clientId)}&app-name=fluxa&app-version=${encodeURIComponent(_appVersion)}`;
  const wantType = contentType === 'series' ? 'tv' : 'movie';

  const headers = { 'Content-Type': 'application/json', 'User-Agent': `Fluxa Desktop/${_appVersion}` };
  const lookup = await tryFetchJson(`https://api.simkl.com/search/id?imdb=${encodeURIComponent(imdbId)}&${simklQuery}`, { headers });
  const simklId = await coreInvoke<number | null>(
    'simklLookupIdForType',
    JSON.stringify({
      lookupJson: JSON.stringify(Array.isArray(lookup) ? lookup : []),
      wantType,
    }),
  );
  if (simklId == null) return [];

  const resource = wantType === 'tv' ? 'tv' : 'movies';
  const detail = await tryFetchJson(`https://api.simkl.com/${resource}/${simklId}?${simklQuery}`, { headers });
  const candidates = await coreInvoke<Array<{ ids?: { simkl?: number }; type?: string }> | null>(
    'simklRecommendationCandidates',
    JSON.stringify({ detailJson: JSON.stringify(detail ?? {}) }),
  );
  if (!candidates?.length) return [];

  const CONCURRENCY = 4;
  const results: (Meta | null)[] = new Array(candidates.length).fill(null);
  let cursor = 0;

  async function worker() {
    while (cursor < candidates!.length) {
      const i = cursor++;
      const rec = candidates![i];
      const recSimklId = rec.ids?.simkl;
      if (recSimklId == null) continue;
      const recResource = rec.type === 'tv' ? 'tv' : 'movies';
      const recDetail = (await tryFetchJson(`https://api.simkl.com/${recResource}/${recSimklId}?${simklQuery}`, { headers })) as {
        ids?: { imdb?: string };
      } | null;
      const imdb = recDetail?.ids?.imdb;
      if (!imdb) continue;
      results[i] = await coreInvoke<Meta | null>(
        'simklRecommendationToMeta',
        JSON.stringify({
          recJson: JSON.stringify(rec),
          resolvedImdb: imdb,
        }),
      );
    }
  }

  await Promise.all(Array.from({ length: Math.min(CONCURRENCY, candidates.length) }, worker));
  return results.filter((m): m is Meta => !!m);
}
