import { useCallback, useEffect, useRef, type MutableRefObject } from 'react';
import { persistPlaybackProgress } from '../core/playbackSession';
import { coreInvoke } from '../core/engine';
import { appPrefs } from '../core/appPrefs';
import { embeddedMpvStatus, type EmbeddedMpvStatus } from '../core/mpvPlayer';
import { persistLastPlaybackSource } from '../core/libraryStorage';
import type { AppState, Meta, Stream, Video } from '../core/types';

type Options = {
  playerUrl: string | null;
  stateRef: MutableRefObject<AppState>;
  closingPlayerRef: MutableRefObject<boolean>;
  inNativePlayerRef: MutableRefObject<boolean>;
  playingMetaRef: MutableRefObject<Meta | null>;
  playingEpisodeRef: MutableRefObject<Video | null>;
  playingStreamRef: MutableRefObject<Stream | null>;
  lastPlaybackStatusRef: MutableRefObject<EmbeddedMpvStatus | null>;
  updateState: (state: Partial<AppState>) => void;
};

export function usePlayerProgressPersistence(options: Options) {
  const {
    playerUrl,
    stateRef,
    closingPlayerRef,
    inNativePlayerRef,
    playingMetaRef,
    playingEpisodeRef,
    playingStreamRef,
    lastPlaybackStatusRef,
    updateState,
  } = options;
  const lastEventSaveAtRef = useRef(0);
  const eventSaveInFlightRef = useRef<Promise<void> | null>(null);
  const saveProgressTick = useCallback(async () => {
    if (closingPlayerRef.current || !inNativePlayerRef.current || !playingMetaRef.current) return;
    await persistLastPlaybackSource(playingMetaRef.current, playingStreamRef.current).catch(() => undefined);
    const status = await embeddedMpvStatus().catch(() => null);
    if (!status) return;
    lastPlaybackStatusRef.current = status;
    try {
      await persistPlaybackProgress({
        meta: playingMetaRef.current,
        episode: playingEpisodeRef.current,
        stream: playingStreamRef.current,
        nextEpisode: null,
        snapshot: {
          timePos: parseFloat(status.timePos ?? '0'),
          duration: parseFloat(status.duration ?? '0'),
        },
        streamIndex: stateRef.current.player.currentStreamIndex ?? null,
        prefs: appPrefs(stateRef.current),
        updateState,
      });
    } catch {}
  }, [
    closingPlayerRef,
    inNativePlayerRef,
    lastPlaybackStatusRef,
    playingEpisodeRef,
    playingMetaRef,
    playingStreamRef,
    stateRef,
    updateState,
  ]);
  const saveProgressOnEvent = useCallback(async () => {
    const now = Date.now();
    const allowed = await coreInvoke<boolean>(
      'playerShouldSaveEventProgress',
      JSON.stringify({ nowMs: now, lastSavedAtMs: lastEventSaveAtRef.current }),
    ).catch(() => false);
    if (!allowed) return;
    if (eventSaveInFlightRef.current) return eventSaveInFlightRef.current;
    lastEventSaveAtRef.current = now;
    const pending = saveProgressTick().finally(() => {
      eventSaveInFlightRef.current = null;
    });
    eventSaveInFlightRef.current = pending;
    return pending;
  }, [saveProgressTick]);
  useEffect(() => {
    if (!playerUrl) return;
    const interval = setInterval(() => {
      void saveProgressTick();
    }, 30000);
    return () => clearInterval(interval);
  }, [playerUrl, saveProgressTick]);
  return { saveProgressTick, saveProgressOnEvent };
}
