export function youtubeVideoId(url: string): string | null {
  try {
    const parsed = new URL(url);
    if (parsed.hostname.includes('youtu.be')) return parsed.pathname.split('/').filter(Boolean)[0] ?? null;
    if (parsed.hostname.includes('youtube.com'))
      return parsed.searchParams.get('v') || parsed.pathname.split('/').filter(Boolean).pop() || null;
  } catch {
    return null;
  }
  return null;
}
