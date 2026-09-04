import { useCallback, useEffect, type Dispatch, type MutableRefObject, type SetStateAction } from 'react';
import { platformInvoke as invoke } from '../platform/invoke';
import { subscribePlayerStatus } from '../core/playerStatusStore';
import {
  coreRecommendationOutroPlan,
  coreTerminalRecommendationEligibility,
} from '../core/engine';
import { fetchTerminalRecommendations } from '../core/detailEffects';
import { appPrefs, prefString } from '../core/appPrefs';
import type { AppState, Meta, Video } from '../core/types';

type Options = {
  playerUrl: string | null;
  stateRef: MutableRefObject<AppState>;
  playingMetaRef: MutableRefObject<Meta | null>;
  playingEpisodeRef: MutableRefObject<Video | null>;
  playingNextEpisodeRef: MutableRefObject<Video | null>;
  recommendationOutroShownRef: MutableRefObject<string | null>;
  pendingResumePercentRef: MutableRefObject<number | null>;
  setRecommendations: Dispatch<SetStateAction<Meta[]>>;
};

export function usePlayerRecommendations({
  playerUrl,
  stateRef,
  playingMetaRef,
  playingEpisodeRef,
  playingNextEpisodeRef,
  recommendationOutroShownRef,
  pendingResumePercentRef,
  setRecommendations,
}: Options) {
  useEffect(() => {
    if (!playerUrl) return;
    const unsubscribe = subscribePlayerStatus((status) => {
      const meta = playingMetaRef.current;
      const episodeKey = playingEpisodeRef.current?.id ?? meta?.id;
      const recommendationDuration = Number.parseFloat(status.duration ?? '');
      const recommendationTimePos = Number.parseFloat(status.timePos ?? '');
      if (
        meta &&
        episodeKey &&
        Number.isFinite(recommendationDuration) &&
        recommendationDuration > 0 &&
        Number.isFinite(recommendationTimePos)
      ) {
        const prefs = appPrefs(stateRef.current);
        const key = meta.type === 'series' ? 'seriesRecommendationOutroPercent' : 'movieRecommendationOutroPercent';
        const thresholdPercent = Number(prefString(prefs, key, '85'));
        void coreRecommendationOutroPlan({
          positionSeconds: recommendationTimePos,
          durationSeconds: recommendationDuration,
          thresholdPercent,
          alreadyShown: recommendationOutroShownRef.current === episodeKey,
        })
          .then((outroPlan) => {
            if (!outroPlan.shouldShow) return;
            recommendationOutroShownRef.current = episodeKey;
            void (async () => {
              if (meta.type === 'series') {
                const episode = playingEpisodeRef.current;
                if (!episode || !meta.videos?.length) return;
                const eligibility = await coreTerminalRecommendationEligibility({
                  contentType: meta.type,
                  videos: meta.videos,
                  currentSeason: episode.season ?? 0,
                  currentEpisode: episode.episode ?? episode.number ?? 0,
                  nowMs: Date.now(),
                });
                if (!eligibility.eligible) return;
              }
              const recommendations = await fetchTerminalRecommendations({
                contentType: meta.type,
                id: meta.id,
                hasNextEpisode: false,
              });
              setRecommendations(recommendations);
            })().catch(() => undefined);
          })
          .catch(() => undefined);
      }
      const percent = pendingResumePercentRef.current;
      if (percent === null) return;
      const duration = Number.parseFloat(status.duration ?? '');
      if (!Number.isFinite(duration) || duration <= 0) return;
      pendingResumePercentRef.current = null;
      const seconds = (percent / 100) * duration;
      void invoke('player_command', { command: `set time-pos ${seconds.toFixed(3)}` }).catch(() => undefined);
    });
    return unsubscribe;
  }, [
    pendingResumePercentRef,
    playerUrl,
    playingEpisodeRef,
    playingMetaRef,
    recommendationOutroShownRef,
    setRecommendations,
    stateRef,
  ]);

  const handleTerminalPlayback = useCallback(async () => {
      const meta = playingMetaRef.current;
      if (!meta || playingNextEpisodeRef.current) return false;
      if (meta.type === 'series') {
        const episode = playingEpisodeRef.current;
        if (!episode || !meta.videos?.length) return false;
        const eligibility = await coreTerminalRecommendationEligibility({
          contentType: meta.type,
          videos: meta.videos,
          currentSeason: episode.season ?? 0,
          currentEpisode: episode.episode ?? episode.number ?? 0,
          nowMs: Date.now(),
        });
        if (!eligibility.eligible) return false;
      }
      const recommendations = await fetchTerminalRecommendations({
        contentType: meta.type,
        id: meta.id,
        hasNextEpisode: false,
      }).catch(() => []);
      if (recommendations.length === 0) return false;
      setRecommendations(recommendations);
      return true;
  }, [playingEpisodeRef, playingMetaRef, playingNextEpisodeRef, setRecommendations]);

  return {
    handleTerminalPlayback,
  };
}
