import React, { useState } from 'react';
import { coreInvoke } from '../../core/engine';
import type { CastMember, Meta, MetaLink } from '../../core/types';

export type NormalizedCastMember = {
  name: string;
  role?: string;
  imageUrl?: string;
};

export async function buildCastMembers(meta: Meta): Promise<NormalizedCastMember[]> {
  const record = meta as Meta & { app_extras?: { cast?: unknown[] }; appExtras?: { cast?: unknown[] } };
  const rawCast = [
    ...castArray(record.cast),
    ...castArray(record.app_extras?.cast),
    ...castArray(record.appExtras?.cast),
  ];
  const fromCast = rawCast.map(normalizeCastMember).filter(Boolean) as NormalizedCastMember[];
  if (fromCast.length > 0) return uniqueCastMembers(fromCast);

  const classified = await coreInvoke<{ cast: MetaLink[] }>('classifyMetaLinks', JSON.stringify(meta.links ?? []));
  return uniqueCastMembers(
    (classified?.cast ?? [])
      .map((link) => ({ name: link.name.trim() }))
      .filter((member) => member.name),
  );
}

function castArray(value: unknown): unknown[] {
  if (Array.isArray(value)) return value;
  return value ? [value] : [];
}

function normalizeCastMember(value: unknown): NormalizedCastMember | null {
  if (typeof value === 'string') {
    const name = value.trim();
    return name ? { name } : null;
  }
  if (!value || typeof value !== 'object') return null;
  const item = value as CastMember & Record<string, unknown>;
  const name = castMemberName(item);
  if (!name) return null;
  return {
    name,
    role: optionalString(item.character) ?? optionalString(item.role) ?? optionalString(item.as) ?? undefined,
    imageUrl: normalizeProfileImage(item.profilePath ?? item.profile_path ?? item.photo ?? item.profile ?? item.image ?? item.img),
  };
}

function castMemberName(item: CastMember & Record<string, unknown>): string {
  const explicit = castNameValue(item.name ?? item.fullName ?? item.full_name ?? item.actor ?? item.person);
  const firstName = optionalString(item.firstName) ?? optionalString(item.first_name) ?? optionalString(item.first) ?? optionalString(item.givenName) ?? optionalString(item.given_name);
  const lastName = optionalString(item.lastName) ?? optionalString(item.last_name) ?? optionalString(item.last) ?? optionalString(item.surname) ?? optionalString(item.familyName) ?? optionalString(item.family_name);
  if (explicit && lastName && !explicit.toLowerCase().includes(lastName.toLowerCase())) return `${explicit} ${lastName}`;
  if (explicit) return explicit;
  return [firstName, lastName].filter(Boolean).join(' ').trim();
}

function castNameValue(value: unknown): string | null {
  const direct = optionalString(value);
  if (direct) return direct;
  if (!value || typeof value !== 'object') return null;
  const item = value as Record<string, unknown>;
  const explicit = optionalString(item.fullName) ?? optionalString(item.full_name) ?? optionalString(item.name);
  const firstName = optionalString(item.firstName) ?? optionalString(item.first_name) ?? optionalString(item.first) ?? optionalString(item.givenName) ?? optionalString(item.given_name);
  const lastName = optionalString(item.lastName) ?? optionalString(item.last_name) ?? optionalString(item.last) ?? optionalString(item.surname) ?? optionalString(item.familyName) ?? optionalString(item.family_name);
  if (explicit && lastName && !explicit.toLowerCase().includes(lastName.toLowerCase())) return `${explicit} ${lastName}`;
  if (explicit) return explicit;
  return [firstName, lastName].filter(Boolean).join(' ').trim() || null;
}

function optionalString(value: unknown): string | null {
  return typeof value === 'string' && value.trim() ? value.trim() : null;
}

function normalizeProfileImage(value: unknown): string | undefined {
  const image = optionalString(value);
  if (!image) return undefined;
  if (image.startsWith('//')) return `https:${image}`;
  if (image.startsWith('/')) return `https://image.tmdb.org/t/p/w185${image}`;
  return image;
}

function uniqueCastMembers(items: NormalizedCastMember[]): NormalizedCastMember[] {
  const seen = new Set<string>();
  const result: NormalizedCastMember[] = [];
  for (const item of items) {
    const key = item.name.trim().toLowerCase();
    if (!key || seen.has(key)) continue;
    seen.add(key);
    result.push(item);
  }
  return result;
}

export const CastAvatar = React.memo(function CastAvatar({ name, role, imageUrl }: { name: string; role: string; imageUrl?: string }) {
  const [imageFailed, setImageFailed] = useState(false);
  const initials = name.split(' ').slice(0, 2).map((w) => w[0] ?? '').join('').toUpperCase();
  const showImage = imageUrl && !imageFailed;
  return (
    <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'center', gap: '0.25rem', width: '6.5rem', flexShrink: 0 }}>
      <div
        style={{
          width: '4.375rem',
          height: '4.375rem',
          borderRadius: '50%',
          background: showImage ? 'transparent' : 'rgba(255,255,255,0.1)',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          border: '1px solid rgba(255,255,255,0.14)',
          overflow: 'hidden',
        }}
      >
        {showImage ? (
          <img src={imageUrl} alt={name} style={{ width: '100%', height: '100%', objectFit: 'cover', display: 'block' }} onError={() => setImageFailed(true)} />
        ) : (
          <span style={{ color: 'rgba(255,255,255,0.7)', fontSize: '1rem', fontWeight: 600 }}>{initials}</span>
        )}
      </div>
      <p style={{ color: '#FFFFFF', fontSize: '0.75rem', fontWeight: 750, lineHeight: '0.875rem', margin: '0.125rem 0 0', textAlign: 'center', overflow: 'hidden', display: '-webkit-box', WebkitLineClamp: 2, WebkitBoxOrient: 'vertical' as const }}>{name}</p>
      <p style={{ color: 'rgba(255,255,255,0.38)', fontSize: '0.625rem', margin: '1px 0 0', textAlign: 'center', overflow: 'hidden', display: '-webkit-box', WebkitLineClamp: 2, WebkitBoxOrient: 'vertical' as const }}>{role}</p>
    </div>
  );
});
