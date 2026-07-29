import { platformFetch } from './httpClient';
import { loadLibrary, saveLibrary, buildContinueWatching, persistStatusListMerge, persistWatchedMerge, persistProgressMerge, profileStorageKey } from './libraryOps';
import type { UserProfile } from './types';
import { replaceExternalContinueWatching } from './externalSyncUtils';
import { coreAnilistEntriesToSync, coreInvoke, coreMergeLibraryItemsById } from './engine';
import { saveProviderLibrary } from './providerLibraries';
import type { ImportCategory } from './importCategories';

type AniListEntry = {
  status?: string | null;
  media?: {
    id?: number;
    title?: { romaji?: string | null; english?: string | null } | null;
    nextAiringEpisode?: { airingAt?: number; episode?: number } | null;
  } | null;
};

type AniListCollectionResponse = {
  MediaListCollection?: {
    lists?: Array<{ entries?: AniListEntry[] | null }> | null;
  } | null;
};

type LibraryItemRecord = Record<string, unknown>;

const ANILIST_COLLECTION_QUERY = `
  query ($userId: Int) {
    MediaListCollection(userId: $userId, type: ANIME) {
      lists {
        entries {
          status
          progress
          updatedAt
          media {
            id
            title { romaji english native }
            coverImage { large extraLarge }
            bannerImage
            episodes
            seasonYear
            genres
            nextAiringEpisode { airingAt episode }
          }
        }
      }
    }
  }
`;

const ANILIST_VIEWER_QUERY = `query { Viewer { id } }`;

export async function syncAniListNow(payload: Record<string, unknown>): Promise<unknown> {
  const token = typeof payload.token === 'string' ? payload.token : undefined;
  if (!token) return { synced: false, error: 'AniList is not connected' };
  const profile = payload.profile as UserProfile | undefined;
  const profileKey = profile ? profileStorageKey(profile) : undefined;

  const viewer = await anilistGraphql<{ Viewer?: { id?: number } }>(ANILIST_VIEWER_QUERY, {}, token);
  const userId = viewer?.Viewer?.id;
  if (!userId) return { synced: false, error: 'AniList account could not be loaded' };

  const data = await anilistGraphql<AniListCollectionResponse>(ANILIST_COLLECTION_QUERY, { userId }, token);
  const entries = (data?.MediaListCollection?.lists ?? [])
    .flatMap((list) => list.entries ?? [])
    .filter((entry): entry is AniListEntry => Boolean(entry?.media?.id));

  const plan = await coreAnilistEntriesToSync(entries, Date.now());
  if (!plan) return { synced: false, error: 'AniList entries could not be processed' };

  const categories = payload.categories as ImportCategory[] | undefined;
  const wants = (category: ImportCategory) => !categories || categories.includes(category);

  const lib = await loadLibrary(profileKey);
  const watchlistBefore = (lib.watchlist as LibraryItemRecord[] | undefined) ?? [];
  const completedBefore = (lib.completed as LibraryItemRecord[] | undefined) ?? [];
  const droppedBefore = (lib.dropped as LibraryItemRecord[] | undefined) ?? [];
  const watchedBefore = (lib.watched as Record<string, boolean> | undefined) ?? {};
  const progressBefore = (lib.progress as Record<string, unknown> | undefined) ?? {};

  if (wants('watchlist')) {
    lib.watchlist = await coreMergeLibraryItemsById(watchlistBefore, plan.watchlist);
    await persistStatusListMerge(watchlistBefore, lib.watchlist as LibraryItemRecord[], 'watchlist', profileKey);
  }
  if (wants('watched')) {
    lib.completed = await coreMergeLibraryItemsById(completedBefore, plan.completed);
    lib.dropped = await coreMergeLibraryItemsById(droppedBefore, plan.dropped);
    lib.watched = { ...watchedBefore, ...plan.watched };
    await persistStatusListMerge(completedBefore, lib.completed as LibraryItemRecord[], 'completed', profileKey);
    await persistStatusListMerge(droppedBefore, lib.dropped as LibraryItemRecord[], 'dropped', profileKey);
    await persistWatchedMerge(watchedBefore, lib.watched as Record<string, boolean>, profileKey);
  }
  if (wants('continueWatching')) {
    lib.progress = { ...progressBefore, ...plan.progress };
    lib.continueWatching = await buildContinueWatching(lib.progress as Record<string, unknown>);
    await persistProgressMerge(progressBefore, lib.progress as Record<string, unknown>, profileKey);
    await replaceExternalContinueWatching({ provider: 'anilist', items: plan.watching, profileKey });
  }
  await saveLibrary(lib, profileKey);
  await saveProviderLibrary('anilist', {
    watchlist: plan.watchlist,
    watching: plan.watching,
    completed: plan.completed,
    dropped: plan.dropped,
  }, profileKey);

  return {
    synced: true,
    provider: 'anilist',
    continueWatchingCount: plan.watching.length,
    watchlistCount: plan.watchlist.length,
    completedCount: plan.completed.length,
    droppedCount: plan.dropped.length,
  };
}

export async function fetchAniListCalendarItems(token: string): Promise<Record<string, unknown>[]> {
  const viewer = await anilistGraphql<{ Viewer?: { id?: number } }>(ANILIST_VIEWER_QUERY, {}, token);
  const userId = viewer?.Viewer?.id;
  if (!userId) return [];

  const data = await anilistGraphql<AniListCollectionResponse>(ANILIST_COLLECTION_QUERY, { userId }, token);
  const entries = (data?.MediaListCollection?.lists ?? []).flatMap((list) => list.entries ?? []);

  return (await coreInvoke<Record<string, unknown>[]>('providerCalendarItems', JSON.stringify({ provider: 'anilist', entries }))) ?? [];
}

export async function pushWatchlistAniList(
  id: string,
  command: 'add' | 'remove',
  token: string,
): Promise<void> {
  const anilistId = parseAniListId(id);
  if (!anilistId) return;
  if (command === 'remove') {
    await deleteAniListEntry(anilistId, token);
    return;
  }
  await setAniListStatus(anilistId, 'PLANNING', token);
}

export async function pushLibraryStatusAniList(
  id: string,
  list: string,
  command: 'add' | 'remove',
  token: string,
): Promise<void> {
  const anilistId = parseAniListId(id);
  if (!anilistId) return;
  if (command === 'remove') {
    await setAniListStatus(anilistId, 'CURRENT', token);
    return;
  }
  if (list === 'completed') {
    await setAniListStatus(anilistId, 'COMPLETED', token);
  } else if (list === 'dropped') {
    await setAniListStatus(anilistId, 'DROPPED', token);
  }
}

async function setAniListStatus(anilistId: number, status: string, token: string): Promise<void> {
  await anilistGraphql(`
    mutation ($mediaId: Int, $status: MediaListStatus) {
      SaveMediaListEntry(mediaId: $mediaId, status: $status) { id status }
    }
  `, { mediaId: anilistId, status }, token);
}

async function deleteAniListEntry(anilistId: number, token: string): Promise<void> {
  const viewer = await anilistGraphql<{ Viewer?: { id?: number } }>(ANILIST_VIEWER_QUERY, {}, token);
  const userId = viewer?.Viewer?.id;
  if (!userId) return;
  const entry = await anilistGraphql<{ MediaList?: { id?: number } }>(
    `query ($mediaId: Int, $userId: Int) { MediaList(mediaId: $mediaId, userId: $userId) { id } }`,
    { mediaId: anilistId, userId },
    token,
  );
  const entryId = entry?.MediaList?.id;
  if (!entryId) return;
  await anilistGraphql(`mutation ($id: Int) { DeleteMediaListEntry(id: $id) { deleted } }`, { id: entryId }, token);
}

async function anilistGraphql<T>(query: string, variables: Record<string, unknown>, token: string, attempt = 0): Promise<T | null> {
  const res = await platformFetch('https://graphql.anilist.co', {
    method: 'POST',
    headers: { Accept: 'application/json', 'Content-Type': 'application/json', Authorization: `Bearer ${token}` },
    body: JSON.stringify({ query, variables }),
  });
  if (res.status === 429 && attempt === 0) {
    const retryAfter = Number(res.headers.get('Retry-After'));
    await new Promise((resolve) => setTimeout(resolve, Math.min(60_000, Math.max(1_000, (Number.isFinite(retryAfter) ? retryAfter : 1) * 1_000))));
    return anilistGraphql(query, variables, token, 1);
  }
  const json = await res.json() as { data?: T; errors?: Array<{ message?: string }> };
  if (!res.ok || json.errors?.length) {
    throw new Error(json.errors?.map((e) => e.message).filter(Boolean).join('; ') || `AniList request failed: HTTP ${res.status}`);
  }
  return json.data ?? null;
}

function parseAniListId(id: string): number | null {
  const match = id.match(/^anilist:(\d+)/i);
  return match ? Number(match[1]) : null;
}
