import { useEffect, useRef } from 'react';
import { listen } from '@tauri-apps/api/event';
import { t } from '../i18n';
import type { Meta, Stream, Video } from '../core/types';
import { appPrefs } from '../core/appPrefs';
import { embeddedMpvSetTitle, embeddedMpvSetLoadingArtwork, embeddedMpvStop } from '../core/mpvPlayer';
import { fetchStreamsForEpisode } from '../core/effectRunner';
import { playerArtwork, playerDisplayTitle } from '../core/playerUtils';
import { coreResolveNextEpisode, coreSelectNextEpisodeStream } from '../core/engine';
import type { AppState } from '../core/types';
function presentNativePlayerError(message: string): string {
  const sourceFailure = /\b(loading failed|failed to open|http(?:\s+error)?|403|401|forbidden|unauthorized|not found|timed?\s*out|connection (?:refused|reset|failed)|network|no such host|certificate)\b/i.test(message);
  if (!sourceFailure) return message;
  return `Source error: the selected stream is unavailable, expired, or blocked.\n${message}`;
}

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
  onPlayerError,
  onEpisodePlaybackFailed,
  showEpisodeTransitionLoading,
}: {
  stateRef: React.MutableRefObject<AppState>;
  closingPlayerRef: React.MutableRefObject<boolean>;
  playingMetaRef: React.MutableRefObject<Meta | null>;
  playingStreamRef: React.MutableRefObject<Stream | null>;
  playingEpisodeRef: React.MutableRefObject<Video | null>;
  playingNextEpisodeRef: React.MutableRefObject<Video | null>;
  prefetchedNextEpRef: React.MutableRefObject<{ episodeId: string; stream: Stream } | null>;
  closePlayer: () => Promise<void>;
  handlePlay: (stream: Stream, meta?: Meta, episode?: Video | null, resumeAtSeconds?: number, totalDurationSeconds?: number, sourceCandidates?: Stream[], openSourcePickerOnFailure?: boolean) => Promise<void>;
  onPlayerError: (message: string) => Promise<void>;
  onEpisodePlaybackFailed?: (meta: Meta, episode: Video, message: string) => Promise<void> | void;
  showEpisodeTransitionLoading: (meta: Meta, episode: Video, stream: Stream) => void;
}) {
  const episodeTransitionActiveRef = useRef(false);

  useEffect(() => {
    const unlisteners: Array<() => void> = [];
    let cancelled = false;

    listen('native-player-close-requested', () => { void closePlayer(); })
      .then((fn) => { if (cancelled) fn(); else unlisteners.push(fn); })
      .catch(() => undefined);

    listen<string>('native-player-error', (event) => {
      void onPlayerError(presentNativePlayerError(event.payload));
    })
      .then((fn) => { if (cancelled) fn(); else unlisteners.push(fn); })
      .catch(() => undefined);

    return () => { cancelled = true; unlisteners.forEach((fn) => fn()); };
  }, [closePlayer, onPlayerError]);

  useEffect(() => {
    const unlisteners: Array<() => void> = [];
    let cancelled = false;

    listen('native-player-next-episode', () => {
      if (closingPlayerRef.current || episodeTransitionActiveRef.current) return;
      episodeTransitionActiveRef.current = true;
      const meta = playingMetaRef.current;
      const currentStream = playingStreamRef.current;
      const currentEp = playingEpisodeRef.current;

      void (async () => {
        try {
        let nextEp = playingNextEpisodeRef.current;
        if (!nextEp && meta?.videos?.length && currentEp) {
          nextEp = (await coreResolveNextEpisode(
            JSON.stringify(meta.videos),
            currentEp.season ?? 0,
            currentEp.episode ?? currentEp.number ?? 0,
            Date.now(),
            true,
          )) as typeof nextEp;
        }
        if (!nextEp || !meta || !currentStream) return;

        showEpisodeTransitionLoading(meta, nextEp, currentStream);
        await embeddedMpvStop().catch(() => undefined);
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
        let chosenStream: Stream | null = null;
        let sourceCandidates: Stream[] | undefined;
        const prefetched = prefetchedNextEpRef.current;
        const prefetchedIsTorrent = !!(prefetched?.stream.isTorrent || prefetched?.stream.infoHash);
        if (prefetched?.episodeId === nextEp.id && !prefetchedIsTorrent) {
          chosenStream = prefetched.stream;
          prefetchedNextEpRef.current = null;
        } else {
          if (prefetched?.episodeId === nextEp.id) prefetchedNextEpRef.current = null;
          try {
            const result = await fetchStreamsForEpisode(nextEp.id, meta.type);
            const streams = result.streams as Stream[];
            if (streams.length > 0) {
              sourceCandidates = streams;
              chosenStream = (await coreSelectNextEpisodeStream(JSON.stringify(streams), JSON.stringify(currentStream), JSON.stringify(prefs), nextEp.id)) as Stream | null;
            }
          } catch {}
        }
        if (!chosenStream) {
          if (!closingPlayerRef.current && onEpisodePlaybackFailed) await onEpisodePlaybackFailed(meta, nextEp, t('player.no_playable_url'));
          else if (!closingPlayerRef.current) await onPlayerError(t('player.no_playable_url'));
          return;
        }
        try { await handlePlay(chosenStream, meta, nextEp, undefined, undefined, sourceCandidates, true); } catch {}
        } finally {
          episodeTransitionActiveRef.current = false;
        }
      })();
    })
      .then((fn) => { if (cancelled) fn(); else unlisteners.push(fn); })
      .catch(() => undefined);

    listen<string>('native-player-play-episode', (event) => {
      if (closingPlayerRef.current || episodeTransitionActiveRef.current) return;
      episodeTransitionActiveRef.current = true;
      const episodeId = event.payload;
      const meta = playingMetaRef.current;
      const currentStream = playingStreamRef.current;
      if (!meta || !currentStream) {
        episodeTransitionActiveRef.current = false;
        return;
      }
      const ep = meta.videos?.find((v) => v.id === episodeId) ?? null;
      if (!ep) {
        episodeTransitionActiveRef.current = false;
        return;
      }

      void (async () => {
        try {
        showEpisodeTransitionLoading(meta, ep, currentStream);
        await embeddedMpvStop().catch(() => undefined);
        const nextTitle = playerDisplayTitle(meta, ep, currentStream);
        const nextArtwork = playerArtwork(meta, ep);
        void embeddedMpvSetTitle(nextTitle.contentTitle, nextTitle.episodeLine).catch(() => undefined);
        void embeddedMpvSetLoadingArtwork(
          nextTitle.contentTitle ?? 'Fluxa',
          nextTitle.episodeLine,
          nextArtwork.background,
          nextArtwork.logo,
        ).catch(() => undefined);

        const prefs = appPrefs(stateRef.current);
        let chosenStream: Stream | null = null;
        let sourceCandidates: Stream[] | undefined;
        try {
          const result = await fetchStreamsForEpisode(ep.id, meta.type);
          const streams = result.streams as Stream[];
          if (streams.length > 0) {
            sourceCandidates = streams;
            chosenStream = (await coreSelectNextEpisodeStream(JSON.stringify(streams), JSON.stringify(currentStream), JSON.stringify(prefs), ep.id)) as Stream | null;
          }
        } catch {}
        if (!chosenStream) {
          if (!closingPlayerRef.current && onEpisodePlaybackFailed) await onEpisodePlaybackFailed(meta, ep, t('player.no_playable_url'));
          else if (!closingPlayerRef.current) await onPlayerError(t('player.no_playable_url'));
          return;
        }
        try { await handlePlay(chosenStream, meta, ep, undefined, undefined, sourceCandidates, true); } catch {}
        } finally {
          episodeTransitionActiveRef.current = false;
        }
      })();
    })
      .then((fn) => { if (cancelled) fn(); else unlisteners.push(fn); })
      .catch(() => undefined);

    return () => { cancelled = true; unlisteners.forEach((fn) => fn()); };
  }, [handlePlay, stateRef, closingPlayerRef, playingMetaRef, playingStreamRef, playingEpisodeRef, playingNextEpisodeRef, prefetchedNextEpRef, episodeTransitionActiveRef, showEpisodeTransitionLoading, onEpisodePlaybackFailed]);
}
