import { coreInvoke } from './engine';
import type { AddonDescriptor, AddonManifest, AddonResourceSpec, CatalogDef } from './types';

type LegacyAddon = Partial<AddonDescriptor> & {
  manifest?: AddonManifest;
  id?: string;
  name?: string;
  version?: string;
  description?: string;
  logo?: string;
  background?: string;
  resources?: Array<string | AddonResourceSpec>;
  types?: string[];
  catalogs?: CatalogDef[];
};

export async function normalizeAddonDescriptor(addon: LegacyAddon): Promise<AddonDescriptor> {
  const normalized = await coreInvoke<AddonDescriptor>('normalizeAddonDescriptor', JSON.stringify({ addonJson: JSON.stringify(addon) }));
  if (!normalized) throw new Error('Invalid addon descriptor');
  return normalized;
}

export function addonKey(addon: AddonDescriptor): string {
  return addon.transportUrl || addon.manifest.id;
}

export function addonManifest(addon: AddonDescriptor): AddonManifest {
  return addon.manifest;
}

export function addonName(addon: AddonDescriptor): string {
  const manifest = addonManifest(addon);
  return manifest.name || manifest.id || addon.transportUrl || 'Unknown Addon';
}

export function addonVersion(addon: AddonDescriptor): string | null {
  return addonManifest(addon).version ?? null;
}

export function addonLogo(addon: AddonDescriptor): string | null {
  return addonManifest(addon).logo ?? addonManifest(addon).background ?? null;
}

export function addonResources(addon: AddonDescriptor): Array<string | AddonResourceSpec> {
  return addonManifest(addon).resources ?? [];
}

export function addonTypes(addon: AddonDescriptor): string[] {
  return addonManifest(addon).types ?? [];
}

export function addonCatalogs(addon: AddonDescriptor): CatalogDef[] {
  return addonManifest(addon).catalogs ?? [];
}
