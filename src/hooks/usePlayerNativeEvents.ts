import { useEffect } from 'react';
import { listen } from '@tauri-apps/api/event';
import type { Meta, Stream, Video } from '../core/types';
import { appPrefs } from '../core/appPrefs';
import { embeddedMpvSetTitle, embeddedMpvSetLoadingArtwork, embeddedMpvStop } from '../core/mpvPlayer';
import { fetchStreamsForEpisode } from '../core/effectRunner';
import { playerArtwork, playerDisplayTitle, resolveNextEpisode, isEpisodeReleasedForPlayback, selectNextEpisodeStream } from '../core/playerUtils';
import type { AppState } from '../core/types';

export function usePlayerNativeEvents({
  stateRef,
  closingPlayerRef,
  playingMetaRef,
  playingStreamRef,
  playingEpisodeRef,
  playingNextEpisodeRef,
  prefetchedNextEpRef,
  closePlayer,
  handlePlay,
}: {
  stateRef: React.MutableRefObject<AppState>;
  closingPlayerRef: React.MutableRefObject<boolean>;
  playingMetaRef: React.MutableRefObject<Meta | null>;
  playingStreamRef: React.MutableRefObject<Stream | null>;
  playingEpisodeRef: React.MutableRefObject<Video | null>;
  playingNextEpisodeRef: React.MutableRefObject<Video | null>;
  prefetchedNextEpRef: React.MutableRefObject<{ episodeId: string; stream: Stream } | null>;
  closePlayer: () => Promise<void>;
  handlePlay: (stream: Stream, meta?: Meta, episode?: Video | null, resumeAtSeconds?: number, totalDurationSeconds?: number) => Promise<void>;
}) {
  useEffect(() => {
    const unlisteners: Array<() => void> = [];
    let cancelled = false;

    listen('native-player-close-requested', () => { void closePlayer(); })
      .then((fn) => { if (cancelled) fn(); else unlisteners.push(fn); })
      .catch(() => undefined);

    listen<string>('native-player-error', (event) => {
      void closePlayer();
      alert(event.payload);
    })
      .then((fn) => { if (cancelled) fn(); else unlisteners.push(fn); })
      .catch(() => undefined);

    return () => { cancelled = true; unlisteners.forEach((fn) => fn()); };
  }, [closePlayer]);

  useEffect(() => {
    const unlisteners: Array<() => void> = [];
    let cancelled = false;

    listen('native-player-next-episode', () => {
      if (closingPlayerRef.current) return;
      const meta = playingMetaRef.current;
      const currentStream = playingStreamRef.current;
      const currentEp = playingEpisodeRef.current;

      let nextEp = playingNextEpisodeRef.current;
      if (nextEp && !isEpisodeReleasedForPlayback(nextEp)) nextEp = null;
      if (!nextEp && meta?.videos?.length && currentEp) {
        nextEp = resolveNextEpisode(meta.videos, currentEp, true);
      }
      if (!nextEp || !meta || !currentStream) return;

      void embeddedMpvStop().catch(() => undefined);
      void (async () => {
        const nextTitle = playerDisplayTitle(meta, nextEp, currentStream);
        const nextArtwork = playerArtwork(meta, nextEp);
        void embeddedMpvSetTitle(nextTitle.contentTitle, nextTitle.episodeLine).catch(() => undefined);
        void embeddedMpvSetLoadingArtwork(
          nextTitle.contentTitle ?? 'Fluxa',
          nextTitle.episodeLine,
          nextArtwork.background,
          nextArtwork.logo,
        ).catch(() => undefined);

        const prefs = appPrefs(stateRef.current);
        let chosenStream: Stream = currentStream;
        const prefetched = prefetchedNextEpRef.current;
        if (prefetched?.episodeId === nextEp!.id) {
          chosenStream = prefetched.stream;
          prefetchedNextEpRef.current = null;
        } else {
          try {
            const timeout = new Promise<never>((_, reject) =>
              setTimeout(() => reject(new Error('stream fetch timeout')), 8000));
            const result = await Promise.race([fetchStreamsForEpisode(nextEp!.id, meta.type), timeout]);
            const streams = result.streams as Stream[];
            if (streams.length > 0) {
              chosenStream = selectNextEpisodeStream(streams, currentStream, prefs) ?? currentStream;
            }
          } catch {}
        }
        try { await handlePlay(chosenStream, meta, nextEp); } catch {}
      })();
    })
      .then((fn) => { if (cancelled) fn(); else unlisteners.push(fn); })
      .catch(() => undefined);

    listen<string>('native-player-play-episode', (event) => {
      if (closingPlayerRef.current) return;
      const episodeId = event.payload;
      const meta = playingMetaRef.current;
      const stream = playingStreamRef.current;
      if (!meta || !stream) return;
      const ep = meta.videos?.find((v) => v.id === episodeId) ?? null;
      if (!ep) return;
      void handlePlay(stream, meta, ep);
    })
      .then((fn) => { if (cancelled) fn(); else unlisteners.push(fn); })
      .catch(() => undefined);

    return () => { cancelled = true; unlisteners.forEach((fn) => fn()); };
  }, [handlePlay, stateRef, closingPlayerRef, playingMetaRef, playingStreamRef, playingEpisodeRef, playingNextEpisodeRef, prefetchedNextEpRef]);
}
