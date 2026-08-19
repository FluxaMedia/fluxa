import React, { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { getLanguage, t } from '../../i18n';
import type { DetailState, LibraryItem, Meta, MetaLink, Stream, Trailer, Video } from '../../core/types';
import type { posterPrefsFromState } from '../../core/posterPrefs';
import { MS } from './detailStyles';
import { type NormalizedCastMember } from './castSection';
import { youtubeVideoId, type TrailerMetadata } from './TrailerCarousel';
import { InlineSourceList, MovieSourcePanel } from './SourcePanel';
import { type ProgressEntry } from './EpisodePanel';
import { ModernTabBar } from './DetailButtons';
import { useSeasonWatched } from '../../hooks/useSeasonWatched';
import { DetailsTabContent, EpisodesTabContent, RelatedTabContent } from './ModernDetailTabs';
import { useTrailerPlayback } from '../../hooks/useTrailerPlayback';
import { ModernDetailHero } from './ModernDetailHero';
import { ModernDetailActionRow } from './ModernDetailActionRow';
import { ModernDetailMetaBlock } from './ModernDetailMetaBlock';
import { ModernDetailSeasonSection } from './ModernDetailSeasonSection';

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
  isFavorite: boolean;
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
  onToggleFavorite: () => void;
  onOpenComments?: () => void;
  onBgError: () => void;
};

export function ModernDetailLayout({
  displayMeta,
  bgUrl,
  isSeries,
  detail,
  meta,
  episodes,
  filteredEps,
  seasonNumbers,
  selectedSeason,
  selectedEpisode,
  showSources,
  playbackFailure,
  streams,
  episodePlan,
  similarItems,
  displayTrailers,
  trailerMetadata,
  castMembers,
  directorLinks,
  peopleImages,
  watchedMap,
  progressMap,
  continueWatchingEntry,
  isInWatchlist,
  isDropped,
  isCompleted,
  isFavorite,
  omdbRatings,
  mdblistRatings,
  fanartArtwork,
  availableAddons,
  streamAddonCount,
  poster,
  trailerOnHero,
  detailHeroAutoplayTrailer,
  detailHeroAutoplayTrailerDelaySecs,
  preferredSubtitleLanguage,
  secondarySubtitleLanguage,
  blurUnwatchedEpisodes,
  spoilerHideEpisodeInfo,
  detailSeasonSelectorMode: _detailSeasonSelectorMode,
  episodeCardsLayout,
  onBack,
  onDispatch,
  onNavigateDetail,
  onNavigateGenre,
  onSeasonChange,
  onEpisodeClick,
  onMovieSources,
  onRetryFailed,
  onBackToEpisodes,
  onPlaySource,
  onPlay,
  onToggleWatchlist,
  onToggleCompleted,
  onToggleDropped,
  onToggleFavorite,
  onOpenComments,
  onBgError,
}: ModernDetailProps) {
  const [activeTab, setActiveTab] = useState<'episodes' | 'related' | 'details'>(() => (isSeries ? 'episodes' : 'details'));
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
  const [bgLayers, setBgLayers] = useState<{ url: string; key: number }[]>(() => (bgUrl ? [{ url: bgUrl, key: 0 }] : []));
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
    onDispatch(
      JSON.stringify({
        type: 'detailSecondaryRequested',
        contentType: meta.type,
        id: meta.id,
        language: getLanguage(),
        similarTitlesSource: source,
      }),
    );
  };

  const { seasonWatchedMap, dispatchMarkSeason, toggleEpisodeWatched } = useSeasonWatched({
    meta,
    displayMeta,
    episodes,
    seasonNumbers,
    watchedMap,
    onDispatch,
  });

  const trailerVideoIdsRef = useRef<string[]>([]);
  const trailerVideoIds = useMemo(() => {
    const ids: string[] = [];
    for (const trailer of displayTrailers) {
      const id = youtubeVideoId(trailer.url);
      if (id && !ids.includes(id)) ids.push(id);
    }
    const previous = trailerVideoIdsRef.current;
    if (previous.length === ids.length && previous.every((id, index) => id === ids[index])) {
      return previous;
    }
    trailerVideoIdsRef.current = ids;
    return ids;
  }, [displayTrailers]);

  const trailer = useTrailerPlayback({
    metaId: displayMeta.id,
    trailerVideoIds,
    autoplay: detailHeroAutoplayTrailer,
    autoplayDelaySecs: detailHeroAutoplayTrailerDelaySecs,
    preferredSubtitleLanguage,
    secondarySubtitleLanguage,
  });

  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if (e.key !== 'Escape') return;
      if (prevSeasonDialog) {
        setPrevSeasonDialog(null);
        return;
      }
      if (showSources) {
        onBackToEpisodes();
      }
    };
    window.addEventListener('keydown', handler);
    return () => window.removeEventListener('keydown', handler);
  }, [prevSeasonDialog, showSources, onBackToEpisodes]);

  const toggleSeasonWatched = useCallback(() => {
    const isWatched = seasonWatchedMap[selectedSeason] === true;
    if (isWatched) {
      dispatchMarkSeason([selectedSeason], false);
      return;
    }
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

  const episodeGridStyle = episodeCardsLayout === 'list' ? { ...MS.episodeGrid, gridTemplateColumns: '1fr' } : MS.episodeGrid;

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
    <div className="detail-screen" style={MS.screen} ref={screenRef}>
      <ModernDetailHero
        bgUrl={bgUrl}
        bgLayers={bgLayers}
        bgLoadedKeys={bgLoadedKeys}
        onBgLayerLoad={handleBgLayerLoad}
        onBgError={onBgError}
        trailer={trailer}
        heroInView={heroInView}
        onBack={onBack}
        heroLogo={heroLogo}
        displayMetaName={displayMeta.name}
      />

      <div className="detail-content" style={MS.content}>
        <>
          <ModernDetailActionRow
            continueLabel={isSeries ? continueLabel : null}
            hasProgress={isSeries ? hasProgress : false}
            onPlayClick={() => {
              if (isSeries) {
                if (continueEp) onEpisodeClick(continueEp);
              } else onMovieSources();
            }}
            trailerUrl={trailerOnHero && displayTrailers.length > 0 ? displayTrailers[0].url : undefined}
            isInWatchlist={isInWatchlist}
            onToggleWatchlist={onToggleWatchlist}
            isCompleted={isCompleted}
            onToggleCompleted={onToggleCompleted}
            isDropped={isDropped}
            onToggleDropped={onToggleDropped}
            isFavorite={isFavorite}
            onToggleFavorite={onToggleFavorite}
            onOpenComments={onOpenComments}
          />

          <ModernDetailMetaBlock
            mdblistRatings={mdblistRatings}
            metaGenres={metaGenres}
            onNavigateGenre={onNavigateGenre}
            metaDetails={modernMetaDetails}
            description={displayMeta.description}
          />

          {isSeries && (
            <>
              <ModernDetailSeasonSection
                seasonNumbers={seasonNumbers}
                selectedSeason={selectedSeason}
                onSeasonChange={onSeasonChange}
                seasonWatchedMap={seasonWatchedMap}
                toggleSeasonWatched={toggleSeasonWatched}
                prevSeasonDialog={prevSeasonDialog}
                onDismissPrevSeasonDialog={() => setPrevSeasonDialog(null)}
                onConfirmPrevSeasonDialog={(includePrev) => {
                  if (!prevSeasonDialog) return;
                  const seasons = includePrev ? [...prevSeasonDialog.unwatchedPrev, prevSeasonDialog.season] : [prevSeasonDialog.season];
                  dispatchMarkSeason(seasons, true);
                  setPrevSeasonDialog(null);
                }}
              />

              <ModernTabBar tabs={seriesTabs} active={activeTab} onChange={(id) => setActiveTab(id as typeof activeTab)} />

              {activeTab === 'episodes' && (
                <EpisodesTabContent
                  detail={detail}
                  filteredEps={filteredEps}
                  watchedMap={watchedMap}
                  progressMap={progressMap}
                  metaId={meta.id}
                  continueWatchingEntry={continueWatchingEntry}
                  episodeGridStyle={episodeGridStyle}
                  blurUnwatchedEpisodes={blurUnwatchedEpisodes}
                  spoilerHideEpisodeInfo={spoilerHideEpisodeInfo}
                  onEpisodeClick={onEpisodeClick}
                  toggleEpisodeWatched={toggleEpisodeWatched}
                />
              )}

              {activeTab === 'related' && (
                <RelatedTabContent
                  similarSource={similarSource}
                  onChangeSimilarSource={changeSimilarSource}
                  similarItems={similarItems}
                  poster={poster}
                  onNavigateDetail={onNavigateDetail}
                />
              )}

              {activeTab === 'details' && (
                <DetailsTabContent
                  displayMeta={displayMeta}
                  castMembers={castMembers}
                  directorLinks={directorLinks}
                  peopleImages={peopleImages}
                  displayTrailers={displayTrailers}
                  trailerMetadata={trailerMetadata}
                />
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
                <RelatedTabContent
                  similarSource={similarSource}
                  onChangeSimilarSource={changeSimilarSource}
                  similarItems={similarItems}
                  poster={poster}
                  onNavigateDetail={onNavigateDetail}
                />
              )}

              {activeTab === 'details' && (
                <DetailsTabContent
                  displayMeta={displayMeta}
                  castMembers={castMembers}
                  directorLinks={directorLinks}
                  peopleImages={peopleImages}
                  displayTrailers={displayTrailers}
                  trailerMetadata={trailerMetadata}
                />
              )}
            </>
          )}
        </>
      </div>

      {showSources && selectedEpisode && isSeries && (
        <div className="detail-overlay-backdrop" style={MS.overlayBackdrop} onClick={onBackToEpisodes}>
          <div className="detail-overlay-sheet" style={MS.overlaySheet} onClick={(e) => e.stopPropagation()}>
            <InlineSourceList
              episode={selectedEpisode}
              meta={displayMeta}
              streams={streams}
              isLoading={!!detail.isLoadingStreams}
              availableAddons={availableAddons}
              failedAddons={detail.failedAddons ?? []}
              playbackFailure={playbackFailure}
              streamAddonCount={streamAddonCount}
              onBack={onBackToEpisodes}
              onPlay={onPlaySource}
              onAddonChange={(addon) => onDispatch(JSON.stringify({ type: 'detailSelectedAddonChanged', addon }))}
              onRetryFailed={onRetryFailed}
            />
          </div>
        </div>
      )}

      {showSources && !isSeries && (
        <div className="detail-overlay-backdrop" style={MS.overlayBackdrop} onClick={onBackToEpisodes}>
          <div className="detail-overlay-sheet" style={MS.overlaySheet} onClick={(e) => e.stopPropagation()}>
            <MovieSourcePanel
              meta={displayMeta}
              streams={streams}
              isLoading={!!detail.isLoadingStreams}
              availableAddons={availableAddons}
              failedAddons={detail.failedAddons ?? []}
              playbackFailure={playbackFailure}
              streamAddonCount={streamAddonCount}
              onPlay={(stream) => onPlay(stream, displayMeta, null, undefined, streams)}
              onAddonChange={(addon) => onDispatch(JSON.stringify({ type: 'detailSelectedAddonChanged', addon }))}
              onClose={onBackToEpisodes}
              onRetryFailed={onRetryFailed}
            />
          </div>
        </div>
      )}
    </div>
  );
}
