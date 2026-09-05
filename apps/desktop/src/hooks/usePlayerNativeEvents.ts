import { useEffect, useRef } from 'react';
import { platformListen as listen } from '../platform/browser';
import { platformInvoke as invoke } from '../platform/invoke';
import { t } from '../i18n';
import type { Meta, Stream, Video } from '../core/types';
import { appPrefs } from '../core/appPrefs';
import {
  embeddedMpvSetTitle,
  embeddedMpvSetLoadingArtwork,
  embeddedMpvShowLoading,
  embeddedMpvStatus,
  embeddedMpvStop,
  playerLastStreamError,
  type EmbeddedMpvStatus,
} from '../core/mpvPlayer';
import { fetchStreamsForEpisode } from '../core/effectRunner';
import { playerArtwork, playerDisplayTitle } from '../core/playerUtils';
import { coreResolveNextEpisode, coreSelectNextEpisodeStream } from '../core/engine';
import type { AppState } from '../core/types';
function presentNativePlayerError(message: string): string {
  const sourceFailure =
    /\b(loading failed|failed to open|http(?:\s+error)?|403|401|forbidden|unauthorized|not found|timed?\s*out|connection (?:refused|reset|failed)|network|no such host|certificate)\b/i.test(
      message,
    );
  if (!sourceFailure) return message;
  return t('player.source_error_detail', message);
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
  scrobbleStartedRef,
  dispatchScrobbleLifecycle,
  saveProgressOnEvent,
  onTerminalPlayback,
  debugLog,
}: {
  stateRef: React.MutableRefObject<AppState>;
  closingPlayerRef: React.MutableRefObject<boolean>;
  playingMetaRef: React.MutableRefObject<Meta | null>;
  playingStreamRef: React.MutableRefObject<Stream | null>;
  playingEpisodeRef: React.MutableRefObject<Video | null>;
  playingNextEpisodeRef: React.MutableRefObject<Video | null>;
  prefetchedNextEpRef: React.MutableRefObject<{ episodeId: string; stream: Stream } | null>;
  closePlayer: () => Promise<void>;
  handlePlay: (
    stream: Stream,
    meta?: Meta,
    episode?: Video | null,
    resumeAtSeconds?: number,
    totalDurationSeconds?: number,
    sourceCandidates?: Stream[],
    openSourcePickerOnFailure?: boolean,
  ) => Promise<void>;
  onPlayerError: (message: string) => Promise<void>;
  onEpisodePlaybackFailed?: (meta: Meta, episode: Video, message: string) => Promise<void> | void;
  showEpisodeTransitionLoading: (meta: Meta, episode: Video, stream: Stream) => void;
  scrobbleStartedRef: React.MutableRefObject<boolean>;
  dispatchScrobbleLifecycle: (event: 'start' | 'pause' | 'stop', status: EmbeddedMpvStatus) => Promise<void>;
  saveProgressOnEvent?: () => Promise<void>;
  onTerminalPlayback?: () => Promise<boolean>;
  debugLog: (message: string) => void;
}) {
  const episodeTransitionActiveRef = useRef(false);

  const stopScrobbleForOutgoingEpisode = async () => {
    if (!scrobbleStartedRef.current) return;
    const status = await embeddedMpvStatus().catch(() => null);
    if (!status) return;
    await dispatchScrobbleLifecycle('stop', status).catch(() => undefined);
  };

  useEffect(() => {
    const unlisteners: Array<() => void> = [];
    let cancelled = false;

    listen('native-player-close-requested', () => {
      void (async () => {
        if (onTerminalPlayback && (await onTerminalPlayback())) return;
        await closePlayer();
      })();
    })
      .then((fn) => {
        if (cancelled) fn();
        else unlisteners.push(fn);
      })
      .catch(() => undefined);

    listen<string>('native-player-error', (event) => {
      void (async () => {
        await saveProgressOnEvent?.();
        const proxyDetail = await playerLastStreamError();
        const message = proxyDetail ? t('player.source_error_detail', proxyDetail) : presentNativePlayerError(event.payload);
        await onPlayerError(message);
      })();
    })
      .then((fn) => {
        if (cancelled) fn();
        else unlisteners.push(fn);
      })
      .catch(() => undefined);

    return () => {
      cancelled = true;
      unlisteners.forEach((fn) => fn());
    };
  }, [closePlayer, onPlayerError, onTerminalPlayback, saveProgressOnEvent]);

  useEffect(() => {
    const commands: Record<string, string> = {
      play: 'set pause no',
      pause: 'set pause yes',
      stop: 'stop',
      'volume-up': 'add volume 5',
      'volume-down': 'add volume -5',
      mute: 'cycle mute',
      'skip-forward': 'seek 60 relative',
      'skip-backward': 'seek -60 relative',
    };
    let cancelled = false;
    let unlisten: (() => void) | null = null;
    listen<string>('native-cec-command', (event) => {
      const command = commands[event.payload];
      if (command) void invoke('player_command', { command }).catch(() => undefined);
    })
      .then((fn) => {
        if (cancelled) fn();
        else unlisten = fn;
      })
      .catch(() => undefined);
    return () => {
      cancelled = true;
      unlisten?.();
    };
  }, []);

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
          debugLog(`player-debug:nextEpisode:start from=${currentEp?.id ?? 'none'} to=${nextEp.id}`);

          showEpisodeTransitionLoading(meta, nextEp, currentStream);
          await stopScrobbleForOutgoingEpisode();
          await embeddedMpvStop().catch(() => undefined);
          const nextTitle = playerDisplayTitle(meta, nextEp, currentStream);
          const nextArtwork = playerArtwork(meta, nextEp);
          await embeddedMpvSetTitle(nextTitle.contentTitle, nextTitle.episodeLine).catch(() => undefined);
          await embeddedMpvSetLoadingArtwork(
            nextTitle.contentTitle ?? 'Fluxa',
            nextTitle.episodeLine,
            nextArtwork.background,
            nextArtwork.logo,
          ).catch(() => undefined);
          await embeddedMpvShowLoading(nextTitle.contentTitle, nextTitle.episodeLine).catch(() => undefined);

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
                debugLog(`player-debug:nextEpisode:streams episode=${nextEp.id} count=${streams.length}`);
                chosenStream = (await coreSelectNextEpisodeStream(
                  JSON.stringify(streams),
                  JSON.stringify(currentStream),
                  JSON.stringify(prefs),
                  nextEp.id,
                )) as Stream | null;
              }
            } catch {}
          }
          if (!chosenStream) {
            if (!closingPlayerRef.current && onEpisodePlaybackFailed)
              await onEpisodePlaybackFailed(meta, nextEp, t('player.no_playable_url'));
            else if (!closingPlayerRef.current) await onPlayerError(t('player.no_playable_url'));
            return;
          }
          try {
            debugLog(`player-debug:nextEpisode:play episode=${nextEp.id} stream=${chosenStream.url?.slice(0, 100) ?? 'none'} candidates=${sourceCandidates?.length ?? 0}`);
            await handlePlay(chosenStream, meta, nextEp, undefined, undefined, sourceCandidates, true);
          } catch {}
        } finally {
          episodeTransitionActiveRef.current = false;
        }
      })();
    })
      .then((fn) => {
        if (cancelled) fn();
        else unlisteners.push(fn);
      })
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
      debugLog(`player-debug:episodeSwitch:start from=${playingEpisodeRef.current?.id ?? 'none'} to=${ep.id}`);

      void (async () => {
        try {
          showEpisodeTransitionLoading(meta, ep, currentStream);
          await stopScrobbleForOutgoingEpisode();
          await embeddedMpvStop().catch(() => undefined);
          const nextTitle = playerDisplayTitle(meta, ep, currentStream);
          const nextArtwork = playerArtwork(meta, ep);
          await embeddedMpvSetTitle(nextTitle.contentTitle, nextTitle.episodeLine).catch(() => undefined);
          await embeddedMpvSetLoadingArtwork(
            nextTitle.contentTitle ?? 'Fluxa',
            nextTitle.episodeLine,
            nextArtwork.background,
            nextArtwork.logo,
          ).catch(() => undefined);
          await embeddedMpvShowLoading(nextTitle.contentTitle, nextTitle.episodeLine).catch(() => undefined);

          const prefs = appPrefs(stateRef.current);
          let chosenStream: Stream | null = null;
          let sourceCandidates: Stream[] | undefined;
          try {
            const result = await fetchStreamsForEpisode(ep.id, meta.type);
            const streams = result.streams as Stream[];
            if (streams.length > 0) {
              sourceCandidates = streams;
              debugLog(`player-debug:episodeSwitch:streams episode=${ep.id} count=${streams.length}`);
              chosenStream = (await coreSelectNextEpisodeStream(
                JSON.stringify(streams),
                JSON.stringify(currentStream),
                JSON.stringify(prefs),
                ep.id,
              )) as Stream | null;
            }
          } catch {}
          if (!chosenStream) {
            if (!closingPlayerRef.current && onEpisodePlaybackFailed) await onEpisodePlaybackFailed(meta, ep, t('player.no_playable_url'));
            else if (!closingPlayerRef.current) await onPlayerError(t('player.no_playable_url'));
            return;
          }
          try {
            debugLog(`player-debug:episodeSwitch:play episode=${ep.id} stream=${chosenStream.url?.slice(0, 100) ?? 'none'} candidates=${sourceCandidates?.length ?? 0}`);
            await handlePlay(chosenStream, meta, ep, undefined, undefined, sourceCandidates, true);
          } catch {}
        } finally {
          episodeTransitionActiveRef.current = false;
        }
      })();
    })
      .then((fn) => {
        if (cancelled) fn();
        else unlisteners.push(fn);
      })
      .catch(() => undefined);

    return () => {
      cancelled = true;
      unlisteners.forEach((fn) => fn());
    };
  }, [
    handlePlay,
    stateRef,
    closingPlayerRef,
    playingMetaRef,
    playingStreamRef,
    playingEpisodeRef,
    playingNextEpisodeRef,
    prefetchedNextEpRef,
    episodeTransitionActiveRef,
    showEpisodeTransitionLoading,
    onEpisodePlaybackFailed,
    scrobbleStartedRef,
    dispatchScrobbleLifecycle,
  ]);
}
