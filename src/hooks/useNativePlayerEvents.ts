import { useEffect, useState } from 'react';
import { platformListen as listen } from '../platform/browser';
import { platformInvoke as invoke } from '../platform/invoke';
import { notify } from '../core/notifications';
import { t } from '../i18n';

function debugLog(msg: string) {
  void invoke('debug_log', { msg }).catch(() => {});
}

export function useNativePlayerEvents(flushProgressOnQuit: () => Promise<void>) {
  const [nativePlayerActive, setNativePlayerActive] = useState(false);
  const [softwareVideoActive, setSoftwareVideoActive] = useState(false);

  if (import.meta.env.VITE_FLUXA_TARGET === 'web' || import.meta.env.VITE_FLUXA_TARGET === 'webos') {
    return { nativePlayerActive: false, setNativePlayerActive, softwareVideoActive: false, setSoftwareVideoActive };
  }

  useEffect(() => {
    const unlisteners: Array<() => void> = [];
    let cancelled = false;

    debugLog('App: registering native-player-show/hide listeners');
    listen('native-player-show', () => {
      debugLog('App: received native-player-show');
      setNativePlayerActive(true);
      setSoftwareVideoActive(false);
      document.documentElement.setAttribute('data-native-player-active', 'true');
    }).then((fn) => { debugLog('App: native-player-show listener registered'); if (cancelled) fn(); else unlisteners.push(fn); }).catch((err) => debugLog(`App: native-player-show listen() failed ${String(err)}`));

    listen('native-player-hide', () => {
      debugLog('App: received native-player-hide');
      setNativePlayerActive(false);
      setSoftwareVideoActive(false);
      document.documentElement.removeAttribute('data-native-player-active');
    }).then((fn) => { if (cancelled) fn(); else unlisteners.push(fn); }).catch(() => undefined);

    listen<string>('native-player-software-rendering', (event) => {
      debugLog(`App: software video rendering active: ${event.payload}`);
      setNativePlayerActive(true);
      setSoftwareVideoActive(true);
      document.documentElement.setAttribute('data-native-player-active', 'true');
    }).then((fn) => { if (cancelled) fn(); else unlisteners.push(fn); }).catch(() => undefined);

    return () => { cancelled = true; unlisteners.forEach((fn) => fn()); };
  }, []);

  useEffect(() => {
    const unlisten = listen<{ title?: string; status: string }>('download-progress', (e) => {
      if (e.payload.status === 'downloaded') {
        void notify(t('notifications.download_complete_title'), e.payload.title);
      } else if (e.payload.status === 'failed') {
        void notify(t('notifications.download_failed_title'), e.payload.title);
      }
    });
    return () => { void unlisten.then((fn) => fn()); };
  }, []);

  useEffect(() => {
    let cancelled = false;
    let unlisten: (() => void) | null = null;
    listen('native-app-close-requested', () => {
      void flushProgressOnQuit()
        .catch(() => undefined)
        .finally(() => { void invoke('app_close_flush_done').catch(() => undefined); });
    })
      .then((fn) => { if (cancelled) fn(); else unlisten = fn; })
      .catch(() => undefined);
    return () => { cancelled = true; unlisten?.(); };
  }, [flushProgressOnQuit]);

  return { nativePlayerActive, setNativePlayerActive, softwareVideoActive, setSoftwareVideoActive };
}
