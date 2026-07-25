import { useState, useEffect, useCallback } from 'react';
import { nuvioHealthCheck, nuvioPushWatchProgress, nuvioPushLibrary, nuvioPushWatchHistory } from '../core/nuvioApi';
import { loadLibrary } from '../core/libraryOps';
import { freshNuvioProfile, importNuvioProfileData, recordNuvioSyncMeta, refreshNuvioProfiles } from '../core/nuvioSync';
import type { UserProfile } from '../core/types';
import { coreInvoke } from '../core/engine';

async function pushLocalToNuvio(profile: UserProfile): Promise<void> {
  const freshProfile = await freshNuvioProfile(profile).catch(() => profile);
  const token = freshProfile.nuvioAccessToken!;
  const profileIdx = freshProfile.nuvioProfileIndex ?? 1;
  const lib = await loadLibrary();
  const plan = await coreInvoke<{
    progressEntries: Array<{ content_id: string; content_type: string; video_id: string; position: number; duration: number; last_watched: number; season?: number; episode?: number }>;
    libraryItems: Array<{ content_id: string; content_type: string; name?: string; poster?: string | null; background?: string | null }>;
    historyItems: Array<{ content_id: string; content_type: string; title?: string; season?: number; episode?: number; watched_at: number }>;
  }>('nuvioExportPushPlan', JSON.stringify({ library: lib, nowMs: Date.now() }));
  if (!plan) return;

  await Promise.allSettled([
    plan.progressEntries.length > 0 ? nuvioPushWatchProgress(token, profileIdx, plan.progressEntries) : Promise.resolve(),
    plan.libraryItems.length > 0 ? nuvioPushLibrary(token, profileIdx, plan.libraryItems) : Promise.resolve(),
    plan.historyItems.length > 0 ? nuvioPushWatchHistory(token, profileIdx, plan.historyItems) : Promise.resolve(),
  ]);
}

export function useNuvioConnectivity(activeProfile: UserProfile | null, onSynced?: () => void | Promise<void>) {
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
      } catch {
        down = true;
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
        void (async () => {
          await importNuvioProfileData(profile)
            .then((report) => recordNuvioSyncMeta(report))
            .catch((err) => recordNuvioSyncMeta({ errors: { library: err instanceof Error ? err.message : String(err) } }));
          await refreshNuvioProfiles(profile).catch(() => profile);
          await pushLocalToNuvio(profile).catch(() => undefined);
          await onSynced?.();
        })();
      } else if (!down && !pulledRemote) {
        pulledRemote = true;
        void (async () => {
          await importNuvioProfileData(profile)
            .then((report) => recordNuvioSyncMeta(report))
            .catch((err) => recordNuvioSyncMeta({ errors: { library: err instanceof Error ? err.message : String(err) } }));
          await refreshNuvioProfiles(profile).catch(() => profile);
          await onSynced?.();
        })();
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
