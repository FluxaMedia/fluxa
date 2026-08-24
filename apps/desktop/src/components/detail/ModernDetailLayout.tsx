import React, { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { getLanguage, t } from '../../i18n';
import { platformOpenExternal } from '../../platform/browser';
import type { DetailState, LibraryItem, Meta, MetaLink, Stream, Trailer, Video } from '../../core/types';
import type { posterPrefsFromState } from '../../core/posterPrefs';
import { MS } from './detailStyles';
import { type NormalizedCastMember } from './castSection';
import { youtubeVideoId } from './youtube';
import { InlineSourceList, MovieSourcePanel } from './SourcePanel';
import { type ProgressEntry } from './EpisodePanel';
import { useSeasonWatched } from '../../hooks/useSeasonWatched';
import { DetailsTabContent, EpisodesTabContent, RelatedTabContent } from './ModernDetailTabs';
import { useTrailerPlayback } from '../../hooks/useTrailerPlayback';
import { ModernDetailHero } from './ModernDetailHero';
import { ModernDetailActionRow } from './ModernDetailActionRow';
import { ModernDetailMetaBlock } from './ModernDetailMetaBlock';
import { PrevSeasonDialog, SeasonControls } from './ModernDetailSeasonSection';
import { SimilarSourcePicker } from './ModernDetailParts';
import { RatingsRow } from './RatingBadge';
import { nameList } from './MetaFacts';

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
  hiddenSections: string[];
  episodeCardsLayout: string;
  isInWatchlist: boolean;
  isDropped: boolean;
  isCompleted: boolean;
  isFavorite: boolean;
  omdbRatings?: { rottenTomatoes?: string; metascore?: string } | null;
  mdblistRatings?: Record<string, number> | null;
  fanartArtwork?: { hdLogo?: string } | null;
  heroProgressPercent?: number;
  heroRemainingLabel?: string | null;
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
  heroProgressPercent,
  heroRemainingLabel,
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
  hiddenSections,
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
  const [similarSource, setSimilarSource] = useState('auto');
  const [prevSeasonDialog, setPrevSeasonDialog] = useState<{ season: number; unwatchedPrev: number[] } | null>(null);
  const hidden = useMemo(() => new Set(hiddenSections), [hiddenSections]);

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

  const modernMetaDetails: string[] = [];
  if (displayMeta.releaseInfo) modernMetaDetails.push(displayMeta.releaseInfo);
  if (isSeries && seasonNumbers.length > 0) modernMetaDetails.push(`${seasonNumbers.length} ${t('auto.seasons')}`);
  if (displayMeta.runtime) modernMetaDetails.push(displayMeta.runtime.replace(/(\d\s*h)\s*(\d)/i, '$1 $2'));

  const heroLogo = fanartArtwork?.hdLogo || displayMeta.logo;

  const genres = Array.isArray(displayMeta.genres) ? displayMeta.genres.slice(0, 6) : [];
  const hasMdblistRatings = mdblistRatings != null && Object.keys(mdblistRatings).length > 0;
  const imdbScore = displayMeta.imdbRating ? Number(displayMeta.imdbRating) : null;
  const fallbackRatings: Record<string, number> = {};
  if (imdbScore) fallbackRatings.imdb = imdbScore;
  const tomatoScore = parseInt(omdbRatings?.rottenTomatoes ?? '', 10);
  if (Number.isFinite(tomatoScore)) fallbackRatings.tomatoes = tomatoScore;
  const metascore = parseInt(omdbRatings?.metascore ?? '', 10);
  if (Number.isFinite(metascore)) fallbackRatings.metacritic = metascore;
  const ratingsNode = hasMdblistRatings ? (
    <RatingsRow ratings={mdblistRatings} />
  ) : Object.keys(fallbackRatings).length > 0 ? (
    <RatingsRow ratings={fallbackRatings} />
  ) : null;
  const createdBy = nameList(displayMeta.createdBy);
  const creators = createdBy.length ? createdBy : nameList(displayMeta.director);

  const episodeGridStyle = episodeCardsLayout === 'list' ? { ...MS.episodeGrid, gridTemplateColumns: '1fr' } : MS.episodeGrid;

  return (
    <div className="detail-screen" style={MS.screen} ref={screenRef}>
      {!hidden.has('hero') && <ModernDetailHero
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
        progressPercent={heroProgressPercent}
        remainingLabel={heroRemainingLabel}
      />}

      <div className="detail-content" style={MS.content}>
        {!hidden.has('actions') && <ModernDetailActionRow
          continueLabel={isSeries ? continueLabel : null}
          hasProgress={isSeries ? hasProgress : false}
          onPlayClick={() => {
            if (isSeries) {
              if (continueEp) onEpisodeClick(continueEp);
            } else onMovieSources();
          }}
          onPlayTrailer={
            trailerOnHero
              ? trailer.canPlayTrailer && !trailer.trailerPending
                ? () => {
                    screenRef.current?.scrollTo({ top: 0, behavior: 'smooth' });
                    trailer.startTrailer();
                  }
                : undefined
              : displayTrailers.length > 0
                ? () => platformOpenExternal(displayTrailers[0].url).catch(() => {})
                : undefined
          }
          isInWatchlist={isInWatchlist}
          onToggleWatchlist={onToggleWatchlist}
          isCompleted={isCompleted}
          onToggleCompleted={onToggleCompleted}
          isDropped={isDropped}
          onToggleDropped={onToggleDropped}
          isFavorite={isFavorite}
          onToggleFavorite={onToggleFavorite}
          onOpenComments={onOpenComments}
        />}

        {!hidden.has('meta') && <ModernDetailMetaBlock
          certification={displayMeta.certification}
          metaDetails={modernMetaDetails}
          description={displayMeta.description}
          genres={genres}
          ratings={ratingsNode}
          onNavigateGenre={onNavigateGenre}
        />}

        {isSeries && (
          <>
            <PrevSeasonDialog
              prevSeasonDialog={prevSeasonDialog}
              onDismissPrevSeasonDialog={() => setPrevSeasonDialog(null)}
              onConfirmPrevSeasonDialog={(includePrev) => {
                if (!prevSeasonDialog) return;
                const seasons = includePrev ? [...prevSeasonDialog.unwatchedPrev, prevSeasonDialog.season] : [prevSeasonDialog.season];
                dispatchMarkSeason(seasons, true);
                setPrevSeasonDialog(null);
              }}
            />

            {!hidden.has('episodes') && (
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
                seasonControls={
                  <SeasonControls
                    seasonNumbers={seasonNumbers}
                    selectedSeason={selectedSeason}
                    onSeasonChange={onSeasonChange}
                    seasonWatchedMap={seasonWatchedMap}
                    toggleSeasonWatched={toggleSeasonWatched}
                  />
                }
              />
            )}
          </>
        )}

        {!hidden.has('details') && (
          <DetailsTabContent
            displayMeta={displayMeta}
            castMembers={castMembers}
            directorLinks={directorLinks}
            peopleImages={peopleImages}
            creators={creators}
          />
        )}

        {!hidden.has('related') && (
          <RelatedTabContent
            similarItems={similarItems}
            poster={poster}
            onNavigateDetail={onNavigateDetail}
            sourcePicker={<SimilarSourcePicker value={similarSource} onChange={changeSimilarSource} />}
          />
        )}
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
