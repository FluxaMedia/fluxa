import { coreInvoke } from './engineCoreClient';

export async function coreTmdbContentType(
  contentType: string,
): Promise<string> {
  return (await coreInvoke<string>(
    "tmdbContentType",
    JSON.stringify({ contentType }),
  )) ?? contentType;
}

export async function coreTmdbLanguage(language: string): Promise<string> {
  return (await coreInvoke<string>(
    "tmdbLanguage",
    JSON.stringify({ language }),
  )) ?? language;
}

export async function coreTmdbImageUrl(
  path: string | null,
  size: string,
): Promise<string | null> {
  return coreInvoke("tmdbImageUrl", JSON.stringify({ path, size }));
}

export async function coreTmdbMetaToMeta(
  itemJson: string,
  requestedType: string,
  language: string,
): Promise<unknown | null> {
  return coreInvoke(
    "tmdbMetaToMeta",
    JSON.stringify({ itemJson, requestedType, language }),
  );
}

export async function coreTmdbVideoToTrailer(
  videoJson: string,
): Promise<unknown | null> {
  return coreInvoke("tmdbVideoToTrailer", videoJson);
}

export async function coreTmdbBulkMetas(
  itemsJson: string,
  requestedType: string,
  language: string,
): Promise<unknown[] | null> {
  return coreInvoke(
    "tmdbBulkMetas",
    JSON.stringify({ itemsJson, requestedType, language }),
  );
}

export async function coreTmdbBulkVideosToTrailers(
  itemsJson: string,
): Promise<unknown[] | null> {
  return coreInvoke("tmdbBulkVideosToTrailers", itemsJson);
}

export async function coreTmdbResolveIdHint(
  contentId: string,
): Promise<[string, boolean]> {
  return (await coreInvoke<[string, boolean]>(
    "tmdbResolveIdHint",
    JSON.stringify({ contentId }),
  )) ?? ["", false];
}

export async function coreTmdbBuiltinManifest(): Promise<string> {
  return (await coreInvoke<string>("tmdbBuiltinManifest", "{}")) ?? "{}";
}

export async function coreTmdbBuiltinCatalogUrl(
  contentType: string,
  extra: Record<string, unknown>,
  apiKey: string,
  language: string,
): Promise<string> {
  return (await coreInvoke<string>(
    "tmdbBuiltinCatalogUrl",
    JSON.stringify({ contentType, extra, apiKey, language }),
  )) ?? "";
}

export async function coreTmdbFullMetaToMeta(
  detailsJson: string,
  creditsJson: string,
  imagesJson: string,
  externalIdsJson: string,
  requestedType: string,
  language: string,
): Promise<unknown | null> {
  return coreInvoke(
    "tmdbFullMetaToMeta",
    JSON.stringify({
      detailsJson,
      creditsJson,
      imagesJson,
      externalIdsJson,
      requestedType,
      language,
    }),
  );
}

export async function coreTmdbPickLogo(
  imagesJson: string,
  language: string,
): Promise<{ logo: string | null } | null> {
  return coreInvoke("tmdbPickLogo", JSON.stringify({ imagesJson, language }));
}

export async function coreTmdbEpisodesToVideos(
  seasonJson: string,
  seriesId: string,
): Promise<unknown[] | null> {
  return coreInvoke(
    "tmdbEpisodesToVideos",
    JSON.stringify({ seasonJson, seriesId }),
  );
}

export async function coreTmdbMergeEnrichment(
  baseJson: string,
  tmdbJson: string,
  flagsJson: string,
): Promise<unknown | null> {
  return coreInvoke(
    "tmdbMergeEnrichment",
    JSON.stringify({ baseJson, tmdbJson, flagsJson }),
  );
}
