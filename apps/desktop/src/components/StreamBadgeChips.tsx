import React from 'react';
import type { StreamBadge } from '../core/types';

export function StreamBadgeChips({ badges }: { badges: StreamBadge[] }) {
  const imageBadges = badges.filter((badge) => badge.imageUrl?.trim());
  if (imageBadges.length === 0) return null;
  return (
    <div style={styles.row}>
      {imageBadges.map((badge) => (
        <img key={badge.imageUrl} src={badge.imageUrl} alt={badge.name} title={badge.name} style={styles.image} />
      ))}
    </div>
  );
}

const styles: Record<string, React.CSSProperties> = {
  row: {
    display: 'flex',
    alignItems: 'center',
    gap: '0.3125rem',
    flexWrap: 'wrap',
  },
  image: {
    height: '0.875rem',
    maxWidth: '3.5rem',
    objectFit: 'contain',
  },
};
