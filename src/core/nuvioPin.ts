import { coreInvoke, storageRead, storageWrite, storageDelete } from './engine';
import { nuvioVerifyPin, type NuvioPinVerifyResult } from './nuvioApi';
import type { UserProfile } from './types';

type CachedPin = { salt: string; digest: string; profileUpdatedAt?: string };
type OfflineResult = { unlocked: boolean; reason?: string };

const cacheKey = (profile: UserProfile) => `nuvio_profile_pin_cache_${profile.nuvioUserId ?? profile.id}_${profile.nuvioProfileIndex ?? 1}`;

export async function nuvioPinHash(profileIndex: number, salt: string, pin: string): Promise<string> {
  return (await coreInvoke<string>('nuvioPinHash', JSON.stringify({ profileIndex, salt, pin }))) ?? '';
}

export async function nuvioPinCachePayload(
  profileIndex: number,
  salt: string,
  pin: string,
  profileUpdatedAt?: string,
): Promise<CachedPin | null> {
  const raw = await coreInvoke<string>(
    'nuvioPinCachePayload',
    JSON.stringify({ profileIndex, salt, pin, profileUpdatedAt: profileUpdatedAt ?? '' }),
  );
  return raw ? (JSON.parse(raw) as CachedPin) : null;
}

export async function nuvioVerifyCachedPin(profile: UserProfile, pin: string): Promise<OfflineResult> {
  const cache = await storageRead<CachedPin>(cacheKey(profile));
  const raw = await coreInvoke<string>(
    'nuvioPinVerifyCached',
    JSON.stringify({
      profileIndex: profile.nuvioProfileIndex ?? 1,
      pin,
      pinEnabled: profile.nuvioPinEnabled ?? false,
      profileUpdatedAt: profile.nuvioProfileUpdatedAt ?? '',
      cache,
    }),
  );
  return raw ? (JSON.parse(raw) as OfflineResult) : { unlocked: false, reason: 'requires_online' };
}

export async function nuvioVerifyProfilePin(profile: UserProfile, pin: string): Promise<NuvioPinVerifyResult & { offline?: boolean }> {
  if (!profile.nuvioPinEnabled) return { unlocked: true };
  const cached = await nuvioVerifyCachedPin(profile, pin);
  if (cached.unlocked) return { unlocked: true, offline: true };
  if (cached.reason === 'incorrect' || cached.reason === 'profile_changed') {
    // A stale/incorrect cache must not bypass the server's lockout policy.
    if (cached.reason === 'profile_changed') await storageDelete(cacheKey(profile));
  }
  if (!profile.nuvioAccessToken || profile.nuvioProfileIndex == null) return { unlocked: false };
  const result = await nuvioVerifyPin(profile.nuvioAccessToken, profile.nuvioProfileIndex, pin);
  if (result.unlocked) {
    const salt =
      typeof crypto.randomUUID === 'function' ? crypto.randomUUID() : `${Date.now().toString(16)}-${Math.random().toString(16).slice(2)}`;
    const payload = await nuvioPinCachePayload(profile.nuvioProfileIndex, salt, pin, profile.nuvioProfileUpdatedAt);
    if (payload) await storageWrite(cacheKey(profile), payload);
  }
  return result;
}
