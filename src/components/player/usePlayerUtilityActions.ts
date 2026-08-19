import { useCallback, useEffect, useRef, useState, type MutableRefObject } from 'react';
import { platformInvoke as invoke } from '../../platform/invoke';
import { t } from '../../i18n';
import { fmtTime, sendCmd, type FeedbackFlash } from './PlayerOverlayPrimitives';

export function usePlayerUtilityActions({
  title,
  posRef,
  resetActivity,
  flashFeedback,
}: {
  title: string;
  posRef: MutableRefObject<number>;
  resetActivity: () => void;
  flashFeedback: (icon: FeedbackFlash['icon'], label: string) => void;
}) {
  const [abLoopStage, setAbLoopStage] = useState<'none' | 'a' | 'ab'>('none');
  const cycleAbLoopRef = useRef<() => void>(() => {});
  const takeScreenshotRef = useRef<() => Promise<void>>(async () => {});

  const cycleAbLoop = useCallback(() => {
    resetActivity();
    if (abLoopStage === 'none') {
      sendCmd(`set ab-loop-a ${posRef.current.toFixed(3)}`);
      setAbLoopStage('a');
      flashFeedback('abLoop', t('player.ab_loop_a_set'));
    } else if (abLoopStage === 'a') {
      sendCmd(`set ab-loop-b ${posRef.current.toFixed(3)}`);
      setAbLoopStage('ab');
      flashFeedback('abLoop', t('player.ab_loop_active'));
    } else {
      sendCmd('set ab-loop-a no');
      sendCmd('set ab-loop-b no');
      setAbLoopStage('none');
      flashFeedback('abLoop', t('player.ab_loop_cleared'));
    }
  }, [abLoopStage, flashFeedback, posRef, resetActivity]);

  const takeScreenshot = useCallback(async () => {
    resetActivity();
    try {
      await invoke<string>('player_screenshot', { suggestedName: title || 'fluxa' });
      flashFeedback('screenshot', t('player.screenshot_saved'));
    } catch {
      flashFeedback('screenshot', t('player.screenshot_failed'));
    }
  }, [flashFeedback, resetActivity, title]);

  const copyTimestamp = useCallback(async () => {
    const timestamp = fmtTime(posRef.current);
    try {
      await navigator.clipboard.writeText(timestamp);
    } catch {}
    flashFeedback('subDelay', timestamp);
  }, [flashFeedback, posRef]);

  useEffect(() => {
    cycleAbLoopRef.current = cycleAbLoop;
  }, [cycleAbLoop]);
  useEffect(() => {
    takeScreenshotRef.current = takeScreenshot;
  }, [takeScreenshot]);

  return { abLoopStage, setAbLoopStage, cycleAbLoop, cycleAbLoopRef, takeScreenshot, takeScreenshotRef, copyTimestamp };
}
