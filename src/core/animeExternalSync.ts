import { platformFetch } from './httpClient';
import {
  coreAnilistMediaListStatus,
  coreAnilistSaveMediaListEntryVariables,
  coreAnilistSearchBestMatch,
  coreExtractAnilistIdFromLinks,
  coreShouldAttemptAnimeTracking,
  storageRead,
  storageWrite,
} from './engine';
import { saveProfile } from './profiles';
import type { Meta, UserProfile } from './types';

type AnimeIds = {
  anilistId?: number;
  confidence: 'exact' | 'title-year';
};

type AnimeTrackingUpdate = {
  meta?: Record<string, unknown>;
  episode?: { season?: number; episode?: number; title?: string };
  progressEpisode?: number;
  watched: boolean;
};

type AnimeIdCache = Record<string, AnimeIds>;

const ANIME_ID_CACHE_KEY = 'anime_id_map';

export async function pushAnimeTrackingExternal(
  update: AnimeTrackingUpdate,
  profile: UserProfile,
): Promise<void> {
  if (!update.watched || !update.meta) return;
  if (!profile.anilistAccessToken) return;
  const validProfile = await refreshAnimeTrackingProfile(profile).catch(() => profile);

  const meta = update.meta as unknown as Meta;
  if (!(await coreShouldAttemptAnimeTracking(meta))) return;

  const progressEpisode = update.progressEpisode
    ?? update.episode?.episode
    ?? firstEpisodeNumberFromMeta(meta)
    ?? 1;
  if (!Number.isFinite(progressEpisode) || progressEpisode < 1) return;

  const ids = await resolveAnimeIds(meta);
  const tasks: Promise<void>[] = [];

  if (validProfile.anilistAccessToken && ids?.anilistId) {
    tasks.push(pushAniListProgress(ids.anilistId, progressEpisode, meta, validProfile.anilistAccessToken).catch(() => undefined));
  }

  await Promise.all(tasks);
}

export async function refreshAnimeTrackingProfile(profile: UserProfile): Promise<UserProfile> {
  let updated = profile;
  const now = Math.floor(Date.now() / 1000);

  if (updated.anilistAccessToken && (updated.anilistTokenExpiresAt ?? 0) > 0 && (updated.anilistTokenExpiresAt ?? 0) <= now + 60) {
    updated = {
      ...updated,
      anilistAccessToken: undefined,
      anilistRefreshToken: undefined,
      anilistTokenExpiresAt: undefined,
    };
  }

  if (updated !== profile) await saveProfile(updated);
  return updated;
}

async function resolveAnimeIds(meta: Meta): Promise<AnimeIds | null> {
  const cacheKey = animeCacheKey(meta);
  const cache = (await storageRead<AnimeIdCache>(ANIME_ID_CACHE_KEY)) ?? {};
  if (cacheKey && cache[cacheKey]) return cache[cacheKey];

  const linkedAnilistId = await coreExtractAnilistIdFromLinks(meta);
  if (linkedAnilistId) {
    const resolved = await completeIdsFromAniList({ anilistId: linkedAnilistId }, meta);
    if (cacheKey && resolved) {
      cache[cacheKey] = resolved;
      await storageWrite(ANIME_ID_CACHE_KEY, cache);
    }
    return resolved;
  }

  const searched = await searchAniList(meta);
  if (cacheKey && searched) {
    cache[cacheKey] = searched;
    await storageWrite(ANIME_ID_CACHE_KEY, cache);
  }
  return searched;
}

async function completeIdsFromAniList(ids: Partial<AnimeIds>, meta: Meta): Promise<AnimeIds | null> {
  if (!ids.anilistId) return null;
  const query = `query ($id: Int) { Media(id: $id, type: ANIME) { id } }`;
  const media = await anilistGraphql<{ Media?: { id?: number } }>(query, { id: ids.anilistId });
  return {
    anilistId: media?.Media?.id ?? ids.anilistId,
    confidence: 'exact',
  };
}

async function searchAniList(meta: Meta): Promise<AnimeIds | null> {
  const search = meta.name?.trim();
  if (!search) return null;
  const query = `
    query ($search: String, $year: Int) {
      Page(page: 1, perPage: 5) {
        media(search: $search, type: ANIME, sort: SEARCH_MATCH) {
          id
          seasonYear
          title { romaji english native }
        }
      }
    }
  `;
  const year = typeof meta.year === 'number' ? meta.year : undefined;
  const data = await anilistGraphql<{ Page?: { media?: Array<{ id?: number; seasonYear?: number | null; title?: Record<string, string | null> }> } }>(
    query,
    { search, year },
  );
  const candidates = data?.Page?.media ?? [];
  return coreAnilistSearchBestMatch(meta, candidates);
}

async function anilistGraphql<T>(query: string, variables: Record<string, unknown>, token?: string, attempt = 0): Promise<T | null> {
  const res = await platformFetch('https://graphql.anilist.co', {
    method: 'POST',
    headers: {
      Accept: 'application/json',
      'Content-Type': 'application/json',
      ...(token ? { Authorization: `Bearer ${token}` } : {}),
    },
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

async function pushAniListProgress(anilistId: number, progressEpisode: number, meta: Meta, token: string): Promise<void> {
  const query = `
    mutation ($mediaId: Int, $progress: Int, $status: MediaListStatus) {
      SaveMediaListEntry(mediaId: $mediaId, progress: $progress, status: $status) {
        id
        progress
        status
      }
    }
  `;
  const totalEpisodes = Array.isArray(meta.videos) ? meta.videos.length : 0;
  const status = await coreAnilistMediaListStatus(totalEpisodes, progressEpisode);
  const variables = (await coreAnilistSaveMediaListEntryVariables(`anilist:${anilistId}`, status, Math.max(0, Math.floor(progressEpisode))))
    ?? { mediaId: anilistId, progress: Math.max(0, Math.floor(progressEpisode)), status };
  await anilistGraphql(query, variables, token);
}

function firstEpisodeNumberFromMeta(meta: Meta): number | undefined {
  const first = meta.videos?.[0];
  return first?.episode ?? first?.number;
}

function animeCacheKey(meta: Meta): string {
  return meta.id || `${normalizeTitle(meta.name)}:${meta.year ?? parseYear(meta.releaseInfo) ?? ''}`;
}

function parseYear(value?: string): number | undefined {
  const match = value?.match(/\b(19|20)\d{2}\b/);
  return match ? Number(match[0]) : undefined;
}

function normalizeTitle(value: string): string {
  return value.toLowerCase().replace(/[^a-z0-9]+/g, ' ').replace(/\s+/g, ' ').trim();
}
