import { useCallback, useEffect, type MutableRefObject } from 'react';
import { coreInvoke } from '../core/engine';
import type { EmbeddedMpvStatus } from '../core/mpvPlayer';
import { embeddedMpvStatus } from '../core/mpvPlayer';
import { scrobblePlaybackAction } from '../core/scrobble';
import { saveProfile } from '../core/profiles';
import type { Meta, UserProfile, Video } from '../core/types';

type Options = {
  playerUrl: string | null;
  activeProfileRef: MutableRefObject<UserProfile | null>;
  playingMetaRef: MutableRefObject<Meta | null>;
  playingEpisodeRef: MutableRefObject<Video | null>;
  closingPlayerRef: MutableRefObject<boolean>;
  inNativePlayerRef: MutableRefObject<boolean>;
  lastPlaybackStatusRef: MutableRefObject<EmbeddedMpvStatus | null>;
  scrobbleStartedRef: MutableRefObject<boolean>;
  scrobbleStoppedRef: MutableRefObject<boolean>;
  scrobbleWasPausedRef: MutableRefObject<boolean>;
  onProfileUpdated?: (profile: UserProfile) => void;
};

export function usePlayerScrobbling(options: Options) {
  const { playerUrl, activeProfileRef, playingMetaRef, playingEpisodeRef, closingPlayerRef, inNativePlayerRef, lastPlaybackStatusRef, scrobbleStartedRef, scrobbleStoppedRef, scrobbleWasPausedRef, onProfileUpdated } = options;
  const dispatchScrobbleLifecycle = useCallback(async (event: 'start' | 'pause' | 'stop', status: EmbeddedMpvStatus) => {
    const profile = activeProfileRef.current;
    const meta = playingMetaRef.current;
    if (!profile || !meta) return;
    const timePosSec = parseFloat(status.timePos ?? '0');
    const durationSec = parseFloat(status.duration ?? '0');
    if (!Number.isFinite(timePosSec) || !Number.isFinite(durationSec) || durationSec <= 0) return;
    const action = await coreInvoke<{ action: 'start' | 'pause' | 'stop' }>('playerScrobbleLifecycleAction', JSON.stringify({ event, token: profile.traktAccessToken ?? profile.simklAccessToken, hasStarted: scrobbleStartedRef.current, hasPaused: scrobbleWasPausedRef.current, hasStopped: scrobbleStoppedRef.current, progress: (timePosSec / durationSec) * 100 }));
    if (!action) return;
    scrobblePlaybackAction(profile, meta, playingEpisodeRef.current, timePosSec, durationSec, action.action, (revoked) => { void saveProfile(revoked); onProfileUpdated?.(revoked); });
    if (action.action === 'start') scrobbleStartedRef.current = true;
    if (action.action === 'start') scrobbleWasPausedRef.current = false;
    if (action.action === 'stop') scrobbleStoppedRef.current = true;
  }, [activeProfileRef, onProfileUpdated, playingEpisodeRef, playingMetaRef, scrobbleStartedRef, scrobbleStoppedRef, scrobbleWasPausedRef]);

  useEffect(() => {
    if (!playerUrl) return;
    let cancelled = false;
    const poll = async () => {
      if (cancelled || closingPlayerRef.current || !inNativePlayerRef.current) return;
      const status = await embeddedMpvStatus().catch(() => null);
      if (!status || cancelled) return;
      lastPlaybackStatusRef.current = status;
      const paused = status.pause === 'yes';
      if (!paused) await dispatchScrobbleLifecycle('start', status);
      if (paused && !scrobbleWasPausedRef.current) { scrobbleWasPausedRef.current = true; await dispatchScrobbleLifecycle('pause', status); return; }
      scrobbleWasPausedRef.current = paused;
    };
    void poll();
    const interval = setInterval(() => { void poll(); }, 1000);
    return () => { cancelled = true; clearInterval(interval); };
  }, [closingPlayerRef, dispatchScrobbleLifecycle, inNativePlayerRef, lastPlaybackStatusRef, playerUrl, scrobbleWasPausedRef]);

  return dispatchScrobbleLifecycle;
}
