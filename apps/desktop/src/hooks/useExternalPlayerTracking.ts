import { useEffect } from 'react';
import { platformListen as listen } from '../platform/browser';
import { platformInvoke as invoke } from '../platform/invoke';

function debugLog(msg: string) {
  void invoke('debug_log', { msg }).catch(() => {});
}

export type ExternalPlayerSession = {
  sessionId: string;
  tracking: 'live' | 'callback' | 'passive';
  startedAt: number;
};

export type ExternalPlayerStatus = {
  active: boolean;
  paused?: boolean | null;
  timePos?: number | null;
  duration?: number | null;
};

type Options = {
  session: ExternalPlayerSession | null;
  onStatus: (status: ExternalPlayerStatus) => void;
  onCallback: (progress: number | null, failed: boolean) => void;
};

export function useExternalPlayerTracking({ session, onStatus, onCallback }: Options) {
  useEffect(() => {
    if (!session) return;
    debugLog(`externalPlayerTracking: session=${session.sessionId} tracking=${session.tracking} started`);
    let cancelled = false;
    const unlisten = listen<{ url?: string }>('deep-link-opened', (event) => {
      const raw = event.payload.url ?? '';
      let url: URL;
      try {
        url = new URL(raw);
      } catch {
        return;
      }
      if (url.protocol !== 'fluxa:' || !['external-player', 'external-player-error'].includes(url.hostname)) return;
      if (url.searchParams.get('session') !== session.sessionId) return;
      const rawPosition = url.searchParams.get('position');
      const parsed = rawPosition === null || rawPosition.trim() === '' ? Number.NaN : Number(rawPosition);
      const position = Number.isFinite(parsed) ? parsed : null;
      debugLog(
        `externalPlayerTracking: session=${session.sessionId} deep-link callback position=${position ?? 'unknown'} failed=${url.hostname === 'external-player-error'}`,
      );
      onCallback(position, url.hostname === 'external-player-error');
    });
    if (session.tracking !== 'live') {
      return () => {
        cancelled = true;
        void unlisten.then((fn) => fn());
      };
    }
    let polling = false;
    let lastSeenAt = session.startedAt;
    const poll = async () => {
      if (polling) return;
      polling = true;
      try {
        const status = await invoke<ExternalPlayerStatus | null>('external_player_status', { sessionId: session.sessionId }).catch(
          () => null,
        );
        if (cancelled) return;
        if (status) {
          lastSeenAt = Date.now();
          debugLog(
            `externalPlayerTracking: session=${session.sessionId} status active=${status.active} paused=${status.paused} timePos=${status.timePos} duration=${status.duration}`,
          );
          onStatus(status);
        } else if (Date.now() - lastSeenAt > 5_000) {
          debugLog(`externalPlayerTracking: session=${session.sessionId} no status for ${Date.now() - lastSeenAt}ms, treating as closed`);
          onStatus({ active: false });
        }
      } finally {
        polling = false;
      }
    };
    void poll();
    const statusUnlisten = listen<ExternalPlayerStatus & { sessionId: string }>('external-player-status', (event) => {
      if (event.payload.sessionId !== session.sessionId) return;
      lastSeenAt = Date.now();
      debugLog(
        `externalPlayerTracking: session=${session.sessionId} pushed status paused=${event.payload.paused} timePos=${event.payload.timePos} duration=${event.payload.duration}`,
      );
      onStatus(event.payload);
    });
    const timer = window.setInterval(() => {
      void poll();
    }, 10_000);
    return () => {
      debugLog(`externalPlayerTracking: session=${session.sessionId} tracking stopped`);
      cancelled = true;
      window.clearInterval(timer);
      void unlisten.then((fn) => fn());
      void statusUnlisten.then((fn) => fn());
    };
  }, [onCallback, onStatus, session]);
}
