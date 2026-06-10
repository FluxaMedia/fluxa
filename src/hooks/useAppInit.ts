import React, { useCallback, useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { dispatchAction, getSnapshot, initEngine, storageRead } from '../core/engine';
import { getActiveProfileId, loadProfiles } from '../core/profiles';
import { pumpEffects } from '../core/effectRunner';
import { setLanguage } from '../i18n';
import { prefString } from '../core/appPrefs';
import { startUpdateCheck, type UpdateState } from '../components/UpdateModal';
import type { AppState, UserProfile } from '../core/types';
import type { NavRoute } from '../components/NavSidebar';

interface AppInitResult {
  ready: boolean;
  profilesChecked: boolean;
  activeProfile: UserProfile | null;
  allProfiles: UserProfile[];
  updateModalState: UpdateState;
  setActiveProfile: (p: UserProfile | null) => void;
  setUpdateModalState: (s: UpdateState) => void;
}

export function useAppInit(
  updateState: (s: AppState) => void,
  setActiveRoute: (r: NavRoute) => void,
  storedPrefsRef: React.MutableRefObject<Record<string, unknown>>,
): AppInitResult {
  const [ready, setReady] = useState(false);
  const [profilesChecked, setProfilesChecked] = useState(false);
  const [activeProfile, setActiveProfile] = useState<UserProfile | null>(null);
  const [allProfiles, setAllProfiles] = useState<UserProfile[]>([]);
  const [updateModalState, setUpdateModalState] = useState<UpdateState>({ phase: 'idle' });

  const syncExternalOnStartup = useCallback(async (profile: UserProfile) => {
    try {
      const { syncExternalIntegrationNow } = await import('../core/effectRunner');
      const syncTasks: Promise<unknown>[] = [];
      if (profile.traktAccessToken) {
        const traktClientId = await invoke<string>('get_oauth_client_id', { service: 'trakt' }).catch(() => '');
        syncTasks.push(syncExternalIntegrationNow({
          provider: 'trakt',
          profile,
          token: profile.traktAccessToken,
          clientId: traktClientId,
        }).catch(() => undefined));
      }
      if (profile.simklAccessToken) {
        const simklClientId = await invoke<string>('get_oauth_client_id', { service: 'simkl' }).catch(() => '');
        syncTasks.push(syncExternalIntegrationNow({
          provider: 'simkl',
          profile,
          token: profile.simklAccessToken,
          clientId: simklClientId,
        }).catch(() => undefined));
      }
      if (syncTasks.length > 0) {
        await Promise.all(syncTasks);
        const libResult = await dispatchAction(JSON.stringify({ type: 'libraryHydrateRequested' }));
        if (libResult) updateState(libResult.state);
      }
    } catch {}
  }, [updateState]);

  useEffect(() => {
    (async () => {
      try {
        await initEngine('{}');
        const snap = await getSnapshot();
        const prefs = (await storageRead<Record<string, unknown>>('prefs')) ?? {};
        storedPrefsRef.current = prefs;
        setLanguage(typeof prefs.language === 'string' ? prefs.language : null);
        const startPage = prefString({ ...prefs }, 'startPage', 'home') as NavRoute;
        if (['home', 'search', 'library', 'discover', 'calendar', 'settings'].includes(startPage)) {
          setActiveRoute(startPage);
        }
        if (snap) { const s = snap as AppState; updateState({ ...s, settings: { ...s.settings, values: prefs } }); }
        const libResult = await dispatchAction(JSON.stringify({ type: 'libraryHydrateRequested' }));
        if (libResult) {
          updateState({ ...libResult.state, settings: { ...libResult.state.settings, values: prefs } });
          if (libResult.effects.length > 0) await pumpEffects(libResult.effects, updateState);
        }
      } catch {
      } finally {
        setReady(true);
      }

      let startupProfile: UserProfile | null = null;
      try {
        const [profileId, profiles] = await Promise.all([getActiveProfileId(), loadProfiles()]);
        setAllProfiles(profiles);
        if (profileId) {
          const found = profiles.find((p) => p.id === profileId) ?? null;
          setActiveProfile(found);
          startupProfile = found;
        }
      } catch {
      } finally {
        setProfilesChecked(true);
      }

      if (startupProfile) {
        void syncExternalOnStartup(startupProfile);
      }

      setTimeout(() => {
        void startUpdateCheck((s) => {
          if (s.phase === 'available' || s.phase === 'error') setUpdateModalState(s);
        });
      }, 5000);
    })();
  }, []);

  return {
    ready,
    profilesChecked,
    activeProfile,
    allProfiles,
    updateModalState,
    setActiveProfile,
    setUpdateModalState,
  };
}
