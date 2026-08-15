import { Component } from 'react';
import type { ReactNode } from 'react';
import { platformInvoke as invoke } from '../platform/invoke';
import { getSentryModule } from '../core/sentryRuntime';

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
        <div style={{ background: '#0a0a0a', color: '#ff4444', padding: '1.5rem', fontFamily: 'monospace', fontSize: '0.8125rem', whiteSpace: 'pre-wrap', overflow: 'auto', height: '100%', minHeight: '100vh' }}>
          {this.props.onReset && (
            <button
              onClick={() => {
                this.props.onReset?.();
                this.setState({ error: null });
              }}
              style={{ marginBottom: '1rem', padding: '0.5rem 1rem', background: 'rgba(255,255,255,0.1)', border: '1px solid rgba(255,255,255,0.2)', color: '#fff', borderRadius: '0.25rem', fontFamily: 'inherit', fontSize: 'inherit', cursor: 'pointer' }}
            >
              Go back
            </button>
          )}
          {'React render error: ' + this.state.error.message + '\n\n' + this.state.error.stack}
        </div>
      );
    }
    return this.props.children;
  }
}
