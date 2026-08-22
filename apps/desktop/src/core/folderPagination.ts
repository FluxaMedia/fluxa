import { buildResourceUrl } from './addonManifest';
import { coreInvoke, httpFetchText } from './engine';
import { loadNuvioCollectionSource } from './collectionSources';
import { fetchBuiltinCatalog } from './tmdbAddon';
import { loadPrefs } from './libraryOps';
import { getLanguage } from '../i18n';
import type { Meta, NuvioRemoteCollectionSource } from './types';

export type FolderSourceBatch = { type: string; items: Meta[] };

export type AddonFolderSource = { transportUrl: string; catalogId: string; type: string; genre?: string };
export type FolderSource = AddonFolderSource | NuvioRemoteCollectionSource;

export interface FolderSourceState {
  skip: number;
  exhausted: boolean;
  duplicateStreak: number;
  items: Meta[];
}

export function initFolderSourceState(): FolderSourceState {
  return { skip: 0, exhausted: false, duplicateStreak: 0, items: [] };
}

export async function loadFolderSourcePage(source: FolderSource, skip: number): Promise<FolderSourceBatch> {
  const plan = await coreInvoke<{
    kind: 'remote' | 'builtinTmdb' | 'addon';
    type: string;
    page?: number;
    transportUrl?: string;
    catalogId?: string;
    extra?: Record<string, unknown>;
  }>('folderSourcePagePlan', JSON.stringify({ source, skip }));
  if (!plan) return { type: 'movie', items: [] };
  if (plan.kind === 'remote') {
    return { type: plan.type, items: await loadNuvioCollectionSource(source as NuvioRemoteCollectionSource, plan.page) };
  }

  if (plan.kind === 'builtinTmdb') {
    const prefs = await loadPrefs();
    const { metas } = await fetchBuiltinCatalog(plan.type, plan.extra ?? {}, String(prefs.tmdbApiKey ?? ''), getLanguage());
    return { type: plan.type, items: metas as Meta[] };
  }

  const extraJson = Object.keys(plan.extra ?? {}).length ? JSON.stringify(plan.extra) : undefined;
  const url = await buildResourceUrl(plan.transportUrl!, 'catalog', plan.type, plan.catalogId!, extraJson);
  try {
    const res = await httpFetchText(url);
    if (res.statusCode === 200) {
      const data = JSON.parse(res.body) as { metas?: unknown };
      return { type: plan.type, items: Array.isArray(data?.metas) ? (data.metas as Meta[]) : [] };
    }
  } catch {
    /* skip failed source */
  }
  return { type: plan.type, items: [] as Meta[] };
}
