import React, { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { coreDetailEpisodePlan, coreInvoke } from '../core/engine';
import { prewarmYoutubeTrailerConfig } from '../core/effectRunner';
import { coreSupportsResource } from '../core/addonManifest';
import { loadEnabledAddons } from '../core/libraryOps';
import { appPrefs, prefBool, prefString } from '../core/appPrefs';
import { posterPrefsFromState } from '../core/posterPrefs';
import { seasonPosterUrl } from '../core/seasonPosters';
import type { AppState, LibraryItem, Meta, MetaLink, Stream, Video } from '../core/types';
import { getLanguage, t } from '../i18n';
import { fetchTmdbPeopleImages } from '../core/tmdb';
import { NAV_RAIL_WIDTH, TOP_BAR_H, S } from '../components/detail/detailStyles';
import { buildCastMembers, type NormalizedCastMember } from '../components/detail/castSection';
import { fetchYoutubeTrailerMetadata, youtubeVideoId, type TrailerMetadata } from '../components/detail/TrailerCarousel';
import { type ProgressEntry } from '../components/detail/EpisodePanel';
import { ModernDetailLayout } from '../components/detail/ModernDetailLayout';
import { TraktCommentsDialog } from '../components/detail/TraktCommentsDialog';
import { useSeasonWatched } from '../hooks/useSeasonWatched';
import { imdbButtonFor, setViewingDiscordPresence } from '../core/discordPresence';
import { formatRemaining } from '../core/continueWatchingUtils';
import { resolveTheme } from '../theme/adapter';

void NAV_RAIL_WIDTH;
void TOP_BAR_H;

function useOptimisticToggle(authoritative: boolean): [boolean, () => void] {
  const [override, setOverride] = useState<boolean | null>(null);

  useEffect(() => {
    if (override !== null && override === authoritative) setOverride(null);
  }, [authoritative, override]);

  useEffect(() => {
    if (override === null) return;
    const id = setTimeout(() => setOverride(null), 5000);
    return () => clearTimeout(id);
  }, [override]);

  const value = override ?? authoritative;
  const flip = useCallback(() => setOverride((current) => !(current ?? authoritative)), [authoritative]);
  return [value, flip];
}

interface Props {
  meta: Meta;
  state: Pick<AppState, 'detail' | 'home' | 'library' | 'settings'>;
  onDispatch: (actionJson: string) => void;
  onPlay: (stream: Stream, meta: Meta, episode?: Video | null, resumeAt?: number, sourceCandidates?: Stream[]) => void;
  onNavigateDetail: (meta: Meta) => void;
  onNavigateGenre?: (genre: string) => void;
  onBack: () => void;
  initialEpisode?: Video | null;
  autoShowStreams?: boolean;
  playbackFailure?: string | null;
}

export function DetailScreen({
  meta,
  state,
  onDispatch,
  onPlay,
  onNavigateDetail,
  onNavigateGenre,
  onBack,
  initialEpisode,
  autoShowStreams,
  playbackFailure,
}: Props) {
  const detail = state.detail;
  const [bgError, setBgError] = useState(false);
  const [selectedSeason, setSelectedSeason] = useState(initialEpisode?.season ?? 1);
  const [selectedEpisode, setSelectedEpisode] = useState<Video | null>(initialEpisode ?? null);
  const [showSources, setShowSources] = useState(autoShowStreams ?? false);
  const [streamAddonCount, setStreamAddonCount] = useState(0);
  const [peopleImages, setPeopleImages] = useState<Record<string, string>>({});
  const [trailerMetadata, setTrailerMetadata] = useState<TrailerMetadata>({});
  const [episodePlan, setEpisodePlan] = useState<{
    seasonNumbers?: number[];
    selectedSeason?: number;
    episodes?: Video[];
    selectedEpisode?: Video | null;
  } | null>(null);
  const initialEpisodeRef = useRef(initialEpisode ?? null);
  const autoShowStreamsRef = useRef(autoShowStreams ?? false);
  const userChangedSeasonRef = useRef(false);
  const prevFilteredEpsRef = useRef<{ metaId: string; season: number; episodes: Video[] }>({ metaId: '', season: 0, episodes: [] });

  const [resumeDialog, setResumeDialog] = useState<{ episode: Video; timeOffset: number } | null>(null);
  const [episodeResumeAt, setEpisodeResumeAt] = useState<number | undefined>(undefined);
  const [commentsOpen, setCommentsOpen] = useState(false);

  const isSeries = meta.type === 'series';
  const displayMeta = detail.meta ?? meta;
  const prefs = useMemo(() => appPrefs(state), [state.settings?.values]);
  const detailHiddenSections = useMemo(() => resolveTheme(String(prefs.themeId ?? 'fluxa-dark'), String(prefs.skinConfig ?? '')).skin.detail.hiddenSections, [prefs.themeId, prefs.skinConfig]);
  const trailerOnHero = prefBool(prefs, 'trailerOnHero', false);
  const detailHeroAutoplayTrailer = prefBool(prefs, 'detailHeroAutoplayTrailer', false);
  const detailHeroAutoplayTrailerDelaySecs = Number(prefString(prefs, 'detailHeroAutoplayTrailerDelaySecs', '2'));
  const preferredSubtitleLanguage = prefString(prefs, 'preferredSubtitleLanguage', 'none');
  const secondarySubtitleLanguage = prefString(prefs, 'secondarySubtitleLanguage', 'none');
  const blurUnwatchedEpisodes = prefBool(prefs, 'blurUnwatchedEpisodes', false);
  const spoilerHideEpisodeInfo = prefBool(prefs, 'spoilerHideEpisodeInfo', false);
  const detailSeasonSelectorMode = prefString(prefs, 'detailSeasonSelectorMode', 'tabs');
  const episodeCardsLayout = prefString(prefs, 'episodeCardsLayout', 'standard');
  const traktCommentsEnabled = prefBool(prefs, 'traktCommentsEnabled', false);
  const seasonHeroUrl = prefBool(prefs, 'detailSeasonPostersOnHero', true)
    ? (seasonPosterUrl(displayMeta, selectedSeason) ?? seasonPosterUrl(meta, selectedSeason))
    : undefined;
  const bgUrl = !bgError ? (seasonHeroUrl ?? displayMeta.background ?? displayMeta.poster ?? meta.background ?? meta.poster) : null;

  const libRaw = state.library.lastWrite as Record<string, unknown> | undefined;
  const watchlist = (libRaw?.watchlist as LibraryItem[] | undefined) ?? [];
  const [isInWatchlist, flipWatchlistOverride] = useOptimisticToggle(watchlist.some((item) => item.id === displayMeta.id));
  const [isDropped, flipDroppedOverride] = useOptimisticToggle(
    ((libRaw?.dropped as LibraryItem[] | undefined) ?? []).some((item) => item.id === displayMeta.id),
  );
  const [isCompleted, flipCompletedOverride] = useOptimisticToggle(
    ((libRaw?.completed as LibraryItem[] | undefined) ?? []).some((item) => item.id === displayMeta.id),
  );
  const [isFavorite, flipFavoriteOverride] = useOptimisticToggle(
    ((libRaw?.favorites as LibraryItem[] | undefined) ?? []).some((item) => item.id === displayMeta.id),
  );
  const watchedMap = (libRaw?.watched as Record<string, boolean> | undefined) ?? {};
  const progressMap = (libRaw?.progress as Record<string, ProgressEntry> | undefined) ?? {};

  useEffect(() => {
    setBgError(false);
  }, [displayMeta.id, seasonHeroUrl, displayMeta.background, displayMeta.poster, meta.background, meta.poster]);

  useEffect(() => {
    setViewingDiscordPresence({
      title: displayMeta.name,
      posterUrl: displayMeta.poster ?? meta.poster,
      ...imdbButtonFor(displayMeta.id ?? meta.id),
    });
  }, [displayMeta.id, displayMeta.name, displayMeta.poster, meta.poster]);

  useEffect(() => {
    setSelectedSeason(initialEpisodeRef.current?.season ?? 1);
    setSelectedEpisode(initialEpisodeRef.current);
    setShowSources(autoShowStreamsRef.current);
    userChangedSeasonRef.current = false;
    onDispatch(
      JSON.stringify({
        type: 'detailLoadRequested',
        contentType: meta.type,
        id: meta.id,
        language: getLanguage(),
        sourceAddonTransportUrl: meta.sourceAddonTransportUrl,
        sourceAddonCatalogType: meta.sourceAddonCatalogType,
      }),
    );
  }, [meta.id]);

  useEffect(() => {
    onDispatch(
      JSON.stringify({
        type: 'detailSecondaryRequested',
        contentType: meta.type,
        id: meta.id,
        language: getLanguage(),
        similarTitlesSource: prefString(prefs, 'similarTitlesSource', 'auto'),
      }),
    );
  }, [meta.id]);

  useEffect(() => {
    if (!detail.isLoading || detail.meta) return;
    const timer = setTimeout(() => {
      onDispatch(
        JSON.stringify({
          type: 'detailLoadRequested',
          contentType: meta.type,
          id: meta.id,
          language: getLanguage(),
          sourceAddonTransportUrl: meta.sourceAddonTransportUrl,
          sourceAddonCatalogType: meta.sourceAddonCatalogType,
        }),
      );
    }, 7000);
    return () => clearTimeout(timer);
  }, [detail.isLoading, detail.meta, meta.id, meta.type, onDispatch]);

  useEffect(() => {
    if (!detail.meta || detail.isLoadingStreams || detail.streams?.length) return;
    if (meta.type === 'series') {
      if (autoShowStreamsRef.current && initialEpisodeRef.current?.id) {
        onDispatch(
          JSON.stringify({
            type: 'detailStreamsRequested',
            contentType: meta.type,
            requestIds: [initialEpisodeRef.current.id],
            language: getLanguage(),
          }),
        );
      }
      return;
    }
    onDispatch(JSON.stringify({ type: 'detailStreamsRequested', contentType: meta.type, requestIds: [meta.id], language: getLanguage() }));
  }, [detail.meta, meta.id, meta.type, detail.isLoadingStreams, detail.streams?.length]);

  const [streams, setStreams] = useState<Stream[]>([]);
  useEffect(() => {
    let active = true;
    void coreInvoke<Stream[]>('orderStreamsPlan', JSON.stringify({ streams: detail.streams ?? [], prefs })).then((plan) => {
      if (active) setStreams(plan ?? ((detail.streams ?? []) as Stream[]));
    });
    return () => {
      active = false;
    };
  }, [detail.streams, prefs]);
  const poster = useMemo(() => posterPrefsFromState(state), [state.settings?.values]);

  useEffect(() => {
    let active = true;
    void loadEnabledAddons()
      .then(async (addons) => {
        const supported = await Promise.all(addons.map((addon) => coreSupportsResource(addon.manifest, 'stream', meta.type, meta.id)));
        if (active) setStreamAddonCount(supported.filter(Boolean).length);
      })
      .catch(() => {
        if (active) setStreamAddonCount(0);
      });
    return () => {
      active = false;
    };
  }, [meta.id, meta.type]);
  const displayTrailers = useMemo(
    () => (detail.trailers?.length ? detail.trailers : (displayMeta.trailers ?? [])),
    [detail.trailers, displayMeta.trailers],
  );
  const similarItems = detail.similarItems ?? [];
  const omdbRatings = detail.omdbRatings;
  const mdblistRatings = detail.mdblistRatings;
  const fanartArtwork = detail.fanartArtwork;
  const metaEpisodes = displayMeta.videos ?? [];
  const episodes = useMemo(() => metaEpisodes, [metaEpisodes]);
  const seasonNumbers = episodePlan?.seasonNumbers ?? [];
  const filteredEps = useMemo(() => {
    const plannedSeasonMatches = episodePlan?.selectedSeason == null || episodePlan?.selectedSeason === selectedSeason;
    const computed = plannedSeasonMatches ? (episodePlan?.episodes ?? []) : [];
    if (computed.length > 0) {
      prevFilteredEpsRef.current = { metaId: meta.id, season: selectedSeason, episodes: computed };
      return computed;
    }
    const prev = prevFilteredEpsRef.current;
    if (prev.metaId === meta.id && prev.season === selectedSeason && prev.episodes.length > 0) return prev.episodes;
    return computed;
  }, [episodePlan, selectedSeason, meta.id]);

  const { seasonWatchedMap, dispatchMarkSeason, toggleEpisodeWatched } = useSeasonWatched({
    meta,
    displayMeta,
    episodes,
    seasonNumbers,
    watchedMap,
    onDispatch,
  });

  const changeSeason = useCallback((season: number) => {
    userChangedSeasonRef.current = true;
    setSelectedSeason(season);
    setSelectedEpisode(null);
    setShowSources(false);
  }, []);

  useEffect(() => {
    let cancelled = false;
    setTrailerMetadata({});
    const youtubeTrailers = displayTrailers.filter((trailer) => youtubeVideoId(trailer.url));
    if (youtubeTrailers.length === 0) return;
    Promise.all(
      youtubeTrailers.map(async (trailer) => {
        const metadata = await fetchYoutubeTrailerMetadata(trailer.url);
        return [trailer.url, metadata] as const;
      }),
    ).then((entries) => {
      if (cancelled) return;
      const next: TrailerMetadata = {};
      for (const [url, metadata] of entries) {
        if (metadata) next[url] = metadata;
      }
      setTrailerMetadata(next);
    });
    return () => {
      cancelled = true;
    };
  }, [displayTrailers]);

  useEffect(() => {
    if (!detailHeroAutoplayTrailer) return;
    prewarmYoutubeTrailerConfig().catch((err) => console.error('prewarmYoutubeTrailerConfig failed', err));
  }, [detailHeroAutoplayTrailer]);

  const continueWatchingEntry = (state.home.continueWatching ?? []).find((item) => item.id === meta.id);
  const libProgress = (libRaw?.progress as Record<string, { lastVideoId?: string }> | undefined) ?? {};
  const lastVideoId = continueWatchingEntry?.lastVideoId ?? libProgress[meta.id]?.lastVideoId ?? null;

  useEffect(() => {
    if (!isSeries) {
      setEpisodePlan(null);
      return;
    }
    let cancelled = false;
    coreDetailEpisodePlan({
      episodes,
      selectedSeason,
      selectedEpisodeId: selectedEpisode?.id ?? (userChangedSeasonRef.current ? null : lastVideoId) ?? null,
      metaId: meta.id,
    }).then((plan) => {
      if (cancelled) return;
      setEpisodePlan(plan as typeof episodePlan);
      const planSeason = (plan as { selectedSeason?: number } | null)?.selectedSeason;
      if (planSeason != null && !selectedEpisode && !userChangedSeasonRef.current) setSelectedSeason(planSeason);
    });
    return () => {
      cancelled = true;
    };
  }, [isSeries, episodes, selectedSeason, selectedEpisode?.id, lastVideoId, meta.id]);

  const openEpisodeSources = useCallback(
    (episode: Video) => {
      setSelectedEpisode(episode);
      setShowSources(true);
      onDispatch(
        JSON.stringify({
          type: 'detailStreamsRequested',
          contentType: meta.type,
          requestIds: [episode.id],
          detail: displayMeta,
          seasonEpisodes: filteredEps,
          language: getLanguage(),
        }),
      );
    },
    [meta.type, displayMeta, filteredEps, onDispatch],
  );

  const handleEpisodeClick = useCallback(
    (episode: Video) => {
      const seriesProgress = progressMap[meta.id] ?? continueWatchingEntry;
      const timeOffset = seriesProgress?.timeOffset ?? 0;
      const hasProgress = seriesProgress?.lastVideoId === episode.id && timeOffset > 30;
      if (hasProgress) {
        setResumeDialog({ episode, timeOffset });
      } else {
        setEpisodeResumeAt(undefined);
        openEpisodeSources(episode);
      }
    },
    [progressMap, meta.id, continueWatchingEntry, openEpisodeSources],
  );

  const openMovieSources = useCallback(() => {
    setShowSources(true);
    if (!streams.length && !detail.isLoadingStreams) {
      onDispatch(
        JSON.stringify({ type: 'detailStreamsRequested', contentType: meta.type, requestIds: [meta.id], language: getLanguage() }),
      );
    }
  }, [streams.length, detail.isLoadingStreams, meta.type, meta.id, onDispatch]);

  const retryFailedStreams = useCallback(() => {
    if (isSeries) {
      if (selectedEpisode) openEpisodeSources(selectedEpisode);
    } else {
      onDispatch(
        JSON.stringify({ type: 'detailStreamsRequested', contentType: meta.type, requestIds: [meta.id], language: getLanguage() }),
      );
    }
  }, [isSeries, selectedEpisode, openEpisodeSources, onDispatch, meta.type, meta.id]);

  const heroProgress = useMemo(() => {
    const entry = progressMap[meta.id] ?? continueWatchingEntry;
    const duration = entry?.duration ?? 0;
    const offset = entry?.timeOffset ?? 0;
    if (duration <= 0 || offset <= 30) return null;
    return { percent: Math.min(99, Math.round((offset / duration) * 100)), label: formatRemaining(offset, duration) };
  }, [progressMap, meta.id, continueWatchingEntry]);

  const selectedEpisodeEnriched = useMemo(() => {
    if (!selectedEpisode || selectedEpisode.thumbnail) return selectedEpisode;
    const found = episodes.find((ep) => ep.id === selectedEpisode.id);
    return found?.thumbnail ? found : selectedEpisode;
  }, [selectedEpisode, episodes]);

  useEffect(() => {
    const onKeyDown = (e: KeyboardEvent) => {
      if (e.key !== 'Enter' && e.key !== ' ') return;
      const target = e.target as HTMLElement | null;
      if (target && target !== document.body && target.closest('input, textarea, button, [role="button"], [contenteditable="true"]'))
        return;
      e.preventDefault();
      if (isSeries) {
        const ep = selectedEpisode ?? filteredEps[0] ?? episodes[0];
        if (ep) openEpisodeSources(ep);
      } else {
        openMovieSources();
      }
    };
    window.addEventListener('keydown', onKeyDown);
    return () => window.removeEventListener('keydown', onKeyDown);
  }, [isSeries, selectedEpisode, filteredEps, episodes, openEpisodeSources, openMovieSources]);

  const [castMembers, setCastMembers] = useState<NormalizedCastMember[]>([]);
  const [directorLinks, setDirectorLinks] = useState<MetaLink[]>([]);
  const castSignatureRef = useRef<string | null>(null);
  useEffect(() => {
    const signature = JSON.stringify({ id: displayMeta.id, cast: displayMeta.cast, links: displayMeta.links });
    if (signature === castSignatureRef.current) return;
    castSignatureRef.current = signature;
    let cancelled = false;
    buildCastMembers(displayMeta).then((members) => {
      if (!cancelled) setCastMembers(members.slice(0, 12));
    });
    coreInvoke<{ directors: MetaLink[] }>('classifyMetaLinks', JSON.stringify(displayMeta.links ?? [])).then((result) => {
      if (!cancelled) setDirectorLinks((result?.directors ?? []).slice(0, 2));
    });
    return () => {
      cancelled = true;
    };
  }, [displayMeta]);

  useEffect(() => {
    const prefs = appPrefs(state);
    const apiKey = prefString(prefs, 'tmdbApiKey', '').trim();
    const enabled = prefBool(prefs, 'tmdbCastImagesEnabled', true);
    const peopleLinks = [
      ...directorLinks,
      ...castMembers.filter((member) => !member.imageUrl).map((member) => ({ name: member.name, category: 'cast', url: '' })),
    ];
    setPeopleImages({});
    if (!enabled || !apiKey || peopleLinks.length === 0) return;
    let cancelled = false;
    fetchTmdbPeopleImages({ meta: displayMeta, links: peopleLinks, apiKey, language: prefString(prefs, 'language', getLanguage()) }).then(
      (images) => {
        if (!cancelled) setPeopleImages(images);
      },
    );
    return () => {
      cancelled = true;
    };
  }, [displayMeta.id, displayMeta.type, state.settings?.values, castMembers, directorLinks]);

  const metaParts: string[] = [];
  if (displayMeta.releaseInfo) metaParts.push(displayMeta.releaseInfo);
  if (displayMeta.runtime) metaParts.push(displayMeta.runtime);
  const metaLine = metaParts.join(' · ');

  const resumeDialogEl = resumeDialog ? (
    <ResumeDialog
      timeOffset={resumeDialog.timeOffset}
      onContinue={() => {
        setEpisodeResumeAt(resumeDialog.timeOffset);
        openEpisodeSources(resumeDialog.episode);
        setResumeDialog(null);
      }}
      onStartOver={() => {
        setEpisodeResumeAt(0);
        openEpisodeSources(resumeDialog.episode);
        setResumeDialog(null);
      }}
      onClose={() => setResumeDialog(null)}
    />
  ) : null;

  return (
    <>
      <ModernDetailLayout
        displayMeta={displayMeta}
        bgUrl={bgUrl}
        isSeries={isSeries}
        detail={detail}
        meta={meta}
        episodes={episodes}
        filteredEps={filteredEps}
        seasonNumbers={seasonNumbers}
        selectedSeason={selectedSeason}
        selectedEpisode={selectedEpisode}
        showSources={showSources}
        playbackFailure={playbackFailure}
        streams={streams}
        episodePlan={episodePlan}
        similarItems={similarItems}
        displayTrailers={displayTrailers}
        trailerMetadata={trailerMetadata}
        castMembers={castMembers}
        directorLinks={directorLinks}
        peopleImages={peopleImages}
        watchedMap={watchedMap}
        progressMap={progressMap}
        continueWatchingEntry={continueWatchingEntry}
        isInWatchlist={isInWatchlist}
        isDropped={isDropped}
        isCompleted={isCompleted}
        isFavorite={isFavorite}
        omdbRatings={omdbRatings}
        mdblistRatings={mdblistRatings}
        fanartArtwork={fanartArtwork}
        heroProgressPercent={heroProgress?.percent}
        heroRemainingLabel={heroProgress?.label}
        availableAddons={detail.availableAddons ?? []}
        streamAddonCount={streamAddonCount}
        poster={poster}
        trailerOnHero={trailerOnHero}
        detailHeroAutoplayTrailer={detailHeroAutoplayTrailer}
        detailHeroAutoplayTrailerDelaySecs={detailHeroAutoplayTrailerDelaySecs}
        preferredSubtitleLanguage={preferredSubtitleLanguage}
        secondarySubtitleLanguage={secondarySubtitleLanguage}
        blurUnwatchedEpisodes={blurUnwatchedEpisodes}
        spoilerHideEpisodeInfo={spoilerHideEpisodeInfo}
        detailSeasonSelectorMode={detailSeasonSelectorMode}
        hiddenSections={detailHiddenSections}
        episodeCardsLayout={episodeCardsLayout}
        onBack={onBack}
        onDispatch={onDispatch}
        onNavigateDetail={onNavigateDetail}
        onNavigateGenre={onNavigateGenre}
        onSeasonChange={changeSeason}
        onEpisodeClick={handleEpisodeClick}
        onMovieSources={openMovieSources}
        onRetryFailed={retryFailedStreams}
        onBackToEpisodes={() => setShowSources(false)}
        onPlaySource={(stream) => onPlay(stream, displayMeta, selectedEpisodeEnriched, episodeResumeAt, streams)}
        onPlay={onPlay}
        onToggleWatchlist={() => {
          flipWatchlistOverride();
          onDispatch(JSON.stringify({ type: 'toggleWatchlistRequested', item: displayMeta }));
        }}
        onToggleCompleted={() => {
          flipCompletedOverride();
          onDispatch(JSON.stringify({ type: 'toggleLibraryStatusRequested', list: 'completed', item: displayMeta }));
        }}
        onToggleDropped={() => {
          flipDroppedOverride();
          onDispatch(JSON.stringify({ type: 'toggleLibraryStatusRequested', list: 'dropped', item: displayMeta }));
        }}
        onToggleFavorite={() => {
          flipFavoriteOverride();
          onDispatch(JSON.stringify({ type: 'setFeedbackRequested', id: displayMeta.id, value: !isFavorite, meta: displayMeta }));
        }}
        onOpenComments={traktCommentsEnabled ? () => setCommentsOpen(true) : undefined}
        onBgError={() => setBgError(true)}
      />
      {resumeDialogEl}
      {commentsOpen && <TraktCommentsDialog meta={displayMeta} onClose={() => setCommentsOpen(false)} />}
    </>
  );
}

function DescriptionBlock({ description }: { description: string }) {
  const [expanded, setExpanded] = useState(false);
  const CLAMP = 4;
  const lines = description.split('\n');
  const needsClamp = lines.length > CLAMP || description.length > 320;
  return (
    <div style={{ marginBottom: '1.5rem' }}>
      <p style={S.sectionLabel}>{t('detail.summary')}</p>
      <p
        style={{
          ...S.descText,
          display: '-webkit-box',
          WebkitBoxOrient: 'vertical',
          WebkitLineClamp: expanded ? undefined : CLAMP,
          overflow: expanded ? 'visible' : 'hidden',
        }}
      >
        {description}
      </p>
      {needsClamp && (
        <button
          style={{
            background: 'none',
            border: 'none',
            color: 'rgba(255,255,255,0.45)',
            fontSize: '0.75rem',
            fontWeight: 700,
            cursor: 'pointer',
            padding: '0.25rem 0 0',
            display: 'block',
          }}
          onClick={() => setExpanded((e) => !e)}
        >
          {expanded ? t('auto.read_less') : t('auto.read_more')}
        </button>
      )}
    </div>
  );
}

const FONT = "'Montserrat', -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif";

function formatTimestamp(seconds: number): string {
  const s = Math.floor(seconds);
  const h = Math.floor(s / 3600);
  const m = Math.floor((s % 3600) / 60);
  const sec = s % 60;
  if (h > 0) return `${h}:${String(m).padStart(2, '0')}:${String(sec).padStart(2, '0')}`;
  return `${m}:${String(sec).padStart(2, '0')}`;
}

function ResumeDialog({
  timeOffset,
  onContinue,
  onStartOver,
  onClose,
}: {
  timeOffset: number;
  onContinue: () => void;
  onStartOver: () => void;
  onClose: () => void;
}) {
  return (
    <div
      style={{
        position: 'fixed',
        inset: 0,
        zIndex: 99998,
        background: 'rgba(0,0,0,0.72)',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        backdropFilter: 'blur(0.25rem)',
      }}
      onClick={onClose}
    >
      <div
        style={{
          background: '#18191f',
          border: '1px solid rgba(255,255,255,0.10)',
          borderRadius: '0.75rem',
          padding: '1.75rem 1.75rem 1.5rem',
          maxWidth: '22.5rem',
          width: '90%',
          fontFamily: FONT,
        }}
        onClick={(e) => e.stopPropagation()}
      >
        <p style={{ margin: '0 0 1.5rem', fontSize: '1rem', fontWeight: 600, color: '#fff' }}>{t('detail.resume_dialog_title')}</p>
        <div style={{ display: 'flex', flexDirection: 'column', gap: '0.625rem' }}>
          <button
            style={{
              padding: '0.6875rem 1.125rem',
              borderRadius: '0.5rem',
              border: 'none',
              background: '#fff',
              color: '#000',
              fontSize: '0.8125rem',
              fontWeight: 600,
              fontFamily: FONT,
              cursor: 'pointer',
              textAlign: 'left',
            }}
            onClick={onContinue}
          >
            {t('detail.resume_dialog_continue', formatTimestamp(timeOffset))}
          </button>
          <button
            style={{
              padding: '0.6875rem 1.125rem',
              borderRadius: '0.5rem',
              border: '1px solid rgba(255,255,255,0.12)',
              background: 'transparent',
              color: 'rgba(255,255,255,0.70)',
              fontSize: '0.8125rem',
              fontFamily: FONT,
              cursor: 'pointer',
              textAlign: 'left',
            }}
            onClick={onStartOver}
          >
            {t('detail.resume_dialog_start_over')}
          </button>
        </div>
      </div>
    </div>
  );
}
