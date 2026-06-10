import { storageWrite } from './engine';
import {
  coreAddonResourceRequestPlan,
  manifestFetchPlan,
  mergeLiveManifest,
  parseManifest,
  resolveManifestAssets,
} from './addonManifest';
import { fetchJson } from './httpClient';
import { fetchParsedAddonResource } from './fetchPlanning';
import { loadAddons } from './libraryOps';
import { addonName, normalizeAddonDescriptor } from './addons';
import type { AddonDescriptor } from './types';

export async function fetchAddonManifest(payload: Record<string, unknown>): Promise<unknown> {
  const transportUrl = payload.transportUrl as string;
  const plan = await manifestFetchPlan(transportUrl);
  const candidateUrls = plan?.candidateUrls?.length ? plan.candidateUrls : [transportUrl];

  for (const candidateUrl of candidateUrls) {
    try {
      const data = await fetchJson(candidateUrl);
      const parsed = await parseManifest(JSON.stringify(data), candidateUrl);
      if (!parsed) continue;
      const resolved = await resolveManifestAssets(parsed);
      return normalizeAddonDescriptor((resolved ?? parsed) as AddonDescriptor);
    } catch {
      // Try the next candidate, matching Android's manifest fetch behavior.
    }
  }

  throw new Error(`Unable to fetch addon manifest: ${transportUrl}`);
}

export async function refreshInstalledAddons(_payload: Record<string, unknown>): Promise<unknown> {
  const addons = await loadAddons();
  const refreshed: AddonDescriptor[] = [];
  for (const addon of addons) {
    try {
      const manifest = await fetchAddonManifest({ transportUrl: addon.transportUrl });
      const merged = await mergeLiveManifest(addon, manifest, addonName(addon));
      refreshed.push(normalizeAddonDescriptor((merged ?? manifest) as AddonDescriptor));
    } catch {
      refreshed.push(addon);
    }
  }
  await storageWrite('addons', refreshed);
  return { addons: refreshed };
}

export async function fetchAddonResource(payload: Record<string, unknown>): Promise<unknown> {
  const resource = String(payload.resource ?? '');
  const plan = await coreAddonResourceRequestPlan({
    transportUrl: payload.transportUrl,
    resource,
    contentType: payload.contentType,
    id: payload.id,
    extra: payload.extra,
  });
  const values = await Promise.all(
    (plan?.urls ?? []).map((url) => fetchParsedAddonResource(url, resource, 'addonResource')),
  );
  return values[0] ?? null;
}
