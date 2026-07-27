import {
  coreCalendarItemsFromMeta,
  coreInvoke,
  coreLibraryApplyMarkWatched,
  coreLibraryLocalStatePlan,
  coreMergeProgressMeta,
  coreNextUnairedEpisode,
  corePlaybackProgressMergePlan,
  coreWatchlistTogglePlan,
  libraryProgressDelete,
  libraryProgressRead,
  libraryProgressUpsert,
  libraryStatusSet,
  libraryWatchedSet,
  storageRead,
  storageWrite,
} from "./engine";
import {
  buildContinueWatching,
  effectRunnerLibraryKey,
  loadActiveProfile,
  loadAddons,
  loadLibrary,
  persistContinueWatchingMerge,
  persistProgressMerge,
  persistWatchedMerge,
  saveLibrary,
} from "./libraryOps";
import {
  pushLibraryStatusExternal,
  pushMarkWatchedExternal,
  pushPlaybackProgressExternal,
  pushWatchlistExternal,
  type WatchedEpisodeInfo,
  type WatchProgressInfo,
} from "./externalSync";
import { fetchVideosForSeries, runWithConcurrency } from "./fetchPlanning";
import { fetchTraktCalendarItems } from "./traktExternalSync";
import { fetchSimklCalendarItems } from "./simklExternalSync";
import { fetchAniListCalendarItems } from "./anilistExternalSync";
import { getOAuthClientId, refreshTraktProfile } from "./traktSync";
import { notify } from "./notifications";
import { t } from "../i18n";
import type { LibraryItem } from "./types";

const calendarCache = new Map<string, unknown>();

export function invalidateCalendarCache() {
  calendarCache.clear();
}

const AIR_DATE_REFRESH_CONCURRENCY = 3;

type CalendarMonthItems = Record<string, Record<string, unknown>[]>;

export async function refreshCalendarMonth(year: number, month: number): Promise<void> {
  const monthPrefix = `${Math.trunc(year)}-${String(Math.trunc(month)).padStart(2, "0")}`;
  const library = await loadLibrary();
  const libraryItems = [
    ...((library.watchlist as LibraryItem[] | undefined) ?? []),
    ...((library.continueWatching as LibraryItem[] | undefined) ?? []),
  ];
  const series = libraryItems.filter((item) => item.type === "series" && item.id);
  const addons = await loadAddons();
  const localItems = (await runWithConcurrency(
    series,
    AIR_DATE_REFRESH_CONCURRENCY,
    async (item) => {
      const videos = await fetchVideosForSeries(item.id, addons);
      return coreCalendarItemsFromMeta(
        JSON.stringify({
          id: item.id,
          name: item.name,
          poster: item.poster,
          type: item.type,
          videos,
        }),
        monthPrefix,
      ) as Promise<Record<string, unknown>[]>;
    },
  )).flat();
  const profile = await loadActiveProfile();
  const contentIds = libraryItems.flatMap((item) => item.id ? [item.id] : []);
  const providerItems: Record<string, unknown>[] = [];
  if (profile?.simklAccessToken) {
    const clientId = await getOAuthClientId("simkl");
    providerItems.push(...await fetchSimklCalendarItems(
      profile.simklAccessToken,
      clientId,
      contentIds,
      { year, month },
    ).catch(() => []));
  }
  if (profile?.traktAccessToken) {
    const clientId = await getOAuthClientId("trakt");
    providerItems.push(...await fetchTraktCalendarItems(
      profile.traktAccessToken,
      clientId,
      { year, month },
    ).catch(() => []));
  }
  const latestLibrary = await loadLibrary();
  const calendarMonthItems = (latestLibrary.calendarMonthItems as CalendarMonthItems | undefined) ?? {};
  calendarMonthItems[monthPrefix] = [...localItems, ...providerItems];
  latestLibrary.calendarMonthItems = calendarMonthItems;
  await saveLibrary(latestLibrary);
  invalidateCalendarCache();
}

export async function refreshWatchlistAirDates(): Promise<void> {
  const lib = await loadLibrary();
  const nowMs = Date.now();
  const watchlist = (lib.watchlist as LibraryItem[] | undefined) ?? [];
  const continueWatching =
    (lib.continueWatching as LibraryItem[] | undefined) ?? [];

  const candidates = (await coreInvoke<LibraryItem[]>(
    "airDateRefreshPlan",
    JSON.stringify({
      items: [...watchlist, ...continueWatching],
      nowMs,
    }),
  )) ?? [];
  if (candidates.length === 0) return;

  const addons = await loadAddons();
  const nowIso = new Date(nowMs).toISOString();

  const updates = await runWithConcurrency(
    candidates,
    AIR_DATE_REFRESH_CONCURRENCY,
    async (item) => {
      const videos = await fetchVideosForSeries(item.id, addons);
      const next = videos.length
        ? await coreNextUnairedEpisode(JSON.stringify(videos), nowMs)
        : null;
      return {
        id: item.id,
        nextEpisodeAirDate: next?.released,
        nextEpisodeSeason: next?.season,
        nextEpisodeNumber: next?.episode ?? next?.number,
        nextEpisodeTitle: next?.title ?? next?.name,
        nextEpisodePoster: next?.thumbnail,
        lastAirDateCheckedAt: nowIso,
      };
    },
  );

  const applied = await coreInvoke<
    { watchlist: LibraryItem[]; continueWatching: LibraryItem[] }
  >(
    "applyAirDateUpdates",
    JSON.stringify({
      watchlist,
      continueWatching,
      updates,
    }),
  );
  if (!applied) return;
  lib.watchlist = applied.watchlist;
  lib.continueWatching = applied.continueWatching;

  await saveLibrary(lib);
  invalidateCalendarCache();
}

export async function refreshExternalCalendarItems(): Promise<{ traktError?: string }> {
  const profile = await loadActiveProfile();
  if (!profile) return {};

  const library = await loadLibrary();
  const calendarContentIds = [
    ...((library.watchlist as LibraryItem[] | undefined) ?? []),
    ...((library.continueWatching as LibraryItem[] | undefined) ?? []),
  ].flatMap((item) => item.id ? [item.id] : []);
  const tasks: Promise<Record<string, unknown>[]>[] = [];
  let traktError: string | undefined;
  const activeProfile = profile.traktAccessToken
    ? await refreshTraktProfile(profile).catch(() => profile)
    : profile;

  if (activeProfile.traktAccessToken) {
    tasks.push(
      (async () => {
        const clientId = await getOAuthClientId("trakt");
        return fetchTraktCalendarItems(activeProfile.traktAccessToken!, clientId);
      })().catch((error: unknown) => {
        traktError = error instanceof Error ? error.message : String(error);
        return [];
      }),
    );
  }

  if (profile.simklAccessToken) {
    tasks.push(
      (async () => {
        const clientId = await getOAuthClientId("simkl");
        return fetchSimklCalendarItems(
          profile.simklAccessToken!,
          clientId,
          calendarContentIds,
        );
      })().catch(() => []),
    );
  }

  if (profile.anilistAccessToken) {
    tasks.push(
      fetchAniListCalendarItems(profile.anilistAccessToken).catch(() => []),
    );
  }

  if (tasks.length === 0) return {};

  const results = await Promise.all(tasks);
  library.externalCalendarItems = results.flat();
  await saveLibrary(library);
  invalidateCalendarCache();
  return { traktError };
}

const NOTIFIED_EPISODES_KEY = "notified_released_episode_ids";

async function deriveNextProgressInfo(
  seriesId: string | undefined,
  contentType: string,
  watchedEpisodes: Array<
    { season?: number; episode?: number; number?: number }
  >,
): Promise<WatchProgressInfo | undefined> {
  if (!seriesId || watchedEpisodes.length === 0) return undefined;
  const addons = await loadAddons();
  const videos = await fetchVideosForSeries(seriesId, addons);
  return (await coreInvoke<WatchProgressInfo>(
    "nextProgressInfoPlan",
    JSON.stringify({
      contentId: seriesId,
      contentType,
      videos,
      watchedEpisodes,
      nowMs: Date.now(),
    }),
  )) ?? undefined;
}

export async function notifyReleasedEpisodes(
  payload: Record<string, unknown>,
): Promise<void> {
  const items = (payload.items as Array<Record<string, unknown>> | undefined) ??
    [];
  const todayIso = new Date().toISOString().slice(0, 10);
  const plan = await coreInvoke<
    { items: Array<Record<string, unknown>>; storedKeys: string[] }
  >(
    "calendarNotificationContent",
    JSON.stringify({
      items: items.map((item) => ({
        ...item,
        dateIso: String(item.dateIso ?? "").slice(0, 10),
        metaId: item.contentId ?? item.seriesId ?? item.metaId ?? item.id,
        metaType: item.metaType ?? item.type ?? "series",
        artworkUrl: item.artworkUrl ?? item.poster,
      })),
      todayIso,
      alreadyNotifiedKeys:
        (await storageRead<string[]>(NOTIFIED_EPISODES_KEY)) ?? [],
      profileId: await storageRead<string>("active_profile_id"),
      notificationsEnabled: true,
      alertNewEpisodes: true,
      maxStoredKeys: 500,
    }),
  );
  if (!plan?.items.length) return;

  for (const item of plan.items) {
    const body = (item.episodeTitle as string | undefined) ??
      (item.subtitle as string | undefined);
    void notify(
      t("notifications.new_episode_title", item.title as string),
      body,
    );
  }
  await storageWrite(NOTIFIED_EPISODES_KEY, plan.storedKeys);
}

export async function applyLibraryCommand(
  payload: Record<string, unknown>,
): Promise<unknown> {
  const before = await loadLibrary();
  const command = payload.command as Record<string, unknown> | undefined;
  if (!command) return before;
  const plan = await coreInvoke<{
    library: Record<string, unknown>;
    statusMutation: {
      mediaId: string;
      status: "watchlist" | "completed" | "dropped" | null;
      item?: unknown;
    } | null;
    externalAction: Record<string, unknown>;
  }>(
    "libraryCommandPlan",
    JSON.stringify({
      library: before,
      command,
      nowIso: new Date().toISOString(),
    }),
  );
  if (!plan) return before;

  const after = plan.library;
  const key = await effectRunnerLibraryKey();
  if (plan.statusMutation) {
    await libraryStatusSet(
      key,
      plan.statusMutation.mediaId,
      plan.statusMutation.status,
      plan.statusMutation.item,
    );
  }
  await persistWatchedMerge(
    (before.watched as Record<string, boolean> | undefined) ?? {},
    (after.watched as Record<string, boolean> | undefined) ?? {},
  );
  await persistProgressMerge(
    (before.progress as Record<string, unknown> | undefined) ?? {},
    (after.progress as Record<string, unknown> | undefined) ?? {},
  );
  await persistContinueWatchingMerge(
    (before.continueWatching as Record<string, unknown>[] | undefined) ?? [],
    (after.continueWatching as Record<string, unknown>[] | undefined) ?? [],
  );
  await saveLibrary(after);
  invalidateCalendarCache();

  const action = plan.externalAction;
  const profile = await loadActiveProfile();
  if (action.kind === "watchlist") {
    void pushWatchlistExternal(
      action.item as Record<string, unknown>,
      action.command as "add" | "remove",
      profile,
    );
  } else if (action.kind === "status") {
    void pushLibraryStatusExternal(
      action.item as Record<string, unknown>,
      action.list as "completed" | "dropped",
      action.command as "add" | "remove",
      profile,
    );
  } else if (action.kind === "watched") {
    let progressInfo = action.progressInfo as WatchProgressInfo | undefined;
    if (!progressInfo && typeof action.seriesId === "string") {
      progressInfo = await deriveNextProgressInfo(
        action.seriesId,
        String(
          (action.meta as Record<string, unknown> | undefined)?.type ??
            "series",
        ),
        (command.episodes as
          | Array<
            {
              id?: string;
              name?: string;
              title?: string;
              season?: number;
              episode?: number;
              number?: number;
            }
          >
          | undefined) ?? [],
      );
    }
    void pushMarkWatchedExternal(
      action.videoIds as string[],
      action.watched !== false,
      action.meta as Record<string, unknown> | undefined,
      profile,
      action.episodeInfos as WatchedEpisodeInfo[] | undefined,
      progressInfo,
    );
  }
  return after;
}
export async function writePlaybackProgress(
  payload: Record<string, unknown>,
): Promise<unknown> {
  const before = await loadLibrary();
  const progress = payload.progress as Record<string, unknown> | undefined;
  if (!progress) return {};
  const nowMs = Date.now();
  const plan = await coreInvoke<{
    library: Record<string, unknown>;
    entry: unknown;
    contentId: string;
    externalProgress?: WatchProgressInfo;
  }>(
    "playbackProgressWritePlan",
    JSON.stringify({
      library: before,
      progress,
      nowIso: new Date(nowMs).toISOString(),
      nowMs,
    }),
  );
  if (!plan) return {};
  const dismissed = (before.dismissedContinueWatching as Record<string, string> | undefined) ?? {};
  if (dismissed[plan.contentId]) {
    delete dismissed[plan.contentId];
    plan.library.dismissedContinueWatching = dismissed;
  }
  await libraryProgressUpsert(
    await effectRunnerLibraryKey(),
    plan.contentId,
    plan.entry,
  );
  await persistContinueWatchingMerge(
    (before.continueWatching as Record<string, unknown>[] | undefined) ?? [],
    (plan.library.continueWatching as Record<string, unknown>[] | undefined) ??
      [],
  );
  await saveLibrary(plan.library);
  invalidateCalendarCache();
  if (plan.externalProgress) {
    const meta = progress.meta as Record<string, unknown> | undefined;
    void loadActiveProfile().then((profile) =>
      pushPlaybackProgressExternal(plan.externalProgress!, meta ?? {}, profile)
    );
  }
  return {};
}
export async function writeSettings(
  payload: Record<string, unknown>,
): Promise<unknown> {
  const existing = (await storageRead<Record<string, unknown>>("settings")) ??
    {};
  const updated = { ...existing, ...payload };
  await storageWrite("settings", updated);
  return updated;
}

export async function readLibraryState(): Promise<unknown> {
  return loadLibrary();
}

export async function readPlaybackProgress(
  payload: Record<string, unknown>,
): Promise<unknown> {
  const id = payload.id as string | undefined;
  if (!id) return null;
  return libraryProgressRead(await effectRunnerLibraryKey(), id);
}

export async function readDetailLocalState(
  payload: Record<string, unknown>,
): Promise<unknown> {
  const lib = await loadLibrary();
  return (await coreLibraryLocalStatePlan({ library: lib, ...payload })) ?? {
    progress: null,
    isInWatchlist: false,
    watchedVideoIds: [],
  };
}

export async function readCalendarMonth(
  payload: Record<string, unknown>,
): Promise<unknown> {
  const year = Number(payload.year);
  const month = Number(payload.month);
  const monthPrefix = Number.isFinite(year) && Number.isFinite(month)
    ? `${Math.trunc(year)}-${String(Math.trunc(month)).padStart(2, "0")}`
    : "";
  const plannedItems = Array.isArray(payload.plannedItems)
    ? payload.plannedItems
    : [];

  const cached = calendarCache.get(monthPrefix);
  if (cached) return cached;

  const lib = await loadLibrary();
  const libraryItems = [
    ...((lib.watchlist as unknown[] | undefined) ?? []),
    ...((lib.continueWatching as unknown[] | undefined) ?? []),
  ];
  const calendarMonthItems = (lib.calendarMonthItems as CalendarMonthItems | undefined) ?? {};
  const result = (await coreInvoke(
    "desktopCalendarReadPlan",
    JSON.stringify({
      monthPrefix,
      plannedItems: [...plannedItems, ...(calendarMonthItems[monthPrefix] ?? [])],
      libraryItems,
      externalItems: (lib.externalCalendarItems as unknown[] | undefined) ?? [],
    }),
  )) ?? { items: plannedItems, localItems: [], externalItems: [] };
  calendarCache.set(monthPrefix, result);
  return result;
}
