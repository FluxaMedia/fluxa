import type { UserCollection, UserCollectionFolder } from './types';
import { coreExportCollections, coreImportCollections } from './engine';

function cleanedUrl(raw: string | null | undefined): string | null {
  const s = raw?.trim();
  return s && s.length > 0 ? s : null;
}

function cleanedArtworkUrl(raw: string | null | undefined): string | null {
  const s = cleanedUrl(raw)
    ?.replace(/^['"]|['"]$/g, '')
    .trim();
  if (!s) return null;
  const withScheme = s.startsWith('//') ? `https:${s}` : s;
  const githubBlob = /^https:\/\/github\.com\/([^/]+)\/([^/]+)\/blob\/([^/]+)\/(.+)$/;
  const m = withScheme.match(githubBlob);
  const normalized = m ? `https://raw.githubusercontent.com/${m[1]}/${m[2]}/${m[3]}/${m[4]}` : withScheme;
  return normalized.replace(/ /g, '%20');
}

export function effectiveFolderImageUrl(folder: UserCollectionFolder): string | null {
  return cleanedArtworkUrl(folder.coverImageUrl) ?? cleanedArtworkUrl(folder.imageUrl);
}

export function effectiveFolderShape(folder: UserCollectionFolder): string {
  const raw = (folder.shape ?? 'poster').toLowerCase();
  return raw === 'landscape' ? 'wide' : raw;
}

export function effectiveCatalogId(folder: UserCollectionFolder): string | null {
  return (
    folder.sources?.find((source) => source.provider === 'addon')?.catalogId ??
    folder.catalogSources?.[0]?.catalogId ??
    folder.catalogId ??
    null
  );
}

export function effectiveCatalogType(folder: UserCollectionFolder): string | null {
  return folder.sources?.find((source) => source.provider === 'addon')?.type ?? folder.catalogSources?.[0]?.type ?? null;
}

export async function importCollectionsJson(rawJson: string): Promise<UserCollection[]> {
  return ((await coreImportCollections(rawJson)) ?? []) as UserCollection[];
}

export async function exportCollectionsJson(collections: UserCollection[]): Promise<string> {
  return JSON.stringify((await coreExportCollections(JSON.stringify(collections))) ?? [], null, 2);
}
