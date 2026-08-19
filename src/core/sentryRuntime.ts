type SentryModule = typeof import('@sentry/react');

let sentryModule: SentryModule | null = null;
let initPromise: Promise<void> | null = null;

const TAURI_HTTP_ABORT_RACE = /The resource id \d+ is invalid/;

export function getSentryModule(): SentryModule | null {
  return sentryModule;
}

export function initDiagnosticsSentry(): Promise<void> {
  if (sentryModule) return Promise.resolve();
  if (initPromise) return initPromise;
  initPromise = import('@sentry/react').then((Sentry) => {
    Sentry.init({
      dsn: 'https://9ca93bac9e63dfbd8cc3d84078677fb6@o4511704565678080.ingest.de.sentry.io/4511706868023376',
      integrations: [Sentry.browserTracingIntegration(), Sentry.browserProfilingIntegration()],
      tracesSampleRate: 0.1,
      profilesSampleRate: 0.1,
      beforeSend(event, hint) {
        const reason = hint.originalException;
        const message = reason instanceof Error ? reason.message : String(reason ?? event.message ?? '');
        if (TAURI_HTTP_ABORT_RACE.test(message)) return null;
        return event;
      },
    });
    sentryModule = Sentry;
  });
  return initPromise;
}

export async function withSentrySpan<T>(name: string, op: string, callback: () => Promise<T>): Promise<T> {
  const sentry = getSentryModule();
  if (!sentry) return callback();
  return sentry.startSpan({ name, op }, callback);
}
