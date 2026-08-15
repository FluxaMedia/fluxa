import React from 'react';
import { WelcomeScreen, ProfileSelectionScreen } from './appScreens';
import { setActiveProfileId, createProfileObject, saveProfile, saveProfiles, loadProfiles } from './core/profiles';
import { invalidateLibraryKeyCache } from './core/libraryOps';
import { clearEnginePlugins, hydratePluginsFromNuvio, hydratePluginsFromStorage } from './core/pluginsStorage';
import { storageWrite } from './core/engine';
import { buildLocalNuvioProfiles } from './core/nuvioSync';
import { nuvioListAvatars, nuvioPullProfiles } from './core/nuvioApi';
import { DEFAULT_STATE } from './appConstants';
import type { AppState, UserProfile } from './core/types';

export function AppWelcomeGate({
  dispatch,
  updateState,
  applyStoredPrefs,
  setAllProfiles,
  setActiveProfile,
  setWelcomeCompleted,
  invalidateProfileWork,
}: {
  dispatch: (actionJson: string) => Promise<void> | void;
  updateState: (state: Partial<AppState>) => void;
  applyStoredPrefs: () => Promise<void>;
  setAllProfiles: (profiles: UserProfile[]) => void;
  setActiveProfile: (profile: UserProfile) => void;
  setWelcomeCompleted: (done: boolean) => void;
  invalidateProfileWork: () => void;
}) {
  return (
    <React.Suspense fallback={null}>
      <WelcomeScreen
        onProfileCreated={async (profile) => {
          invalidateProfileWork();
          await storageWrite('welcome_done', true);
          const profiles = await loadProfiles();
          invalidateLibraryKeyCache();
          setAllProfiles(profiles);
          setActiveProfile(profile);
          void dispatch(JSON.stringify({ type: 'addonsRefreshRequested', forceRefresh: false }));
          setWelcomeCompleted(true);
        }}
        onContinueLocal={async () => {
          invalidateProfileWork();
          await storageWrite('welcome_done', true);
          const profile = await createProfileObject('Local', '#FFFFFF');
          const profiles = await saveProfile(profile);
          await setActiveProfileId(profile.id);
          invalidateLibraryKeyCache();
          setAllProfiles(profiles);
          setWelcomeCompleted(true);
          setActiveProfile(profile);
          void dispatch(JSON.stringify({ type: 'addonsRefreshRequested', forceRefresh: false }));
        }}
        onNuvioLogin={async (profile) => {
          invalidateProfileWork();
          await storageWrite('welcome_done', true);
          const savedProfiles = await saveProfile(profile);
          await setActiveProfileId(profile.id);
          let visibleProfiles = savedProfiles;
          let visibleActiveProfile = profile;
          if (profile.nuvioAccessToken) {
            const [remoteProfiles, avatarCatalog] = await Promise.all([
              nuvioPullProfiles(profile.nuvioAccessToken),
              nuvioListAvatars(),
            ]);
            const builtRemoteProfiles = await buildLocalNuvioProfiles(profile, remoteProfiles, avatarCatalog, []);
            const remoteVisibleProfiles = builtRemoteProfiles.map((candidate) =>
              candidate.nuvioProfileIndex === profile.nuvioProfileIndex
                ? { ...candidate, id: profile.id }
                : candidate,
            );
            visibleProfiles = [
              ...savedProfiles.filter((candidate) => !candidate.nuvioAccessToken),
              ...remoteVisibleProfiles,
            ];
            visibleActiveProfile = remoteVisibleProfiles.find((candidate) => candidate.nuvioProfileIndex === profile.nuvioProfileIndex) ?? profile;
          }
          await saveProfiles(visibleProfiles);
          setAllProfiles(visibleProfiles);
          setActiveProfile(visibleActiveProfile);
          await dispatch(JSON.stringify({ type: 'profileActivated', profile: visibleActiveProfile }));
          await applyStoredPrefs();
          await dispatch(JSON.stringify({ type: 'addonsRefreshRequested', forceRefresh: false }));
          await hydratePluginsFromNuvio(visibleActiveProfile);
          await hydratePluginsFromStorage(updateState);
          setWelcomeCompleted(true);
        }}
      />
    </React.Suspense>
  );
}

export function AppProfileGate({
  state,
  stateRef,
  setState,
  dispatch,
  applyStoredPrefs,
  updateState,
  setAllProfiles,
  setActiveProfile,
  setEditProfileOpen,
  setHomeResetKey,
  invalidateProfileWork,
}: {
  state: AppState;
  stateRef: React.MutableRefObject<AppState>;
  setState: (state: AppState) => void;
  dispatch: (actionJson: string) => Promise<void> | void;
  applyStoredPrefs: () => Promise<void>;
  updateState: (s: Partial<AppState>) => void;
  setAllProfiles: (profiles: UserProfile[]) => void;
  setActiveProfile: (profile: UserProfile) => void;
  setEditProfileOpen: (open: boolean) => void;
  setHomeResetKey: (updater: (k: number) => number) => void;
  invalidateProfileWork: () => void;
}) {
  return (
    <React.Suspense fallback={null}>
      <ProfileSelectionScreen
        onProfileSelected={async (profile) => {
          invalidateProfileWork();
          const outgoingRepositories = state.plugins?.repositories ?? [];
          invalidateLibraryKeyCache();
          stateRef.current = DEFAULT_STATE;
          setState(DEFAULT_STATE);
          setActiveProfile(profile);
          setEditProfileOpen(false);
          setHomeResetKey((k) => k + 1);
          await dispatch(JSON.stringify({ type: 'profileActivated', profile }));
          await applyStoredPrefs();
          void dispatch(JSON.stringify({ type: 'addonsRefreshRequested', forceRefresh: false }));
          void dispatch(JSON.stringify({ type: 'homeLoadRequested' }));
          await clearEnginePlugins(outgoingRepositories, updateState);
          await hydratePluginsFromNuvio(profile);
          await hydratePluginsFromStorage(updateState);
        }}
        onProfilesChanged={setAllProfiles}
      />
    </React.Suspense>
  );
}
