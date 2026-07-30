import { useCallback, useReducer, type Dispatch, type SetStateAction } from 'react';
import type { EmbeddedMpvStatus, TorrentStats } from '../../core/mpvPlayer';
import { addSparklineSample } from './PlayerOverlayPrimitives';

type State = {
  paused: boolean;
  muted: boolean;
  volumeLevel: number;
  isBuffering: boolean;
  bufferingProgress: number;
  hdrLabel: string | null;
  statsSnap: EmbeddedMpvStatus | null;
  torrentStatsSnap: TorrentStats | null;
  torrentSpeedHistory: number[];
};

type Action =
  | { type: 'playback'; paused: boolean; muted: boolean; volumeLevel: number }
  | { type: 'paused'; value: SetStateAction<boolean> }
  | { type: 'muted'; value: SetStateAction<boolean> }
  | { type: 'volume'; value: SetStateAction<number> }
  | { type: 'buffering'; active: boolean; progress?: number }
  | { type: 'hdr'; label: string | null }
  | { type: 'stats'; stats: EmbeddedMpvStatus | null }
  | { type: 'torrent'; stats: TorrentStats | null }
  | { type: 'torrent-speed'; speed: number }
  | { type: 'reset-torrent-history' };

const initialState: State = { paused: false, muted: false, volumeLevel: 100, isBuffering: false, bufferingProgress: 0, hdrLabel: null, statsSnap: null, torrentStatsSnap: null, torrentSpeedHistory: [] };

function resolve<T>(value: SetStateAction<T>, previous: T) {
  return typeof value === 'function' ? (value as (current: T) => T)(previous) : value;
}

function reducer(state: State, action: Action): State {
  switch (action.type) {
    case 'playback': return state.paused === action.paused && state.muted === action.muted && state.volumeLevel === action.volumeLevel ? state : { ...state, paused: action.paused, muted: action.muted, volumeLevel: action.volumeLevel };
    case 'paused': { const paused = resolve(action.value, state.paused); return paused === state.paused ? state : { ...state, paused }; }
    case 'muted': { const muted = resolve(action.value, state.muted); return muted === state.muted ? state : { ...state, muted }; }
    case 'volume': { const volumeLevel = resolve(action.value, state.volumeLevel); return volumeLevel === state.volumeLevel ? state : { ...state, volumeLevel }; }
    case 'buffering': { const bufferingProgress = action.progress ?? state.bufferingProgress; return state.isBuffering === action.active && state.bufferingProgress === bufferingProgress ? state : { ...state, isBuffering: action.active, bufferingProgress }; }
    case 'hdr': return state.hdrLabel === action.label ? state : { ...state, hdrLabel: action.label };
    case 'stats': return { ...state, statsSnap: action.stats };
    case 'torrent': return { ...state, torrentStatsSnap: action.stats };
    case 'torrent-speed': return { ...state, torrentSpeedHistory: addSparklineSample(state.torrentSpeedHistory, action.speed) };
    case 'reset-torrent-history': return state.torrentSpeedHistory.length === 0 ? state : { ...state, torrentSpeedHistory: [] };
  }
}

export type PlayerTelemetryControls = {
  setPlayback: (paused: boolean, muted: boolean, volumeLevel: number) => void;
  setPaused: Dispatch<SetStateAction<boolean>>;
  setMuted: Dispatch<SetStateAction<boolean>>;
  setVolumeLevel: Dispatch<SetStateAction<number>>;
  setBuffering: (active: boolean, progress?: number) => void;
  setHdrLabel: (label: string | null) => void;
  setStatsSnap: (stats: EmbeddedMpvStatus | null) => void;
  setTorrentStatsSnap: (stats: TorrentStats | null) => void;
  recordTorrentSpeed: (speed: number) => void;
  resetTorrentSpeedHistory: () => void;
};

export function usePlayerTelemetryState() {
  const [state, dispatch] = useReducer(reducer, initialState);
  const controls: PlayerTelemetryControls = {
    setPlayback: useCallback((paused, muted, volumeLevel) => dispatch({ type: 'playback', paused, muted, volumeLevel }), []),
    setPaused: useCallback((value) => dispatch({ type: 'paused', value }), []),
    setMuted: useCallback((value) => dispatch({ type: 'muted', value }), []),
    setVolumeLevel: useCallback((value) => dispatch({ type: 'volume', value }), []),
    setBuffering: useCallback((active, progress) => dispatch({ type: 'buffering', active, progress }), []),
    setHdrLabel: useCallback((label) => dispatch({ type: 'hdr', label }), []),
    setStatsSnap: useCallback((stats) => dispatch({ type: 'stats', stats }), []),
    setTorrentStatsSnap: useCallback((stats) => dispatch({ type: 'torrent', stats }), []),
    recordTorrentSpeed: useCallback((speed) => dispatch({ type: 'torrent-speed', speed }), []),
    resetTorrentSpeedHistory: useCallback(() => dispatch({ type: 'reset-torrent-history' }), []),
  };
  return { ...state, ...controls };
}
