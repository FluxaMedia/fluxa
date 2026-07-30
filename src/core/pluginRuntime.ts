import { coreInvoke, getSnapshot, httpFetchText, runPluginScraper } from './engine';

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

async function loadScraperCode(scraper: PluginScraperState): Promise<string | null> {
  const cacheKey = `${scraper.repositoryUrl}::${scraper.filename}`;
  const cached = codeCache.get(cacheKey);
  if (cached) return cached;
  try {
    const url = new URL(scraper.filename, repoBaseUrl(scraper.repositoryUrl)).toString();
    const response = await httpFetchText(url);
    if (response.statusCode < 200 || response.statusCode >= 300 || !response.body) return null;
    codeCache.set(cacheKey, response.body);
    return response.body;
  } catch {
    return null;
  }
}

export async function fetchPluginStreams(
  contentType: string,
  tmdbId: string | undefined,
  season: number | undefined,
  episode: number | undefined,
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
      const code = await loadScraperCode(scraper);
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

export async function fetchPluginManifestEffect(payload: Record<string, unknown>): Promise<unknown> {
  const manifestUrl = payload.manifestUrl as string;
  const response = await httpFetchText(manifestUrl);
  if (response.statusCode < 200 || response.statusCode >= 300 || !response.body) {
    throw new Error(`failed to fetch plugin manifest: HTTP ${response.statusCode}`);
  }
  const manifest = await coreInvoke<Record<string, unknown>>('pluginManifestParse', response.body);
  if (!manifest) throw new Error('invalid plugin manifest');
  return { manifestUrl, manifest };
}
