import { coreInvoke, coreSearchResultGrouping } from './engine';
import { coreResourceFetchPlan } from './addonManifest';
import { loadEnabledAddons, loadPrefs } from './libraryOps';
import { fetchPlannedResources, fetchParsedAddonResource, resourceForPlannedRequest } from './fetchPlanning';
import { discoverCatalogOptions } from './homeEffects';
import { fetchBuiltinCatalog, isBuiltinTmdbAddon } from './tmdbAddon';

export async function fetchCatalogPage(payload: Record<string, unknown>, signal?: AbortSignal): Promise<unknown> {
  const values = await fetchPlannedResources({ ...payload, kind: 'catalogPage' }, undefined, signal);
  const rawItems = values.flatMap((value) => ((value as { items?: unknown[] })?.items ?? []));
  const transportUrl = typeof payload.transportUrl === 'string' ? payload.transportUrl : undefined;
  const catalogType = typeof payload.contentType === 'string' ? payload.contentType : undefined;
  const items = rawItems.map((item) => (
    item && typeof item === 'object'
      ? { ...(item as Record<string, unknown>), sourceAddonTransportUrl: transportUrl, sourceAddonCatalogType: catalogType }
      : item
  ));
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
  const query = payload.query as string;
  const language = payload.language as string | undefined;
  const cacheKey = `${language ?? ''}|${query}`;
  const cached = searchResultsCache.get(cacheKey);
  if (cached) return cached;

  searchAbortController?.abort();
  const abortController = new AbortController();
  searchAbortController = abortController;
  const requestSignal = signal ? AbortSignal.any([signal, abortController.signal]) : abortController.signal;

  const addons = await loadEnabledAddons();
  const plan = await coreResourceFetchPlan({ kind: 'search', query, addons });
  const sources: Array<{
    id: string;
    name?: string;
    semanticName?: string;
    type?: string;
    items: unknown[];
    addonName?: string;
    catalogId?: string;
  }> = [];

  await Promise.all((plan?.requests ?? []).map(async (request) => {
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
    const rawItems = ((parsed?.items as unknown[] | undefined) ?? []);
    if (!rawItems.length) return;
    const transportUrl = typeof request.transportUrl === 'string' ? request.transportUrl : undefined;
    const catalogType = request.catalogType ? String(request.catalogType) : undefined;
    const items = rawItems.map((item) => (
      item && typeof item === 'object'
        ? { ...(item as Record<string, unknown>), sourceAddonTransportUrl: transportUrl, sourceAddonCatalogType: catalogType }
        : item
    ));
    const sourceId = String(request.categoryId ?? url);
    const sourceName = request.categoryName ? String(request.categoryName) : (typeof request.addonName === 'string' ? request.addonName : undefined);
    if (searchAbortController === abortController) notifySearchPartialHandlers(query, { id: sourceId, name: sourceName, type: catalogType }, items);
    sources.push({
      id: sourceId,
      name: sourceName,
      type: catalogType,
      items,
      addonName: typeof request.addonName === 'string' ? request.addonName : undefined,
      catalogId: typeof request.catalogId === 'string' ? request.catalogId : undefined,
    });
  }));

  const prefs = await loadPrefs();
  const tmdbApiKey = String(prefs.tmdbApiKey ?? '').trim();
  if (tmdbApiKey) {
    await Promise.all((['movie', 'series'] as const).map(async (type) => {
      const { metas } = await fetchBuiltinCatalog(type, { search: query }, tmdbApiKey, String(prefs.language ?? 'en'), requestSignal);
      if (searchAbortController !== abortController || !metas.length) return;
      notifySearchPartialHandlers(query, { id: `tmdb:${type}`, name: 'TMDB', type }, metas);
      sources.push({ id: `tmdb:${type}`, name: 'TMDB', type, items: metas, addonName: 'TMDB' });
    }));
  }

  const merged = (await coreInvoke<{ results: unknown[]; categories: unknown[] }>(
    'mergeSearchSources',
    JSON.stringify(sources),
  )) ?? { results: [], categories: [] };

  const grouping = await coreSearchResultGrouping({ query, results: merged.results });
  const value = { results: merged.results, categories: merged.categories, grouping };
  if (searchAbortController === abortController) searchResultsCache.set(cacheKey, value);
  return value;
}

let discoverAbortController: AbortController | null = null;

export async function runDiscover(payload: Record<string, unknown>, signal?: AbortSignal): Promise<unknown> {
  discoverAbortController?.abort();
  const abortController = new AbortController();
  discoverAbortController = abortController;
  const requestSignal = signal ? AbortSignal.any([signal, abortController.signal]) : abortController.signal;

  const contentType = payload.contentType as string;
  const filters = payload.filters as { catalogKey?: string; transportUrl?: string; extra?: Record<string, unknown> } | undefined;
  const extra = filters?.extra ?? {};

  if (isBuiltinTmdbAddon(filters?.transportUrl)) {
    const prefs = await loadPrefs();
    const { metas } = await fetchBuiltinCatalog(contentType, extra, String(prefs.tmdbApiKey ?? ''), String(prefs.language ?? 'en'), requestSignal);
    if (discoverAbortController !== abortController) throw new DOMException('superseded', 'AbortError');
    return { results: metas };
  }

  const catalogKey = filters?.catalogKey;
  const addons = await loadEnabledAddons();
  const values = await fetchPlannedResources(
    { kind: 'discover', contentType, catalogKey, extra, addons },
    undefined,
    requestSignal,
  );
  if (discoverAbortController !== abortController) throw new DOMException('superseded', 'AbortError');

  const rawResults = values.flatMap((value) => ((value as { items?: unknown[] })?.items ?? []));
  if (discoverAbortController !== abortController) throw new DOMException('superseded', 'AbortError');
  const results = rawResults.map((item) => (
    item && typeof item === 'object'
      ? { ...(item as Record<string, unknown>), sourceAddonTransportUrl: filters?.transportUrl, sourceAddonCatalogType: contentType }
      : item
  ));
  return { results };
}

export async function readDiscoverCatalogFilters(payload: Record<string, unknown>): Promise<unknown> {
  const contentType = payload.contentType as string;
  const addons = await loadEnabledAddons();
  const catalogOptions = await discoverCatalogOptions(addons, contentType);
  const catalogs = catalogOptions.map((catalog) => ({
    key: catalog.key,
    label: catalog.label,
    transportUrl: catalog.transportUrl,
    type: catalog.type,
    id: catalog.id,
    extras: catalog.extras ?? [],
  }));
  return { catalogs };
}
