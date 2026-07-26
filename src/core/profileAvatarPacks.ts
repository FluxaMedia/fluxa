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

export async function discoverProfileAvatarPacks(repositoryUrl: string): Promise<ProfileAvatarPack[]> {
  const repositoryPlan = await coreInvoke<{ repositoryApiUrl: string } | null>(
    'profileAvatarPackRepositoryPlan',
    JSON.stringify({ repositoryUrl }),
  );
  if (!repositoryPlan) return [];
  const repository = await fetchJson(repositoryPlan.repositoryApiUrl);
  const discoveryPlan = await coreInvoke<{ reference: string; treeApiUrl: string } | null>(
    'profileAvatarPackDiscoveryPlan',
    JSON.stringify({ repositoryUrl, repository }),
  );
  if (!discoveryPlan) return [];
  const tree = await fetchJson(discoveryPlan.treeApiUrl);
  const catalog = await coreInvoke<{ categories: Array<{ manifestUrl: string }> } | null>(
    'profileAvatarPackCatalog',
    JSON.stringify({ repositoryUrl, reference: discoveryPlan.reference, tree }),
  );
  if (!catalog?.categories.length) return [];
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
  return packs.filter((pack): pack is ProfileAvatarPack => pack !== null);
}
