let cachedApiKey = '';

export function setRpdbApiKey(key: string): void {
  cachedApiKey = key.trim();
}

export async function validateRpdbApiKey(key: string): Promise<boolean> {
  const trimmed = key.trim();
  if (!trimmed) return false;
  try {
    const res = await fetch(`https://api.ratingposterdb.com/${trimmed}/isValid`);
    if (!res.ok) return false;
    const data = (await res.json()) as { valid?: boolean };
    return data.valid === true;
  } catch {
    return false;
  }
}

export function rpdbPosterUrl(meta: { id?: string }): string | undefined {
  if (!cachedApiKey || !meta.id) return undefined;
  const imdbMatch = meta.id.match(/^tt\d+/i);
  if (imdbMatch) {
    return `https://api.ratingposterdb.com/${cachedApiKey}/imdb/poster-default/${imdbMatch[0]}.jpg?fallback=true`;
  }
  const tmdbMatch = meta.id.match(/^tmdb:(\d+)/i);
  if (tmdbMatch) {
    return `https://api.ratingposterdb.com/${cachedApiKey}/tmdb/poster-default/${tmdbMatch[1]}.jpg?fallback=true`;
  }
  return undefined;
}
