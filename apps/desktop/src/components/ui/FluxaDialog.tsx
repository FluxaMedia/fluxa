import type { CSSProperties, ReactNode } from 'react';
import { useEscapeKey } from '../../hooks/useEscapeKey';

export function FluxaDialog({
  title,
  description,
  onDismiss,
  children,
  width = 'min(30rem, calc(100vw - 2rem))',
}: {
  title?: ReactNode;
  description?: ReactNode;
  onDismiss: () => void;
  children: ReactNode;
  width?: CSSProperties['width'];
}) {
  useEscapeKey(onDismiss);
  return (
    <div style={dialogOverlay} onClick={onDismiss}>
      <div style={{ ...dialogSurface, width }} onClick={(event) => event.stopPropagation()}>
        {title && <p style={dialogTitle}>{title}</p>}
        {description && <p style={dialogDescription}>{description}</p>}
        {children}
      </div>
    </div>
  );
}

const dialogOverlay: CSSProperties = {
  position: 'fixed',
  inset: 0,
  zIndex: 10000,
  background: 'var(--fluxa-scrim)',
  display: 'flex',
  alignItems: 'center',
  justifyContent: 'center',
  padding: '1rem',
};

const dialogSurface: CSSProperties = {
  boxSizing: 'border-box',
  borderRadius: '1rem',
  background: 'var(--fluxa-surface-raised)',
  border: '1px solid var(--fluxa-border)',
  boxShadow: '0 1.5rem 4rem rgba(0,0,0,0.55)',
  padding: '1.5rem',
  fontFamily: "'Montserrat', -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif",
};

const dialogTitle: CSSProperties = { margin: 0, color: 'var(--fluxa-text-primary)', fontSize: '1rem', fontWeight: 700 };
const dialogDescription: CSSProperties = { margin: '0.625rem 0 1.25rem', color: 'var(--fluxa-text-secondary)', fontSize: '0.8125rem', lineHeight: 1.5 };
