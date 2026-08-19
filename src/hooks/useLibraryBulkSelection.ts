import { useEffect, useMemo, useState } from 'react';
import type { LibraryItem } from '../core/types';

export function useLibraryBulkSelection({
  items,
  tab,
  completed,
  dropped,
  onDispatch,
}: {
  items: LibraryItem[];
  tab: string;
  completed: LibraryItem[];
  dropped: LibraryItem[];
  onDispatch: (actionJson: string) => void;
}) {
  const [bulkMode, setBulkMode] = useState(false);
  const [selectedIds, setSelectedIds] = useState<Set<string>>(() => new Set());
  const [confirmBulkRemove, setConfirmBulkRemove] = useState(false);

  useEffect(() => {
    setSelectedIds((current) => {
      if (current.size === 0) return current;
      const visibleIds = new Set(items.map((item) => item.id));
      const next = new Set([...current].filter((id) => visibleIds.has(id)));
      return next.size === current.size ? current : next;
    });
  }, [items]);

  const selectedItems = useMemo(() => items.filter((item) => selectedIds.has(item.id)), [items, selectedIds]);
  const canRemoveFromCurrentList = tab === 'watchlist' || tab === 'completed' || tab === 'dropped';

  const toggleSelected = (id: string) => {
    setSelectedIds((current) => {
      const next = new Set(current);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });
  };

  const clearSelection = () => setSelectedIds(new Set());

  const runForSelected = async (buildAction: (item: LibraryItem) => Record<string, unknown> | null) => {
    const batch = [...selectedItems];
    clearSelection();
    for (const item of batch) {
      const action = buildAction(item);
      if (action) await Promise.resolve(onDispatch(JSON.stringify(action)));
    }
  };

  const markSelectedWatched = (watched: boolean) => {
    void runForSelected((item) => ({
      type: 'markWatchedRequested',
      seriesId: item.id,
      videoIds: [item.lastVideoId ?? item.id],
      watched,
      meta: item,
      episodes: item.lastVideoId
        ? [
            {
              id: item.lastVideoId,
              name: item.lastEpisodeName,
              season: item.lastEpisodeSeason,
              number: item.lastEpisodeNumber,
              thumbnail: item.lastEpisodeThumbnail,
            },
          ]
        : [],
    }));
  };

  const moveSelectedToStatus = (list: 'completed' | 'dropped') => {
    const existingIds = new Set((list === 'completed' ? completed : dropped).map((item) => item.id));
    void runForSelected((item) =>
      existingIds.has(item.id)
        ? null
        : {
            type: 'toggleLibraryStatusRequested',
            list,
            item,
          },
    );
  };

  const removeSelectedFromCurrentList = () => {
    if (!canRemoveFromCurrentList) return;
    void runForSelected((item) =>
      tab === 'watchlist' ? { type: 'toggleWatchlistRequested', item } : { type: 'toggleLibraryStatusRequested', list: tab, item },
    );
  };

  return {
    bulkMode,
    setBulkMode,
    selectedIds,
    setSelectedIds,
    confirmBulkRemove,
    setConfirmBulkRemove,
    selectedItems,
    canRemoveFromCurrentList,
    toggleSelected,
    clearSelection,
    markSelectedWatched,
    moveSelectedToStatus,
    removeSelectedFromCurrentList,
  };
}
