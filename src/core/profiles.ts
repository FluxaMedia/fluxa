import { storageRead, storageWrite } from './engine';
import type { UserProfile } from './types';

export const PROFILE_COLORS = [
  '#E85D3F',
  '#3F7CFF',
  '#54D17A',
  '#FF8A3D',
  '#C084FC',
  '#FFE45C',
  '#FF5D5D',
  '#38BDF8',
];

export async function loadProfiles(): Promise<UserProfile[]> {
  return (await storageRead<UserProfile[]>('profiles')) ?? [];
}

export async function saveProfiles(profiles: UserProfile[]): Promise<void> {
  await storageWrite('profiles', profiles);
}

export async function getActiveProfileId(): Promise<string | null> {
  return storageRead<string>('active_profile_id');
}

export async function setActiveProfileId(id: string): Promise<void> {
  await storageWrite('active_profile_id', id);
}

export function createProfileObject(name: string, color: string): UserProfile {
  const id =
    typeof crypto !== 'undefined' && typeof crypto.randomUUID === 'function'
      ? crypto.randomUUID()
      : `${Date.now()}-${Math.floor(Math.random() * 1e9)}`;
  return { id, name: name.trim(), color };
}

export async function saveProfile(profile: UserProfile): Promise<UserProfile[]> {
  const profiles = await loadProfiles();
  const idx = profiles.findIndex((p) => p.id === profile.id);
  if (idx >= 0) {
    profiles[idx] = profile;
  } else {
    profiles.push(profile);
  }
  await saveProfiles(profiles);
  return profiles;
}

export async function deleteProfile(id: string): Promise<UserProfile[]> {
  const profiles = (await loadProfiles()).filter((p) => p.id !== id);
  await saveProfiles(profiles);
  return profiles;
}

export function profileInitials(profile: UserProfile): string {
  const name = profile.name ?? profile.email ?? '?';
  return name
    .trim()
    .split(/\s+/)
    .slice(0, 2)
    .map((w) => w[0]?.toUpperCase() ?? '')
    .join('');
}

export function profileColor(profile: UserProfile): string {
  return profile.color ?? PROFILE_COLORS[0];
}

export function isTraktConnected(profile: UserProfile | null | undefined): boolean {
  if (!profile?.traktAccessToken) return false;
  if (profile.traktTokenExpiresAt && Date.now() / 1000 > profile.traktTokenExpiresAt) return false;
  return true;
}
