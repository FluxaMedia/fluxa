import {
  coreParseAddonResourceResult,
  coreResourceFetchPlan,
  coreResourceParsePlan,
} from './addonManifest';
import { _appVersion, platformFetch } from './httpClient';

export type FetchPlanRequest = {
  url?: unknown;
  kind?: unknown;
  resource?: unknown;
  addonName?: unknown;
  stopOnFirstResult?: unknown;
};

export function resourceForPlannedRequest(kind: unknown, requestResource?: unknown, itemResource?: unknown): string {
  const explicit = typeof itemResource === 'string' && itemResource.trim()
    ? itemResource
    : typeof requestResource === 'string' && requestResource.trim()
      ? requestResource
      : '';
  if (explicit) return explicit;

  switch (kind) {
    case 'catalogPage':
    case 'discover':
    case 'search':
      return 'catalog';
    case 'metaDetail':
    case 'seasonEpisodes':
      return 'meta';
    case 'streams':
      return 'stream';
    case 'subtitles':
      return 'subtitles';
    default:
      return typeof kind === 'string' && kind.trim() ? kind : 'catalog';
  }
}

function responseForResourcePayload(resource: string, payload: unknown): unknown {
  switch (resource) {
    case 'catalog':
    case 'metas':
      return { metas: payload };
    case 'stream':
    case 'streams':
      return { streams: payload };
    case 'meta':
      return { meta: payload };
    case 'subtitle':
    case 'subtitles':
      return { subtitles: payload };
    default:
      return payload;
  }
}

export async function fetchParsedAddonResource(
  url: string,
  resource: string,
  kind: unknown,
  addonName?: unknown,
  season?: unknown,
): Promise<Record<string, unknown> | null> {
  let statusCode = 0;
  let body: string | null = null;
  try {
    const response = await platformFetch(url, {
      headers: { 'User-Agent': `Fluxa/${_appVersion}` },
    });
    statusCode = response.status;
    body = await response.text();
  } catch {
    statusCode = 0;
  }
  const result = await coreParseAddonResourceResult(resource, url, statusCode, body);
  if (result.kind !== 'success') return null;

  let payload: unknown;
  try {
    payload = JSON.parse(result.valueJson);
  } catch {
    return null;
  }

  return coreResourceParsePlan({
    kind,
    response: responseForResourcePayload(resource, payload),
    addonName,
    season,
  });
}

export async function fetchPlannedResources(
  request: Record<string, unknown>,
  onPartialResult?: (result: unknown) => void,
): Promise<unknown[]> {
  const plan = await coreResourceFetchPlan(request);
  const requests = (plan?.requests ?? []) as FetchPlanRequest[];

  // When every request has stopOnFirstResult (e.g. metaDetail, seasonEpisodes),
  // fire all addon requests in parallel and take the first non-empty result.
  // This eliminates sequential per-addon latency: instead of waiting for each
  // addon to fail before trying the next, all race simultaneously.
  if (requests.length > 1 && requests.every((r) => r.stopOnFirstResult)) {
    const isNonEmpty = (parsed: Record<string, unknown> | null): parsed is Record<string, unknown> =>
      !!parsed && Object.values(parsed).some((v) => (Array.isArray(v) ? v.length > 0 : v != null));

    try {
      const result = await Promise.any(
        requests.map(async (item) => {
          const url = typeof item.url === 'string' ? item.url : '';
          if (!url) throw new Error('no url');
          const parsed = await fetchParsedAddonResource(
            url,
            resourceForPlannedRequest(item.kind, request.resource, item.resource),
            item.kind,
            item.addonName,
            request.season,
          );
          if (!isNonEmpty(parsed)) throw new Error('empty');
          return parsed;
        }),
      );
      return [result];
    } catch {
      return [];
    }
  }

  // Parallel path: fire all addon requests concurrently and collect all results.
  // Each result is reported via onPartialResult as it arrives for progressive display.
  const values: unknown[] = [];
  await Promise.allSettled(
    requests.map(async (item) => {
      const url = typeof item.url === 'string' ? item.url : '';
      if (!url) return;
      const parsed = await fetchParsedAddonResource(
        url,
        resourceForPlannedRequest(item.kind, request.resource, item.resource),
        item.kind,
        item.addonName,
        request.season,
      );
      if (parsed) {
        values.push(parsed);
        onPartialResult?.(parsed);
      }
    }),
  );
  return values;
}
