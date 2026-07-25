import { effectRunnerLibraryKey } from './libraryOps';
import { storageRead, storageWrite } from './engine';

export type LibraryProvider = 'trakt' | 'simkl' | 'anilist' | 'nuvio' | 'stremio';

export type ProviderLibrarySnapshot = {
  watchlist: Record<string, unknown>[];
  watching: Record<string, unknown>[];
  completed: Record<string, unknown>[];
  dropped: Record<string, unknown>[];
};

type ProviderLibraries = Partial<Record<LibraryProvider, ProviderLibrarySnapshot>>;

async function storageKey(): Promise<string> {
  return (await effectRunnerLibraryKey()).replace(/^library_/, 'provider_libraries_');
}

export async function loadProviderLibraries(): Promise<ProviderLibraries> {
  const libraries = (await storageRead<ProviderLibraries>(await storageKey())) ?? {};
  return Object.fromEntries(
    Object.entries(libraries).map(([provider, snapshot]) => [provider, {
      watchlist: snapshot?.watchlist ?? [],
      watching: snapshot?.watching ?? [],
      completed: snapshot?.completed ?? [],
      dropped: snapshot?.dropped ?? [],
    }]),
  ) as ProviderLibraries;
}

export async function saveProviderLibrary(provider: LibraryProvider, snapshot: ProviderLibrarySnapshot): Promise<void> {
  const libraries = await loadProviderLibraries();
  await storageWrite(await storageKey(), { ...libraries, [provider]: snapshot });
}
