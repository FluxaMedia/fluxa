export type ExternalLinkTarget = {
  id: string;
  label: string;
  href: string;
  callback: boolean;
};

export type ExternalLinkOptions = {
  errorUrl?: string;
  position?: number;
  filename?: string;
  subtitleUrl?: string;
};

function isIos(): boolean {
  const ua = navigator.userAgent;
  if (/iPad|iPhone|iPod/.test(ua)) return true;
  return /Macintosh/.test(ua) && navigator.maxTouchPoints > 1;
}

function isAndroid(): boolean {
  return /Android/.test(navigator.userAgent);
}

function isMac(): boolean {
  return /Macintosh|Mac OS X/.test(navigator.userAgent) && navigator.maxTouchPoints <= 1;
}

function intentUrl(url: string, pkg?: string): string {
  const withoutScheme = url.replace(/^https?:\/\//, '');
  const scheme = url.startsWith('https') ? 'https' : 'http';
  const parts = [`intent://${withoutScheme}#Intent`, `scheme=${scheme}`, 'type=video/*'];
  if (pkg) parts.push(`package=${pkg}`);
  parts.push('end');
  return parts.join(';');
}

function infusePlayUrl(url: string, successUrl: string, options: ExternalLinkOptions): string {
  const params = [`url=${encodeURIComponent(url)}`];
  if (Number.isFinite(options.position) && (options.position ?? 0) > 0) {
    params.push(`position=${Math.floor(options.position ?? 0)}`);
  }
  if (options.filename?.trim()) params.push(`filename=${encodeURIComponent(options.filename.trim())}`);
  if (options.subtitleUrl?.trim()) params.push(`sub=${encodeURIComponent(options.subtitleUrl.trim())}`);
  params.push(`x-success=${encodeURIComponent(successUrl)}`);
  if (options.errorUrl?.trim()) params.push(`x-error=${encodeURIComponent(options.errorUrl.trim())}`);
  return `infuse://x-callback-url/play?${params.join('&')}`;
}

export function externalLinkTargets(url: string, successUrl: string, options: ExternalLinkOptions = {}): ExternalLinkTarget[] {
  const encoded = encodeURIComponent(url);
  const success = encodeURIComponent(successUrl);

  if (isIos()) {
    return [
      { id: 'infuse', label: 'Infuse', href: infusePlayUrl(url, successUrl, options), callback: true },
      { id: 'vlc', label: 'VLC', href: `vlc-x-callback://x-callback-url/stream?url=${encoded}&x-success=${success}`, callback: true },
      { id: 'outplayer', label: 'Outplayer', href: `outplayer://${url}`, callback: false },
      { id: 'nplayer', label: 'nPlayer', href: `nplayer-${url}`, callback: false },
    ];
  }

  if (isAndroid()) {
    return [
      { id: 'vlc', label: 'VLC', href: intentUrl(url, 'org.videolan.vlc'), callback: false },
      { id: 'mx', label: 'MX Player', href: intentUrl(url, 'com.mxtech.videoplayer.ad'), callback: false },
      { id: 'just', label: 'Just Player', href: intentUrl(url, 'com.brouken.player'), callback: false },
      { id: 'system', label: 'System', href: intentUrl(url), callback: false },
    ];
  }

  if (isMac()) {
    return [
      { id: 'infuse', label: 'Infuse', href: infusePlayUrl(url, successUrl, options), callback: true },
      { id: 'iina', label: 'IINA', href: `iina://weblink?url=${encoded}`, callback: false },
    ];
  }

  return [];
}

export function externalHandoffSupported(): boolean {
  return isIos() || isAndroid() || isMac();
}
