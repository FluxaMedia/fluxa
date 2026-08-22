import { useCallback, useEffect, type MutableRefObject } from 'react';
import { persistPlaybackProgress } from '../core/playbackSession';
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
  useEffect(() => {
    if (!playerUrl) return;
    const interval = setInterval(() => {
      void saveProgressTick();
    }, 30000);
    return () => clearInterval(interval);
  }, [playerUrl, saveProgressTick]);
  return saveProgressTick;
}
