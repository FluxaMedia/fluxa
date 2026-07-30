import { coreInvoke, storageRead, storageWrite } from './engine';
import { fetchJson } from './httpClient';

const STORAGE_KEY = 'profile_picker_settings';

export interface ProfileAvatar {
  name: string;
  url: string;
}

export interface ProfileAvatarPack {
  id: string;
  repositoryUrl: string;
  title: string;
  manifestUrl: string;
  avatars: ProfileAvatar[];
}

export interface ProfilePickerSettings {
  backgroundUrl?: string;
  avatarPacks: ProfileAvatarPack[];
}

export type AvatarPackDiscoveryReason = 'invalid_url' | 'repository_not_found' | 'no_packs_found' | 'no_valid_avatars';

export class AvatarPackDiscoveryError extends Error {
  constructor(public reason: AvatarPackDiscoveryReason) {
    super(reason);
    this.name = 'AvatarPackDiscoveryError';
  }
}

export async function loadProfilePickerSettings(): Promise<ProfilePickerSettings> {
  const stored = await storageRead<Partial<ProfilePickerSettings>>(STORAGE_KEY);
  return {
    backgroundUrl: stored?.backgroundUrl,
    avatarPacks: Array.isArray(stored?.avatarPacks) ? stored.avatarPacks : [],
  };
}

export async function saveProfilePickerSettings(settings: ProfilePickerSettings): Promise<void> {
  await storageWrite(STORAGE_KEY, settings);
}

async function discoverDirectManifestPack(repositoryUrl: string): Promise<ProfileAvatarPack | null> {
  const manifestPlan = await coreInvoke<{ manifestUrl: string } | null>(
    'profileAvatarPackManifestPlan',
    JSON.stringify({ repositoryUrl }),
  );
  if (!manifestPlan) return null;
  const pack = await fetchJson(manifestPlan.manifestUrl);
  const parsed = await coreInvoke<{ title: string; manifestUrl: string; avatars: ProfileAvatar[] } | null>(
    'profileAvatarPackParse',
    JSON.stringify({ manifestUrl: manifestPlan.manifestUrl, pack }),
  );
  if (!parsed || !parsed.avatars.length) throw new AvatarPackDiscoveryError('no_valid_avatars');
  return { ...parsed, id: parsed.manifestUrl, repositoryUrl };
}

export async function discoverProfileAvatarPacks(repositoryUrl: string): Promise<ProfileAvatarPack[]> {
  const directPack = await discoverDirectManifestPack(repositoryUrl);
  if (directPack) return [directPack];

  const repositoryPlan = await coreInvoke<{ repositoryApiUrl: string } | null>(
    'profileAvatarPackRepositoryPlan',
    JSON.stringify({ repositoryUrl }),
  );
  if (!repositoryPlan) throw new AvatarPackDiscoveryError('invalid_url');
  const repository = await fetchJson(repositoryPlan.repositoryApiUrl);
  const discoveryPlan = await coreInvoke<{ reference: string; treeApiUrl: string } | null>(
    'profileAvatarPackDiscoveryPlan',
    JSON.stringify({ repositoryUrl, repository }),
  );
  if (!discoveryPlan) throw new AvatarPackDiscoveryError('repository_not_found');
  const tree = await fetchJson(discoveryPlan.treeApiUrl);
  const catalog = await coreInvoke<{ categories: Array<{ manifestUrl: string }> } | null>(
    'profileAvatarPackCatalog',
    JSON.stringify({ repositoryUrl, reference: discoveryPlan.reference, tree }),
  );
  if (!catalog?.categories.length) throw new AvatarPackDiscoveryError('no_packs_found');
  const packs = await Promise.all(catalog.categories.map(async ({ manifestUrl }) => {
    const pack = await fetchJson(manifestUrl);
    const parsed = await coreInvoke<{ title: string; manifestUrl: string; avatars: ProfileAvatar[] } | null>(
      'profileAvatarPackParse',
      JSON.stringify({ manifestUrl, pack }),
    );
    return parsed && parsed.avatars.length
      ? { ...parsed, id: parsed.manifestUrl, repositoryUrl }
      : null;
  }));
  const found = packs.filter((pack): pack is ProfileAvatarPack => pack !== null);
  if (!found.length) throw new AvatarPackDiscoveryError('no_valid_avatars');
  return found;
}

export async function refreshAvatarPackRepository(repositoryUrl: string): Promise<ProfilePickerSettings> {
  const settings = await loadProfilePickerSettings();
  const refreshed = await discoverProfileAvatarPacks(repositoryUrl);
  const kept = settings.avatarPacks.filter((pack) => pack.repositoryUrl !== repositoryUrl);
  const next: ProfilePickerSettings = { ...settings, avatarPacks: [...kept, ...refreshed] };
  await saveProfilePickerSettings(next);
  return next;
}

export async function refreshAllAvatarPacks(): Promise<void> {
  const settings = await loadProfilePickerSettings();
  const repositoryUrls = Array.from(new Set(settings.avatarPacks.map((pack) => pack.repositoryUrl)));
  for (const repositoryUrl of repositoryUrls) {
    try {
      await refreshAvatarPackRepository(repositoryUrl);
    } catch {}
  }
}
