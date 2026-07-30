import { useEffect } from 'react';
import { emit } from '@tauri-apps/api/event';
import { sendCmd, type FeedbackFlash } from './PlayerOverlayPrimitives';

export function usePlayerMediaSession({ title, episodeTitle, posterUrl, setPaused, startSeekOverlay, flashFeedback }: { title: string; episodeTitle: string; posterUrl?: string; setPaused: (paused: boolean) => void; startSeekOverlay: () => void; flashFeedback: (icon: FeedbackFlash['icon'], label: string) => void }) {
  useEffect(() => {
    if (!navigator.mediaSession) return;
    navigator.mediaSession.metadata = new MediaMetadata({ title: episodeTitle || title || 'Fluxa', artist: episodeTitle ? title : undefined, artwork: posterUrl ? [{ src: posterUrl }] : undefined });
    navigator.mediaSession.setActionHandler('play', () => { setPaused(false); sendCmd('set pause no'); });
    navigator.mediaSession.setActionHandler('pause', () => { setPaused(true); sendCmd('set pause yes'); });
    navigator.mediaSession.setActionHandler('seekbackward', () => { startSeekOverlay(); flashFeedback('seekBack', '-10s'); sendCmd('seek -10 relative'); });
    navigator.mediaSession.setActionHandler('seekforward', () => { startSeekOverlay(); flashFeedback('seekFwd', '+10s'); sendCmd('seek 10 relative'); });
    navigator.mediaSession.setActionHandler('nexttrack', () => { void emit('native-player-next-episode', null); });
    return () => {
      navigator.mediaSession.setActionHandler('play', null);
      navigator.mediaSession.setActionHandler('pause', null);
      navigator.mediaSession.setActionHandler('seekbackward', null);
      navigator.mediaSession.setActionHandler('seekforward', null);
      navigator.mediaSession.setActionHandler('nexttrack', null);
      navigator.mediaSession.metadata = null;
    };
  }, [episodeTitle, flashFeedback, posterUrl, setPaused, startSeekOverlay, title]);
}
