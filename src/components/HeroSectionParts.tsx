import React, { useState } from 'react';
import type { Meta } from '../core/types';

export function HeroIconBtn({ onClick, title, ariaLabel, children }: { onClick?: () => void; title?: string; ariaLabel?: string; children: React.ReactNode }) {
  const [hovered, setHovered] = useState(false);
  return (
    <button
      title={title}
      aria-label={ariaLabel ?? title}
      onClick={onClick}
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
      style={{
        width: '2.625rem',
        height: '2.625rem',
        borderRadius: '50%',
        background: hovered ? 'rgba(255,255,255,0.18)' : 'rgba(255,255,255,0.10)',
        border: '0.125rem solid rgba(255,255,255,0.55)',
        color: '#FFFFFF',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        cursor: 'pointer',
        padding: 0,
        flexShrink: 0,
        transition: 'background 0.15s',
        boxShadow: '0 0.25rem 1rem rgba(0,0,0,0.35)',
      }}
    >
      {children}
    </button>
  );
}

export function parseReleaseYear(releaseInfo?: string): number | null {
  const match = releaseInfo?.match(/\b(19|20)\d{2}\b/);
  return match ? Number(match[0]) : null;
}

export function readOptionalString(meta: Meta, keys: string[]): string | null {
  const record = meta as unknown as Record<string, unknown>;
  for (const key of keys) {
    const value = record[key];
    if (typeof value === 'string' && value.trim()) return value.trim();
  }
  return null;
}
