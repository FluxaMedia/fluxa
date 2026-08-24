import { useEffect, useRef, useState } from 'react';
import { httpFetchText } from '../core/engine';
import { resolveYoutubeTrailer, type YoutubeTrailerSubtitleTrack } from '../core/effectRunner';
import { normalizeTrailerSubtitleUrl, parseTrailerSubtitleCues, selectTrailerSubtitle, type TrailerCue } from '../core/trailerSubtitles';

const STALL_TIMEOUT_MS = 7000;

export function useTrailerPlayback({
  metaId,
  trailerVideoIds,
  autoplay,
  autoplayDelaySecs,
  preferredSubtitleLanguage,
  secondarySubtitleLanguage,
  isActive = true,
}: {
  metaId: string;
  trailerVideoIds: string[];
  autoplay: boolean;
  autoplayDelaySecs: number;
  preferredSubtitleLanguage?: string;
  secondarySubtitleLanguage?: string;
  isActive?: boolean;
}) {
  const [trailerStreamUrl, setTrailerStreamUrl] = useState<string | null>(null);
  const [trailerAudioUrl, setTrailerAudioUrl] = useState<string | null>(null);
  const [trailerSubtitles, setTrailerSubtitles] = useState<YoutubeTrailerSubtitleTrack[]>([]);
  const [trailerSubtitleCues, setTrailerSubtitleCues] = useState<TrailerCue[]>([]);
  const [activeTrailerSubtitle, setActiveTrailerSubtitle] = useState('');
  const [trailerReady, setTrailerReady] = useState(false);
  const [trailerResolving, setTrailerResolving] = useState(false);
  const [trailerLoading, setTrailerLoading] = useState(false);
  const [trailerMuted, setTrailerMuted] = useState(true);
  const trailerProgressElRef = useRef<HTMLSpanElement | null>(null);
  const lastTrailerProgressAtRef = useRef(0);
  const trailerVideoRef = useRef<HTMLVideoElement | null>(null);
  const trailerAudioRef = useRef<HTMLAudioElement | null>(null);
  const trailerContainerRef = useRef<HTMLDivElement | null>(null);
  const activeTrailerSubtitleRef = useRef('');
  const trailerActive = !!trailerStreamUrl && trailerReady;
  const trailerPending = trailerResolving || trailerLoading || !!trailerStreamUrl;
  const [selectedTrailerSubtitle, setSelectedTrailerSubtitle] = useState<YoutubeTrailerSubtitleTrack | null>(null);
  const [manualStart, setManualStart] = useState(false);
  const canPlayTrailer = trailerVideoIds.length > 0;

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
    if (trailerProgressElRef.current) trailerProgressElRef.current.style.width = '0%';
    setTrailerResolving(false);
    setTrailerLoading(false);
    setTrailerMuted(true);
    setManualStart(false);
  }, [metaId]);

  useEffect(() => {
    if ((!autoplay && !manualStart) || !isActive || trailerVideoIds.length === 0) return;
    let cancelled = false;
    let delayElapsed = manualStart || autoplayDelaySecs <= 0;
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
    }, manualStart ? 0 : autoplayDelaySecs * 1000);

    (async () => {
      for (const id of trailerVideoIds) {
        if (cancelled) return;
        try {
          const resolved = await resolveYoutubeTrailer(id);
          if (cancelled) return;
          if (resolved?.streamUrl) {
            resolvedTrailer = resolved;
            break;
          }
        } catch (err) {
          console.error('resolveYoutubeTrailerUrl failed', err);
        }
      }
      if (cancelled) return;
      resolveFinished = true;
      applyResolvedTrailer();
    })();

    return () => {
      cancelled = true;
      setTrailerResolving(false);
      window.clearTimeout(delayId);
    };
  }, [trailerVideoIds, autoplay, autoplayDelaySecs, isActive, manualStart]);

  useEffect(() => {
    if (isActive) return;
    trailerAudioRef.current?.pause();
    setTrailerStreamUrl(null);
    setTrailerAudioUrl(null);
    setTrailerLoading(false);
  }, [isActive]);

  useEffect(() => {
    let cancelled = false;
    setTrailerSubtitleCues([]);
    setActiveTrailerSubtitle('');
    activeTrailerSubtitleRef.current = '';
    if (!selectedTrailerSubtitle?.url || !trailerStreamUrl) return;

    normalizeTrailerSubtitleUrl(selectedTrailerSubtitle.url)
      .then(httpFetchText)
      .then(async (response) => {
        if (cancelled || response.statusCode < 200 || response.statusCode > 299 || !response.body.trim()) return;
        const cues = await parseTrailerSubtitleCues(response.body);
        if (cancelled) return;
        setTrailerSubtitleCues(cues);
        updateActiveTrailerSubtitle(trailerVideoRef.current?.currentTime ?? 0, cues);
      })
      .catch(() => undefined);

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
    const observer = new IntersectionObserver(
      (entries) => {
        const entry = entries[0];
        if (entry?.isIntersecting && el.paused && !el.ended) {
          el.play().catch(() => {});
        }
      },
      { threshold: 0.05 },
    );
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
    if (el.duration > 0 && trailerProgressElRef.current) {
      trailerProgressElRef.current.style.width = `${(el.currentTime / el.duration) * 100}%`;
    }
    syncTrailerAudio(false);
    updateActiveTrailerSubtitle(el.currentTime);
  };

  const handleTrailerStopped = () => {
    trailerAudioRef.current?.pause();
    setTrailerStreamUrl(null);
    setTrailerAudioUrl(null);
    setTrailerLoading(false);
    setManualStart(false);
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
    const request =
      fullscreenTarget.requestFullscreen?.bind(fullscreenTarget) ?? fullscreenTarget.webkitRequestFullscreen?.bind(fullscreenTarget);
    try {
      const result = request?.();
      if (result && typeof result.catch === 'function') result.catch(() => {});
    } catch {}
  };

  function startTrailer() {
    if (!canPlayTrailer) return;
    setTrailerMuted(false);
    setManualStart(true);
  }

  return {
    canPlayTrailer,
    startTrailer,
    trailerContainerRef,
    trailerVideoRef,
    trailerAudioRef,
    trailerStreamUrl,
    trailerAudioUrl,
    trailerReady,
    trailerActive,
    trailerPending,
    trailerProgressElRef,
    trailerMuted,
    activeTrailerSubtitle,
    handleTrailerPlaying,
    handleTrailerTimeUpdate,
    handleTrailerStopped,
    toggleTrailerMute,
    fullscreenTrailer,
  };
}
