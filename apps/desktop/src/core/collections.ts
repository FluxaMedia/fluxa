import { useEffect, useState } from 'react';
import type { NuvioRemoteCollectionSource, UserCollection, UserCollectionFolder } from './types';
import { coreExportCollections, coreImportCollections, coreInvoke } from './engine';

export type CollectionFolderPresentation = {
  imageUrl: string | null;
  shape: string;
  catalogId: string | null;
  catalogType: string | null;
};

export function useCollectionFolderPresentation(folder: UserCollectionFolder): CollectionFolderPresentation {
  const [presentation, setPresentation] = useState<CollectionFolderPresentation>({
    imageUrl: null,
    shape: 'poster',
    catalogId: null,
    catalogType: null,
  });

  useEffect(() => {
    let active = true;
    void coreInvoke<CollectionFolderPresentation>('collectionFolderPresentation', JSON.stringify(folder))
      .then((next) => {
        if (active && next) setPresentation(next);
      })
      .catch(() => {});
    return () => {
      active = false;
    };
  }, [folder]);

  return presentation;
}

export function remoteSourceKey(source: NuvioRemoteCollectionSource): string {
  if (source.provider === 'trakt') return String(source.traktListId ?? '');
  return source.tmdbId != null ? String(source.tmdbId) : (source.tmdbSourceType ?? '');
}

export async function importCollectionsJson(rawJson: string): Promise<UserCollection[]> {
  return ((await coreImportCollections(rawJson)) ?? []) as UserCollection[];
}

export async function exportCollectionsJson(collections: UserCollection[]): Promise<string> {
  return JSON.stringify((await coreExportCollections(JSON.stringify(collections))) ?? [], null, 2);
}
