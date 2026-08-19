import type { AppState, HomeState, HomeCategory, Meta, LibraryItem } from './types';

// The engine now only sends domains that actually changed since the previous dispatch
// (see StatePatch::diff on the Rust side) — anything absent from `next` keeps prev's
// exact reference via this spread, which is what lets screens gated on
// `prev.state.X === next.state.X` (Calendar, Discover, Library) skip re-rendering.
export function mergeAppState(prev: AppState, next: Partial<AppState>): AppState {
  if (prev === next) return prev;
  const base = { ...prev, ...next };
  if (next.home !== undefined) base.home = mergeHomeState(prev.home, next.home);
  return base;
}

function mergeHomeState(prev: HomeState, next: HomeState): HomeState {
  if (prev === next) return prev;

  const categories = mergeCategories(prev.categories, next.categories);
  const continueWatching = mergeContinueWatching(prev.continueWatching, next.continueWatching);
  const billboard = mergeBillboard(prev.billboard, next.billboard);

  if (
    categories === prev.categories &&
    continueWatching === prev.continueWatching &&
    billboard === prev.billboard &&
    prev.isLoading === next.isLoading &&
    prev.isDirectLoading === next.isDirectLoading &&
    prev.error === next.error &&
    prev.paging === next.paging
  ) {
    return prev;
  }

  return { ...next, categories, continueWatching, billboard };
}

function mergeCategories(prev: HomeCategory[] | undefined, next: HomeCategory[] | undefined): HomeCategory[] | undefined {
  if (next === undefined) return undefined;
  if (!prev) return next;

  const prevById = new Map<string, HomeCategory>(prev.map((c) => [c.id, c]));

  let allSame = prev.length === next.length;
  const merged = next.map((nextCat) => {
    const prevCat = prevById.get(nextCat.id);
    if (!prevCat) {
      allSame = false;
      return nextCat;
    }

    const items = mergeMetaArray(prevCat.items, nextCat.items);

    if (
      items === prevCat.items &&
      prevCat.name === nextCat.name &&
      prevCat.type === nextCat.type &&
      prevCat.addonName === nextCat.addonName &&
      prevCat.hasMore === nextCat.hasMore
    ) {
      return prevCat;
    }

    allSame = false;
    return { ...nextCat, items };
  });

  return allSame ? prev : merged;
}

function mergeMetaArray(prev: Meta[], next: Meta[]): Meta[] {
  if (prev === next) return prev;
  if (prev.length === 0) return next;

  const prevById = new Map<string, Meta>(prev.map((m) => [m.id, m]));

  let allSame = prev.length === next.length;
  const merged = next.map((nextMeta) => {
    const prevMeta = prevById.get(nextMeta.id);
    if (!prevMeta || !metaRenderEqual(prevMeta, nextMeta)) {
      allSame = false;
      return nextMeta;
    }
    return prevMeta;
  });

  return allSame ? prev : merged;
}

function mergeBillboard(prev: Meta | null | undefined, next: Meta | null | undefined): Meta | null | undefined {
  if (!prev || !next) return next;
  if (metaRenderEqual(prev, next)) return prev;
  return next;
}

function mergeContinueWatching(prev: LibraryItem[] | undefined, next: LibraryItem[] | undefined): LibraryItem[] | undefined {
  if (next === undefined) return undefined;
  if (!prev || prev.length === 0) return next;

  const prevById = new Map<string, LibraryItem>(prev.map((i) => [i.id, i]));

  let allSame = prev.length === next.length;
  const merged = next.map((nextItem) => {
    const prevItem = prevById.get(nextItem.id);
    if (!prevItem) {
      allSame = false;
      return nextItem;
    }
    const backfilled = backfillLibraryItem(prevItem, nextItem);
    if (libraryItemRenderEqual(prevItem, backfilled)) return prevItem;
    allSame = false;
    return backfilled;
  });

  return allSame ? prev : merged;
}

function backfillLibraryItem(prev: LibraryItem, next: LibraryItem): LibraryItem {
  if (prev.id !== next.id) return next;
  const name = next.name || prev.name;
  const canReuseEpisodeDetails = !next.lastVideoId || next.lastVideoId === prev.lastVideoId;
  const lastEpisodeName = next.lastEpisodeName || (canReuseEpisodeDetails ? prev.lastEpisodeName : undefined);
  if (name === next.name && lastEpisodeName === next.lastEpisodeName) return next;
  return { ...next, name, lastEpisodeName };
}

function metaRenderEqual(a: Meta, b: Meta): boolean {
  return (
    a.id === b.id &&
    a.name === b.name &&
    a.poster === b.poster &&
    a.background === b.background &&
    a.logo === b.logo &&
    a.year === b.year &&
    a.releaseInfo === b.releaseInfo
  );
}

function libraryItemRenderEqual(a: LibraryItem, b: LibraryItem): boolean {
  return (
    a.id === b.id &&
    a.name === b.name &&
    a.poster === b.poster &&
    a.background === b.background &&
    a.logo === b.logo &&
    a.timeOffset === b.timeOffset &&
    a.duration === b.duration &&
    a.lastVideoId === b.lastVideoId &&
    a.lastEpisodeName === b.lastEpisodeName &&
    a.lastEpisodeSeason === b.lastEpisodeSeason &&
    a.lastEpisodeNumber === b.lastEpisodeNumber &&
    a.lastEpisodeThumbnail === b.lastEpisodeThumbnail &&
    a.continueWatchingBadge === b.continueWatchingBadge &&
    a.newEpisodeReleasedAt === b.newEpisodeReleasedAt
  );
}
