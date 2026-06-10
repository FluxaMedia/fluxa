import type { CatalogSource, UserCollection, UserCollectionFolder } from './types';

function cleanedUrl(raw: string | null | undefined): string | null {
  const s = raw?.trim();
  return s && s.length > 0 ? s : null;
}

function cleanedArtworkUrl(raw: string | null | undefined): string | null {
  const s = cleanedUrl(raw)?.replace(/^['"]|['"]$/g, '').trim();
  if (!s) return null;
  const withScheme = s.startsWith('//') ? `https:${s}` : s;
  const githubBlob = /^https:\/\/github\.com\/([^/]+)\/([^/]+)\/blob\/([^/]+)\/(.+)$/;
  const m = withScheme.match(githubBlob);
  const normalized = m
    ? `https://raw.githubusercontent.com/${m[1]}/${m[2]}/${m[3]}/${m[4]}`
    : withScheme;
  return normalized.replace(/ /g, '%20');
}

function normalizeShape(value: string | null | undefined): string {
  const v = value?.trim().toUpperCase();
  if (v === 'LANDSCAPE' || v === 'WIDE') return 'wide';
  if (v === 'SQUARE') return 'square';
  return 'poster';
}

function exportShape(value: string | null | undefined): string {
  const v = value?.trim().toLowerCase();
  if (v === 'wide' || v === 'landscape') return 'LANDSCAPE';
  if (v === 'square') return 'SQUARE';
  return 'POSTER';
}

export function effectiveFolderImageUrl(folder: UserCollectionFolder): string | null {
  return cleanedArtworkUrl(folder.coverImageUrl) ?? cleanedArtworkUrl(folder.imageUrl);
}

export function effectiveFolderShape(folder: UserCollectionFolder): string {
  return folder.shape ?? normalizeShape(null);
}

export function effectiveCatalogId(folder: UserCollectionFolder): string | null {
  return folder.catalogSources?.[0]?.catalogId ?? folder.catalogId ?? null;
}

export function effectiveCatalogType(folder: UserCollectionFolder): string | null {
  return folder.catalogSources?.[0]?.type ?? null;
}

function str(v: unknown): string | null {
  return typeof v === 'string' ? v : null;
}

function bool(v: unknown, fallback: boolean): boolean {
  return typeof v === 'boolean' ? v : fallback;
}

// Mirrors Gson's @SerializedName alternate lookup
function pickStr(obj: Record<string, unknown>, ...keys: string[]): string | null {
  for (const k of keys) {
    const v = obj[k];
    if (typeof v === 'string') return v;
  }
  return null;
}

function parseFolderCoverImageUrl(folder: Record<string, unknown>): string | null {
  // mirrors: @SerializedName("coverImageUrl", alternate=["coverUrl","coverImage","cover","poster","thumbnail","thumb"])
  const coverImageUrl = cleanedArtworkUrl(
    pickStr(folder, 'coverImageUrl', 'coverUrl', 'coverImage', 'cover', 'poster', 'thumbnail', 'thumb')
  );
  // mirrors: @SerializedName("imageUrl", alternate=["image","image_url","posterUrl","poster_url"])
  const imageUrl = cleanedArtworkUrl(
    pickStr(folder, 'imageUrl', 'image', 'image_url', 'posterUrl', 'poster_url')
  );
  // mirrors normalizedCoverImageUrl(): coverImageUrl.cleanedArtworkUrl() ?: imageUrl.cleanedArtworkUrl()
  return coverImageUrl ?? imageUrl;
}

function parseFolderHeroBackdropUrl(folder: Record<string, unknown>): string | null {
  // mirrors: @SerializedName("heroBackdropUrl", alternate=["background","backdrop","backgroundUrl","backdropUrl"])
  return cleanedUrl(
    pickStr(folder, 'heroBackdropUrl', 'background', 'backdrop', 'backgroundUrl', 'backdropUrl')
  );
}

export function importCollectionsJson(rawJson: string): UserCollection[] {
  let parsed: unknown;
  try { parsed = JSON.parse(rawJson); } catch { return []; }

  const arr: unknown[] = Array.isArray(parsed) ? parsed : [parsed];

  return arr.flatMap((item, i) => {
    if (!item || typeof item !== 'object') return [];
    const col = item as Record<string, unknown>;

    const title = str(col.title)?.trim() ?? '';
    if (!title) return [];

    const id = (str(col.id)?.trim() || null) ?? `imported_${Date.now()}_${i}`;

    const rawFolders = Array.isArray(col.folders) ? col.folders : [];

    const folders: UserCollectionFolder[] = rawFolders.flatMap((f: unknown, fi: number) => {
      if (!f || typeof f !== 'object') return [];
      const folder = f as Record<string, unknown>;

      const folderTitle = str(folder.title)?.trim() ?? '';
      if (!folderTitle) return [];

      const fid = (str(folder.id)?.trim() || null) ?? `folder_${Date.now()}_${fi}`;

      // catalogSources — mirrors: folder.catalogSources.orEmpty().ifEmpty { fallback from catalogId }
      const rawSources = Array.isArray(folder.catalogSources) ? folder.catalogSources : [];
      let sources: CatalogSource[] = rawSources
        .filter((s) => s && typeof s === 'object')
        .map((s) => {
          const o = s as Record<string, unknown>;
          return { catalogId: str(o.catalogId) ?? '', type: str(o.type) ?? 'movie', addonId: str(o.addonId) ?? undefined };
        })
        .filter((s) => s.catalogId.length > 0);

      if (sources.length === 0) {
        const fallbackId = str(folder.catalogId)?.trim();
        if (fallbackId) sources = [{ catalogId: fallbackId, type: 'movie' }];
      }

      const primarySource = sources[0] ?? null;
      const coverImageUrl = parseFolderCoverImageUrl(folder);
      const heroBackdropUrl = parseFolderHeroBackdropUrl(folder);

      return [{
        id: fid,
        title: folderTitle,
        // catalogTitle falls back to folderTitle if missing (matches Kotlin: folder.catalogTitle ?: folderTitle)
        catalogTitle: str(folder.catalogTitle) ?? folderTitle,
        catalogId: primarySource?.catalogId ?? str(folder.catalogId) ?? undefined,
        genre: str(folder.genre) ?? undefined,
        shape: normalizeShape(str(folder.tileShape) ?? str(folder.shape)),
        hideTitle: bool(folder.hideTitle, false),
        focusGifEnabled: bool(folder.focusGifEnabled, true),
        catalogSources: sources.length > 0 ? sources : undefined,
        coverEmoji: str(folder.coverEmoji) ?? undefined,
        imageUrl: coverImageUrl ?? undefined,
        coverImageUrl: coverImageUrl ?? undefined,
        focusGifUrl: cleanedUrl(str(folder.focusGifUrl)) ?? undefined,
        titleLogoUrl: cleanedUrl(str(folder.titleLogoUrl)) ?? undefined,
        heroBackdropUrl: heroBackdropUrl ?? undefined,
      }];
    });

    return [{
      id,
      title,
      // collection imageUrl = first folder's cover image (matches Kotlin)
      imageUrl: parseFolderCoverImageUrl((Array.isArray(col.folders) ? col.folders : [])
        .find((f) => f && typeof f === 'object') as Record<string, unknown> ?? {}) ?? undefined,
      showOnHome: bool(col.showOnHome, true),   // default true on desktop (Android defaults false)
      itemIds: [],
      folders,
      showAllTab: bool(col.showAllTab, true),
      viewMode: str(col.viewMode) ?? 'FOLLOW_LAYOUT',
      pinToTop: bool(col.pinToTop, false),
      focusGlowEnabled: bool(col.focusGlowEnabled, true),
    }];
  });
}

export function exportCollectionsJson(collections: UserCollection[]): string {
  const data = collections.map((col) => ({
    id: col.id,
    title: col.title,
    showAllTab: col.showAllTab ?? true,
    viewMode: col.viewMode ?? 'FOLLOW_LAYOUT',
    showOnHome: col.showOnHome ?? true,
    pinToTop: col.pinToTop ?? false,
    focusGlowEnabled: col.focusGlowEnabled ?? true,
    folders: (col.folders ?? []).map((folder) => ({
      id: folder.id,
      title: folder.title,
      tileShape: exportShape(folder.shape),
      hideTitle: folder.hideTitle ?? false,
      focusGifEnabled: folder.focusGifEnabled ?? true,
      catalogSources: folder.catalogSources?.length
        ? folder.catalogSources
        : folder.catalogId
          ? [{ catalogId: folder.catalogId, type: 'movie' }]
          : [],
      coverEmoji: folder.coverEmoji,
      coverImageUrl: folder.coverImageUrl ?? folder.imageUrl,
      focusGifUrl: folder.focusGifUrl,
      titleLogoUrl: folder.titleLogoUrl,
      heroBackdropUrl: folder.heroBackdropUrl,
    })),
  }));
  return JSON.stringify(data, null, 2);
}
