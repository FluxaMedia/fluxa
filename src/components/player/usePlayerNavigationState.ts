import { useCallback, useReducer, type Dispatch, type SetStateAction } from 'react';
import type { EpisodeInfo } from './EpisodePanel';
import type { ActiveSkip, Chapter, SkipSegment } from './PlayerOverlayPrimitives';

type State = {
  chapters: Chapter[];
  skipSegments: SkipSegment[];
  nextEpSubtitle: string;
  nextEpThreshold: number;
  autoPlayNextEpisode: boolean;
  autoPlayCountdownSecs: number;
  countdown: number | null;
  nextEpDismissed: boolean;
  episodes: EpisodeInfo[];
  activeSkip: ActiveSkip | null;
  autoSkipSegments: boolean;
  showNextEpCard: boolean;
};

type Action =
  | { type: 'chapters'; value: SetStateAction<Chapter[]> }
  | { type: 'skip-segments'; value: SetStateAction<SkipSegment[]> }
  | { type: 'next-subtitle'; value: SetStateAction<string> }
  | { type: 'next-threshold'; value: SetStateAction<number> }
  | { type: 'autoplay'; value: SetStateAction<boolean> }
  | { type: 'autoplay-countdown'; value: SetStateAction<number> }
  | { type: 'countdown'; value: SetStateAction<number | null> }
  | { type: 'next-dismissed'; value: SetStateAction<boolean> }
  | { type: 'episodes'; value: SetStateAction<EpisodeInfo[]> }
  | { type: 'active-skip'; value: SetStateAction<ActiveSkip | null> }
  | { type: 'auto-skip'; value: SetStateAction<boolean> }
  | { type: 'show-next-card'; value: SetStateAction<boolean> };

const initialState: State = { chapters: [], skipSegments: [], nextEpSubtitle: '', nextEpThreshold: 85, autoPlayNextEpisode: false, autoPlayCountdownSecs: 7, countdown: null, nextEpDismissed: false, episodes: [], activeSkip: null, autoSkipSegments: false, showNextEpCard: false };

function resolve<T>(value: SetStateAction<T>, current: T) {
  return typeof value === 'function' ? (value as (previous: T) => T)(current) : value;
}

function reducer(state: State, action: Action): State {
  switch (action.type) {
    case 'chapters': return { ...state, chapters: resolve(action.value, state.chapters) };
    case 'skip-segments': return { ...state, skipSegments: resolve(action.value, state.skipSegments) };
    case 'next-subtitle': return { ...state, nextEpSubtitle: resolve(action.value, state.nextEpSubtitle) };
    case 'next-threshold': return { ...state, nextEpThreshold: resolve(action.value, state.nextEpThreshold) };
    case 'autoplay': return { ...state, autoPlayNextEpisode: resolve(action.value, state.autoPlayNextEpisode) };
    case 'autoplay-countdown': return { ...state, autoPlayCountdownSecs: resolve(action.value, state.autoPlayCountdownSecs) };
    case 'countdown': return { ...state, countdown: resolve(action.value, state.countdown) };
    case 'next-dismissed': return { ...state, nextEpDismissed: resolve(action.value, state.nextEpDismissed) };
    case 'episodes': return { ...state, episodes: resolve(action.value, state.episodes) };
    case 'active-skip': return { ...state, activeSkip: resolve(action.value, state.activeSkip) };
    case 'auto-skip': return { ...state, autoSkipSegments: resolve(action.value, state.autoSkipSegments) };
    case 'show-next-card': return { ...state, showNextEpCard: resolve(action.value, state.showNextEpCard) };
  }
}

export function usePlayerNavigationState() {
  const [state, dispatch] = useReducer(reducer, initialState);
  return {
    ...state,
    setChapters: useCallback<Dispatch<SetStateAction<Chapter[]>>>((value) => dispatch({ type: 'chapters', value }), []),
    setSkipSegments: useCallback<Dispatch<SetStateAction<SkipSegment[]>>>((value) => dispatch({ type: 'skip-segments', value }), []),
    setNextEpSubtitle: useCallback<Dispatch<SetStateAction<string>>>((value) => dispatch({ type: 'next-subtitle', value }), []),
    setNextEpThreshold: useCallback<Dispatch<SetStateAction<number>>>((value) => dispatch({ type: 'next-threshold', value }), []),
    setAutoPlayNextEpisode: useCallback<Dispatch<SetStateAction<boolean>>>((value) => dispatch({ type: 'autoplay', value }), []),
    setAutoPlayCountdownSecs: useCallback<Dispatch<SetStateAction<number>>>((value) => dispatch({ type: 'autoplay-countdown', value }), []),
    setCountdown: useCallback<Dispatch<SetStateAction<number | null>>>((value) => dispatch({ type: 'countdown', value }), []),
    setNextEpDismissed: useCallback<Dispatch<SetStateAction<boolean>>>((value) => dispatch({ type: 'next-dismissed', value }), []),
    setEpisodes: useCallback<Dispatch<SetStateAction<EpisodeInfo[]>>>((value) => dispatch({ type: 'episodes', value }), []),
    setActiveSkip: useCallback<Dispatch<SetStateAction<ActiveSkip | null>>>((value) => dispatch({ type: 'active-skip', value }), []),
    setAutoSkipSegments: useCallback<Dispatch<SetStateAction<boolean>>>((value) => dispatch({ type: 'auto-skip', value }), []),
    setShowNextEpCard: useCallback<Dispatch<SetStateAction<boolean>>>((value) => dispatch({ type: 'show-next-card', value }), []),
  };
}
