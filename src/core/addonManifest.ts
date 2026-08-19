import { coreInvoke } from './engine';

export async function normalizeManifestUrl(rawUrl: string): Promise<string> {
  return (await coreInvoke<string>('normalizeManifestUrl', JSON.stringify({ url: rawUrl }))) ?? rawUrl;
}

export async function manifestFetchPlan(
  rawUrl: string,
): Promise<{ normalizedTransportUrl: string; cacheKey: string; candidateUrls: string[] } | null> {
  return coreInvoke('manifestFetchPlan', JSON.stringify({ url: rawUrl }));
}

export async function parseManifest(body: string, transportUrl: string, unknownName = 'Unknown Addon'): Promise<unknown | null> {
  return coreInvoke('parseManifest', JSON.stringify({ body, transportUrl, unknownName }));
}

export async function resolveManifestAssets(descriptor: unknown): Promise<unknown | null> {
  return coreInvoke('resolveManifestAssets', JSON.stringify(descriptor));
}

export async function mergeLiveManifest(descriptor: unknown, live: unknown | null, unknownName = 'Unknown Addon'): Promise<unknown | null> {
  return coreInvoke(
    'mergeLiveManifest',
    JSON.stringify({
      descriptor: JSON.stringify(descriptor),
      live: live == null ? null : JSON.stringify(live),
      unknownName,
    }),
  );
}

export async function buildResourceUrl(
  transportUrl: string,
  resource: string,
  contentType: string,
  id: string,
  extraJson?: string,
): Promise<string> {
  return (
    (await coreInvoke<string>(
      'buildResourceUrl',
      JSON.stringify({
        transportUrl,
        resource,
        contentType,
        id,
        extraJson: extraJson ?? null,
      }),
    )) ?? ''
  );
}

export async function coreSupportsResource(
  manifest: unknown,
  resourceName: string,
  contentType?: string | null,
  id?: string | null,
): Promise<boolean> {
  return (
    (await coreInvoke<boolean>(
      'supportsResource',
      JSON.stringify({
        manifest: JSON.stringify(manifest),
        resource: resourceName,
        contentType: contentType ?? null,
        id: id ?? null,
      }),
    )) ?? false
  );
}

export async function coreCatalogSupportsExtra(catalog: unknown, extraName: string): Promise<boolean> {
  return (
    (await coreInvoke<boolean>(
      'catalogSupportsExtra',
      JSON.stringify({
        catalog: JSON.stringify(catalog),
        extraName,
      }),
    )) ?? false
  );
}

export async function coreCatalogRequiresExtra(catalog: unknown, extraName: string): Promise<boolean> {
  return (
    (await coreInvoke<boolean>(
      'catalogRequiresExtra',
      JSON.stringify({
        catalog: JSON.stringify(catalog),
        extraName,
      }),
    )) ?? false
  );
}

export async function coreCatalogHasRequiredExtraExcept(catalog: unknown, allowedNames: string[]): Promise<boolean> {
  return (
    (await coreInvoke<boolean>(
      'catalogHasRequiredExtraExcept',
      JSON.stringify({
        catalog: JSON.stringify(catalog),
        allowedNames: JSON.stringify(allowedNames),
      }),
    )) ?? false
  );
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
  return (await coreInvoke<AddonResourceResult>(
    'parseAddonResourceResult',
    JSON.stringify({
      resource,
      url,
      statusCode,
      body,
    }),
  )) as AddonResourceResult;
}

export type ParsedAndPlannedAddonResource =
  | { kind: 'success'; value: Record<string, unknown> }
  | { kind: 'network_error' | 'parse_error' | 'empty'; url: string; statusCode: number; error?: string };

export async function coreParseAndPlanAddonResource(
  resource: string,
  url: string,
  statusCode: number,
  body: string | null,
  kind: string,
  addonName: string | null,
  season: number | null,
): Promise<ParsedAndPlannedAddonResource> {
  return (await coreInvoke<ParsedAndPlannedAddonResource>(
    'parseAndPlanAddonResource',
    JSON.stringify({
      resource,
      url,
      statusCode,
      body,
      kind,
      addonName,
      season,
    }),
  )) as ParsedAndPlannedAddonResource;
}

export async function coreAddonResourceRequestPlan(request: unknown): Promise<{ urls: string[] } | null> {
  return coreInvoke('addonResourceRequestPlan', JSON.stringify(request));
}

export async function coreResourceFetchPlan(request: unknown): Promise<{ requests: Array<Record<string, unknown>> } | null> {
  return coreInvoke('resourceFetchPlan', JSON.stringify(request));
}

export type ResourceFetchExecutionPolicy = {
  requests: Array<Record<string, unknown>>;
  mode: 'race' | 'fanout';
  concurrency: number;
  streamRetry: { maxAttempts: number; fetchTimeoutMs: number; retryTimeoutMs: number };
};

export async function coreResourceFetchExecutionPolicy(request: unknown): Promise<ResourceFetchExecutionPolicy | null> {
  return coreInvoke('resourceFetchExecutionPolicy', JSON.stringify(request));
}

export async function coreResourceParsePlan(request: unknown): Promise<Record<string, unknown> | null> {
  return coreInvoke('resourceParsePlan', JSON.stringify(request));
}

export async function coreAddonCollectionMutationPlan(request: unknown): Promise<{ addons?: unknown[] } | null> {
  return coreInvoke('addonCollectionMutationPlan', JSON.stringify(request));
}
