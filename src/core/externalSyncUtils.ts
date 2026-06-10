import { loadLibrary, saveLibrary } from './libraryOps';
import { fetchMetaDetail } from './detailEffects';

export async function enrichWithAddonMeta(items: Record<string, unknown>[]): Promise<Record<string, unknown>[]> {
  if (items.length === 0) return items;
  const CONCURRENCY = 4;
  const results: Record<string, unknown>[] = new Array(items.length);
  let cursor = 0;

  async function worker() {
    while (cursor < items.length) {
      const i = cursor++;
      const item = items[i];
      if (item.poster || item.background) { results[i] = item; continue; }
      const id = typeof item.id === 'string' ? item.id : '';
      const contentType = typeof item.type === 'string' ? item.type : 'movie';
      if (!id) { results[i] = item; continue; }
      try {
        const meta = await fetchMetaDetail({ id, contentType }) as Record<string, unknown> | null;
        if (!meta) { results[i] = item; continue; }
        const poster = typeof meta.poster === 'string' ? meta.poster : undefined;
        const background = typeof meta.background === 'string' ? meta.background : undefined;
        const logo = typeof meta.logo === 'string' ? meta.logo : undefined;
        let lastEpisodeThumbnail = typeof item.lastEpisodeThumbnail === 'string' ? item.lastEpisodeThumbnail : undefined;
        if (!lastEpisodeThumbnail && contentType === 'series' && Array.isArray(meta.videos)) {
          const season = typeof item.lastEpisodeSeason === 'number' ? item.lastEpisodeSeason : undefined;
          const epNum = typeof item.lastEpisodeNumber === 'number' ? item.lastEpisodeNumber : undefined;
          if (season != null && epNum != null) {
            const ep = (meta.videos as Record<string, unknown>[]).find(
              (v) => Number(v.season) === season && (Number(v.episode ?? v.number) === epNum),
            );
            const thumb = ep?.thumbnail;
            if (typeof thumb === 'string') lastEpisodeThumbnail = thumb;
          }
        }
        results[i] = {
          ...item,
          ...(poster ? { poster } : {}),
          ...(background ? { background } : {}),
          ...(logo ? { logo } : {}),
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
  const provider = typeof payload.provider === 'string' ? payload.provider : undefined;
  const items = Array.isArray(payload.items)
    ? payload.items.filter((item): item is Record<string, unknown> => {
        if (!item || typeof item !== 'object') return false;
        const id = typeof item.id === 'string' ? item.id.trim() : '';
        const timeOffset = Number(item.timeOffset ?? 0);
        const duration = Number(item.duration ?? 0);
        return !!id && timeOffset > 0 && duration > 0;
      })
    : [];
  if (provider) {
    const existing = (lib.externalContinueWatching as Record<string, unknown>[] | undefined) ?? [];
    const otherProviders = existing.filter((item) => item.reason !== provider);
    const combined = [...otherProviders, ...items];
    const byId = new Map<string, Record<string, unknown>>();
    for (const item of combined) {
      const id = typeof item.id === 'string' ? item.id : '';
      if (!id) continue;
      const prev = byId.get(id);
      if (!prev) { byId.set(id, item); continue; }
      const prevTime = new Date(String(prev.savedAt ?? 0)).getTime();
      const itemTime = new Date(String(item.savedAt ?? 0)).getTime();
      if (itemTime > prevTime) byId.set(id, item);
    }
    lib.externalContinueWatching = [...byId.values()];
  } else {
    lib.externalContinueWatching = items;
  }
  await saveLibrary(lib);
  return { count: items.length };
}
