import { platformFetch } from './httpClient';
import { profileStorageKey } from './libraryOps';
import type { UserProfile } from './types';
import { replaceExternalContinueWatching } from './externalSyncUtils';
import { coreAnilistEntriesToSync, coreAnilistGraphqlQueries, coreAnilistSaveMediaListEntryVariables, coreInvoke } from './engine';
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

  const categories = payload.categories as ImportCategory[] | undefined;
  const dryRun = payload.dryRun === true;

  const plan = await coreAnilistEntriesToSync(entries, Date.now(), categories, dryRun);
  if (!plan) return { synced: false, error: 'AniList entries could not be processed' };

  if (plan.watching != null) {
    await replaceExternalContinueWatching({ provider: 'anilist', items: plan.watching, profileKey });
  }
  if (!dryRun) {
    await saveProviderLibrary('anilist', {
      watchlist: plan.watchlist ?? [],
      watching: plan.watching ?? [],
      completed: plan.completed ?? [],
      dropped: plan.dropped ?? [],
      favorites: [],
    }, profileKey);
  }

  return {
    synced: true,
    provider: 'anilist',
    continueWatchingCount: plan.watchingCount,
    watchlistCount: plan.watchlistCount,
    completedCount: plan.completedCount,
    droppedCount: plan.droppedCount,
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
  const queries = await coreAnilistGraphqlQueries();
  const variables = (await coreAnilistSaveMediaListEntryVariables(`anilist:${anilistId}`, status as 'COMPLETED' | 'CURRENT'))
    ?? { mediaId: anilistId, status };
  await anilistGraphql(
    queries?.saveMediaListEntry ?? `mutation ($mediaId: Int, $status: MediaListStatus) { SaveMediaListEntry(mediaId: $mediaId, status: $status) { id } }`,
    variables,
    token,
  );
}

async function deleteAniListEntry(anilistId: number, token: string): Promise<void> {
  const queries = await coreAnilistGraphqlQueries();
  const lookupQuery = queries?.mediaListEntryLookup ?? `query ($mediaId: Int) { Media(id: $mediaId) { mediaListEntry { id } } }`;
  const entry = await anilistGraphql<{ Media?: { mediaListEntry?: { id?: number } } }>(lookupQuery, { mediaId: anilistId }, token);
  const entryId = entry?.Media?.mediaListEntry?.id;
  if (!entryId) return;
  const deleteQuery = queries?.deleteMediaListEntry ?? `mutation ($id: Int) { DeleteMediaListEntry(id: $id) { deleted } }`;
  await anilistGraphql(deleteQuery, { id: entryId }, token);
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
