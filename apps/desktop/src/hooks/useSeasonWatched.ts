import { useCallback, useEffect, useState } from 'react';
import { coreInvoke } from '../core/engine';
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
  const [seasonWatchedMap, setSeasonWatchedMap] = useState<Record<number, boolean>>({});
  useEffect(() => {
    let active = true;
    void coreInvoke<Record<number, boolean>>('seasonWatchedPlan', JSON.stringify({ episodes, seasonNumbers, watchedMap })).then((plan) => {
      if (active) setSeasonWatchedMap(plan ?? {});
    });
    return () => {
      active = false;
    };
  }, [seasonNumbers, episodes, watchedMap]);

  const dispatchMarkSeason = useCallback(
    (seasons: number[], watched: boolean) => {
      void coreInvoke<Record<string, unknown>>(
        'markSeasonsActionPlan',
        JSON.stringify({
          episodes,
          seasons,
          watched,
          meta: { id: meta.id, name: displayMeta.name, type: meta.type },
          nowMs: Date.now(),
        }),
      ).then((action) => {
        if (action) onDispatch(JSON.stringify(action));
      });
    },
    [episodes, meta.id, displayMeta.name, meta.type, onDispatch],
  );

  const toggleEpisodeWatched = useCallback(
    (ep: Video, currentlyWatched: boolean) => {
      void coreInvoke<Record<string, unknown>>(
        'markSeasonsActionPlan',
        JSON.stringify({
          episodes: [ep],
          seasons: [ep.season ?? 1],
          watched: !currentlyWatched,
          meta: { id: meta.id, name: displayMeta.name, type: meta.type },
          nowMs: Date.now(),
        }),
      ).then((action) => {
        if (action) onDispatch(JSON.stringify(action));
      });
    },
    [meta.id, displayMeta.name, meta.type, onDispatch],
  );

  return { seasonWatchedMap, dispatchMarkSeason, toggleEpisodeWatched };
}
