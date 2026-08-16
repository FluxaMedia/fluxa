import { useCallback, useEffect, type MutableRefObject } from 'react';
import { platformListen as listen } from '../platform/browser';
import type { EmbeddedMpvStatus } from '../core/mpvPlayer';
import { embeddedMpvStatus } from '../core/mpvPlayer';
import { runScrobbleLifecycle } from '../core/playbackSession';
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
    const action = await runScrobbleLifecycle({
      event,
      profile,
      meta,
      episode: playingEpisodeRef.current,
      snapshot: {
        timePos: parseFloat(status.timePos ?? '0'),
        duration: parseFloat(status.duration ?? '0'),
      },
      flags: {
        hasStarted: scrobbleStartedRef.current,
        hasPaused: scrobbleWasPausedRef.current,
        hasStopped: scrobbleStoppedRef.current,
      },
      onProfileUpdated,
    });
    if (action === 'start') {
      scrobbleStartedRef.current = true;
      scrobbleWasPausedRef.current = false;
    }
    if (action === 'stop') scrobbleStoppedRef.current = true;
  }, [activeProfileRef, onProfileUpdated, playingEpisodeRef, playingMetaRef, scrobbleStartedRef, scrobbleStoppedRef, scrobbleWasPausedRef]);

  useEffect(() => {
    if (!playerUrl) return;
    let cancelled = false;
    const handlePauseChanged = async (paused: boolean) => {
      if (cancelled || closingPlayerRef.current || !inNativePlayerRef.current) return;
      const status = await embeddedMpvStatus().catch(() => null);
      if (!status || cancelled) return;
      lastPlaybackStatusRef.current = status;
      if (!paused) {
        scrobbleWasPausedRef.current = false;
        await dispatchScrobbleLifecycle('start', status);
        return;
      }
      if (!scrobbleWasPausedRef.current) {
        scrobbleWasPausedRef.current = true;
        await dispatchScrobbleLifecycle('pause', status);
      }
    };
    let unlisten: (() => void) | null = null;
    listen<boolean>('native-player-pause-changed', (event) => { void handlePauseChanged(event.payload); })
      .then((fn) => { if (cancelled) fn(); else unlisten = fn; })
      .catch(() => undefined);
    void embeddedMpvStatus().then((status) => { if (!cancelled && status) void handlePauseChanged(status.pause === 'yes'); }).catch(() => undefined);
    return () => { cancelled = true; unlisten?.(); };
  }, [closingPlayerRef, dispatchScrobbleLifecycle, inNativePlayerRef, lastPlaybackStatusRef, playerUrl, scrobbleWasPausedRef]);

  return dispatchScrobbleLifecycle;
}
