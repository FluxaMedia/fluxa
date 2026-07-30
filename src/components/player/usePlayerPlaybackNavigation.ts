import { useEffect, type Dispatch, type MutableRefObject, type SetStateAction } from 'react';
import { emit, listen } from '@tauri-apps/api/event';
import { playerGetPlaybackInfo } from '../../core/mpvPlayer';
import type { EpisodeInfo } from './EpisodePanel';
import { parseChapters, parseEpisodes, parseSegments, type ActiveSkip, type Chapter, type SkipSegment } from './PlayerOverlayPrimitives';

type Setter<T> = Dispatch<SetStateAction<T>>;

type Bindings = {
  chaptersRef: MutableRefObject<Chapter[]>;
  setChapters: Setter<Chapter[]>;
  setSkipSegments: Setter<SkipSegment[]>;
  setNextEpSubtitle: Setter<string>;
  setNextEpDismissed: Setter<boolean>;
  setNextEpThreshold: Setter<number>;
  setAutoPlayNextEpisode: Setter<boolean>;
  setAutoPlayCountdownSecs: Setter<number>;
  setAutoSkipSegments: Setter<boolean>;
  setEpisodes: Setter<EpisodeInfo[]>;
  showNextEpCard: boolean;
  autoPlayNextEpisode: boolean;
  nextEpDismissed: boolean;
  autoPlayCountdownSecs: number;
  setCountdown: Setter<number | null>;
  activeSkip: ActiveSkip | null;
  setActiveSkip: Setter<ActiveSkip | null>;
  countdown: number | null;
  pausedRef: MutableRefObject<boolean>;
  resetActivity: () => void;
};

export function usePlayerPlaybackNavigation(options: Bindings) {
  const { chaptersRef, setChapters, setSkipSegments, setNextEpSubtitle, setNextEpDismissed, setNextEpThreshold, setAutoPlayNextEpisode, setAutoPlayCountdownSecs, setAutoSkipSegments, setEpisodes, showNextEpCard, autoPlayNextEpisode, nextEpDismissed, autoPlayCountdownSecs, setCountdown, activeSkip, setActiveSkip, countdown, pausedRef, resetActivity } = options;
  useEffect(() => {
    const poll = async () => {
      try {
        const info = await playerGetPlaybackInfo();
        const chapters = parseChapters(info.chaptersJson);
        chaptersRef.current = chapters;
        setChapters(chapters);
        setSkipSegments(parseSegments(info.skipSegmentsJson));
        setNextEpSubtitle((previous: string) => {
          const next = info.nextEpSubtitle ?? '';
          if (next !== previous) setNextEpDismissed(false);
          return next;
        });
        setNextEpThreshold(info.nextEpThresholdPercent ?? 85);
        setAutoPlayNextEpisode(info.autoPlayNextEpisode ?? false);
        setAutoPlayCountdownSecs(info.autoPlayCountdownSecs ?? 7);
        setAutoSkipSegments(info.autoSkipSegments ?? false);
        setEpisodes(parseEpisodes(info.episodesJson));
      } catch {}
    };
    void poll();
    const interval = setInterval(() => { void poll(); }, 2000);
    let unlisten: (() => void) | undefined;
    void listen('player-skip-info-updated', () => { void poll(); }).then((stop) => { unlisten = stop; });
    return () => { clearInterval(interval); unlisten?.(); };
  }, []);
  useEffect(() => {
    if (!showNextEpCard || !autoPlayNextEpisode || nextEpDismissed) { setCountdown(null); return; }
    setCountdown(autoPlayCountdownSecs);
  }, [showNextEpCard, autoPlayNextEpisode, nextEpDismissed, autoPlayCountdownSecs]);
  useEffect(() => { if (showNextEpCard && activeSkip?.type === 'outro') setActiveSkip(null); }, [showNextEpCard, activeSkip]);
  useEffect(() => {
    if (countdown === null) return;
    const id = setInterval(() => { if (!pausedRef.current) setCountdown((value: number | null) => value !== null && value > 0 ? value - 1 : value); }, 1000);
    return () => clearInterval(id);
  }, [countdown === null]);
  useEffect(() => { if (countdown === 0) { resetActivity(); void emit('native-player-next-episode', null); } }, [countdown]);
}
