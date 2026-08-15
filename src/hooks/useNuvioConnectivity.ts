import { useState, useEffect, useCallback } from 'react';
import { nuvioHealthCheck } from '../core/nuvioApi';
import type { UserProfile } from '../core/types';

export function useNuvioConnectivity(activeProfile: UserProfile | null, onSynced?: (changed: boolean) => void | Promise<void>) {
  const [serverDown, setServerDown] = useState(false);
  const [justRecovered, setJustRecovered] = useState(false);
  const [dismissed, setDismissed] = useState(false);

  useEffect(() => {
    const token = activeProfile?.nuvioAccessToken;
    if (!token) {
      setServerDown(false);
      return;
    }

    const profile = activeProfile!;
    let cancelled = false;
    let isCurrentlyDown = false;
    let pulledRemote = false;
    let timer: ReturnType<typeof setTimeout>;

    const run = async () => {
      if (cancelled) return;
      let down = isCurrentlyDown;
      try {
        const result = await nuvioHealthCheck();
        down = result?.status !== 'healthy' && result?.status !== 'ok';
      } catch (error) {
        console.error('[fluxa:nuvio:health-check]', error);
        down = import.meta.env.VITE_FLUXA_TARGET !== 'web' && import.meta.env.VITE_FLUXA_TARGET !== 'webos';
      }
      if (cancelled) return;

      if (down && !isCurrentlyDown) {
        isCurrentlyDown = true;
        setDismissed(false);
        setServerDown(true);
        setJustRecovered(false);
      } else if (!down && isCurrentlyDown) {
        isCurrentlyDown = false;
        setServerDown(false);
        setJustRecovered(true);
        setDismissed(false);
        setTimeout(() => { if (!cancelled) setJustRecovered(false); }, 2000);
        void onSynced?.(false);
      } else if (!down && !pulledRemote) {
        pulledRemote = true;
      }

      timer = setTimeout(run, isCurrentlyDown ? 30_000 : 60_000);
    };

    void run();

    return () => {
      cancelled = true;
      clearTimeout(timer);
    };
  }, [activeProfile?.nuvioAccessToken, activeProfile?.id, onSynced]);

  const dismiss = useCallback(() => setDismissed(true), []);

  return { serverDown, justRecovered, dismissed, dismiss };
}
