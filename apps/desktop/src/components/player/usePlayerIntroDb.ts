import { useEffect, useState } from 'react';
import { corePlaybackExternalIds, corePlaybackIntroLookupContentId } from '../../core/engine';

export function usePlayerIntroDb(metaId: string | undefined, enabled: boolean) {
  const [ids, setIds] = useState<{ imdbId: string | null; tmdbId: number | null }>({ imdbId: null, tmdbId: null });
  useEffect(() => {
    if (!enabled || !metaId) {
      setIds({ imdbId: null, tmdbId: null });
      return;
    }
    let cancelled = false;
      void corePlaybackIntroLookupContentId(metaId)
      .then((id) => corePlaybackExternalIds(id))
      .then(({ imdbId, tmdbId }) => {
        if (cancelled) return;
        setIds({ imdbId, tmdbId });
      })
      .catch(() => undefined);
    return () => {
      cancelled = true;
    };
  }, [enabled, metaId]);
  return ids;
}
