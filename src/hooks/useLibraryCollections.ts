import { useLayoutEffect, useRef, useState } from 'react';
import { exportCollectionsJson, importCollectionsJson } from '../core/collections';
import { saveProfile } from '../core/profiles';
import { nuvioPushCollections } from '../core/nuvioApi';
import { freshNuvioProfile } from '../core/nuvioSync';
import { loadNuvioCollectionSource } from '../core/collectionSources';
import { coreInvoke } from '../core/engine';
import type { HomeCategory, Meta, NuvioRemoteCollectionSource, UserCollection, UserCollectionFolder, UserProfile } from '../core/types';

export function useLibraryCollections({
  activeProfile,
  onProfileUpdated,
  homeCategories,
}: {
  activeProfile?: UserProfile | null;
  onProfileUpdated?: (profile: UserProfile) => void;
  homeCategories: HomeCategory[];
}) {
  const collections: UserCollection[] = activeProfile?.libraryCollections ?? [];
  const [viewAllFolder, setViewAllFolder] = useState<{ title: string; items: Meta[]; groups: Array<{ type: string; items: Meta[] }> } | null>(null);
  const [editingCollection, setEditingCollection] = useState<UserCollection | 'new' | null>(null);
  const collectionsScrollRef = useRef<HTMLDivElement>(null);
  const savedScrollRef = useRef(0);

  useLayoutEffect(() => {
    if (!viewAllFolder && collectionsScrollRef.current) collectionsScrollRef.current.scrollTop = savedScrollRef.current;
  }, [viewAllFolder]);

  async function getItemsForFolder(folder: UserCollectionFolder): Promise<{ items: Meta[]; groups: Array<{ type: string; items: Meta[] }>; remoteSources: NuvioRemoteCollectionSource[] }> {
    return ((await coreInvoke('collectionFolderItemsPlan', JSON.stringify({ folder, categories: homeCategories }))) as { items: Meta[]; groups: Array<{ type: string; items: Meta[] }>; remoteSources: NuvioRemoteCollectionSource[] } | null) ?? { items: [], groups: [], remoteSources: [] };
  }

  async function openFolder(folder: UserCollectionFolder, title: string) {
    savedScrollRef.current = collectionsScrollRef.current?.scrollTop ?? 0;
    const local = await getItemsForFolder(folder);
    setViewAllFolder({ title, ...local });
    const specialSources = local.remoteSources;
    if (!specialSources.length) return;
    const batches = await Promise.all(specialSources.map((source) => loadNuvioCollectionSource(source)));
    const merged = (await coreInvoke<{ items: Meta[]; groups: Array<{ type: string; items: Meta[] }> }>('mergeFolderSources', JSON.stringify([local.items, ...batches]))) ?? local;
    setViewAllFolder({ title, ...merged });
  }

  async function saveCollections(next: UserCollection[]) {
    if (!activeProfile) return;
    const updated: UserProfile = { ...activeProfile, libraryCollections: next };
    await saveProfile(updated);
    onProfileUpdated?.(updated);
    if (!updated.nuvioAccessToken) return;

    try {
      const freshProfile = await freshNuvioProfile(updated);
      const token = freshProfile.nuvioAccessToken;
      if (!token) return;
      await nuvioPushCollections(token, freshProfile.nuvioProfileIndex ?? 1, next);
      if (freshProfile !== updated) onProfileUpdated?.(freshProfile);
    } catch {
    }
  }

  async function handleSaveCollection(col: UserCollection) {
    const existing = collections.findIndex((c) => c.id === col.id);
    const next = existing >= 0
      ? collections.map((c) => (c.id === col.id ? col : c))
      : [...collections, col];
    await saveCollections(next);
    setEditingCollection(null);
  }

  async function handleDeleteCollection(id: string) {
    await saveCollections(collections.filter((c) => c.id !== id));
  }

  async function handleImportJson(json: string) {
    const imported = await importCollectionsJson(json);
    if (!imported.length) return;
    const merged = (await coreInvoke<UserCollection[]>('collectionMergePlan', JSON.stringify({ existing: collections, incoming: imported }))) ?? collections;
    await saveCollections(merged);
    setEditingCollection(null);
  }

  async function handleExportAll() {
    const json = await exportCollectionsJson(collections);
    await navigator.clipboard.writeText(json);
  }

  return {
    collections,
    viewAllFolder, setViewAllFolder,
    editingCollection, setEditingCollection,
    collectionsScrollRef,
    openFolder,
    saveCollections,
    handleSaveCollection,
    handleDeleteCollection,
    handleImportJson,
    handleExportAll,
  };
}
