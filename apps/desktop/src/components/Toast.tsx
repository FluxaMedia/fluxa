import { AlertTriangle, Check, X } from 'lucide-react';
import { useState } from 'react';
import { FluxaButton, FluxaIconButton } from './ui/FluxaButton';

interface ToastAction {
  label: string;
  onClick: () => void;
  primary?: boolean;
}

interface ToastProps {
  variant?: 'warning' | 'error' | 'success';
  title: string;
  message: string;
  details?: string;
  detailsLabel?: string;
  detailsHideLabel?: string;
  actions?: ToastAction[];
  onClose?: () => void;
}

const VARIANT_COLORS: Record<NonNullable<ToastProps['variant']>, { icon: string; iconBg: string }> = {
  warning: { icon: 'var(--fluxa-warning)', iconBg: 'var(--fluxa-warning-soft)' },
  error: { icon: 'var(--fluxa-error)', iconBg: 'var(--fluxa-error-soft)' },
  success: { icon: 'var(--fluxa-success)', iconBg: 'var(--fluxa-success-soft)' },
};

export function Toast({ variant = 'warning', title, message, details, detailsLabel, detailsHideLabel, actions, onClose }: ToastProps) {
  const [detailsOpen, setDetailsOpen] = useState(false);
  const colors = VARIANT_COLORS[variant];
  const Icon = variant === 'success' ? Check : AlertTriangle;

  return (
    <div
      className="fluxa-toast"
      style={{
        width: 'min(23.75rem, 100%)',
        boxSizing: 'border-box',
        background: 'var(--fluxa-surface-raised)',
        border: '1px solid var(--fluxa-border)',
        borderRadius: '0.75rem',
        boxShadow: '0 0.75rem 2.5rem rgba(0,0,0,0.5)',
        padding: '0.875rem 1rem',
        pointerEvents: 'auto',
      }}
    >
      <div style={{ display: 'flex', alignItems: 'flex-start', gap: '0.75rem' }}>
        <span
          style={{
            width: '2rem',
            height: '2rem',
            borderRadius: '0.5rem',
            background: colors.iconBg,
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            flexShrink: 0,
          }}
        >
          <Icon size={16} color={colors.icon} />
        </span>
        <div style={{ minWidth: 0, flex: 1 }}>
          <p style={{ color: 'var(--fluxa-text-primary)', fontSize: '0.875rem', fontWeight: 700, margin: 0 }}>{title}</p>
          <p style={{ color: 'var(--fluxa-text-secondary)', fontSize: '0.8125rem', margin: '0.1875rem 0 0', lineHeight: 1.4 }}>{message}</p>
          {details && (
            <>
              <FluxaButton
                variant="ghost"
                size="sm"
                onClick={() => setDetailsOpen((v) => !v)}
                style={{
                  minHeight: 'auto',
                  padding: 0,
                  marginTop: '0.5rem',
                  color: 'var(--fluxa-text-muted)',
                }}
              >
                {detailsOpen ? detailsHideLabel : detailsLabel}
              </FluxaButton>
              {detailsOpen && (
                <pre
                  style={{
                    margin: '0.5rem 0 0',
                    padding: '0.5rem 0.625rem',
                    background: 'var(--fluxa-background)',
                    border: '1px solid var(--fluxa-border)',
                    borderRadius: '0.375rem',
                    color: 'var(--fluxa-text-secondary)',
                    fontSize: '0.6875rem',
                    fontFamily: "'Cascadia Mono', 'Consolas', monospace",
                    lineHeight: 1.5,
                    whiteSpace: 'pre-wrap',
                    wordBreak: 'break-word',
                    maxHeight: '7.5rem',
                    overflowY: 'auto',
                  }}
                >
                  {details}
                </pre>
              )}
            </>
          )}
          {actions && actions.length > 0 && (
            <div style={{ display: 'flex', gap: '0.5rem', marginTop: '0.75rem' }}>
              {actions.map((action) => (
                <FluxaButton
                  variant={action.primary ? 'primary' : 'secondary'}
                  size="sm"
                  key={action.label}
                  onClick={action.onClick}
                >
                  {action.label}
                </FluxaButton>
              ))}
            </div>
          )}
        </div>
        {onClose && (
          <FluxaIconButton
            ariaLabel="Close"
            size="sm"
            variant="ghost"
            onClick={onClose}
            style={{
              margin: '-0.125rem -0.25rem 0 0',
              color: 'rgba(255,255,255,0.4)',
              flexShrink: 0,
            }}
          >
            <X size={15} />
          </FluxaIconButton>
        )}
      </div>
      <style>{`
        @keyframes fluxa-toast-in {
          from { opacity: 0; transform: translateX(0.875rem); }
          to { opacity: 1; transform: translateX(0); }
        }
        .fluxa-toast {
          animation: fluxa-toast-in 0.22s cubic-bezier(0.16, 1, 0.3, 1);
        }
      `}</style>
    </div>
  );
}
