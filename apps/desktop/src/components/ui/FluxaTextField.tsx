import type { InputHTMLAttributes } from 'react';

export function FluxaTextField({ style, ...props }: InputHTMLAttributes<HTMLInputElement>) {
  return (
    <input
      {...props}
      style={{
        width: '100%',
        boxSizing: 'border-box',
        minHeight: '2.75rem',
        padding: '0.75rem 0.875rem',
        borderRadius: '0.625rem',
        border: '1px solid var(--fluxa-border-strong)',
        background: 'var(--fluxa-surface)',
        color: 'var(--fluxa-text-primary)',
        outline: 'none',
        font: 'inherit',
        ...style,
      }}
    />
  );
}
