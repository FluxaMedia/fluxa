import { coreDiscoverSortPlan, coreSearchResultGrouping } from './engine';
import { coreResourceFetchPlan } from './addonManifest';
import { loadAddons } from './libraryOps';
import { fetchPlannedResources, fetchParsedAddonResource, resourceForPlannedRequest } from './fetchPlanning';
import { discoverCatalogOptions } from './homeEffects';

export async function fetchCatalogPage(payload: Record<string, unknown>): Promise<unknown> {
  const values = await fetchPlannedResources({ ...payload, kind: 'catalogPage' });
  const metas = values.flatMap((value) => ((value as { items?: unknown[] })?.items ?? []));
  return { metas };
}

export async function runSearch(payload: Record<string, unknown>): Promise<unknown> {
  const query = payload.query as string;
  const addons = await loadAddons();
  const plan = await coreResourceFetchPlan({ kind: 'search', query, addons });
  const categories: Array<{
    id: string;
    name: string;
    semanticName: string;
    type: string;
    items: unknown[];
    addonName?: string;
    catalogId?: string;
  }> = [];
  const results: unknown[] = [];

  await Promise.all((plan?.requests ?? []).map(async (request) => {
    const url = typeof request.url === 'string' ? request.url : '';
    if (!url) return;
    const parsed = await fetchParsedAddonResource(
      url,
      resourceForPlannedRequest(request.kind, undefined, request.resource),
      request.kind,
      request.addonName,
    );
    const items = ((parsed?.items as unknown[] | undefined) ?? []);
    if (!items.length) return;
    results.push(...items);
    categories.push({
      id: String(request.categoryId ?? url),
      name: String(request.categoryName ?? request.addonName ?? 'Search results'),
      semanticName: String(request.categoryName ?? request.addonName ?? 'Search results'),
      type: String(request.catalogType ?? 'mixed'),
      items,
      addonName: typeof request.addonName === 'string' ? request.addonName : undefined,
      catalogId: typeof request.catalogId === 'string' ? request.catalogId : undefined,
    });
  }));

  const grouping = await coreSearchResultGrouping({ query, results });
  return { results, categories, grouping };
}

export async function runDiscover(payload: Record<string, unknown>): Promise<unknown> {
  const contentType = payload.contentType as string;
  const filters = payload.filters as Record<string, string> | undefined;
  const genre = (payload.genre as string | null | undefined) ?? filters?.genre;
  const sortBy = payload.sortBy as string | undefined;
  const addons = await loadAddons();
  const values = await fetchPlannedResources({ kind: 'discover', contentType, genre, addons });
  const results = values.flatMap((value) => ((value as { items?: unknown[] })?.items ?? []));

  const coreSort = await coreDiscoverSortPlan({
    items: results,
    sortBy: sortBy === 'top' ? 'rating' : sortBy === 'newest' ? 'year' : 'default',
    ascending: false,
    contentTypeFilter: contentType,
    genreFilter: genre,
  }) as { items?: unknown[] } | null;

  return { results: coreSort?.items ?? results };
}

export async function readDiscoverCatalogFilters(payload: Record<string, unknown>): Promise<unknown> {
  const contentType = payload.contentType as string;
  const addons = await loadAddons();
  const catalogOptions = await discoverCatalogOptions(addons, contentType);
  const catalogs = catalogOptions.map((catalog) => ({
    key: catalog.key,
    name: catalog.label,
    addonName: catalog.label,
    transportUrl: catalog.transportUrl,
    type: catalog.type,
    id: catalog.id,
    genres: catalog.genres ?? [],
    requiresGenre: catalog.requiresGenre ?? false,
  }));
  return { catalogs };
}
