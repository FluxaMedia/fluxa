import { useCallback, useMemo } from 'react';
import type { Meta, Video } from '../core/types';

export function useSeasonWatched({
  meta,
  displayMeta,
  episodes,
  seasonNumbers,
  watchedMap,
  onDispatch,
}: {
  meta: Meta;
  displayMeta: Meta;
  episodes: Video[];
  seasonNumbers: number[];
  watchedMap: Record<string, boolean>;
  onDispatch: (actionJson: string) => void;
}) {
  const seasonWatchedMap = useMemo(() => {
    const map: Record<number, boolean> = {};
    for (const season of seasonNumbers) {
      const sEps = episodes.filter((ep) => (ep.season ?? 1) === season);
      if (sEps.length > 0) map[season] = sEps.every((ep) => watchedMap[ep.id] === true);
    }
    return map;
  }, [seasonNumbers, episodes, watchedMap]);

  const dispatchMarkSeason = useCallback((seasons: number[], watched: boolean) => {
    const now = Date.now();
    const allEps = episodes.filter((ep) => {
      if (!seasons.includes(ep.season ?? 1)) return false;
      if (watched && ep.released && new Date(ep.released).getTime() > now) return false;
      return true;
    });
    if (allEps.length === 0) return;
    onDispatch(JSON.stringify({
      type: 'markWatchedRequested',
      seriesId: meta.id,
      videoIds: allEps.map((ep) => ep.id),
      watched,
      meta: { id: meta.id, name: displayMeta.name, type: meta.type },
      episodes: allEps.map((ep) => ({ id: ep.id, name: ep.name ?? ep.title, season: ep.season, number: ep.episode ?? ep.number, thumbnail: ep.thumbnail })),
    }));
  }, [episodes, meta.id, displayMeta.name, meta.type, onDispatch]);

  const toggleEpisodeWatched = useCallback((ep: Video, currentlyWatched: boolean) => {
    onDispatch(JSON.stringify({
      type: 'markWatchedRequested',
      seriesId: meta.id,
      videoIds: [ep.id],
      watched: !currentlyWatched,
      meta: { id: meta.id, name: displayMeta.name, type: meta.type },
      episodes: [{ id: ep.id, name: ep.name ?? ep.title, season: ep.season, number: ep.episode ?? ep.number, thumbnail: ep.thumbnail }],
    }));
  }, [meta.id, displayMeta.name, meta.type, onDispatch]);

  return { seasonWatchedMap, dispatchMarkSeason, toggleEpisodeWatched };
}
