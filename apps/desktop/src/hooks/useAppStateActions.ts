import { startTransition, useCallback, type MutableRefObject } from 'react';
import { platformInvoke as invoke } from '../platform/invoke';
import { loadPrefs } from '../core/libraryOps';
import { dispatchAction } from '../core/engine';
import { pumpEffects } from '../core/effectRunner';
import { setLanguage } from '../i18n';
import { prefBool, prefString } from '../core/appPrefs';
import { setRpdbApiKey } from '../core/rpdb';
import { AppStateStore } from '../core/appStateStore';
import { mergeAppState } from '../core/mergeState';
import { AsyncScope } from '../core/asyncScope';
import { clearSearchResultsCache } from '../core/searchResultsCache';
import type { AppState } from '../core/types';

export function useAppStateActions({
  store,
  stateRef,
  storedPrefsRef,
  profileScopeRef,
  profileAbortControllerRef,
  isWebTarget,
}: {
  store: AppStateStore;
  stateRef: MutableRefObject<AppState>;
  storedPrefsRef: MutableRefObject<Record<string, unknown>>;
  profileScopeRef: MutableRefObject<AsyncScope>;
  profileAbortControllerRef: MutableRefObject<AbortController>;
  isWebTarget: boolean;
}) {
  const overlayPrefs = useCallback((merged: AppState): AppState => {
    const prefs = storedPrefsRef.current;
    if (Object.keys(prefs).length === 0 || merged.settings.values === prefs) return merged;
    return { ...merged, settings: { ...merged.settings, values: prefs } };
  }, [storedPrefsRef]);

  const updateState = useCallback((patch: Partial<AppState>) => {
    const next = overlayPrefs(mergeAppState(stateRef.current, patch));
    stateRef.current = next;
    store.replace(next);
  }, [overlayPrefs, stateRef, store]);

  const updateStateDeferred = useCallback((patch: Partial<AppState>) => {
    const next = overlayPrefs(mergeAppState(stateRef.current, patch));
    stateRef.current = next;
    startTransition(() => store.replace(next));
  }, [overlayPrefs, stateRef, store]);

  const replaceState = useCallback((next: AppState) => {
    stateRef.current = next;
    store.replace(next);
  }, [stateRef, store]);

  const invalidateProfileWork = useCallback(() => {
    profileScopeRef.current.invalidate();
    profileAbortControllerRef.current.abort();
    profileAbortControllerRef.current = new AbortController();
    clearSearchResultsCache();
  }, [profileAbortControllerRef, profileScopeRef]);

  const dispatch = useCallback(async (actionJson: string) => {
    const profileRevision = profileScopeRef.current.capture();
    const profileSignal = profileAbortControllerRef.current.signal;
    const result = await dispatchAction(actionJson);
    if (!result || !profileScopeRef.current.isCurrent(profileRevision)) return;
    try {
      const action = JSON.parse(actionJson) as { type?: string };
      if (action.type === 'settingsChanged') {
        storedPrefsRef.current = await loadPrefs();
      }
    } catch {}
    updateState(result.state);
    if (result.effects.length > 0) {
      await pumpEffects(
        result.effects,
        (patch) => {
          if (profileScopeRef.current.isCurrent(profileRevision)) updateStateDeferred(patch);
        },
        profileSignal,
      ).catch(() => undefined);
    }
  }, [profileAbortControllerRef, profileScopeRef, storedPrefsRef, updateState, updateStateDeferred]);

  const applyStoredPrefs = useCallback(async () => {
    const freshPrefs = await loadPrefs();
    storedPrefsRef.current = freshPrefs;
    setLanguage(typeof freshPrefs.language === 'string' ? freshPrefs.language : null);
    setRpdbApiKey(prefString(freshPrefs, 'rpdbApiKey', ''));
    if (!isWebTarget) {
      void invoke('discord_presence_configure', { enabled: prefBool(freshPrefs, 'discordRichPresenceEnabled', true) });
      void invoke('set_diagnostic_mode', { enabled: prefBool(freshPrefs, 'diagnosticMode', false) });
    }
    updateState({ settings: { values: freshPrefs } });
  }, [isWebTarget, storedPrefsRef, updateState]);

  return {
    updateState,
    updateStateDeferred,
    replaceState,
    invalidateProfileWork,
    dispatch,
    applyStoredPrefs,
  };
}
