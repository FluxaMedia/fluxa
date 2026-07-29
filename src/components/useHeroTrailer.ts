import { useEffect, useRef, useState } from 'react';
import { youtubeVideoId } from './detail/TrailerCarousel';
import { httpFetchText } from '../core/engine';
import { resolveYoutubeTrailer, type YoutubeTrailerSubtitleTrack } from '../core/effectRunner';
import { normalizeTrailerSubtitleUrl, parseTrailerSubtitleCues, selectTrailerSubtitle, type TrailerCue } from '../core/trailerSubtitles';
import type { Meta } from '../core/types';

const STALL_TIMEOUT_MS = 7000;

export function useHeroTrailer({
  activeMeta,
  isActive,
  autoplayTrailer,
  autoplayTrailerDelaySecs,
  preferredSubtitleLanguage,
  secondarySubtitleLanguage,
}: {
  activeMeta: Meta;
  isActive: boolean;
  autoplayTrailer: boolean;
  autoplayTrailerDelaySecs: number;
  preferredSubtitleLanguage?: string;
  secondarySubtitleLanguage?: string;
}) {
  const trailerVideoId = (() => {
    for (const trailer of activeMeta.trailers ?? []) {
      const id = youtubeVideoId(trailer.url);
      if (id) return id;
    }
    return null;
  })();

  const [trailerStreamUrl, setTrailerStreamUrl] = useState<string | null>(null);
  const [trailerAudioUrl, setTrailerAudioUrl] = useState<string | null>(null);
  const [trailerSubtitles, setTrailerSubtitles] = useState<YoutubeTrailerSubtitleTrack[]>([]);
  const [trailerSubtitleCues, setTrailerSubtitleCues] = useState<TrailerCue[]>([]);
  const [activeTrailerSubtitle, setActiveTrailerSubtitle] = useState('');
  const [trailerReady, setTrailerReady] = useState(false);
  const [trailerResolving, setTrailerResolving] = useState(false);
  const [trailerLoading, setTrailerLoading] = useState(false);
  const [trailerProgress, setTrailerProgress] = useState(0);
  const [trailerMuted, setTrailerMuted] = useState(true);
  const lastTrailerProgressAtRef = useRef(0);
  const trailerVideoRef = useRef<HTMLVideoElement | null>(null);
  const trailerAudioRef = useRef<HTMLAudioElement | null>(null);
  const trailerContainerRef = useRef<HTMLDivElement | null>(null);
  const activeTrailerSubtitleRef = useRef('');
  const trailerActive = !!trailerStreamUrl && trailerReady;
  const trailerPending = trailerResolving || trailerLoading || !!trailerStreamUrl;
  const [selectedTrailerSubtitle, setSelectedTrailerSubtitle] = useState<YoutubeTrailerSubtitleTrack | null>(null);

  useEffect(() => {
    let cancelled = false;
    selectTrailerSubtitle(trailerSubtitles, preferredSubtitleLanguage, secondarySubtitleLanguage).then((track) => {
      if (!cancelled) setSelectedTrailerSubtitle(track);
    });
    return () => {
      cancelled = true;
    };
  }, [trailerSubtitles, preferredSubtitleLanguage, secondarySubtitleLanguage]);

  useEffect(() => {
    setTrailerStreamUrl(null);
    setTrailerAudioUrl(null);
    setTrailerSubtitles([]);
    setTrailerSubtitleCues([]);
    setActiveTrailerSubtitle('');
    activeTrailerSubtitleRef.current = '';
    setTrailerReady(false);
    setTrailerProgress(0);
    setTrailerResolving(false);
    setTrailerLoading(false);
    setTrailerMuted(true);
  }, [activeMeta.id]);

  useEffect(() => {
    if (!autoplayTrailer || !isActive || !trailerVideoId) return;
    let cancelled = false;
    let delayElapsed = autoplayTrailerDelaySecs <= 0;
    let resolvedTrailer: Awaited<ReturnType<typeof resolveYoutubeTrailer>> | null = null;
    let resolveFinished = false;
    setTrailerResolving(true);

    const applyResolvedTrailer = () => {
      if (cancelled || !delayElapsed || !resolveFinished) return;
      if (resolvedTrailer?.streamUrl) {
        setTrailerSubtitles(resolvedTrailer.subtitles ?? []);
        setTrailerAudioUrl(resolvedTrailer.audioUrl ?? null);
        setTrailerReady(false);
        setTrailerLoading(true);
        setTrailerStreamUrl(resolvedTrailer.streamUrl);
      }
      setTrailerResolving(false);
      if (!resolvedTrailer?.streamUrl) setTrailerLoading(false);
    };

    const delayId = window.setTimeout(() => {
      delayElapsed = true;
      if (!resolveFinished) setTrailerLoading(true);
      applyResolvedTrailer();
    }, autoplayTrailerDelaySecs * 1000);

    resolveYoutubeTrailer(trailerVideoId).then((resolved) => {
      if (cancelled) return;
      resolvedTrailer = resolved;
      resolveFinished = true;
      applyResolvedTrailer();
    }).catch((err) => {
      console.error('resolveYoutubeTrailerUrl failed', err);
      resolveFinished = true;
      if (!cancelled) {
        setTrailerResolving(false);
        if (delayElapsed) setTrailerLoading(false);
      }
    });

    return () => {
      cancelled = true;
      setTrailerResolving(false);
      window.clearTimeout(delayId);
    };
  }, [trailerVideoId, autoplayTrailer, autoplayTrailerDelaySecs, isActive]);

  useEffect(() => {
    let cancelled = false;
    setTrailerSubtitleCues([]);
    setActiveTrailerSubtitle('');
    activeTrailerSubtitleRef.current = '';
    if (!selectedTrailerSubtitle?.url || !trailerStreamUrl) return;

    normalizeTrailerSubtitleUrl(selectedTrailerSubtitle.url).then(httpFetchText).then(async (response) => {
      if (cancelled || response.statusCode < 200 || response.statusCode > 299 || !response.body.trim()) return;
      const cues = await parseTrailerSubtitleCues(response.body);
      if (cancelled) return;
      setTrailerSubtitleCues(cues);
      updateActiveTrailerSubtitle(trailerVideoRef.current?.currentTime ?? 0, cues);
    }).catch(() => undefined);

    return () => {
      cancelled = true;
    };
  }, [selectedTrailerSubtitle?.url, trailerStreamUrl]);

  function updateActiveTrailerSubtitle(time: number, cues = trailerSubtitleCues) {
    const text = cues.find((cue) => time >= cue.start && time <= cue.end)?.text ?? '';
    if (text !== activeTrailerSubtitleRef.current) {
      activeTrailerSubtitleRef.current = text;
      setActiveTrailerSubtitle(text);
    }
  }

  function syncTrailerAudio(shouldPlay = false) {
    if (!trailerAudioUrl) return;
    const video = trailerVideoRef.current;
    const audio = trailerAudioRef.current;
    if (!video || !audio) return;
    if (Number.isFinite(video.currentTime) && Math.abs(audio.currentTime - video.currentTime) > 0.35) {
      audio.currentTime = video.currentTime;
    }
    audio.muted = trailerMuted;
    audio.volume = trailerMuted ? 0 : 1;
    if (trailerMuted || video.paused || video.ended) {
      audio.pause();
    } else if (shouldPlay || audio.paused) {
      audio.play().catch(() => {});
    }
  }

  useEffect(() => {
    if (!trailerStreamUrl) return;
    lastTrailerProgressAtRef.current = Date.now();
    const id = window.setInterval(() => {
      if (Date.now() - lastTrailerProgressAtRef.current > STALL_TIMEOUT_MS) {
        setTrailerStreamUrl(null);
        setTrailerLoading(false);
      }
    }, 2000);
    return () => window.clearInterval(id);
  }, [trailerStreamUrl]);

  useEffect(() => {
    if (!trailerStreamUrl) return;
    const el = trailerVideoRef.current;
    if (!el) return;
    const observer = new IntersectionObserver((entries) => {
      const entry = entries[0];
      if (entry?.isIntersecting && el.paused && !el.ended) {
        el.play().catch(() => {});
      }
    }, { threshold: 0.05 });
    observer.observe(el);
    return () => observer.disconnect();
  }, [trailerStreamUrl]);

  useEffect(() => {
    const el = trailerVideoRef.current;
    if (!el) return;
    el.muted = trailerMuted;
    el.volume = trailerMuted ? 0 : 1;
    syncTrailerAudio(!trailerMuted);
  }, [trailerMuted, trailerAudioUrl]);

  const handleTrailerPlaying = () => {
    setTrailerReady(true);
    setTrailerLoading(false);
    lastTrailerProgressAtRef.current = Date.now();
    if (trailerVideoRef.current) {
      trailerVideoRef.current.muted = trailerMuted;
      trailerVideoRef.current.volume = trailerMuted ? 0 : 1;
    }
    syncTrailerAudio(true);
    updateActiveTrailerSubtitle(trailerVideoRef.current?.currentTime ?? 0);
  };

  const handleTrailerTimeUpdate = (el: HTMLVideoElement) => {
    lastTrailerProgressAtRef.current = Date.now();
    if (el.duration > 0) setTrailerProgress(el.currentTime / el.duration);
    syncTrailerAudio(false);
    updateActiveTrailerSubtitle(el.currentTime);
  };

  const handleTrailerStopped = () => {
    trailerAudioRef.current?.pause();
    setTrailerStreamUrl(null);
    setTrailerAudioUrl(null);
    setTrailerLoading(false);
  };

  const toggleTrailerMute = () => {
    const newMutedState = !trailerMuted;
    setTrailerMuted(newMutedState);
    if (trailerVideoRef.current) {
      trailerVideoRef.current.muted = newMutedState;
      trailerVideoRef.current.volume = newMutedState ? 0 : 1;
      if (!newMutedState && trailerVideoRef.current.paused) {
        trailerVideoRef.current.play().catch(() => {});
      }
    }
    if (trailerAudioRef.current && trailerVideoRef.current) {
      trailerAudioRef.current.muted = newMutedState;
      trailerAudioRef.current.volume = newMutedState ? 0 : 1;
      if (newMutedState) {
        trailerAudioRef.current.pause();
      } else {
        trailerAudioRef.current.currentTime = trailerVideoRef.current.currentTime;
        trailerAudioRef.current.play().catch(() => {});
      }
    }
  };

  const fullscreenTrailer = () => {
    const container = trailerContainerRef.current;
    if (!container) return;
    const fullscreenTarget = container as HTMLDivElement & {
      webkitRequestFullscreen?: () => Promise<void> | void;
    };
    const request = fullscreenTarget.requestFullscreen?.bind(fullscreenTarget)
      ?? fullscreenTarget.webkitRequestFullscreen?.bind(fullscreenTarget);
    try {
      const result = request?.();
      if (result && typeof result.catch === 'function') result.catch(() => {});
    } catch {}
  };

  return {
    trailerContainerRef, trailerVideoRef, trailerAudioRef,
    trailerStreamUrl, trailerAudioUrl, trailerReady, trailerActive, trailerPending, trailerProgress, trailerMuted, activeTrailerSubtitle,
    handleTrailerPlaying, handleTrailerTimeUpdate, handleTrailerStopped, toggleTrailerMute, fullscreenTrailer,
  };
}
