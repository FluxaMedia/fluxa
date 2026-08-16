import { useEffect, useRef } from 'react';

type Sentinel = { release: () => Promise<void>; released: boolean };
type WakeLockNavigator = Navigator & { wakeLock?: { request: (type: 'screen') => Promise<Sentinel> } };

export function useWakeLock(active: boolean): void {
  const sentinelRef = useRef<Sentinel | null>(null);

  useEffect(() => {
    const api = (navigator as WakeLockNavigator).wakeLock;
    if (!api || !active) return undefined;

    let cancelled = false;

    const acquire = async () => {
      if (cancelled || sentinelRef.current) return;
      try {
        sentinelRef.current = await api.request('screen');
      } catch {
        sentinelRef.current = null;
      }
    };

    const onVisibility = () => {
      if (document.visibilityState !== 'visible') return;
      if (sentinelRef.current?.released !== false) sentinelRef.current = null;
      void acquire();
    };

    void acquire();
    document.addEventListener('visibilitychange', onVisibility);

    return () => {
      cancelled = true;
      document.removeEventListener('visibilitychange', onVisibility);
      void sentinelRef.current?.release().catch(() => undefined);
      sentinelRef.current = null;
    };
  }, [active]);
}
