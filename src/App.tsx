import React, { useCallback, useEffect, useRef, useState } from 'react';
import { NavSidebar, TopBar, type NavRoute } from './components/NavSidebar';
import { ProfileChip } from './components/ProfileChip';
import { CalendarRoute, DetailRoute, DiscoverRoute, GlobalSearchRoute, HomeRoute, LibraryRoute, SearchRoute, SettingsRoute } from './components/AppRouteHosts';
import { PlaybackHost } from './components/PlaybackHost';
import { AppShell } from './components/AppShell';
import { AppBootstrap } from './components/AppBootstrap';
import { isBrowserTarget, platformListen } from './platform/browser';
import { ensureWebosProxy, IS_WEBOS, webosExitApp, webosStageReady } from './platform/webos';
import { platformInvoke as invoke } from './platform/invoke';
import { setBrowsingDiscordPresence } from './core/discordPresence';
import { appStyles, BROWSING_LABELS, DEFAULT_STATE } from './appConstants';
import { useNativePlayerEvents } from './hooks/useNativePlayerEvents';
import { useGlobalShortcuts } from './hooks/useGlobalShortcuts';
import { useDetailNavigation } from './hooks/useDetailNavigation';
import { useAppLayoutPrefs } from './hooks/useAppLayoutPrefs';
import { ErrorBoundary } from './components/ErrorBoundary';
import { UpdateModal, startUpdateCheck } from './components/UpdateModal';
import { AppProfileGate, AppWelcomeGate } from './AppGateScreens';
import { NuvioStatusBanner } from './components/NuvioStatusBanner';
import { OfflineBanner } from './components/OfflineBanner';
import { P2PDialog } from './components/P2PDialog';
import { useNuvioConnectivity } from './hooks/useNuvioConnectivity';
import { useOnlineStatus } from './hooks/useOnlineStatus';
import { setActiveProfileId, loadProfiles } from './core/profiles';
import { invalidateLibraryKeyCache, loadPrefs } from './core/libraryOps';
import { clearSearchResultsCache } from './core/searchResultsCache';
import { storageWrite, storageRead } from './core/engine';
import { getLanguage, setLanguage } from './i18n';
import { dispatchAction } from './core/engine';
import { pumpEffects } from './core/effectRunner';
import { appPrefs, prefBool, prefString } from './core/appPrefs';
import { setRpdbApiKey } from './core/rpdb';
import { AppStateStore, appStateSliceEqual, useAppStateSelector } from './core/appStateStore';
import { mergeAppState } from './core/mergeState';
import { AsyncScope } from './core/asyncScope';
import { usePlayer } from './hooks/usePlayer';
import { useAppInit } from './hooks/useAppInit';
import type { AppState, Meta, Stream, Video, UserProfile } from './core/types';
import { WebPlayerOverlay } from './components/WebPlayerOverlay';

const settingsStateEqual = appStateSliceEqual('settings');
const profileStateEqual = appStateSliceEqual('plugins');

export default function App() {
  const storeRef = useRef<AppStateStore | null>(null);
  if (!storeRef.current) storeRef.current = new AppStateStore(DEFAULT_STATE);
  const store = storeRef.current;
  const [editProfileOpen, setEditProfileOpen] = useState(false);
  const [activeRoute, setActiveRoute] = useState<NavRoute>('home');
  const [homeScrolled, setHomeScrolled] = useState(false);
  const [globalSearchQuery, setGlobalSearchQuery] = useState('');
  const [pendingAddonUrl, setPendingAddonUrl] = useState<string | null>(null);
  const [p2pDialog, setP2PDialog] = useState<{ mode: 'first-time' | 'disabled'; pendingPlay: () => void } | null>(null);
  const storedPrefsRef = useRef<Record<string, unknown>>({});
  const stateRef = useRef<AppState>(store.getState());
  const profileScopeRef = useRef(new AsyncScope());
  const profileAbortControllerRef = useRef(new AbortController());
  const settingsState = useAppStateSelector(store, (appState) => appState, settingsStateEqual);
  const profileState = useAppStateSelector(store, (appState) => appState, profileStateEqual);
  const lastNonSettingsRouteRef = useRef<NavRoute>('home');
  const lastNonSearchRouteRef = useRef<NavRoute>('home');
  const episodePlaybackFailureRef = useRef<(meta: Meta, episode: Video, message: string) => Promise<void>>(async () => {});
  const handleEpisodePlaybackFailed = useCallback((meta: Meta, episode: Video, message: string) => episodePlaybackFailureRef.current(meta, episode, message), []);

  const detailNav = useDetailNavigation();
  const {
    detailMeta, setDetailMeta, detailInitialEpisode, setDetailInitialEpisode,
    detailAutoShowStreams, setDetailAutoShowStreams, detailResumeAt, setDetailResumeAt,
    detailResumePercent,
    detailPlaybackError, setDetailPlaybackError, discoverInitialGenre, setDiscoverInitialGenre,
    guardedPlayRef, handleNavigateDetail, handleResumeFromContinueWatching,
    handleStartOverContinueWatching, handlePlayManually, resetDetail,
  } = detailNav;

  const overlayPrefs = useCallback((merged: AppState): AppState => {
    const prefs = storedPrefsRef.current;
    if (Object.keys(prefs).length === 0 || merged.settings.values === prefs) return merged;
    return { ...merged, settings: { ...merged.settings, values: prefs } };
  }, []);

  const updateState = useCallback((s: Partial<AppState>) => {
    const overlaid = overlayPrefs(mergeAppState(stateRef.current, s));
    stateRef.current = overlaid;
    store.replace(overlaid);
  }, [overlayPrefs, store]);

  const updateStateDeferred = useCallback((s: Partial<AppState>) => {
    const overlaid = overlayPrefs(mergeAppState(stateRef.current, s));
    stateRef.current = overlaid;
    React.startTransition(() => store.replace(overlaid));
  }, [overlayPrefs, store]);

  const replaceState = useCallback((next: AppState) => {
    stateRef.current = next;
    store.replace(next);
  }, [store]);

  const invalidateProfileWork = useCallback(() => {
    profileScopeRef.current.invalidate();
    profileAbortControllerRef.current.abort();
    profileAbortControllerRef.current = new AbortController();
    clearSearchResultsCache();
  }, []);

  const {
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
  } = useAppInit(updateState, setActiveRoute, storedPrefsRef);

  const { playerLoadingOverlay, playerUrl, playerTorrentTelemetryContext, playerPlaybackError, playerSubtitleWarning, dismissSubtitleWarning, playerTitle, playerEpisodeTitle, playerEpisode, playerUsesTorrent, playerPosterUrl, playerLogoUrl, playerMetaId, playerSubtitleUrl, playerSubtitles, playerCodecs, playerResumeAt, playbackSnapshotRef, reportPlaybackEvent, playerStreamHeaders, playingStreamRef, playingMetaRef, handlePlay, closePlayer, notifyFirstFrame, flushProgressOnQuit, skipSegmentCoverage } = usePlayer({
    stateRef,
    activeProfile,
    updateState,
    onProfileUpdated: setActiveProfile,
    onEpisodePlaybackFailed: handleEpisodePlaybackFailed,
  });

  const openEpisodeSourcePicker = useCallback(async (meta: Meta, episode: Video, message: string) => {
    await closePlayer();
    setDetailInitialEpisode(episode);
    setDetailAutoShowStreams(true);
    setDetailResumeAt(undefined);
    setDetailPlaybackError(message);
    setDetailMeta(meta);
  }, [closePlayer, setDetailInitialEpisode, setDetailAutoShowStreams, setDetailResumeAt, setDetailPlaybackError, setDetailMeta]);

  useEffect(() => {
    episodePlaybackFailureRef.current = openEpisodeSourcePicker;
  }, [openEpisodeSourcePicker]);

  useEffect(() => {
    if (ready) webosStageReady();
  }, [ready]);

  useEffect(() => {
    void ensureWebosProxy();
  }, []);

  const nativeEvents = useNativePlayerEvents(flushProgressOnQuit);
  const isWebTarget = isBrowserTarget();
  const nativePlayerActive = isWebTarget ? Boolean(playerUrl) : nativeEvents.nativePlayerActive;
  const softwareVideoActive = isWebTarget ? false : nativeEvents.softwareVideoActive;

  const guardedPlay = useCallback(async (
    stream: Stream,
    meta: Meta,
    episode: Video | null | undefined,
    resumeAt?: number,
    totalDuration?: number,
    sourceCandidates?: Stream[],
    resumePercent?: number,
  ) => {
    setDetailPlaybackError(null);
    const isP2P = !!(stream.isTorrent || stream.infoHash);
    if (!isP2P) {
      await handlePlay(stream, meta, episode, resumeAt, totalDuration, sourceCandidates, undefined, resumePercent);
      return;
    }

    const prefs = appPrefs(stateRef.current);
    const p2pEnabled = prefBool(prefs, 'p2pEnabled', true);
    const proceed = () => void handlePlay(stream, meta, episode, resumeAt, totalDuration, sourceCandidates, undefined, resumePercent);

    if (!p2pEnabled) {
      setP2PDialog({ mode: 'disabled', pendingPlay: proceed });
      return;
    }

    const warned = await storageRead<boolean>('p2p_warned').catch(() => false);
    if (!warned) {
      setP2PDialog({ mode: 'first-time', pendingPlay: proceed });
      return;
    }

    proceed();
  }, [handlePlay, stateRef, setDetailPlaybackError]);

  useEffect(() => {
    guardedPlayRef.current = guardedPlay;
  }, [guardedPlay, guardedPlayRef]);

  const [homeResetKey, setHomeResetKey] = useState(0);

  const navigateRoute = useCallback((route: NavRoute) => {
    if (route !== 'settings') {
      lastNonSettingsRouteRef.current = route;
    } else if (activeRoute !== 'settings') {
      lastNonSettingsRouteRef.current = activeRoute;
    }
    if (route !== 'search') {
      lastNonSearchRouteRef.current = route;
    } else if (activeRoute !== 'search' && activeRoute !== 'settings') {
      lastNonSearchRouteRef.current = activeRoute;
    }
    if (route === 'home') {
      setHomeResetKey((k) => k + 1);
    }
    setActiveRoute(route);
    setDetailMeta(null);
  }, [activeRoute, setDetailMeta]);

  useEffect(() => {
    if (isWebTarget) return undefined;
    const unlisten = platformListen<{ url?: string }>('deep-link-opened', (e) => {
      const raw = e.payload.url ?? '';
      const match = raw.match(/^fluxa:\/\/addon\/(.+)$/i);
      if (!match) return;
      let addonUrl = decodeURIComponent(match[1]);
      if (addonUrl.startsWith('stremio://')) addonUrl = addonUrl.replace(/^stremio:\/\//, 'https://');
      if (!/^https?:\/\//i.test(addonUrl)) return;
      setPendingAddonUrl(addonUrl);
      navigateRoute('settings');
    });
    return () => { void unlisten.then((fn) => fn()); };
  }, [isWebTarget, navigateRoute]);

  const goBack = useCallback(() => {
    if (detailMeta) {
      void closePlayer();
      resetDetail();
      return;
    }
    if (activeRoute === 'settings') { navigateRoute(lastNonSettingsRouteRef.current); return; }
    if (activeRoute === 'search') { navigateRoute(lastNonSearchRouteRef.current); return; }
    if (IS_WEBOS) webosExitApp();
  }, [detailMeta, activeRoute, navigateRoute, closePlayer, resetDetail]);

  const { searchFocusSignal, setSearchFocusSignal } = useGlobalShortcuts({ nativePlayerActive, navigateRoute, goBack });

  const dispatch = useCallback(async (actionJson: string) => {
    const profileRevision = profileScopeRef.current.capture();
    const profileSignal = profileAbortControllerRef.current.signal;
    const result = await dispatchAction(actionJson);
    if (!result || !profileScopeRef.current.isCurrent(profileRevision)) return;
    try {
      const action = JSON.parse(actionJson) as { type?: string };
      if (action.type === 'settingsChanged') {
        const freshPrefs = await loadPrefs();
        storedPrefsRef.current = freshPrefs;
      }
    } catch {}
    updateState(result.state);
    if (result.effects.length > 0) {
      await pumpEffects(result.effects, (patch) => {
        if (profileScopeRef.current.isCurrent(profileRevision)) updateStateDeferred(patch);
      }, profileSignal).catch(() => undefined);
    }
  }, [updateState, updateStateDeferred]);

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
  }, [isWebTarget, updateState]);

  const switchToNoProfile = useCallback(async () => {
    invalidateProfileWork();
    setActiveProfile(null);
    await setActiveProfileId('');
    await applyStoredPrefs();
  }, [applyStoredPrefs, invalidateProfileWork, setActiveProfile]);

  const activeProfileId = activeProfile?.id;
  const handleNuvioSynced = useCallback(async (changed: boolean) => {
    if (!changed) return;
    invalidateLibraryKeyCache();
    const profiles = await loadProfiles();
    setAllProfiles(profiles);
    if (activeProfileId) {
      const freshActiveProfile = profiles.find((p) => p.id === activeProfileId);
      if (freshActiveProfile) {
        setActiveProfile(freshActiveProfile);
        await dispatch(JSON.stringify({ type: 'profileActivated', profile: freshActiveProfile }));
      }
    }
    await applyStoredPrefs();
    void dispatch(JSON.stringify({ type: 'addonsRefreshRequested', forceRefresh: false }));
    void dispatch(JSON.stringify({ type: 'homeLoadRequested', force: true, language: getLanguage() }));
  }, [activeProfileId, applyStoredPrefs, dispatch, setAllProfiles, setActiveProfile]);

  const { serverDown, justRecovered, dismissed, dismiss } = useNuvioConnectivity(activeProfile, handleNuvioSynced);
  const isOnline = useOnlineStatus();

  const leaveSearch = useCallback(() => {
    setGlobalSearchQuery('');
    navigateRoute(lastNonSearchRouteRef.current);
  }, [navigateRoute]);

  const handleLibraryBack = useCallback(() => { setActiveRoute('home'); }, []);
  const handleProfileUpdated = useCallback((updated: UserProfile) => { setActiveProfile(updated); }, [setActiveProfile]);
  const handleSearchQueryChange = useCallback((query: string) => { setGlobalSearchQuery(query); }, []);
  const handleDiscoverBack = useCallback(() => { setDiscoverInitialGenre(null); setActiveRoute('home'); }, [setDiscoverInitialGenre]);

  const prefs = React.useMemo(() => appPrefs(settingsState), [settingsState.settings?.values]);
  const { rootStyle, isTopBar, navBarPosition, navItemsAlign, sidebarAlwaysOpen, sidebarOffset, mirrorSearchToLeft } = useAppLayoutPrefs({
    state: settingsState, prefs, nativePlayerActive, updateState, storedPrefsRef,
  });

  React.useEffect(() => {
    if (detailMeta || nativePlayerActive) return;
    setBrowsingDiscordPresence(BROWSING_LABELS[activeRoute] ?? 'Browsing');
  }, [activeRoute, detailMeta, nativePlayerActive]);

  const welcomeGate = (
    <AppWelcomeGate
        dispatch={dispatch}
        updateState={updateState}
        applyStoredPrefs={applyStoredPrefs}
        setAllProfiles={setAllProfiles}
        setActiveProfile={setActiveProfile}
        setWelcomeCompleted={setWelcomeCompleted}
        invalidateProfileWork={invalidateProfileWork}
    />
  );

  const profileGate = (
    <AppProfileGate
        state={profileState}
        stateRef={stateRef}
        setState={replaceState}
        dispatch={dispatch}
        applyStoredPrefs={applyStoredPrefs}
        updateState={updateState}
        setAllProfiles={setAllProfiles}
        setActiveProfile={setActiveProfile}
        setEditProfileOpen={setEditProfileOpen}
        setHomeResetKey={setHomeResetKey}
        invalidateProfileWork={invalidateProfileWork}
    />
  );

  const showDetail = detailMeta !== null;
  const bannerOffset = (serverDown && !dismissed) || justRecovered ? 36 : 0;

  const navigation = !nativePlayerActive && (isTopBar ? (
        <TopBar
          activeRoute={activeRoute}
          onNavigate={navigateRoute}
          transparent={activeRoute === 'home' && !showDetail && !homeScrolled}
          position={navBarPosition}
          itemsAlign={navItemsAlign}
          topOffset={bannerOffset}
        />
      ) : (
        <NavSidebar
          activeRoute={activeRoute}
          onNavigate={navigateRoute}
          position={navBarPosition}
          itemsAlign={navItemsAlign}
          topOffset={bannerOffset}
          alwaysOpen={sidebarAlwaysOpen}
        />
      ));

  const globalControls = !nativePlayerActive ? (
        <div
          style={{
            position: 'fixed',
            top: 18 + bannerOffset,
            left: mirrorSearchToLeft ? 20 : undefined,
            right: mirrorSearchToLeft ? undefined : 20,
            zIndex: 46,
            display: 'flex',
            alignItems: 'center',
            gap: '0.5rem',
            pointerEvents: 'none',
          }}
        >
          <GlobalSearchRoute
            store={store}
            query={globalSearchQuery}
            onSearch={(query) => { setGlobalSearchQuery(query); navigateRoute('search'); }}
            onBack={leaveSearch}
            focusSignal={searchFocusSignal}
            onDispatch={dispatch}
            onNavigateDetail={handleNavigateDetail}
          />
          <div style={{ pointerEvents: 'auto', flexShrink: 0 }}>
            <ProfileChip
              profile={activeProfile!}
              allProfiles={allProfiles}
              onSwitchProfile={() => void switchToNoProfile()}
              onSwitchToProfile={async (p) => {
                invalidateProfileWork();
                await setActiveProfileId(p.id);
                invalidateLibraryKeyCache();
                stateRef.current = DEFAULT_STATE;
                replaceState(DEFAULT_STATE);
                setActiveProfile(p);
                setHomeResetKey((k) => k + 1);
                await dispatch(JSON.stringify({ type: 'profileActivated', profile: p }));
                await applyStoredPrefs();
                void dispatch(JSON.stringify({ type: 'addonsRefreshRequested', forceRefresh: false }));
                void dispatch(JSON.stringify({ type: 'homeLoadRequested' }));
              }}
              onOpenSettings={() => navigateRoute('settings')}
              onEditProfile={() => setEditProfileOpen(true)}
            />
          </div>
        </div>
      ) : null;

  const content = (
      <ErrorBoundary
        resetKeys={[activeRoute, detailMeta?.id]}
        onReset={() => { setDetailMeta(null); setActiveRoute('home'); }}
      >
        {showDetail && (
          <React.Suspense fallback={null}><DetailRoute
            store={store}
            key={detailMeta!.id}
            meta={detailMeta!}
            onDispatch={dispatch}
            onPlay={(stream, meta, episode, resumeAt, sourceCandidates) => void guardedPlay(stream, meta, episode, resumeAt !== undefined ? resumeAt : (detailAutoShowStreams ? detailResumeAt : undefined), undefined, sourceCandidates, resumeAt === undefined && detailAutoShowStreams ? detailResumePercent : undefined)}
            onNavigateDetail={handleNavigateDetail}
            onNavigateGenre={(genre) => { setDiscoverInitialGenre(genre); setDetailMeta(null); navigateRoute('discover'); }}
            onBack={() => { void closePlayer(); resetDetail(); }}
            initialEpisode={detailInitialEpisode}
            autoShowStreams={detailAutoShowStreams}
            playbackFailure={detailPlaybackError}
          /></React.Suspense>
        )}
        <div style={{ display: !showDetail && activeRoute === 'home' ? 'contents' : 'none' }}>
          <HomeRoute
            store={store}
            onDispatch={dispatch}
            onNavigateDetail={handleNavigateDetail}
            onPlay={setDetailMeta}
            onResume={handleResumeFromContinueWatching}
            onStartOver={handleStartOverContinueWatching}
            onPlayManually={handlePlayManually}
            isActive={!showDetail && activeRoute === 'home'}
            onScrolledChange={setHomeScrolled}
            resetKey={homeResetKey}
            deferStaleRefresh={externalSyncPending}
          />
        </div>
        {!showDetail && activeRoute === 'calendar' && (
          <React.Suspense fallback={null}>
            <CalendarRoute
              store={store}
              onDispatch={dispatch}
            />
          </React.Suspense>
        )}
        {!showDetail && activeRoute === 'discover' && (
          <React.Suspense fallback={null}>
            <DiscoverRoute
              store={store}
              onDispatch={dispatch}
              onNavigateDetail={handleNavigateDetail}
              onBack={handleDiscoverBack}
              initialGenre={discoverInitialGenre}
            />
          </React.Suspense>
        )}
        {!showDetail && activeRoute === 'library' && (
          <React.Suspense fallback={null}>
            <LibraryRoute
              store={store}
              onDispatch={dispatch}
              onNavigateDetail={handleNavigateDetail}
              onBack={handleLibraryBack}
              activeProfile={activeProfile!}
              onProfileUpdated={handleProfileUpdated}
              onPlayLocal={(stream, meta, episode) => { void handlePlay(stream, meta, episode); }}
            />
          </React.Suspense>
        )}
        {!showDetail && activeRoute === 'search' ? (
          <React.Suspense fallback={null}>
            <SearchRoute
              store={store}
              onDispatch={dispatch}
              onNavigateDetail={handleNavigateDetail}
              query={globalSearchQuery}
              onQueryChange={handleSearchQueryChange}
              onBack={leaveSearch}
            />
          </React.Suspense>
        ) : !showDetail && activeRoute === 'settings' ? (
          <React.Suspense fallback={null}>
            <SettingsRoute
              store={store}
              onDispatch={dispatch}
              activeProfile={activeProfile}
              onProfileUpdated={(updated) => setActiveProfile(updated)}
              onSwitchProfile={() => void switchToNoProfile()}
              onBack={() => navigateRoute(lastNonSettingsRouteRef.current)}
              onCheckForUpdates={() => void startUpdateCheck(setUpdateModalState)}
              initialAddonUrl={pendingAddonUrl}
            />
          </React.Suspense>
        ) : null}
      </ErrorBoundary>
  );

  const notices = isOnline ? (
        <NuvioStatusBanner
          serverDown={serverDown}
          justRecovered={justRecovered}
          dismissed={dismissed}
          onDismiss={dismiss}
        />
      ) : (
        <OfflineBanner online={isOnline} />
      );
  const dialogs = <>
      {p2pDialog && (
        <P2PDialog
          mode={p2pDialog.mode}
          onCancel={() => setP2PDialog(null)}
          onConfirm={() => {
            void storageWrite('p2p_warned', true);
            setP2PDialog(null);
            p2pDialog.pendingPlay();
          }}
          onEnableP2P={() => {
            void dispatch(JSON.stringify({ type: 'settingsChanged', key: 'p2pEnabled', value: true }));
          }}
        />
      )}
      <UpdateModal state={updateModalState} onClose={() => setUpdateModalState({ phase: 'idle' })} />
  </>;
  const playback = (
      isWebTarget && playerUrl ? <WebPlayerOverlay url={playerUrl} title={playerTitle} subtitles={playerSubtitles} codecs={playerCodecs} resumeAt={playerResumeAt} snapshotRef={playbackSnapshotRef} onPlaybackEvent={reportPlaybackEvent} contentId={playerMetaId} contentType={playerEpisode ? 'series' : 'movie'} videoId={playerEpisode?.id} onClose={closePlayer} onFirstFrame={notifyFirstFrame} /> : <PlaybackHost
        active={nativePlayerActive}
        loading={playerLoadingOverlay}
        closePlayer={closePlayer}
        notifyFirstFrame={notifyFirstFrame}
        title={playerTitle}
        episodeTitle={playerEpisodeTitle}
        episode={playerEpisode}
        usesTorrent={playerUsesTorrent}
        posterUrl={playerPosterUrl}
        logoUrl={playerLogoUrl}
        metaId={playerMetaId}
        subtitleUrl={playerSubtitleUrl}
        streamHeaders={playerStreamHeaders}
        streamRef={playingStreamRef}
        metaRef={playingMetaRef}
        playbackUrl={playerUrl}
        torrentTelemetryContext={playerTorrentTelemetryContext}
        prefs={prefs}
        playbackError={playerPlaybackError}
        subtitleWarning={playerSubtitleWarning}
        dismissSubtitleWarning={dismissSubtitleWarning}
        softwareVideoActive={softwareVideoActive}
        bannerOffset={bannerOffset}
        skipSegmentCoverage={skipSegmentCoverage}
        dispatch={dispatch}
      />
  );

  const shell = (
    <AppShell
      rootStyle={rootStyle}
      animations={prefBool(prefs, 'animationsEnabled', true) ? 'on' : 'off'}
      density={prefString(prefs, 'interfaceDensity', 'medium')}
      reduceMotion={prefBool(prefs, 'reduceMotion', false) ? 'true' : 'false'}
      reducedEffects={prefBool(prefs, 'reducedEffects', false) ? 'true' : 'false'}
      navigation={navigation}
      globalControls={globalControls}
      contentStyle={{ ...appStyles.content, top: (isTopBar && navBarPosition === 'top' && activeRoute !== 'home' && !showDetail ? 76 : 0) + bannerOffset, paddingLeft: sidebarAlwaysOpen && navBarPosition !== 'right' ? sidebarOffset : 0, paddingRight: sidebarAlwaysOpen && navBarPosition === 'right' ? sidebarOffset : 0, display: nativePlayerActive ? 'none' : undefined }}
      content={content}
      notices={notices}
      dialogs={dialogs}
      playback={playback}
    />
  );

  return (
    <AppBootstrap
      ready={ready}
      profilesChecked={profilesChecked}
      welcomeCompleted={welcomeCompleted}
      profileReady={!!activeProfile && !editProfileOpen}
      loading={<span style={appStyles.loadingText}>fluxa</span>}
      welcome={welcomeGate}
      profile={profileGate}
    >
      {shell}
    </AppBootstrap>
  );
}
