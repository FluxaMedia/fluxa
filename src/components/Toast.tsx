import { AlertTriangle, Check, X } from 'lucide-react';
import { useState } from 'react';

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
  warning: { icon: '#f0b74a', iconBg: 'rgba(240,183,74,0.13)' },
  error: { icon: '#ff7b7b', iconBg: 'rgba(255,90,90,0.13)' },
  success: { icon: '#8fd19e', iconBg: 'rgba(143,209,158,0.13)' },
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
        background: 'rgba(20,22,28,0.97)',
        border: '1px solid rgba(255,255,255,0.1)',
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
          <p style={{ color: '#fff', fontSize: '0.875rem', fontWeight: 700, margin: 0 }}>{title}</p>
          <p style={{ color: 'rgba(255,255,255,0.62)', fontSize: '0.8125rem', margin: '0.1875rem 0 0', lineHeight: 1.4 }}>{message}</p>
          {details && (
            <>
              <button
                onClick={() => setDetailsOpen((v) => !v)}
                style={{
                  background: 'none',
                  border: 'none',
                  padding: 0,
                  marginTop: '0.5rem',
                  color: 'rgba(255,255,255,0.42)',
                  fontSize: '0.7188rem',
                  fontWeight: 600,
                  cursor: 'pointer',
                }}
              >
                {detailsOpen ? detailsHideLabel : detailsLabel}
              </button>
              {detailsOpen && (
                <pre
                  style={{
                    margin: '0.5rem 0 0',
                    padding: '0.5rem 0.625rem',
                    background: 'rgba(0,0,0,0.4)',
                    border: '1px solid rgba(255,255,255,0.06)',
                    borderRadius: '0.375rem',
                    color: 'rgba(255,255,255,0.55)',
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
                <button
                  key={action.label}
                  onClick={action.onClick}
                  style={{
                    background: action.primary ? '#fff' : 'rgba(255,255,255,0.08)',
                    border: action.primary ? 'none' : '1px solid rgba(255,255,255,0.14)',
                    borderRadius: '0.5rem',
                    padding: '0.4375rem 0.875rem',
                    color: action.primary ? '#000' : 'rgba(255,255,255,0.85)',
                    fontSize: '0.8125rem',
                    fontWeight: 700,
                    cursor: 'pointer',
                  }}
                >
                  {action.label}
                </button>
              ))}
            </div>
          )}
        </div>
        {onClose && (
          <button
            onClick={onClose}
            style={{
              background: 'none',
              border: 'none',
              padding: '0.125rem',
              margin: '-0.125rem -0.25rem 0 0',
              color: 'rgba(255,255,255,0.4)',
              cursor: 'pointer',
              flexShrink: 0,
              display: 'flex',
            }}
          >
            <X size={15} />
          </button>
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
