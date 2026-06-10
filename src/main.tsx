import React from 'react';
import ReactDOM from 'react-dom/client';
import App from './App';
import { ErrorBoundary } from './components/ErrorBoundary';
import './index.css';

try {
  ReactDOM.createRoot(document.getElementById('root')!).render(
    <ErrorBoundary>
      <App />
    </ErrorBoundary>,
  );
} catch (err) {
  const root = document.getElementById('root')!;
  root.style.cssText = 'background:#0a0a0a;color:#ff4444;padding:24px;font-family:monospace;font-size:13px;white-space:pre-wrap;overflow:auto';
  root.textContent = 'React mount error:\n' + (err instanceof Error ? err.stack : String(err));
}
