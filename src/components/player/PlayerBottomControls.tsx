import {
  AudioLines,
  Captions,
  Fullscreen,
  GalleryVerticalEnd,
  Gauge,
  Pause,
  Play,
  RotateCcw,
  RotateCw,
  Share2,
  SkipForward,
} from 'lucide-react';
import { useRef, useState, type CSSProperties, type PointerEvent, type MutableRefObject, type RefObject } from 'react';
import { platformEmit as emit } from '../../platform/browser';
import { t } from '../../i18n';
import { useIsTouch } from '../../platform/viewport';
import { VolumeBar } from './VolumeBar';
import { PlayerSeekBar } from './PlayerSeekBar';
import { IconVolume, type Chapter } from './PlayerOverlayPrimitives';

export function PlayerBottomControls(props: {
  style: CSSProperties;
  showEpisodePanel: boolean;
  onControlsHover: (hovering: boolean) => void;
  seekbarRef: RefObject<HTMLDivElement | null>;
  seekFillRef: RefObject<HTMLDivElement | null>;
  seekBufferRef: RefObject<HTMLDivElement | null>;
  seekDotRef: RefObject<HTMLDivElement | null>;
  segmentFillRefs: MutableRefObject<(HTMLDivElement | null)[]>;
  segmentBufferRefs: MutableRefObject<(HTMLDivElement | null)[]>;
  durationRef: MutableRefObject<number>;
  chaptersRef: MutableRefObject<Chapter[]>;
  chapterSegments: Array<{ start: number; end: number }> | null;
  skipMarkers: Array<{ start: number; end: number }>;
  onSeekStart: (event: PointerEvent) => void;
  paused: boolean;
  muted: boolean;
  volumeLevel: number;
  onTogglePause: () => void;
  onSeek: (seconds: number) => void;
  onToggleMute: () => void;
  onVolumeWheel: (delta: number) => void;
  onSetVolume: (value: number) => void;
  currentTimeRef: RefObject<HTMLSpanElement | null>;
  durationLabelRef: RefObject<HTMLSpanElement | null>;
  isTorrentStream: boolean;
  torrentButtonRef: RefObject<HTMLButtonElement | null>;
  showTorrentPopover: boolean;
  onToggleTorrent: () => void;
  nextEpisodeSubtitle: string | null;
  episodeCount: number;
  onToggleEpisodes: () => void;
  subtitleButtonRef: RefObject<HTMLButtonElement | null>;
  audioButtonRef: RefObject<HTMLButtonElement | null>;
  speedButtonRef: RefObject<HTMLButtonElement | null>;
  onOpenTracks: (type: 'audio' | 'sub' | 'speed') => void;
  playbackSpeed: number;
  onToggleFullscreen: () => void;
}) {
  const [showVolumeSlider, setShowVolumeSlider] = useState(false);
  const [volumeScrolling, setVolumeScrolling] = useState(false);
  const volumeHideTimer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const volumeScrollTimer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const isTouch = useIsTouch();
  const {
    style,
    showEpisodePanel,
    onControlsHover,
    seekbarRef,
    seekFillRef,
    seekBufferRef,
    seekDotRef,
    segmentFillRefs,
    segmentBufferRefs,
    durationRef,
    chaptersRef,
    chapterSegments,
    skipMarkers,
    onSeekStart,
    paused,
    muted,
    volumeLevel,
    onTogglePause,
    onSeek,
    onToggleMute,
    onVolumeWheel,
    onSetVolume,
    currentTimeRef,
    durationLabelRef,
    isTorrentStream,
    torrentButtonRef,
    showTorrentPopover,
    onToggleTorrent,
    nextEpisodeSubtitle,
    episodeCount,
    onToggleEpisodes,
    subtitleButtonRef,
    audioButtonRef,
    speedButtonRef,
    onOpenTracks,
    playbackSpeed,
    onToggleFullscreen,
  } = props;
  return (
    <div
      className="fluxa-player-controls"
      style={{
        ...style,
        position: 'relative',
        paddingRight: showEpisodePanel ? 380 : 0,
        background: 'transparent',
        zIndex: 2,
        overflow: 'visible',
      }}
      onMouseEnter={() => onControlsHover(true)}
      onMouseLeave={() => onControlsHover(false)}
    >
      <PlayerSeekBar
        barRef={seekbarRef}
        fillRef={seekFillRef}
        bufferRef={seekBufferRef}
        dotRef={seekDotRef}
        segmentFillRefs={segmentFillRefs}
        segmentBufferRefs={segmentBufferRefs}
        durationRef={durationRef}
        chaptersRef={chaptersRef}
        chapterSegments={chapterSegments}
        skipMarkers={skipMarkers}
        onSeekStart={onSeekStart}
      />
      <div style={{ display: 'flex', alignItems: 'center', padding: '0 0.5rem 0.875rem', gap: 0 }}>
        <button
          onClick={(event) => {
            event.stopPropagation();
            onTogglePause();
          }}
          className="fluxa-ibtn"
          style={{ ...iconBtn, width: '3rem', height: '3rem' }}
          title={paused ? t('player.play') : t('player.pause')}
        >
          {paused ? <Play size={26} fill="currentColor" strokeWidth={0} /> : <Pause size={26} fill="currentColor" strokeWidth={0} />}
        </button>
        <button
          onClick={(event) => {
            event.stopPropagation();
            onSeek(-10);
          }}
          className="fluxa-ibtn"
          style={iconBtn}
          title={t('player.seek_back')}
        >
          <RotateCcw size={22} />
        </button>
        <button
          onClick={(event) => {
            event.stopPropagation();
            onSeek(10);
          }}
          className="fluxa-ibtn"
          style={iconBtn}
          title={t('player.seek_forward')}
        >
          <RotateCw size={22} />
        </button>
        <div
          style={{ display: 'flex', alignItems: 'center', position: 'relative', flexShrink: 0 }}
          onMouseEnter={() => {
            if (isTouch) return;
            if (volumeHideTimer.current) clearTimeout(volumeHideTimer.current);
            setShowVolumeSlider(true);
          }}
          onMouseLeave={() => {
            if (isTouch) return;
            volumeHideTimer.current = setTimeout(() => setShowVolumeSlider(false), 200);
          }}
          onWheel={(event) => {
            event.stopPropagation();
            onVolumeWheel(event.deltaY < 0 ? 5 : -5);
            setShowVolumeSlider(true);
            setVolumeScrolling(true);
            if (volumeScrollTimer.current) clearTimeout(volumeScrollTimer.current);
            volumeScrollTimer.current = setTimeout(() => setVolumeScrolling(false), 700);
          }}
        >
          <button
            onClick={(event) => {
              event.stopPropagation();
              if (isTouch && !showVolumeSlider) {
                setShowVolumeSlider(true);
                return;
              }
              onToggleMute();
            }}
            className="fluxa-ibtn"
            style={iconBtn}
            title={muted ? t('player.unmute') : t('player.mute')}
          >
            <IconVolume muted={muted} level={volumeLevel} />
          </button>
          <div
            style={{
              width: showVolumeSlider ? '5.75rem' : 0,
              flexShrink: 0,
              opacity: showVolumeSlider ? 1 : 0,
              pointerEvents: showVolumeSlider ? 'auto' : 'none',
              transition: 'width 0.18s ease, opacity 0.18s ease',
              overflow: 'hidden',
              display: 'flex',
              alignItems: 'center',
              paddingLeft: showVolumeSlider ? '0.25rem' : 0,
            }}
          >
            <VolumeBar value={muted ? 0 : volumeLevel} max={130} forceTooltip={volumeScrolling} onChange={onSetVolume} />
          </div>
        </div>
        <div
          style={{
            display: 'flex',
            alignItems: 'baseline',
            gap: '0.1875rem',
            paddingLeft: '0.625rem',
            pointerEvents: 'none',
            flexShrink: 0,
          }}
        >
          <span ref={currentTimeRef} style={currentTimeStyle}>
            0:00
          </span>
          <span style={{ fontSize: '0.75rem', color: 'rgba(255,255,255,0.3)' }}>/</span>
          <span ref={durationLabelRef} style={durationStyle}>
            0:00
          </span>
        </div>
        <div style={{ flex: 1 }} />
        {isTorrentStream && (
          <button
            ref={torrentButtonRef}
            onClick={(event) => {
              event.stopPropagation();
              onToggleTorrent();
            }}
            className="fluxa-ibtn"
            style={{ ...iconBtn, color: showTorrentPopover ? 'var(--primary-accent-color)' : '#fff' }}
            title={t('player.torrent_stats_title')}
          >
            <Share2 size={20} />
          </button>
        )}
        {nextEpisodeSubtitle && (
          <button
            onClick={(event) => {
              event.stopPropagation();
              void emit('native-player-next-episode', null);
            }}
            className="fluxa-ibtn"
            style={iconBtn}
            title={t('player.next_label', nextEpisodeSubtitle)}
          >
            <SkipForward size={22} />
          </button>
        )}
        {episodeCount > 0 && (
          <button
            onClick={(event) => {
              event.stopPropagation();
              onToggleEpisodes();
            }}
            className="fluxa-ibtn"
            style={iconBtn}
            title={t('player.episodes')}
          >
            <GalleryVerticalEnd size={22} />
          </button>
        )}
        <button
          ref={subtitleButtonRef}
          onClick={(event) => {
            event.stopPropagation();
            onOpenTracks('sub');
          }}
          className="fluxa-ibtn"
          style={iconBtn}
          title={t('player.subtitles')}
        >
          <Captions size={22} />
        </button>
        <button
          ref={audioButtonRef}
          onClick={(event) => {
            event.stopPropagation();
            onOpenTracks('audio');
          }}
          className="fluxa-ibtn"
          style={iconBtn}
          title={t('player.audio')}
        >
          <AudioLines size={22} />
        </button>
        <button
          ref={speedButtonRef}
          onClick={(event) => {
            event.stopPropagation();
            onOpenTracks('speed');
          }}
          className="fluxa-ibtn"
          style={iconBtn}
          title={t('player.speed_label', playbackSpeed === 1 ? t('player.normal') : `${playbackSpeed}×`)}
        >
          <Gauge size={22} />
        </button>
        <button
          onClick={(event) => {
            event.stopPropagation();
            onToggleFullscreen();
          }}
          className="fluxa-ibtn"
          style={iconBtn}
          title={t('player.fullscreen')}
        >
          <Fullscreen size={22} />
        </button>
      </div>
    </div>
  );
}

const iconBtn: CSSProperties = {
  background: 'none',
  border: 'none',
  color: '#fff',
  cursor: 'pointer',
  display: 'flex',
  alignItems: 'center',
  justifyContent: 'center',
  width: '2.75rem',
  height: '2.75rem',
  borderRadius: '0.5rem',
  padding: 0,
  flexShrink: 0,
};
const currentTimeStyle: CSSProperties = {
  fontSize: '0.8125rem',
  fontWeight: 700,
  fontVariantNumeric: 'tabular-nums',
  color: 'rgba(255,255,255,0.9)',
  letterSpacing: '0.0125rem',
};
const durationStyle: CSSProperties = {
  fontSize: '0.75rem',
  fontVariantNumeric: 'tabular-nums',
  color: 'rgba(255,255,255,0.4)',
  letterSpacing: '0.0125rem',
};
