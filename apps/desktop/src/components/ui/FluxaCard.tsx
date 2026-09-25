import type { CSSProperties, HTMLAttributes } from 'react';

export function FluxaCard({ style, ...props }: HTMLAttributes<HTMLDivElement>) {
  return (
    <div
      {...props}
      style={{
        background: 'var(--fluxa-surface-card, var(--fluxa-surface-raised))',
        border: '1px solid var(--fluxa-border)',
        borderRadius: '0.875rem',
        ...style,
      } as CSSProperties}
    />
  );
}
