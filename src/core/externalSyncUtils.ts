import { loadLibrary, saveLibrary, loadPrefs, persistContinueWatchingMerge } from './libraryOps';
import { fetchMetaDetail, fetchTmdbPosterFallback } from './detailEffects';
import { coreReplaceExternalContinueWatching } from './engine';
import { prefString } from './appPrefs';

export async function enrichWithAddonMeta(items: Record<string, unknown>[]): Promise<Record<string, unknown>[]> {
  if (items.length === 0) return items;
  const CONCURRENCY = 4;
  const results: Record<string, unknown>[] = new Array(items.length);
  let cursor = 0;
  const prefs = await loadPrefs();
  const tmdbApiKey = prefString(prefs, 'tmdbApiKey');
  const language = prefString(prefs, 'language', 'en');

  async function worker() {
    while (cursor < items.length) {
      const i = cursor++;
      const item = items[i];
      const id = typeof item.id === 'string' ? item.id : '';
      const contentType = typeof item.type === 'string' ? item.type : 'movie';
      const needsEpisodeMeta = contentType === 'series' &&
        typeof item.lastEpisodeSeason === 'number' &&
        typeof item.lastEpisodeNumber === 'number' &&
        (!item.lastEpisodeName || !item.lastEpisodeThumbnail);
      if ((item.poster || item.background) && !needsEpisodeMeta) { results[i] = item; continue; }
      if (!id) { results[i] = item; continue; }
      try {
        const meta = await fetchMetaDetail({ id, contentType }) as Record<string, unknown> | null;
        let poster = meta && typeof meta.poster === 'string' ? meta.poster : undefined;
        let background = meta && typeof meta.background === 'string' ? meta.background : undefined;
        const logo = meta && typeof meta.logo === 'string' ? meta.logo : undefined;
        if (!poster && !background) {
          const tmdbFallback = await fetchTmdbPosterFallback({ contentType, id, language, apiKey: tmdbApiKey });
          poster = tmdbFallback?.poster;
          background = tmdbFallback?.background;
        }
        let lastEpisodeThumbnail = typeof item.lastEpisodeThumbnail === 'string' ? item.lastEpisodeThumbnail : undefined;
        let lastEpisodeName = typeof item.lastEpisodeName === 'string' ? item.lastEpisodeName : undefined;
        if (meta && contentType === 'series' && Array.isArray(meta.videos)) {
          const season = typeof item.lastEpisodeSeason === 'number' ? item.lastEpisodeSeason : undefined;
          const epNum = typeof item.lastEpisodeNumber === 'number' ? item.lastEpisodeNumber : undefined;
          if (season != null && epNum != null) {
            const ep = (meta.videos as Record<string, unknown>[]).find(
              (v) => Number(v.season) === season && (Number(v.episode ?? v.number) === epNum),
            );
            const thumb = ep?.thumbnail;
            if (!lastEpisodeThumbnail && typeof thumb === 'string') lastEpisodeThumbnail = thumb;
            const name = ep?.name ?? ep?.title;
            if (!lastEpisodeName && typeof name === 'string' && name.trim()) lastEpisodeName = name;
          }
        }
        results[i] = {
          ...item,
          ...(poster ? { poster } : {}),
          ...(background ? { background } : {}),
          ...(logo ? { logo } : {}),
          ...(lastEpisodeName ? { lastEpisodeName } : {}),
          ...(lastEpisodeThumbnail ? { lastEpisodeThumbnail } : {}),
        };
      } catch {
        results[i] = item;
      }
    }
  }

  await Promise.all(Array.from({ length: Math.min(CONCURRENCY, items.length) }, worker));
  return results;
}

export async function replaceExternalContinueWatching(payload: Record<string, unknown>): Promise<unknown> {
  const lib = await loadLibrary();
  const prefs = await loadPrefs();
  const provider = typeof payload.provider === 'string' ? payload.provider : null;
  const dismissed = (lib.dismissedContinueWatching as Record<string, unknown> | undefined) ?? {};
  const items = (Array.isArray(payload.items) ? payload.items : []).filter((item) => {
    const id = item && typeof item === 'object' ? (item as Record<string, unknown>).id : undefined;
    return typeof id !== 'string' || !dismissed[id];
  });
  const existing = (lib.externalContinueWatching as Record<string, unknown>[]) ?? [];
  const merged = await coreReplaceExternalContinueWatching(
    JSON.stringify(existing),
    provider,
    JSON.stringify(items),
    prefString(prefs, 'syncCwSourceOfTruth'),
    prefString(prefs, 'syncCwRanking'),
    Number(prefs.continueWatchingDays) || 0,
  );
  lib.externalContinueWatching = merged;
  await persistContinueWatchingMerge(existing, merged as Record<string, unknown>[]);
  await saveLibrary(lib);
  return { count: merged.length };
}
