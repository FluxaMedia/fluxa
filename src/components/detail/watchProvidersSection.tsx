import React, { useState } from 'react';
import { color, fade, fontSize, radius } from '../../design';
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
  size = '3.25rem',
}: {
  name: string;
  logo?: string;
  onClick: () => void;
  size?: string;
}) {
  const [imageFailed, setImageFailed] = useState(false);
  const showImage = logo && !imageFailed;
  return (
    <button
      onClick={onClick}
      title={name}
      style={{
        width: size,
        height: size,
        flexShrink: 0,
        padding: 0,
        border: `1px solid ${color.lineStrong}`,
        borderRadius: radius.md,
        background: showImage ? 'transparent' : color.fillHover,
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
        <span style={{ color: color.textBody, fontSize: fontSize.xs, fontWeight: 700 }}>{name.slice(0, 2).toUpperCase()}</span>
      )}
    </button>
  );
});

export function openWatchProvidersLink(link?: string) {
  if (!link) return;
  void platformOpenExternal(link).catch(() => {});
}
