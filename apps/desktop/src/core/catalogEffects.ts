import { coreInvoke, coreSearchResultGrouping } from './engine';
import { buildResourceUrl, coreResourceFetchPlan } from './addonManifest';
import { loadEnabledAddons, loadPrefs } from './libraryOps';
import { fetchPlannedResources, fetchParsedAddonResource, resourceForPlannedRequest } from './fetchPlanning';
import { fetchBuiltinCatalog, isBuiltinTmdbAddon, withBuiltinTmdbAddon } from './tmdbAddon';
import { startPerfSpan } from './performance';
import type { UserProfile } from './models';

export async function fetchCatalogPage(payload: Record<string, unknown>, signal?: AbortSignal): Promise<unknown> {
  const perf = startPerfSpan('catalog.page', { contentType: payload.contentType, catalogId: payload.catalogId });
  const values = await fetchPlannedResources({ ...payload, kind: 'catalogPage', traceId: perf.traceId }, undefined, signal);
  const rawItems = values.flatMap((value) => (value as { items?: unknown[] })?.items ?? []);
  const transportUrl = typeof payload.transportUrl === 'string' ? payload.transportUrl : undefined;
  const catalogType = typeof payload.contentType === 'string' ? payload.contentType : undefined;
  const items = rawItems.map((item) =>
    item && typeof item === 'object'
      ? { ...(item as Record<string, unknown>), sourceAddonTransportUrl: transportUrl, sourceAddonCatalogType: catalogType }
      : item,
  );
  perf.end({ items: items.length });
  return { items };
}

const searchResultsCache = new Map<string, unknown>();
let searchAbortController: AbortController | null = null;

export interface PartialSearchSource {
  id: string;
  name?: string;
  type?: string;
}

type SearchPartialHandler = (query: string, source: PartialSearchSource, items: unknown[]) => void;

const _searchPartialHandlers = new Set<SearchPartialHandler>();

export function addSearchPartialHandler(fn: SearchPartialHandler): () => void {
  _searchPartialHandlers.add(fn);
  return () => _searchPartialHandlers.delete(fn);
}

function notifySearchPartialHandlers(query: string, source: PartialSearchSource, items: unknown[]) {
  for (const handler of _searchPartialHandlers) handler(query, source, items);
}

export async function runSearch(payload: Record<string, unknown>, signal?: AbortSignal): Promise<unknown> {
  const perf = startPerfSpan('search', { query: payload.query });
  const startedAt = performance.now();
  const query = payload.query as string;
  const language = payload.language as string | undefined;
  const cacheKey = `${language ?? ''}|${query}`;
  console.debug('[fluxa:search:start]', JSON.stringify({ query }));
  const cached = searchResultsCache.get(cacheKey);
  if (cached) {
    console.debug('[fluxa:search:cache-hit]', JSON.stringify({ query, ms: Math.round(performance.now() - startedAt) }));
    return cached;
  }

  searchAbortController?.abort();
  const abortController = new AbortController();
  searchAbortController = abortController;
  const requestSignal = signal ? AbortSignal.any([signal, abortController.signal]) : abortController.signal;
  const addons = await loadEnabledAddons();
  console.debug('[fluxa:search:addons-ready]', JSON.stringify({ query, addons: addons.map((addon) => addon.name), ms: Math.round(performance.now() - startedAt) }));
  const plan = await coreResourceFetchPlan({ kind: 'search', query, addons, traceId: perf.traceId });
  console.debug('[fluxa:search:plan-ready]', JSON.stringify({ query, requests: (plan?.requests ?? []).map((request) => ({ addon: request.addonName, type: request.catalogType, url: request.url })), ms: Math.round(performance.now() - startedAt) }));
  const sources: Array<{
    id: string;
    name?: string;
    semanticName?: string;
    type?: string;
    items: unknown[];
    addonName?: string;
    catalogId?: string;
  }> = [];

  await Promise.all(
    (plan?.requests ?? []).map(async (request) => {
      const url = typeof request.url === 'string' ? request.url : '';
      if (!url) return;
      const parsed = await fetchParsedAddonResource(
        url,
        await resourceForPlannedRequest(request.kind, undefined, request.resource),
        request.kind,
        request.addonName,
        undefined,
        requestSignal,
      );
      console.debug('[fluxa:search:source-parsed]', JSON.stringify({ query, addon: request.addonName, type: request.catalogType, items: Array.isArray(parsed?.items) ? parsed.items.length : 0, ms: Math.round(performance.now() - startedAt) }));
      const rawItems = (parsed?.items as unknown[] | undefined) ?? [];
      if (!rawItems.length) return;
      const transportUrl = typeof request.transportUrl === 'string' ? request.transportUrl : undefined;
      const catalogType = request.catalogType ? String(request.catalogType) : undefined;
      const items = rawItems.map((item) =>
        item && typeof item === 'object'
          ? { ...(item as Record<string, unknown>), sourceAddonTransportUrl: transportUrl, sourceAddonCatalogType: catalogType }
          : item,
      );
      const sourceId = String(request.categoryId ?? url);
      const sourceName = request.categoryName
        ? String(request.categoryName)
        : typeof request.addonName === 'string'
          ? request.addonName
          : undefined;
      if (searchAbortController === abortController)
        notifySearchPartialHandlers(query, { id: sourceId, name: sourceName, type: catalogType }, items);
      sources.push({
        id: sourceId,
        name: sourceName,
        type: catalogType,
        items,
        addonName: typeof request.addonName === 'string' ? request.addonName : undefined,
        catalogId: typeof request.catalogId === 'string' ? request.catalogId : undefined,
      });
    }),
  );

  const prefs = await loadPrefs();
  const tmdbApiKey = String(prefs.tmdbApiKey ?? '').trim();
  if (tmdbApiKey) {
    await Promise.all(
      (['movie', 'series'] as const).map(async (type) => {
        const { metas } = await fetchBuiltinCatalog(type, { search: query }, tmdbApiKey, String(prefs.language ?? 'en'), requestSignal);
        if (searchAbortController !== abortController || !metas.length) return;
        notifySearchPartialHandlers(query, { id: `tmdb:${type}`, name: 'TMDB', type }, metas);
        sources.push({ id: `tmdb:${type}`, name: 'TMDB', type, items: metas, addonName: 'TMDB' });
      }),
    );
  }

  const merged = (await coreInvoke<{ results: unknown[]; categories: unknown[] }>('mergeSearchSources', JSON.stringify(sources))) ?? {
    results: [],
    categories: [],
  };

  const grouping = await coreSearchResultGrouping({ query, results: merged.results });
  const value = { results: merged.results, categories: merged.categories, grouping };
  if (searchAbortController === abortController) searchResultsCache.set(cacheKey, value);
  perf.end({ sources: sources.length, results: merged.results.length });
  console.debug('[fluxa:search:end]', JSON.stringify({ query, sources: sources.length, results: merged.results.length, ms: Math.round(performance.now() - startedAt) }));
  return value;
}

let discoverAbortController: AbortController | null = null;

export async function runDiscover(payload: Record<string, unknown>, signal?: AbortSignal): Promise<unknown> {
  const perf = startPerfSpan('discover', { contentType: payload.contentType });
  discoverAbortController?.abort();
  const abortController = new AbortController();
  discoverAbortController = abortController;
  const requestSignal = signal ? AbortSignal.any([signal, abortController.signal]) : abortController.signal;

  const contentType = payload.contentType as string;
  const filters = (payload.filters ?? {}) as Record<string, unknown>;
  const sourceRequests = await coreInvoke<Array<Record<string, unknown>>>(
    'discoverSourceRequests',
    JSON.stringify({ contentType, filters }),
  );
  const prefs = await loadPrefs();
  const apiKey = String(prefs.tmdbApiKey ?? '').trim();
  const sources = await Promise.all((sourceRequests ?? []).map(async (source) => {
    const transportUrl = String(source.transportUrl ?? '');
    const type = String(source.type ?? contentType);
    const catalogId = String(source.catalogId ?? '');
    const extra = (source.extra as Record<string, unknown> | undefined) ?? {};
    const genre = typeof extra.genre === 'string' ? extra.genre : null;
    let items: unknown[] = [];
    if (isBuiltinTmdbAddon(transportUrl)) {
      const result = await fetchBuiltinCatalog(type, extra, apiKey, String(prefs.language ?? 'en'), requestSignal);
      items = result.metas;
    } else {
      const url = await buildResourceUrl(transportUrl, 'catalog', type, catalogId, JSON.stringify(extra));
      const resource = await resourceForPlannedRequest('discover');
      const parsed = url ? await fetchParsedAddonResource(url, resource, 'discover', transportUrl, undefined, requestSignal) : null;
      items = (parsed?.items as unknown[] | undefined) ?? [];
    }
    return { transportUrl, catalogId, type, genre, items };
  }));
  if (discoverAbortController !== abortController) throw new DOMException('superseded', 'AbortError');
  const merged = await coreInvoke<{ results: unknown[]; resultSources: unknown }>(
    'mergeDiscoverSources',
    JSON.stringify({ sources }),
  );
  const result = merged ?? { results: [], resultSources: {} };
  perf.end({ sources: sources.length, results: result.results.length });
  return result;
}

export async function readDiscoverCatalogFilters(payload: Record<string, unknown>): Promise<unknown> {
  const profile = payload.profile as UserProfile | null | undefined;
  const addons = await withBuiltinTmdbAddon(await loadEnabledAddons(profile), await loadPrefs());
  return { addons };
}
