import { platformInvoke } from '../platform/invoke';
import { withSentrySpan } from './sentryRuntime';
import type { DispatchResult, EffectResult } from './types';
import type { CoreMethod } from './coreMethods';

let engineHandle: number | null = null;

export async function initEngine(initialJson: string = '{}'): Promise<void> {
  if (engineHandle !== null) return;
  engineHandle = await platformInvoke<number>('engine_init', { initialJson });
}

export async function dispatchAction(actionJson: string): Promise<DispatchResult | null> {
  let label = 'dispatch';
  try {
    label = `dispatch:${(JSON.parse(actionJson) as { type?: string }).type ?? '?'}`;
  } catch {}
  return withSentrySpan(label, 'fluxa.ipc', async () => {
    const raw = await platformInvoke<string | null>('engine_dispatch', { actionJson });
    if (!raw) return null;
    return JSON.parse(raw) as DispatchResult;
  });
}

export async function completeEffect(result: EffectResult): Promise<DispatchResult | null> {
  return withSentrySpan(`completeEffect:${result.effectId}`, 'fluxa.ipc', async () => {
    const raw = await platformInvoke<string | null>('engine_complete_effect', {
      resultJson: JSON.stringify(result),
    });
    if (!raw) return null;
    return JSON.parse(raw) as DispatchResult;
  });
}

export async function getSnapshot(): Promise<unknown | null> {
  const raw = await platformInvoke<string | null>('engine_snapshot');
  if (!raw) return null;
  return JSON.parse(raw);
}

export async function httpFetchText(url: string): Promise<{ statusCode: number; body: string }> {
  const response = await platformInvoke<{ status_code: number; body: string }>('http_fetch_text', { url });
  return { statusCode: response.status_code, body: response.body };
}

export async function httpExecuteText(
  url: string,
  method: string,
  headers: Record<string, string>,
  body?: unknown,
): Promise<{ statusCode: number; body: string }> {
  const response = await platformInvoke<{ status_code: number; body: string }>('http_execute_text', { url, method, headers, body });
  return { statusCode: response.status_code, body: response.body };
}

export async function registerTrailerProxyUrl(url: string): Promise<string> {
  return platformInvoke<string>('register_trailer_proxy_url', { url });
}

export async function runPluginScraper(
  code: string,
  repositoryUrl: string,
  scraperId: string,
  scraperSettingsJson: string,
  tmdbId: string,
  mediaType: string,
  season: number | null,
  episode: number | null,
): Promise<string> {
  if (typeof window !== 'undefined') {
    const { runWebPluginScraper } = await import('../platform/web/pluginRuntime');
    return runWebPluginScraper(code, scraperId, scraperSettingsJson, tmdbId, mediaType, season, episode);
  }
  return platformInvoke<string>('run_plugin_scraper', {
    code,
    repositoryUrl,
    scraperId,
    scraperSettingsJson,
    tmdbId,
    mediaType,
    season,
    episode,
  });
}

export interface YoutubeTrailerSubtitleTrack {
  languageTag: string;
  label: string;
  url: string;
  mimeType: string;
  isAuto: boolean;
}
