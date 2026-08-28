import { Component } from 'react';
import type { ReactNode } from 'react';
import { platformInvoke as invoke } from '../platform/invoke';
import { getSentryModule } from '../core/sentryRuntime';
import { t } from '../i18n';

interface Props {
  children: ReactNode;
  resetKeys?: unknown[];
  onReset?: () => void;
}

interface State {
  error: Error | null;
}

export class ErrorBoundary extends Component<Props, State> {
  constructor(props: Props) {
    super(props);
    this.state = { error: null };
  }

  static getDerivedStateFromError(error: Error) {
    return { error };
  }

  componentDidCatch(error: Error) {
    void invoke('debug_log', { msg: `ErrorBoundary caught: ${error.message}\n${error.stack}` }).catch(() => {});
    getSentryModule()?.captureException(error);
  }

  componentDidUpdate(prevProps: Props) {
    if (!this.state.error || !this.props.resetKeys) return;
    const prevKeys = prevProps.resetKeys ?? [];
    const changed = this.props.resetKeys.some((key, i) => key !== prevKeys[i]);
    if (changed) this.setState({ error: null });
  }

  render() {
    if (this.state.error) {
        return (
        <div
          style={{
            background: '#0a0a0a',
            color: '#fff',
            padding: '2rem',
            height: '100%',
            minHeight: '100vh',
            display: 'grid',
            placeItems: 'center',
          }}
        >
          <div style={{ maxWidth: '32rem', textAlign: 'center' }}>
            <h2 style={{ margin: '0 0 0.75rem', fontSize: '1.35rem' }}>{t('error.generic_title')}</h2>
            <p style={{ margin: '0 0 1.5rem', color: 'rgba(255,255,255,0.65)', lineHeight: 1.5 }}>
              {t('error.generic_message')}
            </p>
            <button
              onClick={() => {
                if (this.props.onReset) {
                  this.props.onReset();
                  this.setState({ error: null });
                  return;
                }
                window.location.reload();
              }}
              style={{
                padding: '0.65rem 1rem',
                background: '#fff',
                border: 0,
                color: '#0a0a0a',
                borderRadius: '0.5rem',
                fontSize: '0.9rem',
                fontWeight: 700,
                cursor: 'pointer',
              }}
            >
              {this.props.onReset ? t('common.back') : t('common.retry')}
            </button>
          </div>
        </div>
      );
    }
    return this.props.children;
  }
}
