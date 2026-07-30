import { useEffect, useState } from 'react';
import { corePlaybackIntroLookupContentId } from '../../core/engine';

export function usePlayerIntroDb(metaId: string | undefined, enabled: boolean) {
  const [imdbId, setImdbId] = useState<string | null>(null);
  useEffect(() => {
    if (!enabled || !metaId) {
      setImdbId(null);
      return;
    }
    let cancelled = false;
    void corePlaybackIntroLookupContentId(metaId).then((id) => {
      if (!cancelled) setImdbId(id || null);
    }).catch(() => undefined);
    return () => { cancelled = true; };
  }, [enabled, metaId]);
  return imdbId;
}
