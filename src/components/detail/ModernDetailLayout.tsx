import React, { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { ArrowLeft, Bookmark, BookmarkCheck, Check, CheckCircle2, ChevronDown, Circle, Film, Maximize2, MessageCircle, Volume2, VolumeX, XCircle } from 'lucide-react';
import { open as shellOpen } from '@tauri-apps/plugin-shell';
import { MovieCard } from '../MovieCard';
import { getLanguage, t } from '../../i18n';
import type { DetailState, LibraryItem, Meta, MetaLink, Stream, Trailer, Video } from '../../core/types';
import type { posterPrefsFromState } from '../../core/posterPrefs';
import { MS, S, spinnerStyle } from './detailStyles';
import { CastAvatar, type NormalizedCastMember } from './castSection';
import { TrailerCarousel, youtubeVideoId, type TrailerMetadata } from './TrailerCarousel';
import { InlineSourceList, MovieSourcePanel } from './SourcePanel';
import { SeasonDropdown, seasonLabel, formatEpDate as _formatEpDate, type ProgressEntry } from './EpisodePanel';
import { ModernIconBtn, ModernPlayButton, ModernTabBar } from './DetailButtons';
import { ModernEpisodeCard } from './ModernEpisodeCard';
import { useSeasonWatched } from '../../hooks/useSeasonWatched';
import { httpFetchText } from '../../core/engine';
import { resolveYoutubeTrailer, type YoutubeTrailerSubtitleTrack } from '../../core/effectRunner';
import { normalizeTrailerSubtitleUrl, parseTrailerSubtitleCues, selectTrailerSubtitle, type TrailerCue } from '../../core/trailerSubtitles';
import { RatingsRow } from './RatingBadge';

const STALL_TIMEOUT_MS = 7000;

export type ModernDetailProps = {
  displayMeta: Meta;
  bgUrl: string | null | undefined;
  isSeries: boolean;
  detail: DetailState;
  meta: Meta;
  episodes: Video[];
  filteredEps: Video[];
  seasonNumbers: number[];
  selectedSeason: number;
  selectedEpisode: Video | null;
  showSources: boolean;
  playbackFailure?: string | null;
  streams: Stream[];
  episodePlan: { seasonNumbers?: number[]; selectedSeason?: number; episodes?: Video[]; selectedEpisode?: Video | null } | null;
  similarItems: Meta[];
  displayTrailers: Trailer[];
  trailerMetadata: TrailerMetadata;
  castMembers: NormalizedCastMember[];
  directorLinks: MetaLink[];
  peopleImages: Record<string, string>;
  watchedMap: Record<string, boolean>;
  progressMap: Record<string, ProgressEntry>;
  continueWatchingEntry?: LibraryItem | null;
  trailerOnHero: boolean;
  detailHeroAutoplayTrailer: boolean;
  detailHeroAutoplayTrailerDelaySecs: number;
  preferredSubtitleLanguage?: string;
  secondarySubtitleLanguage?: string;
  blurUnwatchedEpisodes: boolean;
  spoilerHideEpisodeInfo: boolean;
  detailSeasonSelectorMode: string;
  episodeCardsLayout: string;
  isInWatchlist: boolean;
  isDropped: boolean;
  isCompleted: boolean;
  omdbRatings?: { rottenTomatoes?: string; metascore?: string } | null;
  mdblistRatings?: Record<string, number> | null;
  fanartArtwork?: { hdLogo?: string } | null;
  availableAddons: string[];
  streamAddonCount: number;
  poster: ReturnType<typeof posterPrefsFromState>;
  onBack: () => void;
  onDispatch: (actionJson: string) => void;
  onNavigateDetail: (meta: Meta) => void;
  onNavigateGenre?: (genre: string) => void;
  onSeasonChange: (season: number) => void;
  onEpisodeClick: (ep: Video) => void;
  onMovieSources: () => void;
  onRetryFailed: () => void;
  onBackToEpisodes: () => void;
  onPlaySource: (stream: Stream) => void;
  onPlay: (stream: Stream, meta: Meta, episode?: Video | null, resumeAt?: number, sourceCandidates?: Stream[]) => void;
  onToggleWatchlist: () => void;
  onToggleCompleted: () => void;
  onToggleDropped: () => void;
  onOpenComments?: () => void;
  onBgError: () => void;
};

function GenreTag({ label, onClick }: { label: string; onClick?: () => void }) {
  const [hovered, setHovered] = useState(false);
  return (
    <span
      style={{ ...MS.genreTag, textDecoration: hovered ? 'underline' : 'none' }}
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
      onClick={onClick}
      role="button"
      tabIndex={0}
      onKeyDown={(e) => { if (e.key === 'Enter' || e.key === ' ') onClick?.(); }}
    >
      {label}
    </span>
  );
}

const similarSourceOptions = ['auto', 'trakt', 'simkl', 'tmdb'] as const;

function SimilarSourcePicker({ value, onChange }: { value: string; onChange: (value: string) => void }) {
  const [open, setOpen] = useState(false);
  const pickerRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const closeOnOutsideClick = (event: MouseEvent) => {
      if (!pickerRef.current?.contains(event.target as Node)) setOpen(false);
    };
    const closeOnEscape = (event: KeyboardEvent) => {
      if (event.key === 'Escape') setOpen(false);
    };
    document.addEventListener('mousedown', closeOnOutsideClick);
    document.addEventListener('keydown', closeOnEscape);
    return () => {
      document.removeEventListener('mousedown', closeOnOutsideClick);
      document.removeEventListener('keydown', closeOnEscape);
    };
  }, []);

  const labelFor = (source: string) => t(`detail.similar_source_${source}`);

  return (
    <div ref={pickerRef} style={MS.similarSourcePicker}>
      <button
        type="button"
        aria-label={t('detail.similar_source')}
        aria-expanded={open}
        onClick={() => setOpen((current) => !current)}
        style={MS.similarSourceButton}
      >
        <span>{labelFor(value)}</span>
        <ChevronDown size={15} strokeWidth={2.5} style={{ transform: open ? 'rotate(180deg)' : undefined, transition: 'transform 0.16s ease' }} />
      </button>
      {open && (
        <div role="menu" style={MS.similarSourceMenu}>
          {similarSourceOptions.map((source) => {
            const selected = source === value;
            return (
              <button
                key={source}
                type="button"
                role="menuitemradio"
                aria-checked={selected}
                onClick={() => {
                  setOpen(false);
                  if (!selected) onChange(source);
                }}
                style={{ ...MS.similarSourceMenuItem, ...(selected ? MS.similarSourceMenuItemActive : {}) }}
              >
                <span>{labelFor(source)}</span>
                {selected && <Check size={15} strokeWidth={3} />}
              </button>
            );
          })}
        </div>
      )}
    </div>
  );
}

export function ModernDetailLayout({
  displayMeta, bgUrl, isSeries, detail, meta, episodes, filteredEps, seasonNumbers,
  selectedSeason, selectedEpisode, showSources, playbackFailure, streams, episodePlan, similarItems,
  displayTrailers, trailerMetadata, castMembers, directorLinks, peopleImages,
  watchedMap, progressMap, continueWatchingEntry, isInWatchlist, isDropped, isCompleted,
  omdbRatings, mdblistRatings, fanartArtwork, availableAddons, streamAddonCount, poster,
  trailerOnHero, detailHeroAutoplayTrailer, detailHeroAutoplayTrailerDelaySecs, preferredSubtitleLanguage, secondarySubtitleLanguage,
  blurUnwatchedEpisodes, spoilerHideEpisodeInfo, detailSeasonSelectorMode: _detailSeasonSelectorMode, episodeCardsLayout,
  onBack, onDispatch, onNavigateDetail, onNavigateGenre, onSeasonChange, onEpisodeClick,
  onMovieSources, onRetryFailed, onBackToEpisodes, onPlaySource, onPlay,
  onToggleWatchlist, onToggleCompleted, onToggleDropped, onOpenComments, onBgError,
}: ModernDetailProps) {
  const [activeTab, setActiveTab] = useState<'episodes' | 'related' | 'details'>(() => isSeries ? 'episodes' : 'details');
  const [similarSource, setSimilarSource] = useState('auto');
  const [prevSeasonDialog, setPrevSeasonDialog] = useState<{ season: number; unwatchedPrev: number[] } | null>(null);

  const screenRef = useRef<HTMLDivElement | null>(null);
  const [heroInView, setHeroInView] = useState(true);
  useEffect(() => {
    const el = screenRef.current;
    if (!el) return;
    const onScroll = () => setHeroInView(el.scrollTop < 120);
    el.addEventListener('scroll', onScroll, { passive: true });
    return () => el.removeEventListener('scroll', onScroll);
  }, []);

  const bgLayerKeyRef = useRef(0);
  const [bgLayers, setBgLayers] = useState<{ url: string; key: number }[]>(() => bgUrl ? [{ url: bgUrl, key: 0 }] : []);
  const [bgLoadedKeys, setBgLoadedKeys] = useState<Set<number>>(() => new Set());
  useEffect(() => {
    setBgLayers((layers) => {
      const top = layers[layers.length - 1];
      if (top?.url === bgUrl) return layers;
      if (!bgUrl) return [];
      bgLayerKeyRef.current += 1;
      return [...layers, { url: bgUrl, key: bgLayerKeyRef.current }];
    });
  }, [bgUrl]);
  const handleBgLayerLoad = useCallback((key: number) => {
    setBgLoadedKeys((prev) => new Set(prev).add(key));
    setBgLayers((layers) => {
      const idx = layers.findIndex((layer) => layer.key === key);
      return idx <= 0 ? layers : layers.slice(idx);
    });
  }, []);

  const changeSimilarSource = (source: string) => {
    setSimilarSource(source);
    onDispatch(JSON.stringify({ type: 'detailSecondaryRequested', contentType: meta.type, id: meta.id, language: getLanguage(), similarTitlesSource: source }));
  };

  const { seasonWatchedMap, dispatchMarkSeason, toggleEpisodeWatched } = useSeasonWatched({
    meta, displayMeta, episodes, seasonNumbers, watchedMap, onDispatch,
  });

  const trailerVideoId = useMemo(() => {
    for (const trailer of displayTrailers) {
      const id = youtubeVideoId(trailer.url);
      if (id) return id;
    }
    return null;
  }, [displayTrailers]);
  const [trailerStreamUrl, setTrailerStreamUrl] = useState<string | null>(null);
  const [trailerAudioUrl, setTrailerAudioUrl] = useState<string | null>(null);
  const [trailerSubtitles, setTrailerSubtitles] = useState<YoutubeTrailerSubtitleTrack[]>([]);
  const [trailerSubtitleCues, setTrailerSubtitleCues] = useState<TrailerCue[]>([]);
  const [activeTrailerSubtitle, setActiveTrailerSubtitle] = useState('');
  const [trailerReady, setTrailerReady] = useState(false);
  const [trailerProgress, setTrailerProgress] = useState(0);
  const [trailerMuted, setTrailerMuted] = useState(true);
  const lastTrailerProgressAtRef = useRef(0);
  const trailerVideoRef = useRef<HTMLVideoElement | null>(null);
  const trailerAudioRef = useRef<HTMLAudioElement | null>(null);
  const trailerContainerRef = useRef<HTMLDivElement | null>(null);
  const activeTrailerSubtitleRef = useRef('');
  const trailerActive = !!trailerStreamUrl && trailerReady;
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
    setTrailerMuted(true);
  }, [displayMeta.id]);

  useEffect(() => {
    if (!detailHeroAutoplayTrailer || !trailerVideoId) return;
    let cancelled = false;
    let delayElapsed = detailHeroAutoplayTrailerDelaySecs <= 0;
    let resolvedTrailer: Awaited<ReturnType<typeof resolveYoutubeTrailer>> | null = null;
    let resolveFinished = false;

    const applyResolvedTrailer = () => {
      if (cancelled || !delayElapsed || !resolveFinished) return;
      if (resolvedTrailer?.streamUrl) {
        setTrailerSubtitles(resolvedTrailer.subtitles ?? []);
        setTrailerAudioUrl(resolvedTrailer.audioUrl ?? null);
        setTrailerReady(false);
        setTrailerStreamUrl(resolvedTrailer.streamUrl);
      }
    };

    const delayId = window.setTimeout(() => {
      delayElapsed = true;
      applyResolvedTrailer();
    }, detailHeroAutoplayTrailerDelaySecs * 1000);

    resolveYoutubeTrailer(trailerVideoId).then((resolved) => {
      if (cancelled) return;
      resolvedTrailer = resolved;
      resolveFinished = true;
      applyResolvedTrailer();
    }).catch((err) => {
      console.error('resolveYoutubeTrailerUrl failed', err);
    });

    return () => {
      cancelled = true;
      window.clearTimeout(delayId);
    };
  }, [trailerVideoId, detailHeroAutoplayTrailer, detailHeroAutoplayTrailerDelaySecs]);

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
      }
    }, 2000);
    return () => window.clearInterval(id);
  }, [trailerStreamUrl]);

  useEffect(() => {
    const el = trailerVideoRef.current;
    if (!el) return;
    el.muted = trailerMuted;
    el.volume = trailerMuted ? 0 : 1;
    syncTrailerAudio(!trailerMuted);
  }, [trailerMuted, trailerAudioUrl]);

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

  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if (e.key !== 'Escape') return;
      if (prevSeasonDialog) { setPrevSeasonDialog(null); return; }
      if (showSources) { onBackToEpisodes(); }
    };
    window.addEventListener('keydown', handler);
    return () => window.removeEventListener('keydown', handler);
  }, [prevSeasonDialog, showSources, onBackToEpisodes]);

  const toggleSeasonWatched = useCallback(() => {
    const isWatched = seasonWatchedMap[selectedSeason] === true;
    if (isWatched) { dispatchMarkSeason([selectedSeason], false); return; }
    const unwatchedPrev = seasonNumbers.filter((s) => s > 0 && s < selectedSeason && !seasonWatchedMap[s]);
    if (unwatchedPrev.length > 0) {
      setPrevSeasonDialog({ season: selectedSeason, unwatchedPrev });
    } else {
      dispatchMarkSeason([selectedSeason], true);
    }
  }, [selectedSeason, seasonWatchedMap, seasonNumbers, dispatchMarkSeason]);

  const continueEp = episodePlan?.selectedEpisode ?? filteredEps[0] ?? episodes[0];
  const hasProgress = episodePlan?.selectedEpisode != null;
  const continueLabel = continueEp
    ? t('format.season_episode_short', continueEp.season ?? 1, continueEp.episode ?? continueEp.number ?? 1)
    : null;

  const hasMdblistRatings = mdblistRatings != null && Object.keys(mdblistRatings).length > 0;
  const modernMetaDetails: string[] = [];
  if (!hasMdblistRatings) {
    if (displayMeta.imdbRating) modernMetaDetails.push(`IMDb ${displayMeta.imdbRating}/10`);
    if (omdbRatings?.rottenTomatoes) modernMetaDetails.push(`RT ${omdbRatings.rottenTomatoes}`);
    if (omdbRatings?.metascore) modernMetaDetails.push(`Metascore ${omdbRatings.metascore}`);
  }
  if (displayMeta.releaseInfo) modernMetaDetails.push(displayMeta.releaseInfo);
  if (displayMeta.runtime) modernMetaDetails.push(displayMeta.runtime);
  if (isSeries && seasonNumbers.length > 0) modernMetaDetails.push(`${seasonNumbers.length} ${t('auto.seasons')}`);
  const metaGenres = Array.isArray(displayMeta.genres) ? displayMeta.genres.slice(0, 3) : [];

  const heroLogo = fanartArtwork?.hdLogo || displayMeta.logo;

  const episodeGridStyle = episodeCardsLayout === 'list'
    ? { ...MS.episodeGrid, gridTemplateColumns: '1fr' }
    : MS.episodeGrid;

  const seriesTabs = [
    { id: 'episodes', label: t('auto.episodes') },
    { id: 'details', label: t('common.details') },
    { id: 'related', label: t('auto.similar_titles') },
  ];

  const movieTabs = [
    { id: 'details', label: t('common.details') },
    { id: 'related', label: t('auto.similar_titles') },
  ];

  return (
    <div style={MS.screen} ref={screenRef}>
      {bgUrl ? (
        <div style={MS.pageBgWrap}>
          {bgLayers.map((layer, index) => (
            <img
              key={layer.key}
              src={layer.url}
              alt=""
              style={{
                ...MS.pageBgImg,
                position: 'absolute',
                inset: 0,
                opacity: index === 0 || bgLoadedKeys.has(layer.key) ? 1 : 0,
              }}
              onLoad={() => handleBgLayerLoad(layer.key)}
              onError={onBgError}
            />
          ))}

          <div ref={trailerContainerRef} style={MS.heroTrailerContainer}>
            {trailerStreamUrl && (
              <video
                ref={trailerVideoRef}
                key={trailerStreamUrl}
                style={{ ...MS.heroTrailerFrame, opacity: trailerReady ? 1 : 0, transition: 'opacity 0.6s ease' }}
                src={trailerStreamUrl}
                autoPlay
                playsInline
                onPlaying={() => {
                  setTrailerReady(true);
                  lastTrailerProgressAtRef.current = Date.now();
                  if (trailerVideoRef.current) {
                    trailerVideoRef.current.muted = trailerMuted;
                    trailerVideoRef.current.volume = trailerMuted ? 0 : 1;
                  }
                  syncTrailerAudio(true);
                  updateActiveTrailerSubtitle(trailerVideoRef.current?.currentTime ?? 0);
                }}
                onTimeUpdate={(e) => {
                  const el = e.currentTarget;
                  lastTrailerProgressAtRef.current = Date.now();
                  if (el.duration > 0) setTrailerProgress(el.currentTime / el.duration);
                  syncTrailerAudio(false);
                  updateActiveTrailerSubtitle(el.currentTime);
                }}
                onEnded={() => {
                  trailerAudioRef.current?.pause();
                  setTrailerStreamUrl(null);
                  setTrailerAudioUrl(null);
                }}
                onError={() => {
                  trailerAudioRef.current?.pause();
                  setTrailerStreamUrl(null);
                  setTrailerAudioUrl(null);
                }}
              />
            )}
            {trailerAudioUrl && (
              <audio ref={trailerAudioRef} key={trailerAudioUrl} src={trailerAudioUrl} preload="auto" />
            )}
          </div>

          <div style={MS.pageBgGradLeft} />
          <div style={MS.pageBgGradBottom} />
        </div>
      ) : (
        <div style={MS.heroPlaceholder} />
      )}

      {trailerActive && heroInView && (
        <div style={MS.trailerOverlayWrap}>
          {activeTrailerSubtitle && (
            <div style={MS.heroTrailerSubtitleOverlay}>{activeTrailerSubtitle}</div>
          )}

          <button
            style={{ ...MS.heroTrailerFullscreenButton, pointerEvents: 'auto' }}
            onClick={fullscreenTrailer}
            aria-label="Fullscreen trailer"
            title="Fullscreen trailer"
          >
            <Maximize2 size={16} />
          </button>

          <button
            style={{ ...MS.heroTrailerMuteButton, pointerEvents: 'auto' }}
            onClick={() => {
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
            }}
            aria-label={trailerMuted ? 'Unmute' : 'Mute'}
          >
            {trailerMuted ? <VolumeX size={18} /> : <Volume2 size={18} />}
          </button>
          <div style={MS.heroTrailerProgressTrack}>
            <span style={{ ...MS.heroTrailerProgressFill, width: `${trailerProgress * 100}%` }} />
          </div>
        </div>
      )}

      <div style={MS.heroWrap}>
        <button style={MS.backBtn} onClick={onBack}>
          <ArrowLeft size={18} color="rgba(255,255,255,0.85)" />
        </button>

        <div style={{ ...MS.logoWrap, opacity: trailerActive ? 0 : 1, transition: 'opacity 0.4s ease' }}>
          {heroLogo ? (
            <img src={heroLogo} alt={displayMeta.name} style={MS.logo} onError={(e) => { (e.currentTarget as HTMLImageElement).style.display = 'none'; }} />
          ) : (
            <h1 style={MS.titleHero}>{displayMeta.name}</h1>
          )}
        </div>
      </div>

      <div style={MS.content}>
        <>
          <div style={MS.actionRow}>
            <ModernPlayButton
              continueLabel={isSeries ? continueLabel : null}
              hasProgress={isSeries ? hasProgress : false}
              onClick={() => {
                if (isSeries) { if (continueEp) onEpisodeClick(continueEp); }
                else onMovieSources();
              }}
            />
            {trailerOnHero && displayTrailers.length > 0 && (
              <ModernIconBtn title={t('detail.watch_trailer')} onClick={() => shellOpen(displayTrailers[0].url).catch(() => {})}>
                <Film size={18} />
              </ModernIconBtn>
            )}
            <ModernIconBtn title={isInWatchlist ? t('detail.in_library') : t('detail.add_to_library')} active={isInWatchlist} onClick={onToggleWatchlist}>
              {isInWatchlist ? <BookmarkCheck size={18} /> : <Bookmark size={18} />}
            </ModernIconBtn>
            <ModernIconBtn title={isCompleted ? t('library.unmark_completed') : t('library.mark_completed')} active={isCompleted} onClick={onToggleCompleted}>
              <CheckCircle2 size={18} />
            </ModernIconBtn>
            <ModernIconBtn title={isDropped ? t('library.unmark_dropped') : t('library.mark_dropped')} active={isDropped} onClick={onToggleDropped}>
              <XCircle size={18} />
            </ModernIconBtn>
            {onOpenComments && <ModernIconBtn title={t('detail.trakt_comments')} onClick={onOpenComments}>
              <MessageCircle size={18} />
            </ModernIconBtn>}
          </div>

          <div style={MS.metaBlock}>
            {hasMdblistRatings && (
              <div style={{ marginBottom: '0.625rem' }}>
                <RatingsRow ratings={mdblistRatings} />
              </div>
            )}
            {(metaGenres.length > 0 || modernMetaDetails.length > 0) && (
              <p style={MS.metaInfoLine}>
                {metaGenres.map((g, i) => (
                  <React.Fragment key={g}>
                    <GenreTag label={g} onClick={() => onNavigateGenre?.(g)} />
                    {(i < metaGenres.length - 1 || modernMetaDetails.length > 0) && <span style={MS.metaDot}> • </span>}
                  </React.Fragment>
                ))}
                {modernMetaDetails.length > 0 && <span style={MS.metaDetailsText}>{modernMetaDetails.join(' • ')}</span>}
              </p>
            )}
            {displayMeta.description && <p style={MS.descText}>{displayMeta.description}</p>}
          </div>

          {isSeries && (
            <>
              <div style={MS.seasonRowModern}>
                <SeasonDropdown seasons={seasonNumbers} selected={selectedSeason} onChange={onSeasonChange} buttonStyle={MS.seasonBtn} seasonWatched={seasonWatchedMap} hideButtonIndicator />
                <button
                  onClick={toggleSeasonWatched}
                  title={seasonWatchedMap[selectedSeason] ? t('detail.mark_season_unwatched') : t('detail.mark_season_watched')}
                  style={MS.seasonWatchedBtn}
                >
                  {seasonWatchedMap[selectedSeason] ? (
                    <CheckCircle2 size={18} color="rgba(255,255,255,0.75)" />
                  ) : (
                    <Circle size={18} color="rgba(255,255,255,0.28)" />
                  )}
                  <span style={MS.seasonWatchedLabel}>
                    {t(seasonWatchedMap[selectedSeason] ? 'detail.mark_season_unwatched' : 'detail.mark_season_watched')}
                  </span>
                </button>
              </div>

              {prevSeasonDialog && (
                <div style={MS.overlayBackdrop} onClick={() => setPrevSeasonDialog(null)}>
                  <div style={{ ...MS.overlaySheet, maxWidth: '25rem', padding: '1.75rem' }} onClick={(e) => e.stopPropagation()}>
                    <p style={{ color: '#fff', fontSize: '0.9375rem', fontWeight: 700, margin: '0 0 0.625rem' }}>
                      {t('detail.prev_seasons_dialog_title')}
                    </p>
                    <p style={{ color: 'rgba(255,255,255,0.55)', fontSize: '0.8125rem', margin: '0 0 1.5rem', lineHeight: '1.25rem' }}>
                      {t('detail.prev_seasons_dialog_body', prevSeasonDialog.unwatchedPrev.map((s) => seasonLabel(s)).join(', '))}
                    </p>
                    <div style={{ display: 'flex', gap: '0.625rem', justifyContent: 'flex-end' }}>
                      <button style={{ background: 'rgba(255,255,255,0.1)', border: 'none', color: '#fff', borderRadius: '0.5rem', padding: '0.5625rem 1.25rem', fontSize: '0.8125rem', fontWeight: 600, cursor: 'pointer' }} onClick={() => { dispatchMarkSeason([prevSeasonDialog.season], true); setPrevSeasonDialog(null); }}>
                        {t('detail.prev_seasons_dialog_no')}
                      </button>
                      <button style={{ background: 'var(--primary-accent-color)', border: 'none', color: 'var(--primary-accent-foreground-color, #fff)', borderRadius: '0.5rem', padding: '0.5625rem 1.25rem', fontSize: '0.8125rem', fontWeight: 600, cursor: 'pointer' }} onClick={() => { dispatchMarkSeason([...prevSeasonDialog.unwatchedPrev, prevSeasonDialog.season], true); setPrevSeasonDialog(null); }}>
                        {t('detail.prev_seasons_dialog_yes')}
                      </button>
                    </div>
                  </div>
                </div>
              )}

              <ModernTabBar
                tabs={seriesTabs}
                active={activeTab}
                onChange={(id) => setActiveTab(id as typeof activeTab)}
              />

              {activeTab === 'episodes' && (
                <div style={{ ...MS.episodeSection, minHeight: '12.5rem' }}>
                  {detail.isLoading && filteredEps.length === 0 ? (
                    <div style={{ display: 'flex', justifyContent: 'center', padding: '2.5rem' }}><div style={spinnerStyle} /></div>
                  ) : (
                    <>
                      <p style={MS.episodeCount}>{t('format.episode_count', filteredEps.length)}</p>
                      <div style={episodeGridStyle}>
                        {filteredEps.map((ep, i) => {
                          const isWatched = watchedMap[ep.id] === true;
                          const metaProgress = progressMap[meta.id];
                          const showProg = !isWatched && metaProgress?.lastVideoId === ep.id && (metaProgress.duration ?? 0) > 0;
                          const progressPct = isWatched ? 100 : (showProg ? Math.min(99, Math.round((metaProgress!.timeOffset / metaProgress!.duration) * 100)) : 0);
                          const minutesRemaining = showProg ? Math.max(0, Math.round((metaProgress!.duration - metaProgress!.timeOffset) / 60)) : 0;
                          const isCwEp = continueWatchingEntry?.lastVideoId === ep.id;
                          const cwBadge = isCwEp ? (continueWatchingEntry?.continueWatchingBadge ?? null) : null;
                          const cwScheduledDate = cwBadge === 'scheduledEpisode' ? (continueWatchingEntry as LibraryItem & { newEpisodeReleasedAt?: string })?.newEpisodeReleasedAt : undefined;
                          return (
                            <ModernEpisodeCard
                              key={ep.id}
                              episode={ep}
                              number={i + 1}
                              isWatched={isWatched}
                              progressPct={progressPct}
                              minutesRemaining={minutesRemaining}
                              cwBadge={cwBadge}
                              cwScheduledDate={cwScheduledDate}
                              blurUnwatched={blurUnwatchedEpisodes}
                              spoilerHide={spoilerHideEpisodeInfo}
                              onClick={() => onEpisodeClick(ep)}
                              onToggleWatched={() => toggleEpisodeWatched(ep, isWatched)}
                            />
                          );
                        })}
                      </div>
                    </>
                  )}
                </div>
              )}

              {activeTab === 'related' && (
                <div style={{ ...MS.relatedSection, minHeight: '12.5rem' }}>
                  <SimilarSourcePicker value={similarSource} onChange={changeSimilarSource} />
                  {similarItems.length === 0 ? (
                    <p style={MS.episodeCount}>{t('auto.no_similar_titles')}</p>
                  ) : (
                    <div style={MS.relatedGrid}>
                      {similarItems.slice(0, 24).map((item) => (
                        <MovieCard key={`${item.type}:${item.id}`} meta={item} width={poster.width} height={poster.height} radius={poster.radius} hideTitle={poster.hideTitles} layout={poster.layout} onClick={onNavigateDetail} />
                      ))}
                    </div>
                  )}
                </div>
              )}

              {activeTab === 'details' && (
                <div style={{ ...MS.detailsTab, minHeight: '12.5rem' }}>
                  {displayMeta.description && (
                    <div style={MS.detailsSection}>
                      <h3 style={MS.detailsSectionTitle}>{t('detail.summary')}</h3>
                      <p style={MS.detailsText}>{displayMeta.description}</p>
                    </div>
                  )}
                  {displayMeta.awards && (
                    <div style={MS.detailsSection}>
                      <h3 style={MS.detailsSectionTitle}>{t('detail.awards')}</h3>
                      <p style={{ ...MS.detailsText, color: '#54D17A', fontWeight: 700 }}>{displayMeta.awards}</p>
                    </div>
                  )}
                  {(castMembers.length > 0 || directorLinks.length > 0) && (
                    <div style={MS.detailsSection}>
                      <h3 style={MS.detailsSectionTitle}>{t('detail.cast_crew')}</h3>
                      <div style={S.castRow}>
                        {directorLinks.map((l) => <CastAvatar key={`dir-${l.name}`} name={l.name} role={t('detail.director')} imageUrl={peopleImages[l.name]} />)}
                        {castMembers.map((member) => <CastAvatar key={`cast-${member.name}:${member.role ?? ''}`} name={member.name} role={member.role || t('detail.actor')} imageUrl={member.imageUrl ?? peopleImages[member.name]} />)}
                      </div>
                    </div>
                  )}
                  {displayTrailers.length > 0 && (
                    <div style={MS.detailsSection}>
                      <h3 style={MS.detailsSectionTitle}>{t('auto.trailers')}</h3>
                      <TrailerCarousel trailers={displayTrailers} trailerMetadata={trailerMetadata} />
                    </div>
                  )}
                </div>
              )}
            </>
          )}

          {!isSeries && (
            <>
              <ModernTabBar
                tabs={movieTabs}
                active={activeTab === 'episodes' ? 'related' : activeTab}
                onChange={(id) => setActiveTab(id as typeof activeTab)}
              />

              {(activeTab === 'related' || activeTab === 'episodes') && (
                <div style={{ ...MS.relatedSection, minHeight: '12.5rem' }}>
                  <SimilarSourcePicker value={similarSource} onChange={changeSimilarSource} />
                  {similarItems.length === 0 ? (
                    <p style={MS.episodeCount}>{t('auto.no_similar_titles')}</p>
                  ) : (
                    <div style={MS.relatedGrid}>
                      {similarItems.slice(0, 24).map((item) => (
                        <MovieCard key={`${item.type}:${item.id}`} meta={item} width={poster.width} height={poster.height} radius={poster.radius} hideTitle={poster.hideTitles} layout={poster.layout} onClick={onNavigateDetail} />
                      ))}
                    </div>
                  )}
                </div>
              )}

              {activeTab === 'details' && (
                <div style={{ ...MS.detailsTab, minHeight: '12.5rem' }}>
                  {displayMeta.description && (
                    <div style={MS.detailsSection}>
                      <h3 style={MS.detailsSectionTitle}>{t('detail.summary')}</h3>
                      <p style={MS.detailsText}>{displayMeta.description}</p>
                    </div>
                  )}
                  {displayMeta.awards && (
                    <div style={MS.detailsSection}>
                      <h3 style={MS.detailsSectionTitle}>{t('detail.awards')}</h3>
                      <p style={{ ...MS.detailsText, color: '#54D17A', fontWeight: 700 }}>{displayMeta.awards}</p>
                    </div>
                  )}
                  {(castMembers.length > 0 || directorLinks.length > 0) && (
                    <div style={MS.detailsSection}>
                      <h3 style={MS.detailsSectionTitle}>{t('detail.cast_crew')}</h3>
                      <div style={S.castRow}>
                        {directorLinks.map((l) => <CastAvatar key={`dir-${l.name}`} name={l.name} role={t('detail.director')} imageUrl={peopleImages[l.name]} />)}
                        {castMembers.map((member) => <CastAvatar key={`cast-${member.name}:${member.role ?? ''}`} name={member.name} role={member.role || t('detail.actor')} imageUrl={member.imageUrl ?? peopleImages[member.name]} />)}
                      </div>
                    </div>
                  )}
                  {displayTrailers.length > 0 && (
                    <div style={MS.detailsSection}>
                      <h3 style={MS.detailsSectionTitle}>{t('auto.trailers')}</h3>
                      <TrailerCarousel trailers={displayTrailers} trailerMetadata={trailerMetadata} />
                    </div>
                  )}
                </div>
              )}
            </>
          )}
        </>
      </div>

      {showSources && selectedEpisode && isSeries && (
        <div style={MS.overlayBackdrop} onClick={onBackToEpisodes}>
          <div style={MS.overlaySheet} onClick={(e) => e.stopPropagation()}>
            <InlineSourceList episode={selectedEpisode} meta={displayMeta} streams={streams} isLoading={!!detail.isLoadingStreams} availableAddons={availableAddons} failedAddons={detail.failedAddons ?? []} playbackFailure={playbackFailure} streamAddonCount={streamAddonCount} onBack={onBackToEpisodes} onPlay={onPlaySource} onAddonChange={(addon) => onDispatch(JSON.stringify({ type: 'detailSelectedAddonChanged', addon }))} onRetryFailed={onRetryFailed} />
          </div>
        </div>
      )}

      {showSources && !isSeries && (
        <div style={MS.overlayBackdrop} onClick={onBackToEpisodes}>
          <div style={MS.overlaySheet} onClick={(e) => e.stopPropagation()}>
            <MovieSourcePanel meta={displayMeta} streams={streams} isLoading={!!detail.isLoadingStreams} availableAddons={availableAddons} failedAddons={detail.failedAddons ?? []} playbackFailure={playbackFailure} streamAddonCount={streamAddonCount} onPlay={(stream) => onPlay(stream, displayMeta, null, undefined, streams)} onAddonChange={(addon) => onDispatch(JSON.stringify({ type: 'detailSelectedAddonChanged', addon }))} onClose={onBackToEpisodes} onRetryFailed={onRetryFailed} />
          </div>
        </div>
      )}
    </div>
  );
}
