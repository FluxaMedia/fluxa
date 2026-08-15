const companionUrl = import.meta.env.VITE_FLUXA_COMPANION_URL || 'http://127.0.0.1:19876';

function isLocalCompanionUrl(sourceUrl: string): boolean {
  try {
    const host = new URL(sourceUrl).hostname;
    return host === '127.0.0.1' || host === 'localhost' || host === '[::1]';
  } catch {
    return false;
  }
}

export function transcodeUrl(sourceUrl: string, startSeconds?: number, headers?: Record<string, string>): string {
  const input = isLocalCompanionUrl(sourceUrl) ? sourceUrl : proxyUrl(sourceUrl, headers ?? {});
  const url = `${companionUrl}/transcode?url=${encodeURIComponent(input)}`;
  return startSeconds && startSeconds > 0 ? `${url}&start=${Math.floor(startSeconds)}` : url;
}

export function proxyUrl(sourceUrl: string, headers: Record<string, string>): string {
  return `${companionUrl}/proxy?url=${encodeURIComponent(sourceUrl)}&h=${encodeURIComponent(JSON.stringify(headers))}`;
}

function extensionOf(sourceUrl: string): string {
  try {
    return new URL(sourceUrl).pathname.split('.').pop()?.toLowerCase() ?? '';
  } catch {
    return sourceUrl.split('?')[0].split('.').pop()?.toLowerCase() ?? '';
  }
}

function codecString(videoCodec: string | null | undefined, audioCodec: string | null | undefined): string {
  const video = videoCodec?.toLowerCase().includes('h264') || videoCodec?.toLowerCase().includes('avc')
    ? 'avc1.42E01E'
    : videoCodec?.toLowerCase().includes('vp9')
      ? 'vp09.00.10.08'
      : videoCodec?.toLowerCase().includes('vp8')
        ? 'vp8'
        : videoCodec?.toLowerCase().includes('av1')
          ? 'av01.0.05M.08'
          : videoCodec ?? '';
  const audio = audioCodec?.toLowerCase().includes('aac') || audioCodec?.toLowerCase().includes('mp4a')
    ? 'mp4a.40.2'
    : audioCodec?.toLowerCase().includes('opus')
      ? 'opus'
      : audioCodec?.toLowerCase().includes('vorbis')
        ? 'vorbis'
        : audioCodec?.toLowerCase().includes('mp3')
          ? 'mp3'
          : audioCodec ?? '';
  return [video, audio].filter(Boolean).join(', ');
}

export function canDirectPlay(
  sourceUrl: string,
  videoCodec: string | null | undefined,
  audioCodec: string | null | undefined,
): boolean {
  const extension = extensionOf(sourceUrl);
  if (['mkv', 'avi', 'wmv', 'flv', 'm2ts', 'ts'].includes(extension)) return false;
  const video = document.createElement('video');
  const codecs = codecString(videoCodec, audioCodec);
  const mime = extension === 'webm'
    ? `video/webm${codecs ? `; codecs="${codecs}"` : ''}`
    : extension === 'm3u8'
      ? 'application/vnd.apple.mpegurl'
      : extension === 'ogg' || extension === 'ogv'
        ? `video/ogg${codecs ? `; codecs="${codecs}"` : ''}`
        : `video/mp4${codecs ? `; codecs="${codecs}"` : ''}`;
  const browserSupport = video.canPlayType(mime);
  if (browserSupport === '') return false;
  return true;
}

export async function probeStream(sourceUrl: string, headers?: Record<string, string>): Promise<{ videoCodec: string | null; audioCodec: string | null; duration: number | null } | null> {
  try {
    const input = isLocalCompanionUrl(sourceUrl) ? sourceUrl : proxyUrl(sourceUrl, headers ?? {});
    const response = await fetch(`${companionUrl}/probe?url=${encodeURIComponent(input)}`);
    return response.ok ? await response.json() : null;
  } catch {
    return null;
  }
}
