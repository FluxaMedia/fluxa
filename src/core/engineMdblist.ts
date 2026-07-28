import { coreInvoke } from './engine';

export async function coreMdblistMediaInfoUrl(
  provider: string,
  mediaType: string,
  mediaId: string,
  appendToResponse?: string,
): Promise<string | null> {
  return coreInvoke(
    'mdblistMediaInfoUrl',
    JSON.stringify({ provider, mediaType, mediaId, appendToResponse }),
  );
}

export async function coreMdblistMediaRatingsFromResponse(
  responseJson: string,
): Promise<Record<string, number> | null> {
  return coreInvoke('mdblistMediaRatingsFromResponse', responseJson);
}
