import { useCallback, useState } from 'react';
import { embeddedMpvAddSubtitle, playerGetTrackOptions, type PlayerTrackOption } from '../../core/mpvPlayer';
import type { PlayerSubtitleSource } from '../../core/playerUtils';
import { sendCmd } from './PlayerOverlayPrimitives';

export type PlayerTrackPopover = 'audio' | 'sub' | 'speed' | null;

const PENDING_PREFIX = 'pending:';

function subtitleLabel(subtitle: PlayerSubtitleSource): string {
  return subtitle.addonName || subtitle.label || subtitle.lang || 'Subtitle';
}

// mpv only knows the tracks that were already sub-added. Everything the addons
// resolved but we did not download yet is listed alongside them as a pending
// row, and downloaded the moment the viewer picks it.
function withPendingSubtitles(tracks: PlayerTrackOption[], subtitles: PlayerSubtitleSource[]): PlayerTrackOption[] {
  const loaded = new Set(tracks.map((track) => track.source).filter(Boolean));
  const pending = subtitles
    .filter((subtitle) => !loaded.has(subtitle.url) && !loaded.has(subtitleLabel(subtitle)))
    .map((subtitle) => ({
      id: `${PENDING_PREFIX}${subtitle.url}`,
      label: subtitleLabel(subtitle),
      selected: false,
      lang: subtitle.lang ?? null,
      source: subtitleLabel(subtitle),
      external: true,
      format: null,
    }));
  return [...tracks, ...pending];
}

export function usePlayerTrackControls(resetActivity: () => void, subtitles: PlayerSubtitleSource[] = []) {
  const [trackPopover, setTrackPopover] = useState<PlayerTrackPopover>(null);
  const [playbackSpeed, setPlaybackSpeed] = useState(1);
  const [audioTracks, setAudioTracks] = useState<PlayerTrackOption[]>([]);
  const [subTracks, setSubTracks] = useState<PlayerTrackOption[]>([]);
  const [loadingTrackId, setLoadingTrackId] = useState<string | null>(null);
  const [failedTrackId, setFailedTrackId] = useState<string | null>(null);

  const openTrackPopover = useCallback(
    async (type: Exclude<PlayerTrackPopover, null>) => {
      resetActivity();
      if (trackPopover === type) {
        setTrackPopover(null);
        return;
      }
      if (type === 'audio') {
        try {
          setAudioTracks(await playerGetTrackOptions('audio'));
        } catch {}
      } else if (type === 'sub') {
        setFailedTrackId(null);
        try {
          setSubTracks(withPendingSubtitles(await playerGetTrackOptions('sub'), subtitles));
        } catch {
          setSubTracks(withPendingSubtitles([], subtitles));
        }
      }
      setTrackPopover(type);
    },
    [resetActivity, subtitles, trackPopover],
  );

  const setSpeed = useCallback((speed: number) => {
    sendCmd(`set speed ${speed.toFixed(2)}`);
    setPlaybackSpeed(speed);
    setTrackPopover(null);
  }, []);

  const selectSubtitleTrack = useCallback((id: string) => {
    sendCmd(`set sid ${id}`);
    setTrackPopover(null);
    setSubTracks((tracks) => tracks.map((track) => ({ ...track, selected: track.id === id })));
  }, []);

  const loadPendingSubtitle = useCallback(
    async (pendingId: string) => {
      const url = pendingId.slice(PENDING_PREFIX.length);
      const subtitle = subtitles.find((entry) => entry.url === url);
      if (!subtitle) return;
      setFailedTrackId(null);
      setLoadingTrackId(pendingId);
      const before = new Set((await playerGetTrackOptions('sub').catch(() => [])).map((track) => track.id));
      try {
        await embeddedMpvAddSubtitle(url, subtitleLabel(subtitle), subtitle.lang);
      } catch {
        setLoadingTrackId(null);
        setFailedTrackId(pendingId);
        return;
      }
      // sub-add does not report the new track id, so the track list is re-read
      // and the entry that was not there before is the one that was just added.
      const after = await playerGetTrackOptions('sub').catch(() => []);
      const added = after.find((track) => !before.has(track.id));
      setLoadingTrackId(null);
      if (!added) {
        setFailedTrackId(pendingId);
        setSubTracks(withPendingSubtitles(after, subtitles));
        return;
      }
      setSubTracks(withPendingSubtitles(after, subtitles).map((track) => ({ ...track, selected: track.id === added.id })));
      sendCmd(`set sid ${added.id}`);
      setTrackPopover(null);
    },
    [subtitles],
  );

  const selectTrack = useCallback(
    (type: 'audio' | 'sub', id: string) => {
      if (type === 'audio') {
        sendCmd(`set aid ${id}`);
        sendCmd('seek 0.1 relative exact');
        setTrackPopover(null);
        setAudioTracks((tracks) => tracks.map((track) => ({ ...track, selected: track.id === id })));
        return;
      }
      if (id.startsWith(PENDING_PREFIX)) {
        void loadPendingSubtitle(id);
        return;
      }
      selectSubtitleTrack(id);
    },
    [loadPendingSubtitle, selectSubtitleTrack],
  );

  const disableSubs = useCallback(() => {
    sendCmd('set sid no');
    setSubTracks((tracks) => tracks.map((track) => ({ ...track, selected: false })));
    setTrackPopover(null);
  }, []);

  return {
    trackPopover,
    setTrackPopover,
    playbackSpeed,
    setPlaybackSpeed,
    audioTracks,
    subTracks,
    loadingTrackId,
    failedTrackId,
    openTrackPopover,
    setSpeed,
    selectTrack,
    disableSubs,
  };
}
