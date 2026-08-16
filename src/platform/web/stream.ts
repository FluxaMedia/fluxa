import { ensureWebosProxy, IS_WEBOS } from '../webos';
const companionUrl = import.meta.env.VITE_FLUXA_COMPANION_URL || 'http://127.0.0.1:19876';

function isLocalCompanionUrl(sourceUrl: string): boolean {
  try {
    const host = new URL(sourceUrl).hostname;
    return host === '127.0.0.1' || host === 'localhost' || host === '[::1]';
  } catch {
    return false;
  }
}

export function transcodeUrl(sourceUrl: string, startSeconds?: number, headers?: Record<string, string>, target?: BrowserTranscodeTarget): string {
  const input = isLocalCompanionUrl(sourceUrl) ? sourceUrl : proxyUrl(sourceUrl, headers ?? {});
  const params = new URLSearchParams({ url: input });
  if (startSeconds && startSeconds > 0) params.set('start', String(Math.floor(startSeconds)));
  if (target) {
    params.set('videoCodec', target.videoCodec);
    params.set('audioCodec', target.audioCodec);
    params.set('container', target.container);
  }
  return `${companionUrl}/transcode?${params.toString()}`;
}

export interface BrowserTranscodeTarget {
  videoCodec: string;
  audioCodec: string;
  container: 'mp4' | 'webm';
}

function browserSupportsVideo(codec: string | null | undefined): boolean {
  const normalized = codec?.toLowerCase() ?? '';
  const mime = normalized.includes('av1')
    ? 'video/webm; codecs="av01.0.08M.08"'
    : normalized.includes('vp9')
      ? 'video/webm; codecs="vp09.00.10.08"'
      : normalized.includes('vp8')
        ? 'video/webm; codecs="vp8"'
        : normalized.includes('hevc') || normalized.includes('h265')
          ? 'video/mp4; codecs="hvc1.2.4.L153.B0"'
          : normalized.includes('h264') || normalized.includes('avc')
            ? 'video/mp4; codecs="avc1.640028"'
            : '';
  return Boolean(mime && document.createElement('video').canPlayType(mime));
}

function browserSupportsAudio(codec: string | null | undefined): boolean {
  const normalized = codec?.toLowerCase() ?? '';
  const mime = normalized.includes('opus')
    ? 'audio/webm; codecs="opus"'
    : normalized.includes('vorbis')
      ? 'audio/webm; codecs="vorbis"'
      : normalized.includes('aac') || normalized.includes('mp4a')
        ? 'audio/mp4; codecs="mp4a.40.2"'
        : normalized.includes('mp3')
          ? 'audio/mpeg'
          : '';
  return Boolean(mime && document.createElement('audio').canPlayType(mime));
}

export function chooseBrowserTranscodeTarget(videoCodec: string | null | undefined, audioCodec: string | null | undefined): BrowserTranscodeTarget {
  const normalizedVideo = videoCodec?.toLowerCase() ?? '';
  const normalizedAudio = audioCodec?.toLowerCase() ?? '';
  const targetVideo = browserSupportsVideo(normalizedVideo) ? normalizedVideo : 'h264';
  const targetAudio = browserSupportsAudio(normalizedAudio) ? normalizedAudio : 'aac';
  const webmVideo = ['vp8', 'vp9', 'av1'].some((codec) => targetVideo.includes(codec));
  const webmAudio = !targetAudio || targetAudio.includes('opus') || targetAudio.includes('vorbis');
  const webm = webmVideo && webmAudio;
  return { videoCodec: targetVideo, audioCodec: targetAudio, container: webm ? 'webm' : 'mp4' };
}

export function proxyUrl(sourceUrl: string, headers: Record<string, string>): string {
  return `${companionUrl}/proxy?url=${encodeURIComponent(sourceUrl)}&h=${encodeURIComponent(JSON.stringify(headers))}`;
}

export async function fetchThroughProxyIfNeeded(sourceUrl: string, headers: Record<string, string> = {}): Promise<Response> {
  if (isLocalCompanionUrl(sourceUrl)) return fetch(sourceUrl);
  if (IS_WEBOS) await ensureWebosProxy();
  const attempts = IS_WEBOS
    ? [proxyUrl(sourceUrl, headers), sourceUrl]
    : [sourceUrl, proxyUrl(sourceUrl, headers)];
  let lastError: unknown;
  for (const candidate of attempts) {
    try {
      const response = await fetch(candidate);
      if (response.ok) return response;
      lastError = new Error(`request failed: ${response.status}`);
    } catch (error) {
      lastError = error;
    }
  }
  throw lastError instanceof Error ? lastError : new Error(String(lastError));
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
