import { useCallback, useEffect, useRef, useState, type RefObject } from 'react';
import { platformInvoke } from '../platform/invoke';
import { choosePlaybackUrl, probeStream } from '../platform/web/stream';
import { loadEnabledAddons } from '../core/libraryOps';
import { resolvePlaybackSubtitles } from '../core/subtitles';
import type { PlayerSubtitleSource } from '../core/playerUtils';
import { corePlaybackPreparePlan } from '../core/engine';
import { appPrefs } from '../core/appPrefs';
import { fetchPlaybackSkipSegments, type IntroSegmentResult } from '../core/effectRunner';
import { corePlaybackIntroLookupContentId, coreResolveNextEpisode } from '../core/engine';
import { persistLastPlaybackSource } from '../core/libraryStorage';
import {
  persistPlaybackProgress,
  runScrobbleLifecycle,
  snapshotIsUsable,
  type PlaybackSnapshot,
  type ScrobbleEvent,
} from '../core/playbackSession';
import type { AppState, Meta, Stream, UserProfile, Video } from '../core/types';
import type { PlayerLoadingOverlayState } from './usePlayer';

interface UsePlayerOptions {
  stateRef: React.MutableRefObject<AppState>;
  activeProfile: UserProfile | null;
  updateState: (s: Partial<AppState>) => void;
  onProfileUpdated?: (profile: UserProfile) => void;
  onEpisodePlaybackFailed?: (meta: Meta, episode: Video, message: string) => Promise<void> | void;
}

export interface WebPlayerResult {
  playerLoadingOverlay: PlayerLoadingOverlayState | null;
  playerUrl: string | null;
  playerTorrentTelemetryContext: null;
  playerTitle: string | undefined;
  playerEpisodeTitle: string | undefined;
  playerEpisode: Video | null;
  playerUsesTorrent: boolean;
  playerPosterUrl: string | undefined;
  playerLogoUrl: string | undefined;
  playerMetaId: string | undefined;
  playerSubtitleUrl: string | undefined;
  playerSubtitles: PlayerSubtitleSource[];
  playerCodecs: { videoCodec: string | null; audioCodec: string | null } | null;
  playerResumeAt: number | undefined;
  playerSkipSegments: IntroSegmentResult[];
  playerNextEpisode: Video | null;
  playNextEpisode: () => Promise<void>;
  playbackSnapshotRef: RefObject<PlaybackSnapshot | null>;
  reportPlaybackEvent: (event: ScrobbleEvent) => void;
  playerStreamHeaders: Record<string, string> | undefined;
  playingStreamRef: RefObject<Stream | null>;
  playingMetaRef: RefObject<Meta | null>;
  playerPlaybackError: string | null;
  playerSubtitleWarning: string[] | null;
  dismissSubtitleWarning: () => void;
  handlePlay: (stream: Stream, meta?: Meta, episode?: Video | null, resumeAtSeconds?: number, totalDurationSeconds?: number, sourceCandidates?: Stream[], openSourcePickerOnFailure?: boolean, resumePercent?: number) => Promise<void>;
  closePlayer: () => Promise<void>;
  notifyFirstFrame: () => void;
  flushProgressOnQuit: () => Promise<void>;
  skipSegmentCoverage: Record<string, string[]>;
}

export function useWebPlayer({ stateRef, activeProfile, updateState, onProfileUpdated, onEpisodePlaybackFailed: _onEpisodePlaybackFailed }: UsePlayerOptions): WebPlayerResult {
  const [playerUrl, setPlayerUrl] = useState<string | null>(null);
  const [playerCodecs, setPlayerCodecs] = useState<{ videoCodec: string | null; audioCodec: string | null } | null>(null);
  const [playerTitle, setPlayerTitle] = useState<string>();
  const [playerEpisodeTitle, setPlayerEpisodeTitle] = useState<string>();
  const [playerEpisode, setPlayerEpisode] = useState<Video | null>(null);
  const [playerUsesTorrent, setPlayerUsesTorrent] = useState(false);
  const [playerPosterUrl, setPlayerPosterUrl] = useState<string>();
  const [playerLogoUrl, setPlayerLogoUrl] = useState<string>();
  const [playerMetaId, setPlayerMetaId] = useState<string>();
  const [playerStreamHeaders, setPlayerStreamHeaders] = useState<Record<string, string>>();
  const [playerSubtitles, setPlayerSubtitles] = useState<PlayerSubtitleSource[]>([]);
  const [playerLoadingOverlay, setPlayerLoadingOverlay] = useState<PlayerLoadingOverlayState | null>(null);
  const [playerPlaybackError, setPlayerPlaybackError] = useState<string | null>(null);
  const [playerResumeAt, setPlayerResumeAt] = useState<number>();
  const [playerSkipSegments, setPlayerSkipSegments] = useState<IntroSegmentResult[]>([]);
  const [playerNextEpisode, setPlayerNextEpisode] = useState<Video | null>(null);
  const [skipSegmentCoverage, setSkipSegmentCoverage] = useState<Record<string, string[]>>({});
  const playingStreamRef = useRef<Stream | null>(null);
  const playingMetaRef = useRef<Meta | null>(null);
  const playingEpisodeRef = useRef<Video | null>(null);
  const playbackSnapshotRef = useRef<PlaybackSnapshot | null>(null);
  const activeProfileRef = useRef<UserProfile | null>(activeProfile);
  const scrobbleStartedRef = useRef(false);
  const scrobbleWasPausedRef = useRef(false);
  const scrobbleStoppedRef = useRef(false);

  useEffect(() => { activeProfileRef.current = activeProfile; }, [activeProfile]);

  const resolveSkipSegments = useCallback(async (meta?: Meta, episode?: Video | null) => {
    setPlayerSkipSegments([]);
    setSkipSegmentCoverage({});
    const prefs = appPrefs(stateRef.current);
    const useSkipSegments = prefs.useSkipSegments === true;
    const useAnimeSkip = prefs.useAnimeSkip === true;
    if ((!useSkipSegments && !useAnimeSkip) || !episode) return;
    try {
      const resolvedId = useSkipSegments && meta?.id ? await corePlaybackIntroLookupContentId(meta.id) : '';
      const imdbId = resolvedId.startsWith('tt') ? resolvedId : '';
      const tmdbId = !imdbId && /^\d+$/.test(resolvedId) ? Number(resolvedId) : undefined;
      const { segments, coverage } = await fetchPlaybackSkipSegments({
        imdbId,
        tmdbId,
        season: episode.season ?? 1,
        episode: episode.episode ?? episode.number ?? 1,
        title: meta?.name ?? '',
        useSkipSegments,
        useAnimeSkip,
        animeSkipClientId: typeof prefs.animeSkipClientId === 'string' ? prefs.animeSkipClientId : undefined,
      });
      setPlayerSkipSegments(segments);
      setSkipSegmentCoverage(coverage);
    } catch {
      setPlayerSkipSegments([]);
    }
  }, [stateRef]);

  const resolveFollowingEpisode = useCallback(async (meta?: Meta, episode?: Video | null) => {
    setPlayerNextEpisode(null);
    if (!meta || !episode) return;
    try {
      const next = await coreResolveNextEpisode(
        JSON.stringify(meta.videos ?? []),
        episode.season ?? 1,
        episode.episode ?? episode.number ?? 1,
        Date.now(),
        true,
      );
      setPlayerNextEpisode((next as Video | null) ?? null);
    } catch {
      setPlayerNextEpisode(null);
    }
  }, []);

  const saveProgress = useCallback(async () => {
    const snapshot = playbackSnapshotRef.current;
    const meta = playingMetaRef.current;
    if (!meta || !snapshotIsUsable(snapshot)) return;
    await persistLastPlaybackSource(meta, playingStreamRef.current).catch(() => undefined);
    await persistPlaybackProgress({
      meta,
      episode: playingEpisodeRef.current,
      stream: playingStreamRef.current,
      nextEpisode: null,
      snapshot,
      streamIndex: stateRef.current.player.currentStreamIndex ?? null,
      prefs: appPrefs(stateRef.current),
      updateState,
    }).catch(() => undefined);
  }, [stateRef, updateState]);

  const reportPlaybackEvent = useCallback((event: ScrobbleEvent) => {
    const snapshot = playbackSnapshotRef.current;
    if (!snapshotIsUsable(snapshot)) return;
    void runScrobbleLifecycle({
      event,
      profile: activeProfileRef.current,
      meta: playingMetaRef.current,
      episode: playingEpisodeRef.current,
      snapshot,
      flags: {
        hasStarted: scrobbleStartedRef.current,
        hasPaused: scrobbleWasPausedRef.current,
        hasStopped: scrobbleStoppedRef.current,
      },
      onProfileUpdated,
    }).then((action) => {
      if (action === 'start') {
        scrobbleStartedRef.current = true;
        scrobbleWasPausedRef.current = false;
      }
      if (action === 'pause') scrobbleWasPausedRef.current = true;
      if (action === 'stop') scrobbleStoppedRef.current = true;
    }).catch(() => undefined);
  }, [onProfileUpdated]);

  useEffect(() => {
    if (!playerUrl) return undefined;
    const interval = setInterval(() => { void saveProgress(); }, 30000);
    return () => clearInterval(interval);
  }, [playerUrl, saveProgress]);

  useEffect(() => {
    if (!playerUrl) return undefined;
    const flush = () => { void saveProgress(); };
    window.addEventListener('pagehide', flush);
    return () => window.removeEventListener('pagehide', flush);
  }, [playerUrl, saveProgress]);

  const handlePlay = useCallback(async (stream: Stream, meta?: Meta, episode?: Video | null, resumeAtSeconds?: number) => {
    const source = stream.playableUrl ?? stream.url;
    if (!source) return;
    const isTorrent = Boolean(stream.isTorrent || stream.infoHash);
    setPlayerPlaybackError(null);
    setPlayerLoadingOverlay({ title: meta?.name, episodeLine: episode?.title });
    try {
      const url = isTorrent
        ? await platformInvoke<string>('start_torrent_stream', { streamJson: JSON.stringify(stream), title: meta?.name ?? null, preferences: null })
        : source;
      const headers = stream.behaviorHints?.proxyHeaders as Record<string, string> | undefined;
      const subtitlePromise = Promise.all([
        loadEnabledAddons(),
        corePlaybackPreparePlan({ stream, meta, episode, preferredPlayer: 'web' }),
      ])
        .then(([addons, plan]) => resolvePlaybackSubtitles(
          stream,
          meta,
          episode,
          typeof plan?.subtitleExtraArgs === 'string' ? plan.subtitleExtraArgs : undefined,
          addons,
        ))
        .catch(() => ({ subtitles: [], failedAddons: [] }));
      const probe = isTorrent ? null : await probeStream(url, headers);
      const resolvedSubtitles = await subtitlePromise;
      const playback = choosePlaybackUrl(url, source, probe, headers, resumeAtSeconds);
      const playbackUrl = playback.url;
      playingStreamRef.current = stream;
      playingMetaRef.current = meta ?? null;
      playingEpisodeRef.current = episode ?? null;
      playbackSnapshotRef.current = null;
      scrobbleStartedRef.current = false;
      scrobbleWasPausedRef.current = false;
      scrobbleStoppedRef.current = false;
      setPlayerResumeAt(playback.mode === 'transcode' ? undefined : resumeAtSeconds);
      setPlayerUrl(playbackUrl);
      setPlayerTitle(meta?.name);
      setPlayerEpisodeTitle(episode?.title);
      setPlayerEpisode(episode ?? null);
      setPlayerUsesTorrent(isTorrent);
      setPlayerPosterUrl(meta?.background ?? meta?.poster);
      setPlayerLogoUrl(meta?.logo);
      setPlayerMetaId(meta?.id);
      setPlayerStreamHeaders(headers);
      setPlayerSubtitles(resolvedSubtitles.subtitles);
      setPlayerCodecs(probe ? { videoCodec: probe.videoCodec, audioCodec: probe.audioCodec } : null);
      setPlayerLoadingOverlay(null);
      void resolveSkipSegments(meta, episode);
      void resolveFollowingEpisode(meta, episode);
    } catch (error) {
      setPlayerPlaybackError(error instanceof Error ? error.message : String(error));
      setPlayerLoadingOverlay((current) => current ? { ...current, error: error instanceof Error ? error.message : String(error) } : null);
    }
  }, []);

  const playNextEpisode = useCallback(async () => {
    const meta = playingMetaRef.current;
    const stream = playingStreamRef.current;
    if (!meta || !playerNextEpisode || !stream) return;
    await handlePlay(stream, meta, playerNextEpisode);
  }, [handlePlay, playerNextEpisode]);

  const closePlayer = useCallback(async () => {
    await saveProgress();
    reportPlaybackEvent('stop');
    if (playerUsesTorrent) await platformInvoke('stop_torrent_stream').catch(() => undefined);
    setPlayerUrl(null);
    setPlayerLoadingOverlay(null);
    setPlayerUsesTorrent(false);
    setPlayerSubtitles([]);
    setPlayerCodecs(null);
    setPlayerResumeAt(undefined);
    playingStreamRef.current = null;
    playingMetaRef.current = null;
    playingEpisodeRef.current = null;
    playbackSnapshotRef.current = null;
  }, [playerUsesTorrent, reportPlaybackEvent, saveProgress]);

  return {
    playerLoadingOverlay, playerUrl, playerTorrentTelemetryContext: null, playerTitle, playerEpisodeTitle, playerEpisode,
    playerUsesTorrent, playerPosterUrl, playerLogoUrl, playerMetaId, playerSubtitleUrl: undefined, playerSubtitles, playerCodecs, playerResumeAt, playerSkipSegments, playerNextEpisode,
    playNextEpisode, playbackSnapshotRef, reportPlaybackEvent, playerStreamHeaders,
    playingStreamRef, playingMetaRef, playerPlaybackError, playerSubtitleWarning: null, dismissSubtitleWarning: () => {},
    handlePlay, closePlayer, notifyFirstFrame: () => {}, flushProgressOnQuit: saveProgress, skipSegmentCoverage,
  };
}
