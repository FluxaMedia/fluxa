import { useEffect, useState } from 'react';
import { coreTrailerYoutubeVideoIds } from '../core/engineCoreContent';

type TrailerUrl = { url: string };

export function useTrailerVideoIds(trailers: TrailerUrl[] | undefined): string[] {
  const urlsJson = JSON.stringify((trailers ?? []).map((trailer) => trailer.url));
  const [ids, setIds] = useState<string[]>([]);

  useEffect(() => {
    let active = true;
    const urls = JSON.parse(urlsJson) as string[];
    void coreTrailerYoutubeVideoIds(urls).then((next) => {
      if (!active) return;
      setIds((current) => (current.length === next.length && current.every((id, index) => id === next[index]) ? current : next));
    });
    return () => {
      active = false;
    };
  }, [urlsJson]);

  return ids;
}
