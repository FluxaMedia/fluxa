import { platformInvoke as invoke } from '../platform/invoke';

export interface CastDevice {
  id: string;
  name: string;
  kind: 'dlna' | 'chromecast' | 'airplay' | 'roku' | 'fcast';
  host?: string;
  port?: number;
}

let activeKind: CastDevice['kind'] | null = null;

export async function discoverCastDevices(): Promise<CastDevice[]> {
  const [dlna, chromecast, airplay, roku, fcast] = await Promise.all([
    invoke<{ id: string; name: string }[]>('cast_discover_devices').catch(() => []),
    invoke<{ id: string; name: string; host: string; port: number }[]>('chromecast_discover_devices').catch(() => []),
    invoke<{ id: string; name: string; host: string; port: number }[]>('airplay_discover_devices').catch(() => []),
    invoke<{ id: string; name: string; host: string }[]>('roku_discover_devices').catch(() => []),
    invoke<{ id: string; name: string; host: string; port: number }[]>('fcast_discover_devices').catch(() => []),
  ]);
  return [
    ...dlna.map((d) => ({ ...d, kind: 'dlna' as const })),
    ...chromecast.map((d) => ({ ...d, kind: 'chromecast' as const })),
    ...airplay.map((d) => ({ ...d, kind: 'airplay' as const })),
    ...roku.map((d) => ({ ...d, kind: 'roku' as const })),
    ...fcast.map((d) => ({ ...d, kind: 'fcast' as const })),
  ];
}

export async function resolveCastMediaUrl(streamUrl: string): Promise<string> {
  return invoke<string>('cast_resolve_media_url', { streamUrl }).catch(() => streamUrl);
}

export async function proxyMediaUrl(url: string, headers: Record<string, string> | undefined): Promise<string> {
  if (!headers || Object.keys(headers).length === 0) return url;
  try {
    return await invoke<string>('cast_proxy_serve', { url, headers });
  } catch {
    return url;
  }
}

export async function startCasting(
  device: CastDevice,
  mediaUrl: string,
  title: string,
  subtitleUrl?: string,
  positionSecs?: number,
): Promise<void> {
  if (device.kind === 'fcast') {
    await invoke('fcast_connect', { host: device.host, port: device.port, mediaUrl, resumePositionSecs: positionSecs ?? 0 });
  } else if (device.kind === 'dlna') {
    await invoke('cast_set_media', { deviceId: device.id, mediaUrl, title, subtitleUrl: subtitleUrl ?? null });
  } else if (device.kind === 'chromecast') {
    await invoke('chromecast_connect', { host: device.host, port: device.port, mediaUrl, title, subtitleUrl: subtitleUrl ?? null });
  } else if (device.kind === 'airplay') {
    await invoke('airplay_set_media', { host: device.host, port: device.port, mediaUrl });
  } else {
    await invoke('roku_set_media', { host: device.host, mediaUrl, subtitleUrl: subtitleUrl ?? null });
  }
  activeKind = device.kind;
}

export function castPlay(): void {
  if (activeKind === 'roku') {
    void invoke('roku_play_pause').catch(() => undefined);
    return;
  }
  if (activeKind === 'fcast') {
    void invoke('fcast_play').catch(() => undefined);
    return;
  }
  const command = activeKind === 'chromecast' ? 'chromecast_play' : activeKind === 'airplay' ? 'airplay_play' : 'cast_play';
  void invoke(command).catch(() => undefined);
}

export function castPause(): void {
  if (activeKind === 'roku') {
    void invoke('roku_play_pause').catch(() => undefined);
    return;
  }
  if (activeKind === 'fcast') {
    void invoke('fcast_pause').catch(() => undefined);
    return;
  }
  const command = activeKind === 'chromecast' ? 'chromecast_pause' : activeKind === 'airplay' ? 'airplay_pause' : 'cast_pause';
  void invoke(command).catch(() => undefined);
}

export function castSeek(positionSecs: number): void {
  if (activeKind === 'roku') return;
  if (activeKind === 'fcast') {
    void invoke('fcast_seek', { positionSecs }).catch(() => undefined);
    return;
  }
  const command = activeKind === 'chromecast' ? 'chromecast_seek' : activeKind === 'airplay' ? 'airplay_seek' : 'cast_seek';
  void invoke(command, { positionSecs }).catch(() => undefined);
}

export function castSetVolume(level: number): void {
  if (activeKind === 'roku') return;
  if (activeKind === 'fcast') {
    void invoke('fcast_set_volume', { level }).catch(() => undefined);
    return;
  }
  const command =
    activeKind === 'chromecast' ? 'chromecast_set_volume' : activeKind === 'airplay' ? 'airplay_set_volume' : 'cast_set_volume';
  void invoke(command, { level }).catch(() => undefined);
}

export function castDisconnect(): void {
  if (activeKind === 'fcast') {
    void invoke('fcast_disconnect').catch(() => undefined);
    activeKind = null;
    return;
  }
  const command =
    activeKind === 'chromecast'
      ? 'chromecast_disconnect'
      : activeKind === 'airplay'
        ? 'airplay_disconnect'
        : activeKind === 'roku'
          ? 'roku_disconnect'
          : 'cast_disconnect';
  void invoke(command).catch(() => undefined);
  activeKind = null;
}
