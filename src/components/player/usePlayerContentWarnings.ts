import { useCallback, useEffect, useRef, useState } from 'react';
import type { Meta, Video } from '../../core/types';
import { fetchContentWarnings, type ContentWarning } from '../../core/contentWarnings';

export function usePlayerContentWarnings(
  meta: Meta | undefined,
  episode: Video | null | undefined,
  playbackUrl: string | null | undefined,
  canShow: boolean,
) {
  const [warnings, setWarnings] = useState<ContentWarning[]>([]);
  const [isVisible, setIsVisible] = useState(false);
  const [fetchDone, setFetchDone] = useState(false);
  const [warningsDone, setWarningsDone] = useState(false);
  const hasShownRef = useRef(false);
  const fetchedForRef = useRef<string | null>(null);

  useEffect(() => {
    setWarnings([]);
    setIsVisible(false);
    setFetchDone(false);
    setWarningsDone(false);
    hasShownRef.current = false;
    fetchedForRef.current = null;
  }, [playbackUrl]);

  useEffect(() => {
    if (!playbackUrl || fetchedForRef.current === playbackUrl) return;
    fetchedForRef.current = playbackUrl;
    fetchContentWarnings(meta, episode).then((result) => {
      if (fetchedForRef.current !== playbackUrl) return;
      setWarnings(result);
      setFetchDone(true);
      if (result.length === 0) setWarningsDone(true);
    });
  }, [playbackUrl, meta, episode]);

  useEffect(() => {
    if (canShow && fetchDone && !hasShownRef.current && warnings.length > 0) {
      hasShownRef.current = true;
      setIsVisible(true);
    }
  }, [canShow, fetchDone, warnings]);

  const onAnimationComplete = useCallback(() => {
    setIsVisible(false);
    setWarningsDone(true);
  }, []);

  return { warnings, isVisible, onAnimationComplete, warningsDone };
}
