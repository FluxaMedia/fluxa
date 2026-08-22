import { platformInvoke as invoke } from '../../platform/invoke';
import { useCallback, useRef, useState } from 'react';
import { t } from '../../i18n';
import { sendCmd, type FeedbackFlash } from './PlayerOverlayPrimitives';
import type { SubtitleCaptureCue } from './TrackPopover';
import { coreInvoke } from '../../core/engine';

type SubtitleStyleKey =
  | 'subtitleTextOpacity'
  | 'subtitleBackgroundColor'
  | 'subtitleBackgroundOpacity'
  | 'subtitleOutlineColor'
  | 'subtitleOutlineOpacity'
  | 'subtitleForceStyle'
  | 'subtitleCharacterEdge'
  | 'subtitleShadow';

type Options = {
  prefs?: Record<string, unknown>;
  persistPreference: (key: string, value: string | boolean) => void;
  flashFeedback: (icon: FeedbackFlash['icon'], label: string) => void;
};

function mpvColor(color: string, opacity: string) {
  const alpha = Math.round(Math.max(0, Math.min(1, Number(opacity) || 0)) * 255)
    .toString(16)
    .padStart(2, '0')
    .toUpperCase();
  return `#${alpha}${color.replace(/^#/, '')}`;
}

export function usePlayerSubtitleControls({ prefs, persistPreference, flashFeedback }: Options) {
  const [subtitleDelay, setSubtitleDelay] = useState(0);
  const [subtitlePosition, setSubtitlePosition] = useState(() => Number(prefs?.subtitlePosition ?? 100) || 100);
  const [autoSyncingSubtitles, setAutoSyncingSubtitles] = useState(false);
  const [subtitleCaptureCues, setSubtitleCaptureCues] = useState<SubtitleCaptureCue[]>([]);
  const [subtitleCaptureTime, setSubtitleCaptureTime] = useState<number | null>(null);
  const [subtitleFont, setSubtitleFont] = useState(() => String(prefs?.subtitleFont ?? 'default'));
  const subtitleSizeRef = useRef(Number(prefs?.subtitleSize ?? 100) || 100);
  const [subtitleSize, setSubtitleSize] = useState(() => subtitleSizeRef.current);
  const [subtitleColor, setSubtitleColor] = useState(() => String(prefs?.subtitleColor ?? '#FFFFFF'));
  const [subtitleTextOpacity, setSubtitleTextOpacity] = useState(() => String(prefs?.subtitleTextOpacity ?? '1.0'));
  const [subtitleBackgroundColor, setSubtitleBackgroundColor] = useState(() => String(prefs?.subtitleBackgroundColor ?? '#000000'));
  const [subtitleBackgroundOpacity, setSubtitleBackgroundOpacity] = useState(() => String(prefs?.subtitleBackgroundOpacity ?? '0.5'));
  const [subtitleOutlineColor, setSubtitleOutlineColor] = useState(() => String(prefs?.subtitleOutlineColor ?? '#000000'));
  const [subtitleOutlineOpacity, setSubtitleOutlineOpacity] = useState(() => String(prefs?.subtitleOutlineOpacity ?? '1.0'));
  const [subtitleForceStyle, setSubtitleForceStyle] = useState(() => prefs?.subtitleForceStyle === true);
  const [subtitleCharacterEdge, setSubtitleCharacterEdge] = useState(() => String(prefs?.subtitleCharacterEdge ?? 'uniform'));
  const [subtitleShadow, setSubtitleShadow] = useState(() => prefs?.subtitleShadow === true);

  const adjustSubtitleDelay = useCallback((delta: number) => {
    setSubtitleDelay((previous) => {
      const next = Math.round((previous + delta) * 10) / 10;
      sendCmd(`set sub-delay ${next.toFixed(3)}`);
      return next;
    });
  }, []);
  const resetSubtitleDelay = useCallback(() => {
    sendCmd('set sub-delay 0.000');
    setSubtitleDelay(0);
  }, []);
  const chooseSubtitlePosition = useCallback(
    (position: number) => {
      sendCmd(`set sub-pos ${position}`);
      setSubtitlePosition(position);
      persistPreference('subtitlePosition', String(position));
    },
    [persistPreference],
  );
  const autoSyncSubtitles = useCallback(async () => {
    if (autoSyncingSubtitles) return;
    setAutoSyncingSubtitles(true);
    sendCmd('set pause yes');
    try {
      const result = await invoke<{ capturedTime: number; cues: SubtitleCaptureCue[] }>('player_capture_subtitle_cues');
      setSubtitleCaptureTime(result.capturedTime);
      setSubtitleCaptureCues(result.cues);
    } catch {
      flashFeedback('subDelay', t('player.subtitle_auto_sync_failed'));
    } finally {
      setAutoSyncingSubtitles(false);
    }
  }, [autoSyncingSubtitles, flashFeedback]);
  const applySubtitleCapture = useCallback(
    async (cueStart: number) => {
      if (subtitleCaptureTime === null) return;
      const result = await coreInvoke<{ delaySeconds: number }>(
        'subtitleSyncApply',
        JSON.stringify({ capturedTime: subtitleCaptureTime, cueStart }),
      );
      if (!result) return;
      const delay = result.delaySeconds;
      sendCmd(`set sub-delay ${delay.toFixed(3)}`);
      setSubtitleDelay(delay);
      setSubtitleCaptureCues([]);
      flashFeedback('subDelay', `${delay > 0 ? '+' : ''}${delay.toFixed(1)}s`);
    },
    [flashFeedback, subtitleCaptureTime],
  );
  const chooseSubtitleFont = useCallback(
    (font: string) => {
      sendCmd(`set sub-font "${font === 'default' ? 'sans-serif' : font}"`);
      setSubtitleFont(font);
      persistPreference('subtitleFont', font);
    },
    [persistPreference],
  );
  const chooseSubtitleSize = useCallback(
    (size: number) => {
      sendCmd(`set sub-scale ${(size / 100).toFixed(2)}`);
      subtitleSizeRef.current = size;
      setSubtitleSize(size);
      persistPreference('subtitleSize', String(size));
    },
    [persistPreference],
  );
  const adjustSubtitleSize = useCallback(
    (delta: number) => {
      const size = Math.max(50, Math.min(200, subtitleSizeRef.current + delta));
      subtitleSizeRef.current = size;
      sendCmd(`set sub-scale ${(size / 100).toFixed(2)}`);
      setSubtitleSize(size);
      persistPreference('subtitleSize', String(size));
    },
    [persistPreference],
  );
  const chooseSubtitleColor = useCallback(
    (color: string) => {
      sendCmd(`set sub-color "${mpvColor(color, subtitleTextOpacity)}"`);
      setSubtitleColor(color);
      persistPreference('subtitleColor', color);
    },
    [persistPreference, subtitleTextOpacity],
  );
  const chooseSubtitleStyle = useCallback(
    (key: SubtitleStyleKey, value: string | boolean) => {
      if (key === 'subtitleTextOpacity') {
        const opacity = String(value);
        setSubtitleTextOpacity(opacity);
        sendCmd(`set sub-color "${mpvColor(subtitleColor, opacity)}"`);
      } else if (key === 'subtitleBackgroundColor') {
        const color = String(value);
        setSubtitleBackgroundColor(color);
        sendCmd(`set sub-back-color "${mpvColor(color, subtitleBackgroundOpacity)}"`);
      } else if (key === 'subtitleBackgroundOpacity') {
        const opacity = String(value);
        setSubtitleBackgroundOpacity(opacity);
        sendCmd(`set sub-back-color "${mpvColor(subtitleBackgroundColor, opacity)}"`);
      } else if (key === 'subtitleOutlineColor') {
        const color = String(value);
        setSubtitleOutlineColor(color);
        sendCmd(`set sub-border-color "${mpvColor(color, subtitleOutlineOpacity)}"`);
      } else if (key === 'subtitleOutlineOpacity') {
        const opacity = String(value);
        setSubtitleOutlineOpacity(opacity);
        sendCmd(`set sub-border-color "${mpvColor(subtitleOutlineColor, opacity)}"`);
      } else if (key === 'subtitleForceStyle') {
        const enabled = value === true;
        setSubtitleForceStyle(enabled);
        sendCmd(`set sub-ass-override ${enabled ? 'force' : 'no'}`);
      } else if (key === 'subtitleCharacterEdge') {
        const edge = String(value);
        setSubtitleCharacterEdge(edge);
        if (edge === 'none') {
          sendCmd('set sub-border-size 0');
          sendCmd('set sub-shadow-offset 0');
        } else if (edge === 'raised') {
          sendCmd('set sub-border-size 1.5');
          sendCmd('set sub-shadow-offset -1');
        } else if (edge === 'depressed') {
          sendCmd('set sub-border-size 1.5');
          sendCmd('set sub-shadow-offset 1');
        } else if (edge === 'drop-shadow') {
          sendCmd('set sub-border-size 0');
          sendCmd('set sub-shadow-offset 3');
          sendCmd('set sub-shadow-color "#80000000"');
        } else {
          sendCmd('set sub-border-size 3');
          sendCmd('set sub-shadow-offset 0');
        }
      } else {
        const enabled = value === true;
        setSubtitleShadow(enabled);
        sendCmd(`set sub-shadow-offset ${enabled ? '3' : '0'}`);
        if (enabled) sendCmd('set sub-shadow-color "#80000000"');
      }
      persistPreference(key, value);
    },
    [persistPreference, subtitleBackgroundColor, subtitleBackgroundOpacity, subtitleColor, subtitleOutlineColor, subtitleOutlineOpacity],
  );

  return {
    subtitleDelay,
    subtitlePosition,
    autoSyncingSubtitles,
    subtitleCaptureCues,
    subtitleFont,
    subtitleSize,
    subtitleColor,
    subtitleTextOpacity,
    subtitleBackgroundColor,
    subtitleBackgroundOpacity,
    subtitleOutlineColor,
    subtitleOutlineOpacity,
    subtitleForceStyle,
    subtitleCharacterEdge,
    subtitleShadow,
    adjustSubtitleDelay,
    resetSubtitleDelay,
    chooseSubtitlePosition,
    autoSyncSubtitles,
    applySubtitleCapture,
    chooseSubtitleFont,
    chooseSubtitleSize,
    adjustSubtitleSize,
    chooseSubtitleColor,
    chooseSubtitleStyle,
  };
}
