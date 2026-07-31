import { useEffect, useState } from 'react';
import { corePlaybackIntroLookupContentId } from '../../core/engine';

export function usePlayerIntroDb(metaId: string | undefined, enabled: boolean) {
  const [ids, setIds] = useState<{ imdbId: string | null; tmdbId: number | null }>({ imdbId: null, tmdbId: null });
  useEffect(() => {
    if (!enabled || !metaId) {
      setIds({ imdbId: null, tmdbId: null });
      return;
    }
    let cancelled = false;
    void corePlaybackIntroLookupContentId(metaId).then((id) => {
      if (cancelled || !id) return;
      if (id.startsWith('tt')) setIds({ imdbId: id, tmdbId: null });
      else if (/^\d+$/.test(id)) setIds({ imdbId: null, tmdbId: Number(id) });
      else setIds({ imdbId: null, tmdbId: null });
    }).catch(() => undefined);
    return () => { cancelled = true; };
  }, [enabled, metaId]);
  return ids;
}
