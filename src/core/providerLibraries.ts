import { effectRunnerLibraryKey } from './libraryOps';
import { storageRead, storageWrite } from './engine';

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
