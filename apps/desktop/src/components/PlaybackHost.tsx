import React, { type RefObject } from 'react';
import { PlayerLoadingOverlay } from './PlayerLoadingOverlay';
import { ErrorBoundary } from './ErrorBoundary';
import { ReactPlayerOverlay } from '../appScreens';
import type { PlayerSubtitleSource } from '../core/playerUtils';
import type { Meta, Stream, Video } from '../core/types';
import type { PlayerLoadingOverlayState } from '../hooks/usePlayer';
import type { TorrentTelemetryContext } from '../core/mpvPlayer';

interface Props {
  active: boolean;
  loading: PlayerLoadingOverlayState | null;
  closePlayer: () => Promise<void>;
  onSeekPersist?: () => Promise<void>;
  notifyFirstFrame: () => void;
  title?: string;
  episodeTitle?: string;
  episode: Video | null;
  usesTorrent: boolean;
  posterUrl?: string;
  logoUrl?: string;
  metaId?: string;
  subtitleUrl?: string;
  subtitles?: PlayerSubtitleSource[];
  streamHeaders?: Record<string, string>;
  streamRef: RefObject<Stream | null>;
  metaRef: RefObject<Meta | null>;
  playbackUrl: string | null;
  torrentTelemetryContext: TorrentTelemetryContext | null;
  prefs: Record<string, unknown>;
  playbackError: string | null;
  subtitleWarning: string[] | null;
  dismissSubtitleWarning: () => void;
  softwareVideoActive: boolean;
  bannerOffset: number;
  skipSegmentCoverage: Record<string, string[]>;
  dispatch: (actionJson: string) => Promise<void>;
  recommendations?: Meta[];
  onPlayRecommendation?: (meta: Meta) => void;
  onDismissRecommendations?: () => void;
}

export function PlaybackHost({
  active,
  loading,
  closePlayer,
  onSeekPersist,
  notifyFirstFrame,
  title,
  episodeTitle,
  episode,
  usesTorrent,
  posterUrl,
  logoUrl,
  metaId,
  subtitleUrl,
  subtitles,
  streamHeaders,
  streamRef,
  metaRef,
  playbackUrl,
  torrentTelemetryContext,
  prefs,
  playbackError,
  subtitleWarning,
  dismissSubtitleWarning,
  softwareVideoActive,
  bannerOffset,
  skipSegmentCoverage,
  dispatch,
  recommendations = [],
  onPlayRecommendation,
  onDismissRecommendations,
}: Props) {
  if (!active && !loading) return null;
  return (
    <>
      {loading && (
        <PlayerLoadingOverlay
          background={loading.background}
          logo={loading.logo}
          title={loading.title}
          episodeLine={loading.episodeLine}
          status={loading.status}
          error={loading.error}
          isTorrentStream={usesTorrent}
          source={loading.source}
          onBack={closePlayer}
        />
      )}
      {active && (
        <ErrorBoundary>
          <React.Suspense fallback={null}>
            <ReactPlayerOverlay
              closePlayer={closePlayer}
              onSeekPersist={onSeekPersist}
              onFirstFrame={notifyFirstFrame}
              isLoadingOverlayActive={!!loading}
              initialTitle={title}
              initialEpisodeTitle={episodeTitle}
              currentEpisode={episode}
              isTorrentStream={usesTorrent}
              initialPosterUrl={posterUrl}
              initialLogoUrl={logoUrl}
              metaId={metaId}
              initialSubtitleUrl={subtitleUrl}
              subtitles={subtitles}
              initialStreamHeaders={streamHeaders}
              streamRef={streamRef}
              metaRef={metaRef}
              playbackUrl={playbackUrl}
              torrentTelemetryContext={torrentTelemetryContext}
              prefs={prefs}
              onDispatch={dispatch}
              playbackError={playbackError}
              subtitleWarning={subtitleWarning}
              onDismissSubtitleWarning={dismissSubtitleWarning}
              softwareVideoActive={softwareVideoActive}
              bannerOffset={bannerOffset}
              skipSegmentCoverage={skipSegmentCoverage}
              recommendations={recommendations}
              onPlayRecommendation={onPlayRecommendation}
              onDismissRecommendations={onDismissRecommendations}
            />
          </React.Suspense>
        </ErrorBoundary>
      )}
    </>
  );
}
