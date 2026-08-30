import { useCallback, useRef, useState } from 'react';
import { prefetchPlayerArtwork } from '../core/mpvPlayer';
import { playerArtwork } from '../core/playerUtils';
import { readStoredPlaybackSource } from '../core/libraryStorage';
import { coreContinueWatchingResumePlan } from '../core/engineCoreLibrary';
import type { LibraryItem, Meta, Stream, Video } from '../core/types';

type GuardedPlay = (
  stream: Stream,
  meta: Meta,
  episode: Video | null | undefined,
  resumeAt?: number,
  totalDuration?: number,
  sourceCandidates?: Stream[],
  resumePercent?: number,
) => Promise<void>;

export function useDetailNavigation() {
  const [detailMeta, setDetailMeta] = useState<Meta | null>(null);
  const [detailInitialEpisode, setDetailInitialEpisode] = useState<Video | null>(null);
  const [detailAutoShowStreams, setDetailAutoShowStreams] = useState(false);
  const [detailResumeAt, setDetailResumeAt] = useState<number | undefined>(undefined);
  const [detailResumePercent, setDetailResumePercent] = useState<number | undefined>(undefined);
  const [detailPlaybackError, setDetailPlaybackError] = useState<string | null>(null);
  const [discoverInitialGenre, setDiscoverInitialGenre] = useState<string | null>(null);
  const artworkPrefetchRef = useRef<Promise<unknown> | null>(null);
  const guardedPlayRef = useRef<GuardedPlay>(async () => {});

  const prefetchArtworkFor = (meta: Meta, episode: Video | null) => {
    const art = playerArtwork(meta, episode);
    artworkPrefetchRef.current = prefetchPlayerArtwork(art.background, art.logo).catch(() => undefined);
    if (art.background) {
      const i = new Image();
      i.src = art.background;
    }
    if (art.logo) {
      const i = new Image();
      i.src = art.logo;
    }
  };

  const handleNavigateDetail = useCallback((meta: Meta) => {
    setDetailInitialEpisode(null);
    setDetailAutoShowStreams(false);
    setDetailMeta(meta);
    prefetchArtworkFor(meta, null);
  }, []);

  const handleResumeFromContinueWatching = useCallback(async (meta: Meta, resumeAtOverride?: number) => {
    const item = meta as LibraryItem;
    const resumePlan = await coreContinueWatchingResumePlan({ item, videos: meta.videos ?? [], resumeAtOverride });
    const episode = (resumePlan.episode as Video | null) ?? null;
    const resumePercent = resumePlan.resumePercent ?? undefined;
    const resumeAt = resumePlan.resumeAt ?? undefined;

    prefetchArtworkFor(meta, episode);

    void (async () => {
      try {
        const stream = item.lastStream ?? (await readStoredPlaybackSource(meta.id));
        const url = item.lastStreamUrl?.trim();
        const resumeStream: Stream | null = stream ?? (url ? { url, title: item.lastStreamTitle, name: item.lastStreamTitle } : null);
        if (resumeStream) {
          await guardedPlayRef.current(resumeStream, meta, episode, resumeAt, item.duration, undefined, resumePercent);
        } else {
          setDetailInitialEpisode(episode);
          setDetailAutoShowStreams(true);
          setDetailResumeAt(resumeAt ?? undefined);
          setDetailResumePercent(resumePercent);
          setDetailMeta(meta);
        }
      } catch {
        setDetailInitialEpisode(episode);
        setDetailAutoShowStreams(true);
        setDetailResumeAt(resumeAt ?? undefined);
        setDetailResumePercent(resumePercent);
        setDetailMeta(meta);
      }
    })();
  }, []);

  const handleStartOverContinueWatching = useCallback(
    (meta: Meta) => {
      handleResumeFromContinueWatching(meta, 0);
    },
    [handleResumeFromContinueWatching],
  );

  const handlePlayManually = useCallback(async (meta: Meta) => {
    const item = meta as LibraryItem;
    const resumePlan = await coreContinueWatchingResumePlan({ item, videos: meta.videos ?? [] });
    const episode = (resumePlan.episode as Video | null) ?? null;
    const resumePercent = resumePlan.resumePercent ?? undefined;
    setDetailInitialEpisode(episode);
    setDetailAutoShowStreams(true);
    setDetailResumeAt(resumePlan.resumeAt ?? undefined);
    setDetailResumePercent(resumePercent);
    setDetailMeta(meta);
  }, []);

  const resetDetail = useCallback(() => {
    setDetailMeta(null);
    setDetailInitialEpisode(null);
    setDetailAutoShowStreams(false);
    setDetailResumeAt(undefined);
    setDetailResumePercent(undefined);
  }, []);

  return {
    detailMeta,
    setDetailMeta,
    detailInitialEpisode,
    setDetailInitialEpisode,
    detailAutoShowStreams,
    setDetailAutoShowStreams,
    detailResumeAt,
    setDetailResumeAt,
    detailResumePercent,
    setDetailResumePercent,
    detailPlaybackError,
    setDetailPlaybackError,
    discoverInitialGenre,
    setDiscoverInitialGenre,
    guardedPlayRef,
    handleNavigateDetail,
    handleResumeFromContinueWatching,
    handleStartOverContinueWatching,
    handlePlayManually,
    resetDetail,
  };
}
