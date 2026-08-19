import { useCallback, useState } from 'react';
import { playerGetTrackOptions, type PlayerTrackOption } from '../../core/mpvPlayer';
import { sendCmd } from './PlayerOverlayPrimitives';

export type PlayerTrackPopover = 'audio' | 'sub' | 'speed' | null;

export function usePlayerTrackControls(resetActivity: () => void) {
  const [trackPopover, setTrackPopover] = useState<PlayerTrackPopover>(null);
  const [playbackSpeed, setPlaybackSpeed] = useState(1);
  const [audioTracks, setAudioTracks] = useState<PlayerTrackOption[]>([]);
  const [subTracks, setSubTracks] = useState<PlayerTrackOption[]>([]);

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
        try {
          setSubTracks(await playerGetTrackOptions('sub'));
        } catch {}
      }
      setTrackPopover(type);
    },
    [resetActivity, trackPopover],
  );

  const setSpeed = useCallback((speed: number) => {
    sendCmd(`set speed ${speed.toFixed(2)}`);
    setPlaybackSpeed(speed);
    setTrackPopover(null);
  }, []);

  const selectTrack = useCallback((type: 'audio' | 'sub', id: string) => {
    sendCmd(type === 'audio' ? `set aid ${id}` : `set sid ${id}`);
    if (type === 'audio') sendCmd('seek 0.1 relative exact');
    setTrackPopover(null);
    if (type === 'audio') setAudioTracks((tracks) => tracks.map((track) => ({ ...track, selected: track.id === id })));
    else setSubTracks((tracks) => tracks.map((track) => ({ ...track, selected: track.id === id })));
  }, []);

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
    openTrackPopover,
    setSpeed,
    selectTrack,
    disableSubs,
  };
}
