import React, { useState } from 'react';
import type { CastMember, Meta, MetaLink } from '../../core/types';

const AVATAR_COLORS = ['#E8603C', '#9B59B6', '#3498DB', '#27AE60', '#E74C3C', '#1ABC9C', '#D35400', '#8E44AD'];

export function nameAvatarColor(name: string): string {
  let h = 0;
  for (let i = 0; i < name.length; i++) h = name.charCodeAt(i) + ((h << 5) - h);
  return AVATAR_COLORS[Math.abs(h) % AVATAR_COLORS.length];
}

export type NormalizedCastMember = {
  name: string;
  role?: string;
  imageUrl?: string;
};

export function buildCastMembers(meta: Meta): NormalizedCastMember[] {
  const record = meta as Meta & { app_extras?: { cast?: unknown[] }; appExtras?: { cast?: unknown[] } };
  const rawCast = [
    ...castArray(record.cast),
    ...castArray(record.app_extras?.cast),
    ...castArray(record.appExtras?.cast),
  ];
  const fromCast = rawCast.map(normalizeCastMember).filter(Boolean) as NormalizedCastMember[];
  if (fromCast.length > 0) return uniqueCastMembers(fromCast);

  return uniqueCastMembers(
    (meta.links ?? [])
      .filter(isCastLink)
      .map((link) => ({ name: link.name.trim() }))
      .filter((member) => member.name),
  );
}

function castArray(value: unknown): unknown[] {
  if (Array.isArray(value)) return value;
  return value ? [value] : [];
}

function isCastLink(link: MetaLink): boolean {
  const category = link.category.toLowerCase();
  return ['cast', 'actor', 'actors', 'starring'].some((key) => category.includes(key));
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
  const color = nameAvatarColor(name);
  const showImage = imageUrl && !imageFailed;
  return (
    <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'center', gap: 4, width: 92, flexShrink: 0 }}>
      <div
        style={{
          width: showImage ? 70 : 64,
          height: showImage ? 70 : 64,
          borderRadius: showImage ? 8 : '50%',
          background: showImage ? 'transparent' : color,
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          border: showImage ? '1px solid rgba(255,255,255,0.14)' : '2px solid rgba(255,255,255,0.12)',
          overflow: 'hidden',
        }}
      >
        {showImage ? (
          <img src={imageUrl} alt={name} style={{ width: '100%', height: '100%', objectFit: 'cover', display: 'block' }} onError={() => setImageFailed(true)} />
        ) : (
          <span style={{ color: '#fff', fontSize: 17, fontWeight: 800, fontFamily: 'sans-serif' }}>{initials}</span>
        )}
      </div>
      <p style={{ color: '#FFFFFF', fontSize: 12, fontWeight: 750, lineHeight: '14px', minHeight: 28, margin: '2px 0 0', textAlign: 'center', fontFamily: 'sans-serif', overflow: 'hidden', display: '-webkit-box', WebkitLineClamp: 2, WebkitBoxOrient: 'vertical' }}>{name}</p>
      <p style={{ color: 'rgba(255,255,255,0.38)', fontSize: 10, margin: 0, textAlign: 'center', fontFamily: 'sans-serif', overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap', maxWidth: '100%' }}>{role}</p>
    </div>
  );
});
