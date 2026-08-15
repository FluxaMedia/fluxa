import { platformInvoke as invoke } from '../platform/invoke';
import { coreInvoke, getSnapshot, runPluginScraper } from './engine';
import { platformFetch } from './httpClient';

interface PluginScraperState {
  id: string;
  name: string;
  repositoryUrl: string;
  filename: string;
  enabled: boolean;
  supportedTypes?: string[];
  settings?: Record<string, unknown>;
}

const codeCache = new Map<string, string>();

function pluginDebug(message: string) {
  void invoke('debug_log', { msg: `plugin-runtime: ${message}` }).catch(() => undefined);
}

function repoBaseUrl(manifestUrl: string): string {
  const idx = manifestUrl.lastIndexOf('/');
  return idx >= 0 ? manifestUrl.slice(0, idx + 1) : manifestUrl;
}

async function loadScraperCode(scraper: PluginScraperState, signal?: AbortSignal): Promise<string | null> {
  const cacheKey = `${scraper.repositoryUrl}::${scraper.filename}`;
  const cached = codeCache.get(cacheKey);
  if (cached) return cached;
  try {
    const url = new URL(scraper.filename, repoBaseUrl(scraper.repositoryUrl)).toString();
    const response = await platformFetch(url, { signal });
    if (!response.ok) {
      pluginDebug(`code fetch failed scraper=${scraper.id} status=${response.status}`);
      return null;
    }
    const code = await response.text();
    if (!code) {
      pluginDebug(`code fetch returned empty scraper=${scraper.id}`);
      return null;
    }
    codeCache.set(cacheKey, code);
    return code;
  } catch (error) {
    pluginDebug(`code fetch failed scraper=${scraper.id} error=${error instanceof Error ? error.message : String(error)}`);
    return null;
  }
}

export async function fetchPluginStreams(
  contentType: string,
  tmdbId: string | undefined,
  season: number | undefined,
  episode: number | undefined,
  signal?: AbortSignal,
): Promise<Array<Record<string, unknown>>> {
  if (!tmdbId) {
    pluginDebug(`skipped without a plugin content id type=${contentType}`);
    return [];
  }
  const snapshot = (await getSnapshot()) as { plugins?: { scrapers?: PluginScraperState[] } } | null;
  const plan = await coreInvoke<{
    contentId: string;
    mediaType: string;
    season?: number;
    episode?: number;
    scrapers: PluginScraperState[];
  }>('pluginExecutionPlan', JSON.stringify({
    contentId: tmdbId,
    mediaType: contentType,
    season,
    episode,
    scrapers: snapshot?.plugins?.scrapers ?? [],
  }));
  if (!plan?.scrapers.length) {
    pluginDebug(`no compatible scraper type=${contentType} content=${tmdbId} installed=${snapshot?.plugins?.scrapers?.length ?? 0}`);
    return [];
  }

  pluginDebug(`running scrapers=${plan.scrapers.length} type=${plan.mediaType} content=${plan.contentId}`);

  const results = await Promise.allSettled(
    plan.scrapers.map(async (scraper) => {
      const code = await loadScraperCode(scraper, signal);
      if (!code) return [];
      try {
        const raw = await runPluginScraper(
          code,
          scraper.repositoryUrl,
          scraper.id,
          JSON.stringify(scraper.settings ?? {}),
          plan.contentId,
          plan.mediaType,
          plan.season ?? null,
          plan.episode ?? null,
        );
        const streams = (await coreInvoke<Array<Record<string, unknown>>>('pluginStreamResultsToStreams', raw)) ?? [];
        pluginDebug(`completed scraper=${scraper.id} streams=${streams.length}`);
        return streams.map((stream) => ({ ...stream, addonName: scraper.name }));
      } catch (error) {
        pluginDebug(`scraper failed scraper=${scraper.id} error=${error instanceof Error ? error.message : String(error)}`);
        return [];
      }
    }),
  );

  return results.flatMap((result) => {
    if (result.status === 'fulfilled') return result.value;
    pluginDebug(`scraper task rejected error=${result.reason instanceof Error ? result.reason.message : String(result.reason)}`);
    return [];
  });
}

export async function fetchPluginManifestEffect(payload: Record<string, unknown>, signal?: AbortSignal): Promise<unknown> {
  const manifestUrl = payload.manifestUrl as string;
  const response = await platformFetch(manifestUrl, { signal });
  if (!response.ok) {
    throw new Error(`failed to fetch plugin manifest: HTTP ${response.status}`);
  }
  const manifest = await coreInvoke<Record<string, unknown>>('pluginManifestParse', await response.text());
  if (!manifest) throw new Error('invalid plugin manifest');
  return { manifestUrl, manifest };
}
