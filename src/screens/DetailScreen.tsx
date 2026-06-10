import React, { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { coreDetailEpisodePlan } from '../core/engine';
import { appPrefs, prefBool, prefString } from '../core/appPrefs';
import { posterPrefsFromState } from '../core/posterPrefs';
import { seasonPosterUrl } from '../core/seasonPosters';
import { MovieCard } from '../components/MovieCard';
import type { AppState, LibraryItem, Meta, Stream, Video } from '../core/types';
import { getLanguage, t } from '../i18n';
import { fetchTmdbPeopleImages } from '../core/tmdb';
import { NAV_RAIL_WIDTH, TOP_BAR_H, S } from '../components/detail/detailStyles';
import { buildCastMembers, CastAvatar } from '../components/detail/castSection';
import { TrailerCarousel, fetchYoutubeTrailerMetadata, youtubeVideoId, type TrailerMetadata } from '../components/detail/TrailerCarousel';
import { MovieSourcePanel } from '../components/detail/SourcePanel';
import { EpisodePanel, type ProgressEntry } from '../components/detail/EpisodePanel';
import { ModernDetailLayout } from '../components/detail/ModernDetailLayout';

void NAV_RAIL_WIDTH; void TOP_BAR_H;

interface Props {
  meta: Meta;
  state: AppState;
  onDispatch: (actionJson: string) => void;
  onPlay: (stream: Stream, meta: Meta, episode?: Video | null) => void;
  onNavigateDetail: (meta: Meta) => void;
  onNavigateGenre?: (genre: string) => void;
  onBack: () => void;
  initialEpisode?: Video | null;
  autoShowStreams?: boolean;
}

function orderStreamsByPrefs(streams: Stream[], prefs: Record<string, unknown>): Stream[] {
  const mode = prefString(prefs, 'streamSourceSelectionMode', 'manual');
  if (mode === 'regex') {
    const pattern = prefString(prefs, 'streamSourceRegexPattern');
    if (!pattern) return streams;
    try {
      const regex = new RegExp(pattern, 'i');
      return [...streams].sort((a, b) => Number(regex.test(streamText(b))) - Number(regex.test(streamText(a))));
    } catch {
      return streams;
    }
  }
  return streams;
}

function streamText(stream: Stream): string {
  return [stream.name, stream.title, stream.description, stream.url, stream.playableUrl, stream.infoHash].filter(Boolean).join(' ');
}

function IconBtn({ icon, title }: { icon: React.ReactNode; title: string }) {
  const [hovered, setHovered] = useState(false);
  return (
    <button
      title={title}
      style={{ background: 'transparent', border: 'none', cursor: 'pointer', display: 'flex', alignItems: 'center', justifyContent: 'center', width: 38, height: 38, borderRadius: '50%', color: hovered ? '#FFF' : 'rgba(255,255,255,0.5)', transition: 'color 0.15s', padding: 0, flexShrink: 0 }}
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
    >
      {icon}
    </button>
  );
}

function LoadingHeader() {
  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: 14 }}>
      {[300, 180, 360, 240].map((w, i) => (
        <div key={i} style={{ width: w, height: i === 0 ? 48 : 16, background: 'rgba(255,255,255,0.07)', borderRadius: 8, animation: 'pulse 1.6s ease-in-out infinite', animationDelay: `${i * 0.08}s` }} />
      ))}
    </div>
  );
}

void LoadingHeader;

export function DetailScreen({ meta, state, onDispatch, onPlay, onNavigateDetail, onNavigateGenre, onBack, initialEpisode, autoShowStreams }: Props) {
  const detail = state.detail;
  const [bgError, setBgError] = useState(false);
  const [selectedSeason, setSelectedSeason] = useState(initialEpisode?.season ?? 1);
  const [selectedEpisode, setSelectedEpisode] = useState<Video | null>(initialEpisode ?? null);
  const [showSources, setShowSources] = useState(autoShowStreams ?? false);
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
  const prevFilteredEpsRef = useRef<{ metaId: string; season: number; episodes: Video[] }>({ metaId: '', season: 0, episodes: [] });

  const isSeries = meta.type === 'series';
  const displayMeta = detail.meta ?? meta;
  const prefs = useMemo(() => appPrefs(state), [state.settings?.values]);
  const trailerOnHero = prefBool(prefs, 'trailerOnHero', false);
  const blurUnwatchedEpisodes = prefBool(prefs, 'blurUnwatchedEpisodes', false);
  const detailSeasonSelectorMode = prefString(prefs, 'detailSeasonSelectorMode', 'tabs');
  const episodeCardsLayout = prefString(prefs, 'episodeCardsLayout', 'standard');
  const seasonHeroUrl = prefBool(prefs, 'detailSeasonPostersOnHero', true)
    ? seasonPosterUrl(displayMeta, selectedSeason) ?? seasonPosterUrl(meta, selectedSeason)
    : undefined;
  const bgUrl = !bgError
    ? (seasonHeroUrl ?? displayMeta.background ?? displayMeta.poster ?? meta.background ?? meta.poster)
    : null;

  const libRaw = state.library.lastWrite as Record<string, unknown> | undefined;
  const watchlist = (libRaw?.watchlist as LibraryItem[] | undefined) ?? [];
  const isInWatchlist = watchlist.some((item) => item.id === displayMeta.id);
  const watchedMap = (libRaw?.watched as Record<string, boolean> | undefined) ?? {};
  const progressMap = (libRaw?.progress as Record<string, ProgressEntry> | undefined) ?? {};

  useEffect(() => { setBgError(false); }, [displayMeta.id, seasonHeroUrl, displayMeta.background, displayMeta.poster, meta.background, meta.poster]);

  useEffect(() => {
    setSelectedSeason(initialEpisodeRef.current?.season ?? 1);
    setSelectedEpisode(initialEpisodeRef.current);
    setShowSources(autoShowStreamsRef.current);
    onDispatch(JSON.stringify({ type: 'detailLoadRequested', contentType: meta.type, id: meta.id, language: getLanguage() }));
  }, [meta.id]);

  useEffect(() => {
    if (!detail.isLoading || detail.meta) return;
    const timer = setTimeout(() => {
      onDispatch(JSON.stringify({ type: 'detailLoadRequested', contentType: meta.type, id: meta.id, language: getLanguage() }));
    }, 7000);
    return () => clearTimeout(timer);
  }, [detail.isLoading, detail.meta, meta.id, meta.type, onDispatch]);

  useEffect(() => {
    if (!detail.meta || detail.isLoadingStreams || detail.streams?.length) return;
    if (meta.type === 'series') {
      if (autoShowStreamsRef.current && initialEpisodeRef.current?.id) {
        onDispatch(JSON.stringify({ type: 'detailStreamsRequested', contentType: meta.type, requestIds: [initialEpisodeRef.current.id], language: getLanguage() }));
      }
      return;
    }
    onDispatch(JSON.stringify({ type: 'detailStreamsRequested', contentType: meta.type, requestIds: [meta.id], language: getLanguage() }));
  }, [detail.meta, meta.id, meta.type, detail.isLoadingStreams, detail.streams?.length]);

  const streams = useMemo(
    () => orderStreamsByPrefs((detail.visibleStreams ?? detail.streams ?? []) as Stream[], prefs),
    [detail.visibleStreams, detail.streams, prefs],
  );
  const poster = useMemo(() => posterPrefsFromState(state), [state.settings?.values]);
  const displayTrailers = useMemo(
    () => detail.trailers?.length ? detail.trailers : (displayMeta.trailers ?? []),
    [detail.trailers, displayMeta.trailers],
  );
  const similarItems = detail.similarItems ?? [];
  const metaEpisodes = displayMeta.videos ?? [];
  const episodes = useMemo(() => metaEpisodes, [metaEpisodes]);
  const fallbackSeasonNumbers = useMemo(
    () => isSeries ? [...new Set(episodes.map((e) => e.season ?? 1))].sort((a, b) => a - b) : [],
    [isSeries, episodes],
  );
  const fallbackFilteredEps = useMemo(
    () => episodes.filter((e) => (e.season ?? 1) === selectedSeason),
    [episodes, selectedSeason],
  );
  const seasonNumbers = useMemo(
    () => [...new Set([...(episodePlan?.seasonNumbers ?? []), ...fallbackSeasonNumbers, selectedSeason])].sort((a, b) => a - b),
    [episodePlan?.seasonNumbers, fallbackSeasonNumbers, selectedSeason],
  );
  const filteredEps = useMemo(() => {
    const plannedSeasonMatches = episodePlan?.selectedSeason == null || episodePlan?.selectedSeason === selectedSeason;
    const planned = plannedSeasonMatches && episodePlan?.episodes?.length ? episodePlan.episodes : null;
    const computed = planned ?? fallbackFilteredEps;
    if (computed.length > 0) {
      prevFilteredEpsRef.current = { metaId: meta.id, season: selectedSeason, episodes: computed };
      return computed;
    }
    const prev = prevFilteredEpsRef.current;
    if (prev.metaId === meta.id && prev.season === selectedSeason && prev.episodes.length > 0) return prev.episodes;
    return computed;
  }, [episodePlan, selectedSeason, fallbackFilteredEps, meta.id]);

  const changeSeason = useCallback((season: number) => {
    setSelectedSeason(season);
    setSelectedEpisode(null);
    setShowSources(false);
  }, []);

  useEffect(() => {
    let cancelled = false;
    setTrailerMetadata({});
    const youtubeTrailers = displayTrailers.filter((trailer) => youtubeVideoId(trailer.url));
    if (youtubeTrailers.length === 0) return;
    Promise.all(youtubeTrailers.map(async (trailer) => {
      const metadata = await fetchYoutubeTrailerMetadata(trailer.url);
      return [trailer.url, metadata] as const;
    })).then((entries) => {
      if (cancelled) return;
      const next: TrailerMetadata = {};
      for (const [url, metadata] of entries) { if (metadata) next[url] = metadata; }
      setTrailerMetadata(next);
    });
    return () => { cancelled = true; };
  }, [displayTrailers]);

  const continueWatchingEntry = (state.home.continueWatching ?? []).find((item) => item.id === meta.id);
  const libProgress = (libRaw?.progress as Record<string, { lastVideoId?: string }> | undefined) ?? {};
  const lastVideoId = continueWatchingEntry?.lastVideoId ?? libProgress[meta.id]?.lastVideoId ?? null;

  useEffect(() => {
    if (!isSeries) { setEpisodePlan(null); return; }
    let cancelled = false;
    coreDetailEpisodePlan({
      episodes,
      selectedSeason,
      selectedEpisodeId: selectedEpisode?.id ?? lastVideoId ?? null,
      metaId: meta.id,
    }).then((plan) => {
      if (cancelled) return;
      setEpisodePlan(plan as typeof episodePlan);
      const planSeason = (plan as { selectedSeason?: number } | null)?.selectedSeason;
      if (planSeason != null && !selectedEpisode) setSelectedSeason(planSeason);
    });
    return () => { cancelled = true; };
  }, [isSeries, JSON.stringify(episodes), selectedSeason, selectedEpisode?.id, lastVideoId, meta.id]);

  const openEpisodeSources = useCallback((episode: Video) => {
    setSelectedEpisode(episode);
    setShowSources(true);
    onDispatch(JSON.stringify({ type: 'detailStreamsRequested', contentType: meta.type, requestIds: [episode.id], detail: displayMeta, seasonEpisodes: filteredEps, language: getLanguage() }));
  }, [meta.type, displayMeta, filteredEps, onDispatch]);

  const openMovieSources = useCallback(() => {
    setShowSources(true);
    if (!streams.length && !detail.isLoadingStreams) {
      onDispatch(JSON.stringify({ type: 'detailStreamsRequested', contentType: meta.type, requestIds: [meta.id], language: getLanguage() }));
    }
  }, [streams.length, detail.isLoadingStreams, meta.type, meta.id, onDispatch]);

  const castMembers = useMemo(() => buildCastMembers(displayMeta).slice(0, 12), [displayMeta]);

  const directorLinks = useMemo(
    () => (displayMeta.links ?? []).filter((l) => l.category.toLowerCase().includes('director')).slice(0, 2),
    [displayMeta.links],
  );

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
    fetchTmdbPeopleImages({ meta: displayMeta, links: peopleLinks, apiKey, language: prefString(prefs, 'language', getLanguage()) })
      .then((images) => { if (!cancelled) setPeopleImages(images); });
    return () => { cancelled = true; };
  }, [
    displayMeta.id, displayMeta.type, state.settings?.values,
    castMembers.map((member) => `${member.name}:${member.imageUrl ?? ''}`).join('|'),
    directorLinks.map((link) => link.name).join('|'),
  ]);

  const metaParts: string[] = [];
  if (displayMeta.releaseInfo) metaParts.push(displayMeta.releaseInfo);
  if (displayMeta.runtime) metaParts.push(displayMeta.runtime);
  const metaLine = metaParts.join(' · ');

  const viewMode = prefString(prefs, 'detailEpisodeViewMode', 'legacy');
  if (viewMode === 'modern') {
    return (
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
        availableAddons={detail.availableAddons ?? []}
        poster={poster}
        trailerOnHero={trailerOnHero}
        blurUnwatchedEpisodes={blurUnwatchedEpisodes}
        detailSeasonSelectorMode={detailSeasonSelectorMode}
        episodeCardsLayout={episodeCardsLayout}
        onBack={onBack}
        onDispatch={onDispatch}
        onNavigateDetail={onNavigateDetail}
        onNavigateGenre={onNavigateGenre}
        onSeasonChange={changeSeason}
        onEpisodeClick={openEpisodeSources}
        onMovieSources={openMovieSources}
        onBackToEpisodes={() => setShowSources(false)}
        onPlaySource={(stream) => onPlay(stream, displayMeta, selectedEpisode)}
        onPlay={onPlay}
        onBgError={() => setBgError(true)}
      />
    );
  }

  return (
    <div style={S.screen}>
      {bgUrl && (
        <div style={S.bgWrap}>
          <img src={bgUrl} alt="" style={S.bgImg} onError={() => setBgError(true)} />
          <div style={S.bgGradLeft} />
          <div style={S.bgGradBottom} />
        </div>
      )}

      <div style={S.leftPanel}>
        <div style={S.scrollArea}>
          <button style={S.backBtn} onClick={onBack}>
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none">
              <path d="M19 12H5M5 12l7 7M5 12l7-7" stroke="rgba(255,255,255,0.85)" strokeWidth="2.2" strokeLinecap="round" strokeLinejoin="round"/>
            </svg>
            <span style={{ marginLeft: 6 }}>{t('auto.back')}</span>
          </button>

          <>
            <div style={{ marginTop: 28, marginBottom: 8 }}>
              {displayMeta.logo ? (
                <img src={displayMeta.logo} alt={displayMeta.name} style={S.logo} onError={() => {}} />
              ) : (
                <h1 style={S.titleText}>{displayMeta.name}</h1>
              )}
            </div>

            {(metaLine || displayMeta.imdbRating) && (
              <div style={{ display: 'flex', alignItems: 'center', gap: 12, marginBottom: 16, flexWrap: 'wrap' }}>
                {metaLine && <span style={S.metaLine}>{metaLine}</span>}
                {displayMeta.imdbRating && (
                  <span style={{ ...S.imdbBadge }}>
                    <span style={S.imdbLogo}>IMDb</span>
                    <span style={S.imdbRating}>{displayMeta.imdbRating}</span>
                  </span>
                )}
              </div>
            )}

            {trailerOnHero && displayTrailers.length > 0 && (
              <div style={{ maxWidth: 620, marginBottom: 22 }}>
                <TrailerCarousel trailers={displayTrailers.slice(0, 4)} trailerMetadata={trailerMetadata} />
              </div>
            )}

            {displayMeta.genres && displayMeta.genres.length > 0 && (
              <div style={{ display: 'flex', gap: 7, flexWrap: 'wrap', marginBottom: 22 }}>
                {displayMeta.genres.slice(0, 6).map((g) => <span key={g} style={S.genrePill}>{g}</span>)}
              </div>
            )}

            {displayMeta.description && (
              <div style={{ marginBottom: 24 }}>
                <p style={S.sectionLabel}>{t('detail.summary')}</p>
                <p style={S.descText}>{displayMeta.description}</p>
              </div>
            )}

            {displayMeta.awards && (
              <div style={{ marginBottom: 24 }}>
                <p style={S.sectionLabel}>{t('detail.awards')}</p>
                <p style={S.awardsText}>{displayMeta.awards}</p>
              </div>
            )}

            {(castMembers.length > 0 || directorLinks.length > 0) && (
              <div style={{ marginBottom: 20 }}>
                <p style={S.sectionLabel}>{t('detail.cast_crew')}</p>
                <div style={S.castRow}>
                  {directorLinks.map((l) => <CastAvatar key={`dir-${l.name}`} name={l.name} role={t('detail.director')} imageUrl={peopleImages[l.name]} />)}
                  {castMembers.map((member) => (
                    <CastAvatar key={`cast-${member.name}:${member.role ?? ''}`} name={member.name} role={member.role || t('detail.actor')} imageUrl={member.imageUrl ?? peopleImages[member.name]} />
                  ))}
                </div>
              </div>
            )}

            {!trailerOnHero && displayTrailers.length > 0 && (
              <div style={S.trailerSection}>
                <h2 style={S.similarTitle}>{t('auto.trailers')}</h2>
                <TrailerCarousel trailers={displayTrailers} trailerMetadata={trailerMetadata} />
              </div>
            )}

            {similarItems.length > 0 && (
              <div style={S.similarSection}>
                <h2 style={S.similarTitle}>{t('auto.similar_titles')}</h2>
                <div style={S.similarRow}>
                  {similarItems.slice(0, 16).map((item) => (
                    <MovieCard key={`${item.type}:${item.id}`} meta={item} width={poster.width} height={poster.height} radius={poster.radius} hideTitle={poster.hideTitles} layout={poster.layout} onClick={onNavigateDetail} />
                  ))}
                </div>
              </div>
            )}
          </>
        </div>

        <div style={S.actionBar}>
          <button
            style={S.playBtn}
            onClick={() => {
              if (isSeries) {
                const ep = selectedEpisode ?? filteredEps[0] ?? episodes[0];
                if (ep) openEpisodeSources(ep);
              } else {
                openMovieSources();
              }
            }}
          >
            <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor" style={{ marginRight: 7 }}><path d="M8 5v14l11-7z"/></svg>
            {t('common.play')}
          </button>

          <button
            style={{ ...S.secondaryBtn, background: isInWatchlist ? 'rgba(255,255,255,0.14)' : 'transparent' }}
            onClick={() => onDispatch(JSON.stringify({ type: 'toggleWatchlistRequested', item: displayMeta }))}
          >
            {isInWatchlist ? t('detail.in_library') : t('detail.add_to_library')}
          </button>

          <div style={{ flex: 1 }} />

          <IconBtn icon={<svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor"><path d="M12 21.35l-1.45-1.32C5.4 15.36 2 12.28 2 8.5 2 5.42 4.42 3 7.5 3c1.74 0 3.41.81 4.5 2.09C13.09 3.81 14.76 3 16.5 3 19.58 3 22 5.42 22 8.5c0 3.78-3.4 6.86-8.55 11.54L12 21.35z"/></svg>} title={t('detail.favourite')} />
          <IconBtn icon={<svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor"><path d="M1 21h4V9H1v12zm22-11c0-1.1-.9-2-2-2h-6.31l.95-4.57.03-.32c0-.41-.17-.79-.44-1.06L14.17 1 7.59 7.59C7.22 7.95 7 8.45 7 9v10c0 1.1.9 2 2 2h9c.83 0 1.54-.5 1.84-1.22l3.02-7.05c.09-.23.14-.47.14-.73v-2z"/></svg>} title={t('detail.like')} />
          <IconBtn icon={<svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor"><path d="M18 16.08c-.76 0-1.44.3-1.96.77L8.91 12.7c.05-.23.09-.46.09-.7s-.04-.47-.09-.7l7.05-4.11c.54.5 1.25.81 2.04.81 1.66 0 3-1.34 3-3s-1.34-3-3-3-3 1.34-3 3c0 .24.04.47.09.7L8.04 9.81C7.5 9.31 6.79 9 6 9c-1.66 0-3 1.34-3 3s1.34 3 3 3c.79 0 1.5-.31 2.04-.81l7.12 4.16c-.05.21-.08.43-.08.65 0 1.61 1.31 2.92 2.92 2.92s2.92-1.31 2.92-2.92-1.31-2.92-2.92-2.92z"/></svg>} title={t('auto.share')} />
        </div>
      </div>

      {isSeries ? (
        <EpisodePanel
          metaId={meta.id}
          meta={meta}
          seasons={seasonNumbers}
          selectedSeason={selectedSeason}
          onSeasonChange={changeSeason}
          episodes={filteredEps}
          selectedEpisode={selectedEpisode}
          showSources={showSources}
          streams={streams}
          isLoadingStreams={!!detail.isLoadingStreams}
          isLoadingEpisodes={detail.isLoading && filteredEps.length === 0}
          availableAddons={detail.availableAddons ?? []}
          onBackToEpisodes={() => setShowSources(false)}
          onEpisodeClick={openEpisodeSources}
          onPlaySource={(stream) => onPlay(stream, displayMeta, selectedEpisode)}
          watchedMap={watchedMap}
          progressMap={progressMap}
          blurUnwatchedEpisodes={blurUnwatchedEpisodes}
          detailSeasonSelectorMode={detailSeasonSelectorMode}
          episodeCardsLayout={episodeCardsLayout}
        />
      ) : (
        <MovieSourcePanel
          meta={displayMeta}
          streams={streams}
          isLoading={!!detail.isLoadingStreams}
          availableAddons={detail.availableAddons ?? []}
          onPlay={(stream) => onPlay(stream, displayMeta, null)}
        />
      )}
    </div>
  );
}
