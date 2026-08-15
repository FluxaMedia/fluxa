import { useCallback, useEffect, useState } from 'react';
import { platformInvoke as invoke } from '../../platform/invoke';
import { platformListen as listen } from '../../platform/browser';
import { t } from '../../i18n';
import { ANIME4K_MODES, type FeedbackFlash } from './PlayerOverlayPrimitives';

export function usePlayerAnime4k({ prefs, persistPreference, flashFeedback }: { prefs?: Record<string, unknown>; persistPreference: (key: string, value: string) => void; flashFeedback: (icon: FeedbackFlash['icon'], label: string) => void }) {
  const [anime4kEnabled, setAnime4kEnabled] = useState(false);
  const [anime4kMode, setAnime4kMode] = useState(() => String(prefs?.animeUpscalingModePreset ?? 'a'));

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    void invoke<boolean>('player_get_anime4k_enabled').then(setAnime4kEnabled).catch(() => undefined);
    void listen<{ enabled?: boolean }>('player-anime4k-state', (event) => setAnime4kEnabled(event.payload.enabled === true)).then((stop) => { unlisten = stop; });
    return () => unlisten?.();
  }, []);

  const toggleAnime4k = useCallback((enabled: boolean) => {
    setAnime4kEnabled(enabled);
    void invoke('player_set_anime4k_enabled', { enabled, quality: String(prefs?.animeUpscalingQuality ?? 'anime4k_m'), mode: anime4kMode }).catch(() => setAnime4kEnabled(!enabled));
  }, [anime4kMode, prefs]);

  const cycleAnime4kMode = useCallback((direction: 1 | -1) => {
    if (!anime4kEnabled) return;
    const index = ANIME4K_MODES.indexOf(anime4kMode as typeof ANIME4K_MODES[number]);
    const next = ANIME4K_MODES[(index === -1 ? 0 : index + direction + ANIME4K_MODES.length) % ANIME4K_MODES.length];
    void invoke('player_set_anime4k_enabled', { enabled: true, quality: String(prefs?.animeUpscalingQuality ?? 'anime4k_m'), mode: next }).then(() => {
      setAnime4kMode(next);
      persistPreference('animeUpscalingModePreset', next);
      flashFeedback('anime4k', t(`player.anime4k_mode_${next}`));
    }).catch(() => undefined);
  }, [anime4kEnabled, anime4kMode, flashFeedback, persistPreference, prefs]);

  return { anime4kEnabled, toggleAnime4k, cycleAnime4kMode };
}
