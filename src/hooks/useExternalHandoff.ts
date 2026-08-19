import { useCallback, useEffect, useState } from 'react';
import { appPrefs } from '../core/appPrefs';
import {
  clearExternalSession,
  estimateSession,
  readExternalSession,
  writeExternalSession,
  type ExternalSession,
} from '../core/externalSession';
import { persistPlaybackProgress, runScrobbleLifecycle } from '../core/playbackSession';
import type { AppState, UserProfile } from '../core/types';

const RETURN_GRACE_MS = 30_000;

type Options = {
  stateRef: React.MutableRefObject<AppState>;
  activeProfile: UserProfile | null;
  updateState: (state: Partial<AppState>) => void;
  onProfileUpdated?: (profile: UserProfile) => void;
};

export type ExternalHandoffPrompt = {
  session: ExternalSession;
  estimate: { timePos: number; duration: number; finished: boolean };
};

export function useExternalHandoff({ stateRef, activeProfile, updateState, onProfileUpdated }: Options) {
  const [prompt, setPrompt] = useState<ExternalHandoffPrompt | null>(null);

  const evaluate = useCallback(() => {
    const session = readExternalSession();
    if (!session) return;
    const estimate = estimateSession(session, Date.now());
    if (!estimate) {
      clearExternalSession();
      return;
    }
    setPrompt({ session, estimate });
  }, []);

  useEffect(() => {
    const url = new URL(window.location.href);
    if (url.searchParams.get('fluxa_external') === 'done') {
      url.searchParams.delete('fluxa_external');
      window.history.replaceState(null, '', url.toString());
    }
    evaluate();

    let leftAt = 0;
    const onVisibility = () => {
      if (document.visibilityState === 'hidden') {
        leftAt = Date.now();
        return;
      }
      if (leftAt === 0 || Date.now() - leftAt < RETURN_GRACE_MS) return;
      evaluate();
    };
    document.addEventListener('visibilitychange', onVisibility);
    return () => document.removeEventListener('visibilitychange', onVisibility);
  }, [evaluate]);

  const start = useCallback((session: ExternalSession) => {
    writeExternalSession(session);
  }, []);

  const dismiss = useCallback(() => {
    clearExternalSession();
    setPrompt(null);
  }, []);

  const commit = useCallback(
    async (timePos: number, duration: number) => {
      const current = prompt;
      setPrompt(null);
      clearExternalSession();
      if (!current || !(duration > 0)) return;
      const snapshot = { timePos, duration };
      await persistPlaybackProgress({
        meta: current.session.meta,
        episode: current.session.episode,
        stream: current.session.stream,
        nextEpisode: null,
        snapshot,
        streamIndex: null,
        prefs: appPrefs(stateRef.current),
        updateState,
      }).catch(() => undefined);
      const flags = { hasStarted: false, hasPaused: false, hasStopped: false };
      await runScrobbleLifecycle({
        event: 'start',
        profile: activeProfile,
        meta: current.session.meta,
        episode: current.session.episode,
        snapshot,
        flags,
        onProfileUpdated,
      }).catch(() => undefined);
      await runScrobbleLifecycle({
        event: 'stop',
        profile: activeProfile,
        meta: current.session.meta,
        episode: current.session.episode,
        snapshot,
        flags: { ...flags, hasStarted: true },
        onProfileUpdated,
      }).catch(() => undefined);
    },
    [prompt, stateRef, updateState, activeProfile, onProfileUpdated],
  );

  return { prompt, start, dismiss, commit };
}
