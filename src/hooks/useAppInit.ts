import React, { useCallback, useEffect, useState } from 'react';
import { platformInvoke as invoke } from '../platform/invoke';
import { dispatchAction, getSnapshot, initEngine, storageRead } from '../core/engine';
import { getActiveProfileId, loadProfiles } from '../core/profiles';
import { pumpEffects } from '../core/effectRunner';
import { hydratePluginsFromNuvio, hydratePluginsFromStorage } from '../core/pluginsStorage';
import { refreshNuvioProfiles } from '../core/nuvioSync';
import { loadPrefs } from '../core/libraryOps';
import { refreshAllAvatarPacks } from '../core/profileAvatarPacks';
import { getLanguage, setLanguage } from '../i18n';
import { prefBool, prefString } from '../core/appPrefs';
import { setRpdbApiKey } from '../core/rpdb';
import { restoreWindowGeometry } from '../core/windowGeometry';
import { startUpdateCheck, type UpdateState } from '../components/UpdateModal';
import type { AppState, UserProfile } from '../core/types';
import type { NavRoute } from '../components/NavSidebar';
import { isBrowserTarget } from '../platform/browser';

interface AppInitResult {
  ready: boolean;
  profilesChecked: boolean;
  welcomeCompleted: boolean;
  externalSyncPending: boolean;
  activeProfile: UserProfile | null;
  allProfiles: UserProfile[];
  updateModalState: UpdateState;
  setActiveProfile: (p: UserProfile | null) => void;
  setAllProfiles: (profiles: UserProfile[]) => void;
  setUpdateModalState: (s: UpdateState) => void;
  setWelcomeCompleted: (v: boolean) => void;
}

export function useAppInit(
  updateState: (s: Partial<AppState>) => void,
  setActiveRoute: (r: NavRoute) => void,
  storedPrefsRef: React.MutableRefObject<Record<string, unknown>>,
): AppInitResult {
  const [ready, setReady] = useState(false);
  const [profilesChecked, setProfilesChecked] = useState(false);
  const [welcomeCompleted, setWelcomeCompleted] = useState(true);
  const [externalSyncPending, setExternalSyncPending] = useState(true);
  const [activeProfile, setActiveProfile] = useState<UserProfile | null>(null);
  const [allProfiles, setAllProfiles] = useState<UserProfile[]>([]);
  const [updateModalState, setUpdateModalState] = useState<UpdateState>({ phase: 'idle' });

  const loadHomeOnStartup = useCallback(async () => {
    try {
      const homeResult = await dispatchAction(JSON.stringify({ type: 'homeLoadRequested', force: true, language: getLanguage() }));
      if (homeResult) {
        updateState(homeResult.state);
        if (homeResult.effects.length > 0) await pumpEffects(homeResult.effects, updateState);
      }
    } catch {
    } finally {
      setExternalSyncPending(false);
    }
  }, [updateState]);

  useEffect(() => {
    (async () => {
      try {
        void restoreWindowGeometry();
        await initEngine('{}');
        void refreshAllAvatarPacks();
        const snap = await getSnapshot();
        const prefs = await loadPrefs();
        storedPrefsRef.current = prefs;
        const webTarget = isBrowserTarget();
        if (!webTarget) {
          void invoke('player_set_seek_thumbnail_enabled', { enabled: prefBool(prefs, 'seekThumbnailEnabled', false) });
          void invoke('discord_presence_configure', { enabled: prefBool(prefs, 'discordRichPresenceEnabled', true) });
          void invoke('set_diagnostic_mode', { enabled: prefBool(prefs, 'diagnosticMode', false) });
        }
        setRpdbApiKey(prefString(prefs, 'rpdbApiKey', ''));
        setLanguage(typeof prefs.language === 'string' ? prefs.language : null);
        const startPage = prefString({ ...prefs }, 'startPage', 'home') as NavRoute;
        if (['home', 'search', 'library', 'discover', 'calendar', 'settings'].includes(startPage)) {
          setActiveRoute(startPage);
        }
        if (snap) {
          const s = snap as AppState;
          updateState({ ...s, settings: { ...s.settings, values: prefs } });
        }
        const welcomeDone = await storageRead<boolean>('welcome_done');
        if (!welcomeDone) setWelcomeCompleted(false);
        void (async () => {
          const libResult = await dispatchAction(JSON.stringify({ type: 'libraryHydrateRequested' }));
          if (!libResult) return;
          updateState({ ...libResult.state, settings: { ...libResult.state.settings, values: prefs } });
          if (libResult.effects.length > 0) await pumpEffects(libResult.effects, updateState);
        })().catch(() => undefined);
      } catch (err) {
        console.error('app boot sequence failed', err);
      } finally {
        setReady(true);
      }

      try {
        const [profileId, profiles] = await Promise.all([getActiveProfileId(), loadProfiles()]);
        let resolvedProfiles = profiles;
        if (profileId) {
          let found = profiles.find((p) => p.id === profileId) ?? null;
          if (found?.nuvioAccessToken) {
            const refreshed = await refreshNuvioProfiles(found).catch(() => found);
            resolvedProfiles = await loadProfiles();
            found = resolvedProfiles.find((p) => p.id === profileId) ?? refreshed;
          }
          setAllProfiles(resolvedProfiles);
          setActiveProfile(found);
          if (found) {
            await hydratePluginsFromNuvio(found).catch(() => undefined);
            await hydratePluginsFromStorage(updateState);
          }
        }
      } catch {
      } finally {
        setProfilesChecked(true);
      }

      void loadHomeOnStartup();

      if (prefBool(storedPrefsRef.current, 'automaticUpdates', true)) {
        void invoke<boolean>('in_app_updates_supported')
          .then((supported) => {
            if (!supported) return;
            setTimeout(() => {
              void startUpdateCheck((s) => {
                if (s.phase === 'available' || s.phase === 'error') setUpdateModalState(s);
              });
            }, 5000);
          })
          .catch(() => undefined);
      }
    })();
  }, []);

  return {
    ready,
    profilesChecked,
    welcomeCompleted,
    externalSyncPending,
    activeProfile,
    allProfiles,
    updateModalState,
    setActiveProfile,
    setAllProfiles,
    setUpdateModalState,
    setWelcomeCompleted,
  };
}
