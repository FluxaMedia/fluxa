import { useCallback, useEffect, useRef, useState } from 'react';
import { platformInvoke as invoke } from '../../platform/invoke';
import { sendCmd } from './PlayerOverlayPrimitives';
import { castDisconnect, castPause, castPlay, discoverCastDevices, proxyMediaUrl, resolveCastMediaUrl, startCasting, type CastDevice } from '../../core/cast';
import type { EmbeddedMpvStatus } from '../../core/mpvPlayer';

export function usePlayerCasting({ title, episodeTitle, initialSubtitleUrl, initialStreamHeaders, resetActivity }: { title: string; episodeTitle: string; initialSubtitleUrl?: string; initialStreamHeaders?: Record<string, string>; resetActivity: () => void }) {
  const [castPopoverOpen, setCastPopoverOpen] = useState(false);
  const [castDevices, setCastDevices] = useState<CastDevice[]>([]);
  const [castDiscovering, setCastDiscovering] = useState(false);
  const [activeCastDeviceId, setActiveCastDeviceId] = useState<string | null>(null);
  const activeCastDeviceIdRef = useRef<string | null>(null);
  const [activeCastDeviceName, setActiveCastDeviceName] = useState('');
  const [castPaused, setCastPaused] = useState(false);
  const openCastPopoverRef = useRef<() => Promise<void>>(async () => {});

  useEffect(() => {
    activeCastDeviceIdRef.current = activeCastDeviceId;
    return () => { if (activeCastDeviceId) castDisconnect(); };
  }, [activeCastDeviceId]);

  const openCastPopover = useCallback(async () => {
    resetActivity();
    if (castPopoverOpen) {
      setCastPopoverOpen(false);
      return;
    }
    setCastPopoverOpen(true);
    setCastDiscovering(true);
    setCastDevices(await discoverCastDevices());
    setCastDiscovering(false);
  }, [castPopoverOpen, resetActivity]);
  useEffect(() => { openCastPopoverRef.current = openCastPopover; }, [openCastPopover]);

  const selectCastDevice = useCallback(async (device: CastDevice) => {
    let status: EmbeddedMpvStatus | null = null;
    try { status = await invoke<EmbeddedMpvStatus>('player_status'); } catch {}
    const streamUrl = status?.path;
    if (!streamUrl) {
      setCastPopoverOpen(false);
      return;
    }
    const mediaUrl = initialStreamHeaders && Object.keys(initialStreamHeaders).length > 0 ? await proxyMediaUrl(streamUrl, initialStreamHeaders) : await resolveCastMediaUrl(streamUrl);
    try {
      await startCasting(device, mediaUrl, title || episodeTitle || 'Fluxa', initialSubtitleUrl, Number(status?.timePos ?? 0) || 0);
      setActiveCastDeviceId(device.id);
      setActiveCastDeviceName(device.name);
      setCastPaused(false);
      sendCmd('set pause yes');
    } catch {}
    setCastPopoverOpen(false);
  }, [episodeTitle, initialStreamHeaders, initialSubtitleUrl, title]);

  const disconnectCast = useCallback(() => {
    castDisconnect();
    setActiveCastDeviceId(null);
    setActiveCastDeviceName('');
    setCastPopoverOpen(false);
  }, []);

  const toggleCastPause = useCallback(() => {
    if (castPaused) {
      castPlay();
      setCastPaused(false);
      return;
    }
    castPause();
    setCastPaused(true);
  }, [castPaused]);

  return { activeCastDeviceId, activeCastDeviceIdRef, activeCastDeviceName, castDevices, castDiscovering, castPaused, castPopoverOpen, disconnectCast, openCastPopover, openCastPopoverRef, selectCastDevice, setCastPopoverOpen, toggleCastPause };
}
