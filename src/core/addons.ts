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

export function normalizeAddonDescriptor(addon: LegacyAddon): AddonDescriptor {
  if (addon.manifest) {
    return {
      ...addon,
      transportUrl: addon.transportUrl ?? '',
      manifest: addon.manifest,
    };
  }

  const manifest: AddonManifest = {
    id: addon.id ?? '',
    name: addon.name?.trim() || 'Unknown Addon',
    description: addon.description ?? null,
    version: addon.version ?? null,
    resources: addon.resources ?? [],
    types: addon.types ?? [],
    catalogs: addon.catalogs ?? [],
    logo: addon.logo ?? null,
    background: addon.background ?? null,
    configurable: addon.behaviorHints?.configurable ?? null,
  };

  return {
    transportUrl: addon.transportUrl ?? addon.id ?? '',
    manifest,
  };
}

export function addonKey(addon: AddonDescriptor): string {
  return addon.transportUrl || addon.manifest.id;
}

export function addonManifest(addon: AddonDescriptor): AddonManifest {
  return addon.manifest ?? normalizeAddonDescriptor(addon).manifest;
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

function canonicalResourceName(value: string): string {
  const normalized = value.trim().toLowerCase().replace(/s$/, '');
  if (normalized === 'metadata') return 'meta';
  if (normalized === 'subtitle') return 'subtitle';
  return normalized;
}

export function addonHasResource(addon: AddonDescriptor, resourceName: string, contentType?: string | null): boolean {
  const manifest = addonManifest(addon);
  const expected = canonicalResourceName(resourceName);
  const manifestTypes = manifest.types ?? [];

  return addonResources(addon).some((resource) => {
    const name = typeof resource === 'string' ? resource : resource.name ?? resource.type ?? '';
    if (canonicalResourceName(name) !== expected) return false;
    if (!contentType) return true;
    const resourceTypes = typeof resource === 'string'
      ? manifestTypes
      : resource.types ?? (resource.type ? [resource.type] : manifestTypes);
    return resourceTypes.length === 0 || resourceTypes.some((item) => item.toLowerCase() === contentType.toLowerCase());
  });
}
