import { invoke } from '@tauri-apps/api/core';

export async function normalizeManifestUrl(rawUrl: string): Promise<string> {
  return invoke<string>('core_normalize_manifest_url', { rawUrl });
}

export async function manifestFetchPlan(rawUrl: string): Promise<{ normalizedTransportUrl: string; cacheKey: string; candidateUrls: string[] } | null> {
  const raw = await invoke<string | null>('core_manifest_fetch_plan', { rawUrl });
  return raw ? JSON.parse(raw) : null;
}

export async function parseManifest(body: string, transportUrl: string): Promise<unknown | null> {
  const raw = await invoke<string | null>('core_parse_manifest', { body, transportUrl });
  return raw ? JSON.parse(raw) : null;
}

export async function resolveManifestAssets(descriptor: unknown): Promise<unknown | null> {
  const raw = await invoke<string | null>('core_resolve_manifest_assets', {
    descriptorJson: JSON.stringify(descriptor),
  });
  return raw ? JSON.parse(raw) : null;
}

export async function mergeLiveManifest(
  descriptor: unknown,
  live: unknown | null,
  unknownName = 'Unknown Addon',
): Promise<unknown | null> {
  const raw = await invoke<string | null>('core_merge_live_manifest', {
    descriptorJson: JSON.stringify(descriptor),
    liveJson: live == null ? null : JSON.stringify(live),
    unknownName,
  });
  return raw ? JSON.parse(raw) : null;
}

export async function buildResourceUrl(
  transportUrl: string,
  resource: string,
  contentType: string,
  id: string,
  extraJson?: string,
): Promise<string> {
  return invoke<string>('core_build_resource_url', {
    transportUrl,
    resource,
    contentType,
    id,
    extraJson: extraJson ?? null,
  });
}

export async function coreSupportsResource(
  manifest: unknown,
  resourceName: string,
  contentType?: string | null,
  id?: string | null,
): Promise<boolean> {
  return invoke<boolean>('core_supports_resource', {
    manifestJson: JSON.stringify(manifest),
    resourceName,
    contentType: contentType ?? null,
    id: id ?? null,
  });
}

export async function coreCatalogSupportsExtra(catalog: unknown, extraName: string): Promise<boolean> {
  return invoke<boolean>('core_catalog_supports_extra', {
    catalogJson: JSON.stringify(catalog),
    extraName,
  });
}

export async function coreCatalogRequiresExtra(catalog: unknown, extraName: string): Promise<boolean> {
  return invoke<boolean>('core_catalog_requires_extra', {
    catalogJson: JSON.stringify(catalog),
    extraName,
  });
}

export async function coreCatalogHasRequiredExtraExcept(
  catalog: unknown,
  allowedNames: string[],
): Promise<boolean> {
  return invoke<boolean>('core_catalog_has_required_extra_except', {
    catalogJson: JSON.stringify(catalog),
    allowedNamesJson: JSON.stringify(allowedNames),
  });
}

export type AddonResourceResult =
  | {
      kind: 'success';
      url: string;
      statusCode: number;
      cacheMaxAge?: number | null;
      staleRevalidate?: number | null;
      staleError?: number | null;
      valueJson: string;
    }
  | {
      kind: 'network_error' | 'parse_error' | 'empty';
      url: string;
      statusCode: number;
      error?: string;
    };

export async function coreParseAddonResourceResult(
  resource: string,
  url: string,
  statusCode: number,
  body: string | null,
): Promise<AddonResourceResult> {
  const raw = await invoke<string>('core_parse_addon_resource_result', {
    resource,
    url,
    statusCode,
    body,
  });
  return JSON.parse(raw) as AddonResourceResult;
}

export async function coreAddonResourceRequestPlan(request: unknown): Promise<{ urls: string[] } | null> {
  const raw = await invoke<string | null>('core_addon_resource_request_plan', {
    requestJson: JSON.stringify(request),
  });
  return raw ? JSON.parse(raw) : null;
}

export async function coreResourceFetchPlan(request: unknown): Promise<{ requests: Array<Record<string, unknown>> } | null> {
  const raw = await invoke<string | null>('core_resource_fetch_plan', {
    requestJson: JSON.stringify(request),
  });
  return raw ? JSON.parse(raw) : null;
}

export async function coreResourceParsePlan(request: unknown): Promise<Record<string, unknown> | null> {
  const raw = await invoke<string | null>('core_resource_parse_plan', {
    requestJson: JSON.stringify(request),
  });
  return raw ? JSON.parse(raw) : null;
}

export async function coreAddonCollectionMutationPlan(request: unknown): Promise<{ addons?: unknown[] } | null> {
  const raw = await invoke<string | null>('core_addon_collection_mutation_plan', {
    requestJson: JSON.stringify(request),
  });
  return raw ? JSON.parse(raw) : null;
}
