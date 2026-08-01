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
    if (!response.ok) return null;
    const code = await response.text();
    if (!code) return null;
    codeCache.set(cacheKey, code);
    return code;
  } catch {
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
  if (!tmdbId) return [];
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
  if (!plan?.scrapers.length) return [];

  const results = await Promise.allSettled(
    plan.scrapers.map(async (scraper) => {
      const code = await loadScraperCode(scraper, signal);
      if (!code) return [];
      const raw = await runPluginScraper(
        code,
        scraper.id,
        JSON.stringify(scraper.settings ?? {}),
        plan.contentId,
        plan.mediaType,
        plan.season ?? null,
        plan.episode ?? null,
      );
      const streams = (await coreInvoke<Array<Record<string, unknown>>>('pluginStreamResultsToStreams', raw)) ?? [];
      return streams.map((stream) => ({ ...stream, addonName: scraper.name }));
    }),
  );

  return results.flatMap((result) => (result.status === 'fulfilled' ? result.value : []));
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
