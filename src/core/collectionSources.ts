import { platformInvoke } from '../platform/invoke';
import { platformFetch } from './httpClient';
import type { Meta, NuvioRemoteCollectionSource } from './types';
import { loadPrefs } from './libraryOps';
import { prefString } from './appPrefs';
import { coreInvoke } from './engine';

export async function loadNuvioCollectionSource(source: NuvioRemoteCollectionSource, page = 1): Promise<Meta[]> {
  const prefs = await loadPrefs();
  const clientId =
    source.provider === 'trakt' ? await platformInvoke<string>('get_oauth_client_id', { service: 'trakt' }).catch(() => '') : '';
  const plan = await coreInvoke<{
    url: string;
    params: Record<string, string | number>;
    headers: Record<string, string>;
  }>(
    'remoteCollectionRequestPlan',
    JSON.stringify({
      source,
      page,
      clientId,
      apiKey: prefString(prefs, 'tmdbApiKey'),
      language: prefString(prefs, 'language', 'en'),
    }),
  );
  if (!plan) return [];
  try {
    const url = new URL(plan.url);
    for (const [key, value] of Object.entries(plan.params)) url.searchParams.set(key, String(value));
    const response = await platformFetch(url.toString(), { headers: plan.headers });
    if (!response.ok) return [];
    return (await coreInvoke<Meta[]>('remoteCollectionResponsePlan', JSON.stringify({ plan, data: await response.json() }))) ?? [];
  } catch {
    return [];
  }
}
