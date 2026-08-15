import { useEffect, useRef } from 'react';
import { transcodeUrl } from '../platform/web/stream';

interface Props {
  url: string;
  title?: string;
  onClose: () => Promise<void>;
  onFirstFrame: () => void;
}

export function WebPlayerOverlay({ url, title, onClose, onFirstFrame }: Props) {
  const videoRef = useRef<HTMLVideoElement>(null);
  const fallbackUsedRef = useRef(false);

  useEffect(() => {
    const video = videoRef.current;
    if (!video) return;
    fallbackUsedRef.current = false;
    video.src = url;
    video.load();
    void video.play().catch(() => undefined);
    return () => {
      video.pause();
      video.removeAttribute('src');
    };
  }, [url]);

  return (
    <div style={{ position: 'fixed', inset: 0, zIndex: 1000, background: '#000' }}>
      <video
        ref={videoRef}
        title={title}
        controls
        autoPlay
        onPlaying={onFirstFrame}
        onError={() => {
          const video = videoRef.current;
          if (!video || fallbackUsedRef.current || url.includes('/transcode?')) return;
          fallbackUsedRef.current = true;
          video.src = transcodeUrl(url);
          video.load();
          void video.play().catch(() => undefined);
        }}
        style={{ width: '100%', height: '100%', objectFit: 'contain' }}
      />
      <button type="button" onClick={() => { void onClose(); }} style={{ position: 'absolute', top: 16, right: 16, zIndex: 1 }}>×</button>
    </div>
  );
}
