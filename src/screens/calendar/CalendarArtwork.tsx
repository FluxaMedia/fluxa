import { useEffect, useState, type CSSProperties, type ReactNode } from 'react';
import { usePosterSrc } from '../../hooks/usePosterSrc';

export function CalendarArtwork({
  src,
  fallbackSrc,
  style,
  fallback = null,
}: {
  src?: string;
  fallbackSrc?: string;
  style: CSSProperties;
  fallback?: ReactNode;
}) {
  const [currentSrc, setCurrentSrc] = useState(src ?? fallbackSrc);
  const cacheUrl = currentSrc && isTraktArtworkUrl(currentSrc) ? currentSrc : undefined;
  const { src: cachedSrc, failed } = usePosterSrc(cacheUrl);
  const displaySrc = cacheUrl ? cachedSrc : currentSrc;

  useEffect(() => {
    setCurrentSrc(src ?? fallbackSrc);
  }, [src, fallbackSrc]);

  if (!currentSrc || (cacheUrl && (!cachedSrc || failed)) || !displaySrc) return <>{fallback}</>;
  return (
    <img
      key={displaySrc}
      src={displaySrc}
      alt=""
      style={style}
      onError={() => setCurrentSrc((previous) => (previous === fallbackSrc ? undefined : fallbackSrc))}
    />
  );
}

function isTraktArtworkUrl(url: string): boolean {
  try {
    return new URL(url).hostname.endsWith('trakt.tv');
  } catch {
    return false;
  }
}
