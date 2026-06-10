import {
  coreLibraryContinueWatchingItems,
  coreNormalizeLibraryDocument,
  storageRead,
  storageWrite,
} from './engine';
import { normalizeAddonDescriptor } from './addons';
import type { AddonDescriptor, LibraryItem, UserProfile } from './types';

export async function loadAddons(): Promise<AddonDescriptor[]> {
  return ((await storageRead<AddonDescriptor[]>('addons')) ?? []).map(normalizeAddonDescriptor);
}

async function activeProfileStorageSuffix(): Promise<string> {
  const profileId = (await storageRead<string>('active_profile_id'))?.trim();
  return profileId ? profileId.replace(/[^a-zA-Z0-9_-]/g, '_') : 'guest';
}

export async function effectRunnerLibraryKey(): Promise<string> {
  return `library_${await activeProfileStorageSuffix()}`;
}

export async function normalizeLibraryDoc(lib: Record<string, unknown>): Promise<Record<string, unknown>> {
  return coreNormalizeLibraryDocument(JSON.stringify(lib));
}

export async function loadLibrary(): Promise<Record<string, unknown>> {
  const key = await effectRunnerLibraryKey();
  const profileLibrary = await storageRead<Record<string, unknown>>(key);
  if (profileLibrary) return normalizeLibraryDoc(profileLibrary);
  const legacyLibrary = await storageRead<Record<string, unknown>>('library');
  if (legacyLibrary) {
    const migrated = await normalizeLibraryDoc({ ...legacyLibrary, migratedFrom: 'library' });
    await storageWrite(key, migrated);
    return migrated;
  }
  return normalizeLibraryDoc({});
}

export async function saveLibrary(lib: Record<string, unknown>): Promise<void> {
  await storageWrite(await effectRunnerLibraryKey(), await normalizeLibraryDoc(lib));
}

export async function loadPrefs(): Promise<Record<string, unknown>> {
  return (await storageRead<Record<string, unknown>>('prefs')) ?? {};
}

export async function loadActiveProfile(): Promise<UserProfile | null> {
  const profileId = await storageRead<string>('active_profile_id');
  if (!profileId) return null;
  const profiles = (await storageRead<UserProfile[]>('profiles')) ?? [];
  return profiles.find((profile) => profile.id === profileId) ?? null;
}

export async function buildContinueWatching(progressMap: Record<string, unknown>): Promise<unknown[]> {
  const coreItems = Object.values(progressMap).map((entry) => {
    const progress = entry as {
      meta?: { id?: string; name?: string; type?: string; poster?: string; background?: string; logo?: string };
      timeOffset?: number;
      duration?: number;
      lastVideoId?: string | null;
      lastEpisodeName?: string | null;
      lastEpisodeSeason?: number | null;
      lastEpisodeNumber?: number | null;
      lastEpisodeThumbnail?: string | null;
      lastStreamUrl?: string | null;
      lastStreamTitle?: string | null;
      lastStream?: unknown;
      savedAt?: string;
    };
    return {
      _id: progress.meta?.id ?? '',
      name: progress.meta?.name ?? '',
      type: progress.meta?.type ?? '',
      poster: progress.meta?.poster ?? null,
      background: progress.meta?.background ?? null,
      logo: progress.meta?.logo ?? null,
      state: {
        lastWatched: progress.savedAt ?? '',
        timeOffset: Math.floor(progress.timeOffset ?? 0),
        duration: Math.floor(progress.duration ?? 0),
        videoId: progress.lastVideoId ?? null,
        flaggedWatched: 0,
      },
      lastVideoId: progress.lastVideoId ?? null,
      lastEpisodeName: progress.lastEpisodeName ?? null,
      lastEpisodeSeason: progress.lastEpisodeSeason ?? null,
      lastEpisodeNumber: progress.lastEpisodeNumber ?? null,
      lastEpisodeThumbnail: progress.lastEpisodeThumbnail ?? null,
      lastStreamUrl: progress.lastStreamUrl ?? null,
      lastStreamTitle: progress.lastStreamTitle ?? null,
      lastStream: progress.lastStream ?? null,
    };
  });
  const planned = await coreLibraryContinueWatchingItems(coreItems);
  if (planned) {
    const extrasById = new Map(
      coreItems.map((item) => [item._id, {
        savedAt: item.state.lastWatched,
        timeOffset: item.state.timeOffset,
        duration: item.state.duration,
        lastVideoId: item.lastVideoId,
        lastEpisodeName: item.lastEpisodeName,
        lastEpisodeSeason: item.lastEpisodeSeason,
        lastEpisodeNumber: item.lastEpisodeNumber,
        lastEpisodeThumbnail: item.lastEpisodeThumbnail,
        lastStreamUrl: item.lastStreamUrl,
        lastStreamTitle: item.lastStreamTitle,
        lastStream: item.lastStream,
      }]),
    );
    return planned.map((item) => {
      const meta = item as Record<string, unknown>;
      const id = typeof meta.id === 'string' ? meta.id : '';
      return { ...meta, ...(extrasById.get(id) ?? {}) };
    });
  }
  return Object.values(progressMap)
    .map((entry) => entry as { meta?: LibraryItem; timeOffset?: number; duration?: number })
    .filter((p) => {
      const offset = p.timeOffset ?? 0;
      const dur = p.duration ?? 0;
      if (offset === 0 && (p as Record<string, unknown>).lastVideoId) return true;
      return offset > 0 && dur > 0 && offset / dur < 0.95;
    })
    .map((p) => ({
      ...p.meta,
      timeOffset: p.timeOffset,
      duration: p.duration,
      lastEpisodeName: (p as Record<string, unknown>).lastEpisodeName,
      lastEpisodeSeason: (p as Record<string, unknown>).lastEpisodeSeason,
      lastEpisodeNumber: (p as Record<string, unknown>).lastEpisodeNumber,
      lastEpisodeThumbnail: (p as Record<string, unknown>).lastEpisodeThumbnail,
      lastStreamUrl: (p as Record<string, unknown>).lastStreamUrl,
      lastStreamTitle: (p as Record<string, unknown>).lastStreamTitle,
      lastStream: (p as Record<string, unknown>).lastStream,
    }));
}
