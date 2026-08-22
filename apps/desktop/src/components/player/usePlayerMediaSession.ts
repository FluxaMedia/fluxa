import { useEffect } from 'react';
import { platformEmit as emit } from '../../platform/browser';
import { sendCmd, type FeedbackFlash } from './PlayerOverlayPrimitives';

export function usePlayerMediaSession({
  title,
  episodeTitle,
  posterUrl,
  setPaused,
  startSeekOverlay,
  flashFeedback,
}: {
  title: string;
  episodeTitle: string;
  posterUrl?: string;
  setPaused: (paused: boolean) => void;
  startSeekOverlay: () => void;
  flashFeedback: (icon: FeedbackFlash['icon'], label: string) => void;
}) {
  useEffect(() => {
    const mediaSession = typeof navigator !== 'undefined' ? navigator.mediaSession : undefined;
    if (!mediaSession || typeof MediaMetadata === 'undefined') return;
    mediaSession.metadata = new MediaMetadata({
      title: episodeTitle || title || 'Fluxa',
      artist: episodeTitle ? title : undefined,
      artwork: posterUrl ? [{ src: posterUrl }] : undefined,
    });
    mediaSession.setActionHandler('play', () => {
      setPaused(false);
      sendCmd('set pause no');
    });
    mediaSession.setActionHandler('pause', () => {
      setPaused(true);
      sendCmd('set pause yes');
    });
    mediaSession.setActionHandler('seekbackward', () => {
      startSeekOverlay();
      flashFeedback('seekBack', '-10s');
      sendCmd('seek -10 relative');
    });
    mediaSession.setActionHandler('seekforward', () => {
      startSeekOverlay();
      flashFeedback('seekFwd', '+10s');
      sendCmd('seek 10 relative');
    });
    mediaSession.setActionHandler('nexttrack', () => {
      void emit('native-player-next-episode', null);
    });
    return () => {
      mediaSession.setActionHandler('play', null);
      mediaSession.setActionHandler('pause', null);
      mediaSession.setActionHandler('seekbackward', null);
      mediaSession.setActionHandler('seekforward', null);
      mediaSession.setActionHandler('nexttrack', null);
      mediaSession.metadata = null;
    };
  }, [episodeTitle, flashFeedback, posterUrl, setPaused, startSeekOverlay, title]);
}
