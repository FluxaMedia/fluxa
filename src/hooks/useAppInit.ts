import React, { useCallback, useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { dispatchAction, getSnapshot, initEngine, storageRead } from '../core/engine';
import { getActiveProfileId, loadProfiles } from '../core/profiles';
import { pumpEffects, syncExternalIntegrationNow } from '../core/effectRunner';
import { importNuvioProfileData, recordNuvioSyncMeta } from '../core/nuvioSync';
import { setLanguage } from '../i18n';
import { prefBool, prefString } from '../core/appPrefs';
import { setRpdbApiKey } from '../core/rpdb';
import { restoreWindowGeometry } from '../core/windowGeometry';
import { startUpdateCheck, type UpdateState } from '../components/UpdateModal';
import type { AppState, UserProfile } from '../core/types';
import type { NavRoute } from '../components/NavSidebar';

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

  const syncExternalOnStartup = useCallback(async (profile: UserProfile) => {
    try {
      const syncTasks: Promise<unknown>[] = [];
      if (profile.nuvioAccessToken) {
        syncTasks.push(
          importNuvioProfileData(profile)
            .then((report) => recordNuvioSyncMeta(report))
            .catch((err) => recordNuvioSyncMeta({ errors: { library: err instanceof Error ? err.message : String(err) } }))
        );
      }
      if (profile.stremioAuthKey) {
        syncTasks.push(syncExternalIntegrationNow({
          provider: 'stremio',
          profile,
          token: profile.stremioAuthKey,
        }).catch(() => undefined));
      }
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
        const addonsResult = await dispatchAction(JSON.stringify({ type: 'addonsRefreshRequested', forceRefresh: false }));
        if (addonsResult) {
          updateState(addonsResult.state);
          if (addonsResult.effects.length > 0) await pumpEffects(addonsResult.effects, updateState);
        }
        const libResult = await dispatchAction(JSON.stringify({ type: 'libraryHydrateRequested' }));
        if (libResult) updateState(libResult.state);
        const homeResult = await dispatchAction(JSON.stringify({ type: 'homeLoadRequested', force: true }));
        if (homeResult) {
          updateState(homeResult.state);
          if (homeResult.effects.length > 0) await pumpEffects(homeResult.effects, updateState);
        }
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
        const pluginRepositories = await storageRead<string[]>('plugin_repository_urls') ?? [];
        const pluginOverrides = await storageRead<Record<string, boolean>>('plugin_scraper_enabled') ?? {};
        for (const manifestUrl of pluginRepositories) {
          if (!manifestUrl.trim()) continue;
          const result = await dispatchAction(JSON.stringify({ type: 'pluginRepositoryAddRequested', manifestUrl }));
          if (!result) continue;
          updateState(result.state);
          if (result.effects.length > 0) await pumpEffects(result.effects, updateState);
        }
        for (const [scraperId, enabled] of Object.entries(pluginOverrides)) {
          const result = await dispatchAction(JSON.stringify({ type: 'pluginScraperToggled', scraperId, enabled }));
          if (result) updateState(result.state);
        }
        const snap = await getSnapshot();
        const prefs = (await storageRead<Record<string, unknown>>('prefs')) ?? {};
        storedPrefsRef.current = prefs;
        void invoke('player_set_seek_thumbnail_enabled', { enabled: prefBool(prefs, 'seekThumbnailEnabled', false) });
        void invoke('discord_presence_configure', { enabled: prefBool(prefs, 'discordRichPresenceEnabled', true) });
        void invoke('set_diagnostic_mode', { enabled: prefBool(prefs, 'diagnosticMode', false) });
        setRpdbApiKey(prefString(prefs, 'rpdbApiKey', ''));
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
        const welcomeDone = await storageRead<boolean>('welcome_done');
        if (!welcomeDone) setWelcomeCompleted(false);
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
      } else {
        setExternalSyncPending(false);
      }

      if (prefBool(storedPrefsRef.current, 'automaticUpdates', true)) {
        setTimeout(() => {
          void startUpdateCheck((s) => {
            if (s.phase === 'available' || s.phase === 'error') setUpdateModalState(s);
          });
        }, 5000);
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
