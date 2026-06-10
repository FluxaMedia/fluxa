import { Component } from 'react';
import type { ReactNode } from 'react';

export class ErrorBoundary extends Component<{ children: ReactNode }, { error: Error | null }> {
  constructor(props: { children: ReactNode }) {
    super(props);
    this.state = { error: null };
  }

  static getDerivedStateFromError(error: Error) {
    return { error };
  }

  render() {
    if (this.state.error) {
      return (
        <div style={{ background: '#0a0a0a', color: '#ff4444', padding: 24, fontFamily: 'monospace', fontSize: 13, whiteSpace: 'pre-wrap', overflow: 'auto', height: '100vh' }}>
          {'React render error:\n' + this.state.error.stack}
        </div>
      );
    }
    return this.props.children;
  }
}
