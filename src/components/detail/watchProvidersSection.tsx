import React, { useState } from 'react';
import { platformOpenExternal } from '../../platform/browser';
import type { WatchProvider, WatchProviders } from '../../core/types';

export function dedupedWatchProviders(watchProviders?: WatchProviders): WatchProvider[] {
  if (!watchProviders) return [];
  const seen = new Set<string>();
  return [...(watchProviders.flatrate ?? []), ...(watchProviders.rent ?? []), ...(watchProviders.buy ?? [])].filter((provider) => {
    if (seen.has(provider.name)) return false;
    seen.add(provider.name);
    return true;
  });
}

export const WatchProviderLogo = React.memo(function WatchProviderLogo({
  name,
  logo,
  onClick,
}: {
  name: string;
  logo?: string;
  onClick: () => void;
}) {
  const [imageFailed, setImageFailed] = useState(false);
  const showImage = logo && !imageFailed;
  return (
    <button
      onClick={onClick}
      title={name}
      style={{
        width: '3.25rem',
        height: '3.25rem',
        flexShrink: 0,
        padding: 0,
        border: '1px solid rgba(255,255,255,0.14)',
        borderRadius: '0.75rem',
        background: showImage ? 'transparent' : 'rgba(255,255,255,0.1)',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        overflow: 'hidden',
        cursor: 'pointer',
      }}
    >
      {showImage ? (
        <img
          src={logo}
          alt={name}
          style={{ width: '100%', height: '100%', objectFit: 'cover', display: 'block' }}
          onError={() => setImageFailed(true)}
        />
      ) : (
        <span style={{ color: 'rgba(255,255,255,0.7)', fontSize: '0.6875rem', fontWeight: 700 }}>{name.slice(0, 2).toUpperCase()}</span>
      )}
    </button>
  );
});

export function openWatchProvidersLink(link?: string) {
  if (!link) return;
  void platformOpenExternal(link).catch(() => {});
}
