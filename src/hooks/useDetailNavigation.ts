import { useCallback, useRef, useState } from 'react';
import { prefetchPlayerArtwork } from '../core/mpvPlayer';
import { playerArtwork } from '../core/playerUtils';
import { readStoredPlaybackSource } from '../core/libraryStorage';
import type { LibraryItem, Meta, Stream, Video } from '../core/types';

type GuardedPlay = (
  stream: Stream,
  meta: Meta,
  episode: Video | null | undefined,
  resumeAt?: number,
  totalDuration?: number,
  sourceCandidates?: Stream[],
) => Promise<void>;

export function useDetailNavigation() {
  const [detailMeta, setDetailMeta] = useState<Meta | null>(null);
  const [detailInitialEpisode, setDetailInitialEpisode] = useState<Video | null>(null);
  const [detailAutoShowStreams, setDetailAutoShowStreams] = useState(false);
  const [detailResumeAt, setDetailResumeAt] = useState<number | undefined>(undefined);
  const [detailPlaybackError, setDetailPlaybackError] = useState<string | null>(null);
  const [discoverInitialGenre, setDiscoverInitialGenre] = useState<string | null>(null);
  const artworkPrefetchRef = useRef<Promise<unknown> | null>(null);
  const guardedPlayRef = useRef<GuardedPlay>(async () => {});

  const prefetchArtworkFor = (meta: Meta, episode: Video | null) => {
    const art = playerArtwork(meta, episode);
    artworkPrefetchRef.current = prefetchPlayerArtwork(art.background, art.logo).catch(() => undefined);
    if (art.background) { const i = new Image(); i.src = art.background; }
    if (art.logo) { const i = new Image(); i.src = art.logo; }
  };

  const handleNavigateDetail = useCallback((meta: Meta) => {
    setDetailInitialEpisode(null);
    setDetailAutoShowStreams(false);
    setDetailMeta(meta);
    prefetchArtworkFor(meta, null);
  }, []);

  const handleResumeFromContinueWatching = useCallback((meta: Meta, resumeAtOverride?: number) => {
    const item = meta as LibraryItem;
    const matchedVideo = item.lastVideoId ? meta.videos?.find((v) => v.id === item.lastVideoId) : undefined;
    const episode: Video | null = item.lastVideoId ? {
      id: item.lastVideoId,
      name: matchedVideo?.name ?? matchedVideo?.title ?? item.lastEpisodeName,
      season: matchedVideo?.season ?? item.lastEpisodeSeason,
      episode: matchedVideo?.episode ?? matchedVideo?.number ?? item.lastEpisodeNumber,
      number: matchedVideo?.episode ?? matchedVideo?.number ?? item.lastEpisodeNumber,
      thumbnail: matchedVideo?.thumbnail ?? item.lastEpisodeThumbnail,
    } : null;
    const resumeAt = resumeAtOverride ?? item.timeOffset;

    prefetchArtworkFor(meta, episode);

    void (async () => {
      const stream = item.lastStream ?? await readStoredPlaybackSource(meta.id);
      const url = item.lastStreamUrl?.trim();
      const resumeStream: Stream | null = stream ?? (url
        ? { url, title: item.lastStreamTitle, name: item.lastStreamTitle }
        : null);

      if (!resumeStream) {
        setDetailInitialEpisode(episode);
        setDetailAutoShowStreams(true);
        setDetailResumeAt(resumeAt ?? undefined);
        setDetailMeta(meta);
        return;
      }
      await guardedPlayRef.current(resumeStream, meta, episode, resumeAt, item.duration);
    })();
  }, []);

  const handleStartOverContinueWatching = useCallback((meta: Meta) => {
    handleResumeFromContinueWatching(meta, 0);
  }, [handleResumeFromContinueWatching]);

  const handlePlayManually = useCallback((meta: Meta) => {
    const item = meta as LibraryItem;
    const episode: Video | null = item.lastVideoId ? {
      id: item.lastVideoId,
      name: item.lastEpisodeName,
      season: item.lastEpisodeSeason,
      episode: item.lastEpisodeNumber,
      number: item.lastEpisodeNumber,
      thumbnail: item.lastEpisodeThumbnail,
    } : null;
    setDetailInitialEpisode(episode);
    setDetailAutoShowStreams(true);
    setDetailResumeAt(item.timeOffset ?? undefined);
    setDetailMeta(meta);
  }, []);

  const resetDetail = useCallback(() => {
    setDetailMeta(null);
    setDetailInitialEpisode(null);
    setDetailAutoShowStreams(false);
    setDetailResumeAt(undefined);
  }, []);

  return {
    detailMeta, setDetailMeta,
    detailInitialEpisode, setDetailInitialEpisode,
    detailAutoShowStreams, setDetailAutoShowStreams,
    detailResumeAt, setDetailResumeAt,
    detailPlaybackError, setDetailPlaybackError,
    discoverInitialGenre, setDiscoverInitialGenre,
    guardedPlayRef,
    handleNavigateDetail,
    handleResumeFromContinueWatching,
    handleStartOverContinueWatching,
    handlePlayManually,
    resetDetail,
  };
}
