import { platformInvoke } from '../platform/invoke';
import { withSentrySpan } from './sentryRuntime';

export async function coreInvoke<T>(method: string, argsJson: string): Promise<T | null> {
  return withSentrySpan(`coreInvoke:${method}`, 'fluxa.core', async () => {
    const raw = await platformInvoke<string>('core_invoke', { method, argsJson });
    const envelope = JSON.parse(raw) as {
      ok: boolean;
      value?: T;
      error?: { kind: string; message: string };
    };
    if (!envelope.ok) {
      throw new Error(`[core] ${method}: ${envelope.error?.message ?? 'unknown error'}`);
    }
    return envelope.value ?? null;
  });
}
