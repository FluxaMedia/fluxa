import { effectRunnerLibraryKey } from './libraryOps';
import { storageRead, storageWrite } from './engine';
import { loadActiveProfile } from './libraryOps';
import { getOAuthClientId } from './traktSync';
import { nuvioPullLibrary, nuvioPullWatchProgress } from './nuvioApi';
import { coreNuvioLibraryToWatchlist, coreNuvioResolveContinueWatching } from './engineCoreLibrary';

export type LibraryProvider = 'trakt' | 'simkl' | 'anilist' | 'nuvio' | 'stremio';

export type ProviderLibrarySnapshot = {
  watchlist: Record<string, unknown>[];
  watching: Record<string, unknown>[];
  completed: Record<string, unknown>[];
  dropped: Record<string, unknown>[];
  favorites: Record<string, unknown>[];
};

export const PROVIDER_LIBRARIES_CHANGED = 'provider-libraries-changed';

type ProviderLibraries = Partial<Record<LibraryProvider, ProviderLibrarySnapshot>>;

async function storageKey(profileKey?: string): Promise<string> {
  return (profileKey ?? await effectRunnerLibraryKey()).replace(/^library_/, 'provider_libraries_');
}

export async function loadProviderLibraries(profileKey?: string): Promise<ProviderLibraries> {
  const profile = await loadActiveProfile();
  if (profile) {
    const remote: ProviderLibraries = {};
    const tasks: Promise<void>[] = [];
    if (profile.traktAccessToken) {
      tasks.push((async () => {
        const { syncTraktNow } = await import('./traktExternalSync');
        const clientId = await getOAuthClientId('trakt');
        const result = await syncTraktNow({ profile, token: profile.traktAccessToken, clientId, force: true, readOnly: true });
        if (result && typeof result === 'object' && 'snapshot' in result) remote.trakt = (result as { snapshot: ProviderLibrarySnapshot }).snapshot;
      })().catch(() => undefined));
    }
    if (profile.simklAccessToken) {
      tasks.push((async () => {
        const { syncSimklNow } = await import('./simklExternalSync');
        const clientId = await getOAuthClientId('simkl');
        const result = await syncSimklNow({ profile, token: profile.simklAccessToken, clientId, force: true, readOnly: true });
        if (result && typeof result === 'object' && 'snapshot' in result) remote.simkl = (result as { snapshot: ProviderLibrarySnapshot }).snapshot;
      })().catch(() => undefined));
    }
    if (profile.anilistAccessToken) {
      tasks.push((async () => {
        const { syncAniListNow } = await import('./anilistExternalSync');
        const result = await syncAniListNow({ profile, token: profile.anilistAccessToken, readOnly: true });
        if (result && typeof result === 'object' && 'snapshot' in result) remote.anilist = (result as { snapshot: ProviderLibrarySnapshot }).snapshot;
      })().catch(() => undefined));
    }
    if (profile.stremioAuthKey) {
      tasks.push((async () => {
        const { syncStremioNow } = await import('./stremioExternalSync');
        const result = await syncStremioNow({ profile, token: profile.stremioAuthKey, readOnly: true });
        if (result && typeof result === 'object' && 'snapshot' in result) remote.stremio = (result as { snapshot: ProviderLibrarySnapshot }).snapshot;
      })().catch(() => undefined));
    }
    if (profile.nuvioAccessToken) {
      tasks.push((async () => {
        const profileId = profile.nuvioProfileIndex ?? 1;
        const [library, progress] = await Promise.all([
          nuvioPullLibrary(profile.nuvioAccessToken!, profileId),
          nuvioPullWatchProgress(profile.nuvioAccessToken!, profileId),
        ]);
        const watchlist = (await coreNuvioLibraryToWatchlist(library)) ?? [];
        const resolved = (await coreNuvioResolveContinueWatching(progress, {})) ?? [];
        const libraryById = new Map(library.map((item) => [item.content_id, item]));
        const watching = resolved.map((entry) => {
          const progressEntry = entry as Record<string, unknown>;
          const libraryItem = libraryById.get(String(progressEntry.content_id));
          return libraryItem ? {
            ...progressEntry,
            id: progressEntry.content_id,
            type: progressEntry.content_type,
            name: libraryItem.name,
            poster: libraryItem.poster,
            background: libraryItem.background,
            description: libraryItem.description,
            releaseInfo: libraryItem.release_info,
            genres: libraryItem.genres,
            timeOffset: progressEntry.position,
            duration: progressEntry.duration,
            videoId: progressEntry.video_id,
            season: progressEntry.season,
            episode: progressEntry.episode,
          } : progressEntry;
        });
        remote.nuvio = { watchlist: watchlist as Record<string, unknown>[], watching: watching as Record<string, unknown>[], completed: [], dropped: [], favorites: [] };
      })().catch(() => undefined));
    }
    await Promise.all(tasks);
    return remote;
  }
  const libraries = (await storageRead<ProviderLibraries>(await storageKey(profileKey))) ?? {};
  return Object.fromEntries(
    Object.entries(libraries).map(([provider, snapshot]) => [provider, {
      watchlist: snapshot?.watchlist ?? [],
      watching: snapshot?.watching ?? [],
      completed: snapshot?.completed ?? [],
      dropped: snapshot?.dropped ?? [],
      favorites: snapshot?.favorites ?? [],
    }]),
  ) as ProviderLibraries;
}

export async function saveProviderLibrary(provider: LibraryProvider, snapshot: ProviderLibrarySnapshot, profileKey?: string): Promise<void> {
  const libraries = await loadProviderLibraries(profileKey);
  await storageWrite(await storageKey(profileKey), { ...libraries, [provider]: snapshot });
  window.dispatchEvent(new Event(PROVIDER_LIBRARIES_CHANGED));
}
