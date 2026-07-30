import { useCallback, useEffect, useRef, useState, type RefObject } from 'react';
import { t } from '../i18n';
import { getCurrentWindow } from '@tauri-apps/api/window';
import type { EmbeddedMpvStatus, TorrentStats } from '../core/mpvPlayer';
import { embeddedMpvSetCursorVisible } from '../core/mpvPlayer';
import type { Meta, Stream, Video } from '../core/types';
import type { EpisodeInfo } from './player/EpisodePanel';
import { streamShellPlan } from '../core/streamLinks';
import { PlayerBufferingOverlay } from './player/PlayerBufferingOverlay';
import { PlayerStatusToasts } from './player/PlayerStatusToasts';
import { PlayerContextMenu } from './player/PlayerContextMenu';
import { PlayerHeader } from './player/PlayerHeader';
import { usePlayerKeyboardShortcuts } from './player/usePlayerKeyboardShortcuts';
import { usePlayerPlaybackNavigation } from './player/usePlayerPlaybackNavigation';
import { usePlayerLiveTelemetry } from './player/usePlayerLiveTelemetry';
import { usePlayerSubtitleControls } from './player/usePlayerSubtitleControls';
import { usePlayerTelemetryState } from './player/usePlayerTelemetryState';
import { usePlayerNavigationState } from './player/usePlayerNavigationState';
import { usePlayerWindowMode } from './player/usePlayerWindowMode';
import { SoftwareVideoCanvas } from './player/SoftwareVideoCanvas';
import { usePlayerCasting } from './player/usePlayerCasting';
import { PlayerFeedback } from './player/PlayerFeedback';
import { PlayerSkipPrompt } from './player/PlayerSkipPrompt';
import { PlayerBottomControls } from './player/PlayerBottomControls';
import { PlayerMiniMode } from './player/PlayerMiniMode';
import { PlayerStreamLinksMenu } from './player/PlayerStreamLinksMenu';
import { usePlayerSeekInteractions } from './player/usePlayerSeekInteractions';
import { usePlayerMediaSession } from './player/usePlayerMediaSession';
import { usePlayerTrackControls } from './player/usePlayerTrackControls';
import { usePlayerAnime4k } from './player/usePlayerAnime4k';
import { usePlayerUtilityActions } from './player/usePlayerUtilityActions';
import { usePlayerCenterGesture } from './player/usePlayerCenterGesture';
import { usePlayerOverlayInput } from './player/usePlayerOverlayInput';
import { PlayerOverlayDecorations } from './player/PlayerOverlayDecorations';
import { usePlayerTitleReset } from './player/usePlayerTitleReset';
import { usePlayerIntroDb } from './player/usePlayerIntroDb';
import { PlayerSupplementalPanels } from './player/PlayerSupplementalPanels';
import { PlayerTrackPanel } from './player/PlayerTrackPanel';
import { PlayerOverlayStyles } from './player/PlayerOverlayStyles';
import { coreResolveNextEpisode } from '../core/engine';
import { castSetVolume } from '../core/cast';
import { loadShortcutOverrides, onShortcutsChanged, type ShortcutOverrides } from '../core/shortcuts';

import { sendCmd, type Chapter, type FeedbackFlash } from './player/PlayerOverlayPrimitives';

interface Props {
  closePlayer: () => Promise<void>;
  onFirstFrame?: () => void;
  initialTitle?: string;
  initialEpisodeTitle?: string;
  currentEpisode?: Video | null;
  isTorrentStream?: boolean;
  initialPosterUrl?: string;
  initialLogoUrl?: string;
  metaId?: string;
  initialSubtitleUrl?: string;
  initialStreamHeaders?: Record<string, string>;
  streamRef?: RefObject<Stream | null>;
  metaRef?: RefObject<Meta | null>;
  playbackUrl?: string | null;
  playbackError?: string | null;
  subtitleWarning?: string[] | null;
  onDismissSubtitleWarning?: () => void;
  softwareVideoActive?: boolean;
  bannerOffset?: number;
  prefs?: Record<string, unknown>;
  onDispatch?: (actionJson: string) => Promise<void> | void;
}

export function ReactPlayerOverlay({ closePlayer, onFirstFrame, initialTitle, initialEpisodeTitle, currentEpisode, isTorrentStream = false, initialPosterUrl, initialLogoUrl, metaId, initialSubtitleUrl, initialStreamHeaders, streamRef, metaRef, playbackUrl, playbackError, subtitleWarning, onDismissSubtitleWarning, softwareVideoActive = false, bannerOffset = 0, prefs, onDispatch }: Props) {
  const playerTelemetry = usePlayerTelemetryState();
  const { paused, muted, volumeLevel, isBuffering, bufferingProgress, hdrLabel, statsSnap, torrentStatsSnap, torrentSpeedHistory, setPaused, setMuted, setVolumeLevel, resetTorrentSpeedHistory } = playerTelemetry;
  const [controlsVisible, setControlsVisible] = useState(true);
  const [title, setTitle] = useState(initialTitle ?? '');
  const [episodeTitle, setEpisodeTitle] = useState(initialEpisodeTitle ?? '');
  const playerNavigation = usePlayerNavigationState();
  const { chapters, skipSegments, nextEpSubtitle, nextEpThreshold, autoPlayNextEpisode, autoPlayCountdownSecs, countdown, nextEpDismissed, episodes, activeSkip, autoSkipSegments, showNextEpCard, setChapters, setSkipSegments, setNextEpSubtitle, setNextEpThreshold, setAutoPlayNextEpisode, setAutoPlayCountdownSecs, setCountdown, setNextEpDismissed, setEpisodes, setActiveSkip, setAutoSkipSegments, setShowNextEpCard } = playerNavigation;
  const [nextEpThumbnail, setNextEpThumbnail] = useState<string | null>(null);
  useEffect(() => {
    let active = true;
    if (!currentEpisode) {
      setNextEpThumbnail(null);
      return () => { active = false; };
    }
    void coreResolveNextEpisode(
      JSON.stringify(episodes),
      currentEpisode.season ?? 1,
      currentEpisode.episode ?? currentEpisode.number ?? 0,
      Date.now(),
      false,
    ).then((next) => {
      if (active) setNextEpThumbnail((next as EpisodeInfo | null)?.thumbnail ?? null);
    });
    return () => { active = false; };
  }, [episodes, currentEpisode]);
  const [showEpisodePanel, setShowEpisodePanel] = useState(false);
  const autoSkippedKeysRef = useRef<Set<string>>(new Set());
  const [streamLinksMenuPoint, setStreamLinksMenuPoint] = useState<{ x: number; y: number } | null>(null);
  const [streamLinksPlan, setStreamLinksPlan] = useState<{ isTorrent: boolean; sourceLink?: string; downloadLink?: string } | null>(null);
  const streamLinksBtnRef = useRef<HTMLButtonElement | null>(null);
  const [showPlayerSettings, setShowPlayerSettings] = useState(false);
  const [showSegmentMarker, setShowSegmentMarker] = useState(false);
  const introDbSubmitEnabled = prefs?.introDbSubmitEnabled === true;
  const introDbApiKey = typeof prefs?.introDbApiKey === 'string' ? prefs.introDbApiKey : '';
  const introDbImdbId = usePlayerIntroDb(metaId, introDbSubmitEnabled);
  const [showTorrentPopover, setShowTorrentPopover] = useState(false);
  const [feedback, setFeedback] = useState<FeedbackFlash | null>(null);
  const [showSeekOverlay, setShowSeekOverlay] = useState(false);
  const seekOverlayTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const [contextMenu, setContextMenu] = useState<{ x: number; y: number } | null>(null);
  const [showShortcutsHelp, setShowShortcutsHelp] = useState(false);
  const [shortcutOverrides, setShortcutOverrides] = useState<ShortcutOverrides>({});
  useEffect(() => {
    loadShortcutOverrides().then(setShortcutOverrides);
    return onShortcutsChanged(setShortcutOverrides);
  }, []);
  const cycleAnime4kModeRef = useRef<(direction: 1 | -1) => void>(() => {});
  const [showStats, setShowStats] = useState(false);
  const bufferHistoryRef = useRef<number[]>([]);
  const netSpeedHistoryRef = useRef<number[]>([]);
  const liveStatusRef = useRef<EmbeddedMpvStatus | null>(null);
  const torrentStatsRef = useRef<TorrentStats | null>(null);
  const stallCountRef = useRef(0);
  const prevPausedForCacheRef = useRef(false);

  const seekFillRef = useRef<HTMLDivElement>(null);
  const seekBufferRef = useRef<HTMLDivElement>(null);
  const seekDotRef = useRef<HTMLDivElement>(null);
  const currentTimeRef = useRef<HTMLSpanElement>(null);
  const durationRef = useRef<HTMLSpanElement>(null);
  const seekbarRef = useRef<HTMLDivElement>(null);
  const overlayRef = useRef<HTMLDivElement>(null);
  const subTrackBtnRef = useRef<HTMLButtonElement>(null);
  const audioTrackBtnRef = useRef<HTMLButtonElement>(null);
  const speedBtnRef = useRef<HTMLButtonElement>(null);
  const castBtnRef = useRef<HTMLButtonElement>(null);
  const playerSettingsBtnRef = useRef<HTMLButtonElement>(null);
  const torrentBtnRef = useRef<HTMLButtonElement>(null);
  const segFillRefs = useRef<(HTMLDivElement | null)[]>([]);
  const segBufRefs = useRef<(HTMLDivElement | null)[]>([]);
  const skipFillRef = useRef<HTMLDivElement>(null);
  const chapterSegmentsRef = useRef<Array<{ start: number; end: number }> | null>(null);
  const chaptersRef = useRef<Chapter[]>([]);

  const posRef = useRef(0);
  const durRef = useRef(0);
  const pausedRef = useRef(false);
  const lastActivityRef = useRef(Date.now());
  const isOverControlsRef = useRef(false);
  const miniProgressRef = useRef<HTMLDivElement>(null);
  const isDraggingRef = useRef(false);
  const dragPosRef = useRef(0);
  const lastSeekAtRef = useRef(0);
  const feedbackTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const holdTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const holdActiveRef = useRef(false);
  const preSpeedRef = useRef(1.0);
  const controlsVisibleRef = useRef(true);
  const episodePanelOpenRef = useRef(false);
  const firstFrameFiredRef = useRef(false);
  const hasAppliedInitialFillRef = useRef(false);
  useEffect(() => {
    firstFrameFiredRef.current = false;
    hasAppliedInitialFillRef.current = false;
  }, [currentEpisode?.id]);
  const activeSkipKeyRef = useRef<string | null>(null);
  const discordPresenceKeyRef = useRef<string | null>(null);
  const discordPresenceSentAtRef = useRef(0);

  const resetActivity = useCallback(() => {
    lastActivityRef.current = Date.now();
    if (!controlsVisibleRef.current) {
      controlsVisibleRef.current = true;
      setControlsVisible(true);
      if (overlayRef.current) overlayRef.current.classList.remove('fluxa-cursor-hidden');
      getCurrentWindow().setCursorVisible(true).catch(() => {});
      embeddedMpvSetCursorVisible(true).catch(() => {});
    }
  }, []);
  const { trackPopover, setTrackPopover, playbackSpeed, setPlaybackSpeed, audioTracks, subTracks, openTrackPopover, setSpeed, selectTrack, disableSubs } = usePlayerTrackControls(resetActivity);

  const { miniPlayerActive, isFullscreenRef, setPlayerFullscreen, toggleFullscreen, toggleMiniPlayer } = usePlayerWindowMode(resetActivity);
  const { activeCastDeviceId, activeCastDeviceIdRef, activeCastDeviceName, castDevices, castDiscovering, castPaused, castPopoverOpen, disconnectCast, openCastPopover, openCastPopoverRef, selectCastDevice, setCastPopoverOpen, toggleCastPause } = usePlayerCasting({ title, episodeTitle, initialSubtitleUrl, initialStreamHeaders, resetActivity });

  const flashFeedback = useCallback((icon: FeedbackFlash['icon'], label: string) => {
    setFeedback({ icon, label });
    if (feedbackTimerRef.current) clearTimeout(feedbackTimerRef.current);
    feedbackTimerRef.current = setTimeout(() => setFeedback(null), 700);
  }, []);
  const { abLoopStage, setAbLoopStage, cycleAbLoop, cycleAbLoopRef, takeScreenshot, takeScreenshotRef, copyTimestamp } = usePlayerUtilityActions({ title, posRef, resetActivity, flashFeedback });

  const startSeekOverlay = useCallback(() => {
    setShowSeekOverlay(true);
    if (seekOverlayTimerRef.current) clearTimeout(seekOverlayTimerRef.current);
    seekOverlayTimerRef.current = setTimeout(() => setShowSeekOverlay(false), 30000);
  }, []);

  usePlayerMediaSession({ title, episodeTitle, posterUrl: initialPosterUrl, setPaused, startSeekOverlay, flashFeedback });

  const { applyFills, onSeekMouseDown } = usePlayerSeekInteractions({ durRef, lastSeekAtRef, activeCastDeviceIdRef, seekbarRef, seekFillRef, seekBufferRef, seekDotRef, chapterSegmentsRef, segmentFillRefs: segFillRefs, segmentBufferRefs: segBufRefs, isDraggingRef, dragPosRef, startSeekOverlay, resetActivity });

  usePlayerLiveTelemetry({
    skipSegments, nextEpSubtitle, nextEpThreshold, nextEpDismissed, trackPopover, title, episodeTitle, initialPosterUrl, metaId, autoSkipSegments, isTorrentStream, playbackUrl, showStats, showTorrentPopover, onFirstFrame, applyFills, flashFeedback,
    telemetry: playerTelemetry, setShowSeekOverlay, setControlsVisible, setActiveSkip, setShowNextEpCard,
    liveStatusRef, torrentStatsRef, prevPausedForCacheRef, stallCountRef, bufferHistoryRef, netSpeedHistoryRef, posRef, durRef, pausedRef, firstFrameFiredRef, hasAppliedInitialFillRef, currentTimeRef, durationRef, lastSeekAtRef, isDraggingRef, seekOverlayTimerRef, lastActivityRef, controlsVisibleRef, overlayRef, episodePanelOpenRef, isOverControlsRef, miniProgressRef, activeSkipKeyRef, autoSkippedKeysRef, skipFillRef, discordPresenceKeyRef, discordPresenceSentAtRef,
  });

  usePlayerPlaybackNavigation({ chaptersRef, setChapters, setSkipSegments, setNextEpSubtitle, setNextEpDismissed, setNextEpThreshold, setAutoPlayNextEpisode, setAutoPlayCountdownSecs, setAutoSkipSegments, setEpisodes, showNextEpCard, autoPlayNextEpisode, nextEpDismissed, autoPlayCountdownSecs, setCountdown, activeSkip, setActiveSkip, countdown, pausedRef, resetActivity });
  usePlayerTitleReset({ setTitle, setEpisodeTitle, setAbLoopStage, setNextEpisodeDismissed: setNextEpDismissed, autoSkippedKeysRef, stallCountRef, prevPausedForCacheRef, bufferHistoryRef, networkHistoryRef: netSpeedHistoryRef, resetTorrentSpeedHistory });

  const triggerActiveSkip = useCallback(() => {
    if (!activeSkip) return false;
    sendCmd(`set time-pos ${Math.floor(activeSkip.endMs / 1000)}`);
    flashFeedback('seekFwd', activeSkip.label);
    return true;
  }, [activeSkip, flashFeedback]);

  usePlayerKeyboardShortcuts({ closePlayer, contextMenu, setContextMenu, flashFeedback, nextEpSubtitle, playbackSpeed, setPlaybackSpeed, setPlayerFullscreen, toggleFullscreen, toggleMiniPlayer, shortcutOverrides, showEpisodePanel, setShowEpisodePanel, showShortcutsHelp, setShowShortcutsHelp, startSeekOverlay, trackPopover, setTrackPopover, triggerActiveSkip, episodePanelOpenRef, isFullscreenRef, holdTimerRef, holdActiveRef, preSpeedRef, pausedRef, posRef, durRef, chaptersRef, cycleAbLoopRef, openCastPopoverRef, takeScreenshotRef, cycleAnime4kModeRef, setPaused, setShowStats });
  useEffect(() => {
    return () => {
      getCurrentWindow().setCursorVisible(true).catch(() => {});
      embeddedMpvSetCursorVisible(true).catch(() => {});
    };
  }, []);


  const onOverlayWheel = usePlayerOverlayInput({ resetActivity, startSeekOverlay, flashFeedback });

  const setSubtitlePref = useCallback(<K extends string>(key: K, value: string | boolean) => {
    void onDispatch?.(JSON.stringify({ type: 'settingsChanged', key, value }));
  }, [onDispatch]);

  const { anime4kEnabled, toggleAnime4k, cycleAnime4kMode } = usePlayerAnime4k({ prefs, persistPreference: setSubtitlePref, flashFeedback });

  const subtitleControls = usePlayerSubtitleControls({ prefs, persistPreference: setSubtitlePref, flashFeedback });
  useEffect(() => { cycleAnime4kModeRef.current = cycleAnime4kMode; }, [cycleAnime4kMode]);

  const { onCenterMouseDown, releaseCenterHold, onCenterClick } = usePlayerCenterGesture({ playbackSpeed, preSpeedRef, pausedRef, episodePanelOpenRef, showEpisodePanel, setShowEpisodePanel, trackPopover, setTrackPopover, setPaused, resetActivity, flashFeedback, toggleFullscreen, holdTimerRef, holdActiveRef });

  const opacityStyle: React.CSSProperties = {
    opacity: controlsVisible ? 1 : 0,
    transition: 'opacity 0.4s ease',
    pointerEvents: controlsVisible ? 'auto' : 'none',
  };

  const dur = durRef.current;
  const chapterSegments = chapters.length >= 2 && dur > 0
    ? chapters.map((ch, i) => {
        const start = ch.startMs / 1000 / dur;
        const end = i + 1 < chapters.length ? chapters[i + 1].startMs / 1000 / dur : 1;
        return { start, end };
      })
    : null;
  chapterSegmentsRef.current = chapterSegments;
  const skipMarkers = dur > 0
    ? skipSegments
        .map((seg) => ({
          start: Math.max(0, Math.min(1, (seg.startTime / 1000) / dur)),
          end: Math.max(0, Math.min(1, (seg.endTime / 1000) / dur)),
        }))
        .filter((seg) => seg.end > seg.start)
    : [];

  if (miniPlayerActive) {
    return <PlayerMiniMode overlayRef={overlayRef} miniProgressRef={miniProgressRef} opacityStyle={opacityStyle} paused={paused} onTogglePause={() => { resetActivity(); flashFeedback(paused ? 'play' : 'pause', ''); setPaused((value) => !value); sendCmd('cycle pause'); }} onRestore={() => { resetActivity(); void toggleMiniPlayer(); }} onClose={() => { void closePlayer(); }} onActivity={resetActivity} />;
  }

  return (
    <div
      ref={overlayRef}
      style={{ position: 'fixed', inset: 0, zIndex: 9998, display: 'flex', flexDirection: 'column', background: softwareVideoActive ? '#000' : 'transparent' }}
      onWheel={onOverlayWheel}
      onContextMenu={(e) => { e.preventDefault(); resetActivity(); setContextMenu({ x: e.clientX, y: e.clientY }); }}
    >
      {isBuffering && <PlayerBufferingOverlay logoUrl={initialLogoUrl} progress={bufferingProgress} />}
      <PlayerOverlayStyles />

      {softwareVideoActive && <SoftwareVideoCanvas key={currentEpisode?.id} statusRef={liveStatusRef} onFirstFrame={onFirstFrame} />}

      <PlayerOverlayDecorations controlsVisible={controlsVisible} feedback={feedback} muted={muted} volumeLevel={volumeLevel} showSeekOverlay={showSeekOverlay} activeSkip={activeSkip} showNextEpCard={showNextEpCard} nextEpSubtitle={nextEpSubtitle} nextEpThumbnail={nextEpThumbnail} countdown={countdown} autoPlayCountdownSecs={autoPlayCountdownSecs} showEpisodePanel={showEpisodePanel} episodes={episodes} currentEpisode={currentEpisode} skipFillRef={skipFillRef} onActivity={resetActivity} onDismissSkip={() => setActiveSkip(null)} onDismissNextEpisode={() => setNextEpDismissed(true)} onCloseEpisodePanel={() => { setShowEpisodePanel(false); episodePanelOpenRef.current = false; }} />

      <PlayerStatusToasts bannerOffset={bannerOffset} playbackError={playbackError} subtitleWarning={subtitleWarning} onClosePlayer={() => { void closePlayer(); }} onDismissSubtitleWarning={onDismissSubtitleWarning} />

      <PlayerHeader
        style={opacityStyle}
        bannerOffset={bannerOffset}
        title={title}
        episodeTitle={episodeTitle}
        hdrLabel={hdrLabel}
        activeCastDeviceId={activeCastDeviceId}
        activeCastDeviceName={activeCastDeviceName}
        castPaused={castPaused}
        castButtonRef={castBtnRef}
        streamLinksButtonRef={streamLinksBtnRef}
        settingsButtonRef={playerSettingsBtnRef}
        showSegmentMarker={showSegmentMarker}
        canMarkSegments={introDbSubmitEnabled && Boolean(introDbApiKey)}
        onClose={() => { void closePlayer(); }}
        onResetActivity={resetActivity}
        onToggleCastPause={toggleCastPause}
        onOpenCast={() => { void openCastPopover(); }}
        onToggleMiniPlayer={() => { void toggleMiniPlayer(); }}
        onOpenStreamLinks={() => {
          const rect = streamLinksBtnRef.current?.getBoundingClientRect();
          const stream = streamRef?.current;
          if (stream) void streamShellPlan(stream).then(setStreamLinksPlan);
          setStreamLinksMenuPoint(rect ? { x: Math.max(0, rect.right - 216), y: rect.bottom + 8 } : null);
        }}
        onToggleSettings={() => setShowPlayerSettings((value) => !value)}
        onToggleSegmentMarker={() => setShowSegmentMarker((value) => !value)}
      />
      <div style={{ flex: 1, cursor: 'default' }} onMouseDown={onCenterMouseDown} onMouseUp={releaseCenterHold} onMouseLeave={releaseCenterHold} onClick={onCenterClick} />

      {trackPopover && <PlayerTrackPanel type={trackPopover} audioTracks={audioTracks} subTracks={subTracks} playbackSpeed={playbackSpeed} refs={{ audio: audioTrackBtnRef, sub: subTrackBtnRef, speed: speedBtnRef }} onClose={() => setTrackPopover(null)} onSetSpeed={setSpeed} onSelectTrack={selectTrack} onDisableSubs={disableSubs} subtitleControls={subtitleControls} />}

      <PlayerStreamLinksMenu point={streamLinksMenuPoint} onClose={() => setStreamLinksMenuPoint(null)} plan={streamLinksPlan} streamRef={streamRef} metaRef={metaRef} currentEpisode={currentEpisode} />

      <PlayerSupplementalPanels
        cast={castPopoverOpen ? { devices: castDevices, discovering: castDiscovering, activeDeviceId: activeCastDeviceId, anchorRef: castBtnRef, onClose: () => setCastPopoverOpen(false), onSelectDevice: (device) => void selectCastDevice(device), onDisconnect: disconnectCast } : undefined}
        torrent={showTorrentPopover ? { stats: torrentStatsSnap, anchorRef: torrentBtnRef, onClose: () => setShowTorrentPopover(false) } : undefined}
        marker={showSegmentMarker && introDbSubmitEnabled && introDbApiKey ? { onClose: () => setShowSegmentMarker(false), getPosMs: () => posRef.current * 1000, imdbId: introDbImdbId, season: currentEpisode?.season ?? null, episode: currentEpisode?.episode ?? currentEpisode?.number ?? null, apiKey: introDbApiKey } : undefined}
        settings={showPlayerSettings ? { anchorRef: playerSettingsBtnRef, onClose: () => setShowPlayerSettings(false), anime4kEnabled, onToggleAnime4k: toggleAnime4k } : undefined}
        shortcuts={showShortcutsHelp ? { overrides: shortcutOverrides, onClose: () => setShowShortcutsHelp(false) } : undefined}
        stats={showStats ? { stats: statsSnap, torrentStats: torrentStatsSnap, bufferHistory: bufferHistoryRef.current, networkSpeedHistory: netSpeedHistoryRef.current, torrentSpeedHistory, stallCount: stallCountRef.current } : undefined}
      />
      {contextMenu && <PlayerContextMenu point={contextMenu} abLoopStage={abLoopStage} showStats={showStats} onClose={() => setContextMenu(null)} onCycleAbLoop={cycleAbLoop} onCopyTimestamp={() => { void copyTimestamp(); }} onToggleStats={() => setShowStats((value) => !value)} onToggleShortcuts={() => setShowShortcutsHelp((value) => !value)} onOpenAudioTracks={() => { void openTrackPopover('audio'); }} onScreenshot={() => { void takeScreenshot(); }} />}

      <PlayerBottomControls
        style={opacityStyle}
        showEpisodePanel={showEpisodePanel}
        onControlsHover={(hovering) => { isOverControlsRef.current = hovering; }}
        seekbarRef={seekbarRef}
        seekFillRef={seekFillRef}
        seekBufferRef={seekBufferRef}
        seekDotRef={seekDotRef}
        segmentFillRefs={segFillRefs}
        segmentBufferRefs={segBufRefs}
        durationRef={durRef}
        chaptersRef={chaptersRef}
        chapterSegments={chapterSegments}
        skipMarkers={skipMarkers}
        onSeekStart={onSeekMouseDown}
        paused={paused}
        muted={muted}
        volumeLevel={volumeLevel}
        onTogglePause={() => { resetActivity(); flashFeedback(paused ? 'play' : 'pause', ''); setPaused((value) => !value); sendCmd('cycle pause'); }}
        onSeek={(seconds) => { resetActivity(); startSeekOverlay(); flashFeedback(seconds < 0 ? 'seekBack' : 'seekFwd', `${seconds > 0 ? '+' : ''}${seconds}s`); sendCmd(`seek ${seconds} relative`); }}
        onToggleMute={() => { resetActivity(); setMuted((value) => !value); sendCmd('cycle mute'); }}
        onVolumeWheel={(delta) => { resetActivity(); flashFeedback('volume', ''); sendCmd(`add volume ${delta}`); }}
        onSetVolume={(value) => { resetActivity(); flashFeedback('volume', ''); if (activeCastDeviceIdRef.current) { castSetVolume(value / 100); return; } sendCmd(`set volume ${value}`); if (muted && value > 0) sendCmd('set mute no'); }}
        currentTimeRef={currentTimeRef}
        durationLabelRef={durationRef}
        isTorrentStream={isTorrentStream}
        torrentButtonRef={torrentBtnRef}
        showTorrentPopover={showTorrentPopover}
        onToggleTorrent={() => { resetActivity(); setShowTorrentPopover((value) => !value); }}
        nextEpisodeSubtitle={nextEpSubtitle}
        episodeCount={episodes.length}
        onToggleEpisodes={() => { resetActivity(); const next = !showEpisodePanel; setShowEpisodePanel(next); episodePanelOpenRef.current = next; }}
        subtitleButtonRef={subTrackBtnRef}
        audioButtonRef={audioTrackBtnRef}
        speedButtonRef={speedBtnRef}
        onOpenTracks={(type) => { void openTrackPopover(type); }}
        playbackSpeed={playbackSpeed}
        onToggleFullscreen={() => { resetActivity(); void toggleFullscreen(); }}
      />
    </div>
  );
}
