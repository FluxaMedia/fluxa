import { coreInvoke, dispatchAction, storageRead } from './engine';
import { pumpEffects } from './effectRunner';
import type { AppState, UserProfile } from './types';

const LEGACY_REPOSITORY_URLS_KEY = 'plugin_repository_urls';
const LEGACY_SCRAPER_ENABLED_KEY = 'plugin_scraper_enabled';

export async function pluginsOwnerId(): Promise<string> {
  const profiles = (await storageRead<UserProfile[]>('profiles')) ?? [];
  const activeId = (await storageRead<string>('active_profile_id'))?.trim() ?? '';
  const ownerId = await coreInvoke<string>('effectivePluginsOwnerId', JSON.stringify({ profiles, activeProfileId: activeId }));
  return (ownerId || activeId || 'guest').replace(/[^a-zA-Z0-9_-]/g, '_');
}

export async function pluginRepositoryUrlsKey(): Promise<string> {
  return `plugin_repository_urls_${await pluginsOwnerId()}`;
}

export async function pluginScraperEnabledKey(): Promise<string> {
  return `plugin_scraper_enabled_${await pluginsOwnerId()}`;
}

async function readPluginHydrationSource(): Promise<{ repositoryUrls: string[]; scraperOverrides: Record<string, boolean> }> {
  const [scopedRepositoryUrls, legacyRepositoryUrls, scopedScraperOverrides, legacyScraperOverrides] = await Promise.all([
    storageRead<string[]>(await pluginRepositoryUrlsKey()),
    storageRead<string[]>(LEGACY_REPOSITORY_URLS_KEY),
    storageRead<Record<string, boolean>>(await pluginScraperEnabledKey()),
    storageRead<Record<string, boolean>>(LEGACY_SCRAPER_ENABLED_KEY),
  ]);
  const result = await coreInvoke<{ repositoryUrls: string[]; scraperOverrides: Record<string, boolean> }>(
    'pluginStorageFallback',
    JSON.stringify({
      scopedRepositoryUrls: scopedRepositoryUrls ?? null,
      legacyRepositoryUrls: legacyRepositoryUrls ?? null,
      scopedScraperOverrides: scopedScraperOverrides ?? null,
      legacyScraperOverrides: legacyScraperOverrides ?? null,
    }),
  );
  return result ?? { repositoryUrls: [], scraperOverrides: {} };
}

export async function hydratePluginsFromStorage(
  updateState: (s: Partial<AppState>) => void,
): Promise<void> {
  const { repositoryUrls, scraperOverrides } = await readPluginHydrationSource();
  for (const manifestUrl of repositoryUrls) {
    if (!manifestUrl.trim()) continue;
    const result = await dispatchAction(JSON.stringify({ type: 'pluginRepositoryAddRequested', manifestUrl }));
    if (!result) continue;
    updateState(result.state);
    if (result.effects.length > 0) await pumpEffects(result.effects, updateState);
  }
  for (const [scraperId, enabled] of Object.entries(scraperOverrides)) {
    const result = await dispatchAction(JSON.stringify({ type: 'pluginScraperToggled', scraperId, enabled }));
    if (result) updateState(result.state);
  }
}

export async function clearEnginePlugins(
  currentRepositories: Array<{ manifestUrl: string }>,
  updateState: (s: Partial<AppState>) => void,
): Promise<void> {
  for (const repository of currentRepositories) {
    const result = await dispatchAction(JSON.stringify({ type: 'pluginRepositoryRemoveRequested', manifestUrl: repository.manifestUrl }));
    if (result) updateState(result.state);
  }
}
