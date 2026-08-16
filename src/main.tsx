import React from 'react';
import ReactDOM from 'react-dom/client';
import App from './App';
import { ErrorBoundary } from './components/ErrorBoundary';
import { initDiagnosticsSentry } from './core/sentryRuntime';
import { loadPrefs } from './core/libraryOps';
import { prefBool } from './core/appPrefs';
import { startViewportFlags } from './platform/viewport';
import './index.css';
import './mobile.css';

const TAURI_HTTP_ABORT_RACE = /The resource id \d+ is invalid/;

window.addEventListener('unhandledrejection', (event) => {
  const reason = event.reason;
  const message = reason instanceof Error ? reason.message : String(reason);
  if (TAURI_HTTP_ABORT_RACE.test(message)) event.preventDefault();
});

async function bootstrap() {
  startViewportFlags();

  if (import.meta.env.PROD) {
    try {
      const prefs = await loadPrefs();
      if (prefBool(prefs, 'diagnosticMode', false)) {
        await initDiagnosticsSentry();
      }
    } catch {}
  }

  try {
    ReactDOM.createRoot(document.getElementById('root')!).render(
      <ErrorBoundary>
        <App />
      </ErrorBoundary>,
    );
  } catch (err) {
    const root = document.getElementById('root')!;
    root.style.cssText = 'background:#0a0a0a;color:#ff4444;padding:1.5rem;font-family:monospace;font-size:0.8125rem;white-space:pre-wrap;overflow:auto';
    root.textContent = 'React mount error:\n' + (err instanceof Error ? err.stack : String(err));
  }
}

void bootstrap();
