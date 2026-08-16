export const IS_WEBOS = import.meta.env.VITE_FLUXA_TARGET === 'webos'
  || (typeof navigator !== 'undefined' && /Web0S|webOS/i.test(navigator.userAgent));

export type HdrKind = 'none' | 'hdr10' | 'hlg' | 'dolbyVision';

export interface WebOSCapabilities {
  hdr10: boolean;
  dolbyVision: boolean;
  dolbyAtmos: boolean;
  uhd: boolean;
  modelName: string;
  sdkVersion: string;
}

export function lunaCall<T extends WebOSServiceResponse = WebOSServiceResponse>(
  uri: string,
  method: string,
  params: Record<string, unknown> = {},
): Promise<T> {
  return new Promise((resolve, reject) => {
    if (typeof webOS === 'undefined' || !webOS.service) {
      reject(new Error('webOS service bus is not available'));
      return;
    }
    webOS.service.request(uri, {
      method,
      parameters: params,
      onSuccess: (response) => resolve(response as T),
      onFailure: (response) => reject(new Error(String(response.errorText ?? JSON.stringify(response)))),
    });
  });
}

export function lunaSubscribe(
  uri: string,
  method: string,
  params: Record<string, unknown>,
  onEvent: (response: WebOSServiceResponse) => void,
): () => void {
  if (typeof webOS === 'undefined' || !webOS.service) return () => {};
  const request = webOS.service.request(uri, {
    method,
    parameters: { ...params, subscribe: true },
    subscribe: true,
    onSuccess: onEvent,
    onFailure: () => {},
  });
  return () => request.cancel();
}

let capabilitiesPromise: Promise<WebOSCapabilities | null> | null = null;

export function webosCapabilities(): Promise<WebOSCapabilities | null> {
  if (!IS_WEBOS) return Promise.resolve(null);
  capabilitiesPromise ??= new Promise<WebOSCapabilities | null>((resolve) => {
    if (typeof webOS === 'undefined' || typeof webOS.deviceInfo !== 'function') {
      resolve(null);
      return;
    }
    const timer = setTimeout(() => resolve(null), 3000);
    try {
      webOS.deviceInfo((info) => {
        clearTimeout(timer);
        resolve({
          hdr10: info.hdr10 === true,
          dolbyVision: info.dolbyVision === true,
          dolbyAtmos: info.dolbyAtmos === true,
          uhd: info.uhd === true || info.uhd8K === true,
          modelName: String(info.modelNameAscii ?? info.modelName ?? ''),
          sdkVersion: String(info.sdkVersion ?? ''),
        });
      });
    } catch {
      clearTimeout(timer);
      resolve(null);
    }
  });
  return capabilitiesPromise;
}

const CONTAINER_BY_EXTENSION: Record<string, string> = {
  mkv: 'MKV',
  webm: 'WEBM',
  mp4: 'MP4',
  m4v: 'MP4',
  mov: 'MP4',
  avi: 'AVI',
  ts: 'MPEG2-TS',
  m2ts: 'MPEG2-TS',
  mts: 'MPEG2-TS',
  m3u8: 'HLS',
  mpd: 'DASH',
};

export function containerFor(url: string): string {
  const path = url.split('?')[0].split('#')[0].toLowerCase();
  const extension = path.slice(path.lastIndexOf('.') + 1);
  return CONTAINER_BY_EXTENSION[extension] ?? 'MKV';
}

export function hdrKindFrom(videoCodec: string | null | undefined, transferHint?: string | null): HdrKind {
  const codec = `${videoCodec ?? ''}`.toLowerCase();
  const transfer = `${transferHint ?? ''}`.toLowerCase();
  if (codec.includes('dvhe') || codec.includes('dvh1') || codec.includes('dolbyvision') || transfer.includes('dolbyvision')) return 'dolbyVision';
  if (transfer.includes('hlg') || transfer.includes('arib-std-b67')) return 'hlg';
  if (transfer.includes('pq') || transfer.includes('smpte2084') || transfer.includes('hdr10')) return 'hdr10';
  return 'none';
}

export interface MediaOptionHints {
  hdr?: HdrKind;
  multiChannelAudio?: boolean;
  bitstreamAudio?: boolean;
  adaptive?: boolean;
}

export async function applyWebOSMediaOption(
  video: HTMLVideoElement,
  url: string,
  hints: MediaOptionHints = {},
): Promise<void> {
  if (!IS_WEBOS) return;
  const capabilities = await webosCapabilities();
  const container = containerFor(url);
  const requested = hints.hdr ?? 'none';
  const supported = requested === 'dolbyVision'
    ? capabilities?.dolbyVision !== false
    : requested === 'none' || capabilities?.hdr10 !== false;
  const hdr = supported ? requested : 'none';

  const option: Record<string, unknown> = {
    mediaFormat: { type: container },
    transmission: { contentsType: 'VOD' },
    hdrInfo: hdr === 'none'
      ? { isHDRContent: false }
      : { isHDRContent: true, hdrType: hdr },
    audioInfo: {
      isMultiChannel: hints.multiChannelAudio ?? true,
      isBitstreamRequired: hints.bitstreamAudio ?? Boolean(capabilities?.dolbyAtmos),
    },
  };
  if (hints.adaptive || container === 'HLS' || container === 'DASH') {
    option.adaptiveStreaming = { audioOnly: false, seamlessPlay: true };
  }

  const payload = JSON.stringify({ mediaTransportType: 'URI', option });
  try {
    if (typeof video.setMediaOption === 'function') video.setMediaOption(payload);
    else video.mediaOption = payload;
  } catch (error) {
    console.warn('[fluxa:webos:mediaOption]', error);
  }
}

export const WEBOS_PROXY_SERVICE = 'luna://com.fluxa.app.proxy';

let proxyReady: Promise<boolean> | null = null;

export function ensureWebosProxy(): Promise<boolean> {
  if (!IS_WEBOS) return Promise.resolve(false);
  proxyReady ??= lunaCall<WebOSServiceResponse & { port?: number }>(WEBOS_PROXY_SERVICE, 'start')
    .then((response) => response.returnValue === true)
    .catch((error) => {
      console.warn('[fluxa:webos:proxy]', error);
      return false;
    });
  return proxyReady;
}

export function webosPlatformBack(): boolean {
  if (typeof webOS !== 'undefined' && typeof webOS.platformBack === 'function') {
    webOS.platformBack();
    return true;
  }
  return false;
}

export function webosExitApp(): void {
  try {
    if (typeof PalmSystem !== 'undefined') PalmSystem.deactivate();
    window.close();
  } catch {
    /* ignore */
  }
}

export function webosStageReady(): void {
  try {
    if (typeof PalmSystem !== 'undefined') PalmSystem.stageReady();
  } catch {
    /* ignore */
  }
}
