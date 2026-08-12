import React, { useCallback, useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { coreInvoke, dispatchAction, getSnapshot, initEngine, storageRead } from '../core/engine';
import { getActiveProfileId, loadProfiles } from '../core/profiles';
import { pumpEffects, syncExternalIntegrationNow } from '../core/effectRunner';
import { importNuvioProfileData, recordNuvioSyncMeta } from '../core/nuvioSync';
import { hydratePluginsFromStorage } from '../core/pluginsStorage';
import { loadPrefs } from '../core/libraryOps';
import { refreshAllAvatarPacks } from '../core/profileAvatarPacks';
import { getLanguage, setLanguage } from '../i18n';
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
      const prefs = await loadPrefs();
      const sourcePlan = await coreInvoke<{ provider?: 'trakt' | 'simkl' | 'nuvio' | 'anilist' | 'stremio' }>(
        'continueWatchingSourcePlan',
        JSON.stringify({ source: prefs.continueWatchingSource }),
      );
      const selectedProvider = sourcePlan?.provider;
      const reloadLibraryAndHome = async () => {
        const libResult = await dispatchAction(JSON.stringify({ type: 'libraryHydrateRequested' }));
        if (libResult) updateState(libResult.state);
        const homeResult = await dispatchAction(JSON.stringify({ type: 'homeLoadRequested', force: true, language: getLanguage() }));
        if (homeResult) {
          updateState(homeResult.state);
          if (homeResult.effects.length > 0) await pumpEffects(homeResult.effects, updateState);
        }
      };
      const syncSelectedProvider = async () => {
        if (selectedProvider === 'trakt' && profile.traktAccessToken) {
          const clientId = await invoke<string>('get_oauth_client_id', { service: 'trakt' }).catch(() => '');
          await syncExternalIntegrationNow({ provider: 'trakt', profile, token: profile.traktAccessToken, clientId });
          return true;
        }
        if (selectedProvider === 'simkl' && profile.simklAccessToken) {
          const clientId = await invoke<string>('get_oauth_client_id', { service: 'simkl' }).catch(() => '');
          await syncExternalIntegrationNow({ provider: 'simkl', profile, token: profile.simklAccessToken, clientId });
          return true;
        }
        return false;
      };
      if (await syncSelectedProvider().catch(() => false)) {
        await reloadLibraryAndHome();
        setExternalSyncPending(false);
        void (async () => {
          const tasks: Promise<unknown>[] = [];
          if (profile.nuvioAccessToken) tasks.push(importNuvioProfileData(profile).then(recordNuvioSyncMeta).catch((err) => recordNuvioSyncMeta({ errors: { library: err instanceof Error ? err.message : String(err) } })));
          if (profile.anilistAccessToken) tasks.push(syncExternalIntegrationNow({ provider: 'anilist', profile, token: profile.anilistAccessToken }).catch(() => undefined));
          if (profile.stremioAuthKey) tasks.push(syncExternalIntegrationNow({ provider: 'stremio', profile, token: profile.stremioAuthKey }).catch(() => undefined));
          await Promise.allSettled(tasks);
        })();
        return;
      }
      if (profile.nuvioAccessToken && prefs.continueWatchingSource === 'nuvio') {
        const report = await importNuvioProfileData(profile, undefined, undefined, ['continueWatching'])
          .catch((err) => ({ errors: { library: err instanceof Error ? err.message : String(err) }, changed: false }));
        await recordNuvioSyncMeta(report);
        if (report.changed) {
          await reloadLibraryAndHome();
        }
        setExternalSyncPending(false);
        void (async () => {
          const nuvioBackground = importNuvioProfileData(profile, undefined, undefined, ['addons', 'watchlist', 'watched', 'collections'])
            .then(recordNuvioSyncMeta)
            .then(async () => {
              const refreshed = await importNuvioProfileData(profile, undefined, undefined, ['continueWatching']);
              await recordNuvioSyncMeta(refreshed);
              await reloadLibraryAndHome();
            })
            .catch((err) => recordNuvioSyncMeta({ errors: { library: err instanceof Error ? err.message : String(err) } }));
          const tasks: Promise<unknown>[] = [nuvioBackground];
          if (profile.anilistAccessToken) tasks.push(syncExternalIntegrationNow({ provider: 'anilist', profile, token: profile.anilistAccessToken }).catch(() => undefined));
          if (profile.stremioAuthKey) tasks.push(syncExternalIntegrationNow({ provider: 'stremio', profile, token: profile.stremioAuthKey }).catch(() => undefined));
          if (profile.traktAccessToken) {
            const clientId = await invoke<string>('get_oauth_client_id', { service: 'trakt' }).catch(() => '');
            tasks.push(syncExternalIntegrationNow({ provider: 'trakt', profile, token: profile.traktAccessToken, clientId }).catch(() => undefined));
          }
          if (profile.simklAccessToken) {
            const clientId = await invoke<string>('get_oauth_client_id', { service: 'simkl' }).catch(() => '');
            tasks.push(syncExternalIntegrationNow({ provider: 'simkl', profile, token: profile.simklAccessToken, clientId }).catch(() => undefined));
          }
          await Promise.allSettled(tasks);
        })();
        return;
      }
      const syncTasks: Promise<unknown>[] = [];
      let changed = false;
      if (profile.nuvioAccessToken) {
        syncTasks.push(
          importNuvioProfileData(profile)
            .then((report) => { changed = changed || report.changed; return recordNuvioSyncMeta(report); })
            .catch((err) => recordNuvioSyncMeta({ errors: { library: err instanceof Error ? err.message : String(err) } }))
        );
      }
      if (profile.anilistAccessToken) {
        changed = true;
        syncTasks.push(syncExternalIntegrationNow({
          provider: 'anilist',
          profile,
          token: profile.anilistAccessToken,
        }).catch(() => undefined));
      }
      if (profile.stremioAuthKey) {
        changed = true;
        syncTasks.push(syncExternalIntegrationNow({
          provider: 'stremio',
          profile,
          token: profile.stremioAuthKey,
        }).catch(() => undefined));
      }
      if (profile.traktAccessToken) {
        changed = true;
        const traktClientId = await invoke<string>('get_oauth_client_id', { service: 'trakt' }).catch(() => '');
        syncTasks.push(syncExternalIntegrationNow({
          provider: 'trakt',
          profile,
          token: profile.traktAccessToken,
          clientId: traktClientId,
        }).catch(() => undefined));
      }
      if (profile.simklAccessToken) {
        changed = true;
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
        if (!changed) return;
        const addonsResult = await dispatchAction(JSON.stringify({ type: 'addonsRefreshRequested', forceRefresh: false }));
        if (addonsResult) {
          updateState(addonsResult.state);
          if (addonsResult.effects.length > 0) await pumpEffects(addonsResult.effects, updateState);
        }
        const libResult = await dispatchAction(JSON.stringify({ type: 'libraryHydrateRequested' }));
        if (libResult) updateState(libResult.state);
        const homeResult = await dispatchAction(JSON.stringify({ type: 'homeLoadRequested', force: true, language: getLanguage() }));
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
        void refreshAllAvatarPacks();
        const snap = await getSnapshot();
        const prefs = await loadPrefs();
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

      void hydratePluginsFromStorage(updateState).catch(() => undefined);

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
        void invoke<boolean>('in_app_updates_supported').then((supported) => {
          if (!supported) return;
          setTimeout(() => {
            void startUpdateCheck((s) => {
              if (s.phase === 'available' || s.phase === 'error') setUpdateModalState(s);
            });
          }, 5000);
        }).catch(() => undefined);
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
