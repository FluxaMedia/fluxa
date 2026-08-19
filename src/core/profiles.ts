import { coreInvoke, storageRead, storageWrite } from './engine';
import type { UserProfile } from './types';

export const PROFILE_COLORS = ['#E85D3F', '#3F7CFF', '#54D17A', '#FF8A3D', '#C084FC', '#FFE45C', '#FF5D5D', '#38BDF8'];

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

export async function createProfileObject(name: string, color: string): Promise<UserProfile> {
  const id =
    typeof crypto !== 'undefined' && typeof crypto.randomUUID === 'function'
      ? crypto.randomUUID()
      : `${Date.now()}-${Math.floor(Math.random() * 1e9)}`;
  return (await coreInvoke<UserProfile>('createProfilePlan', JSON.stringify({ id, name, color }))) ?? { id, name: name.trim(), color };
}

export async function saveProfile(profile: UserProfile): Promise<UserProfile[]> {
  const profiles = await loadProfiles();
  const next =
    (await coreInvoke<UserProfile[]>('profileMutationPlan', JSON.stringify({ operation: 'save', profiles, profile }))) ?? profiles;
  await saveProfiles(next);
  return next;
}

export async function deleteProfile(id: string): Promise<UserProfile[]> {
  const profiles = await loadProfiles();
  const next = (await coreInvoke<UserProfile[]>('profileMutationPlan', JSON.stringify({ operation: 'delete', profiles, id }))) ?? profiles;
  await saveProfiles(next);
  return next;
}

export async function hashPin(pin: string): Promise<string> {
  return (await coreInvoke<string>('profilePinHash', JSON.stringify({ pin }))) ?? '';
}

export async function verifyPin(profile: UserProfile, pin: string): Promise<boolean> {
  return (await coreInvoke<boolean>('profilePinMatches', JSON.stringify({ profileJson: JSON.stringify(profile), pin }))) ?? false;
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

export async function profileConnectionState(profile: UserProfile | null | undefined): Promise<{ trakt: boolean; simkl: boolean }> {
  return (
    (await coreInvoke<{ trakt: boolean; simkl: boolean }>(
      'profileConnectionState',
      JSON.stringify({
        profileJson: JSON.stringify(profile ?? null),
        nowEpochSeconds: Math.floor(Date.now() / 1000),
      }),
    )) ?? { trakt: false, simkl: false }
  );
}
