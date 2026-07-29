import React, { useCallback, useEffect, useRef, useState } from 'react';
import { NavSidebar, TopBar, type NavRoute } from './components/NavSidebar';
import { ProfileChip } from './components/ProfileChip';
import { GlobalSearchBar } from './components/GlobalSearchBar';
import { PlayerLoadingOverlay } from './components/PlayerLoadingOverlay';
import { ReactPlayerOverlay } from './components/ReactPlayerOverlay';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';
import { setBrowsingDiscordPresence } from './core/discordPresence';
import { appStyles, accentForegroundColor, computeAutoUiScale, BROWSING_LABELS, DEFAULT_STATE } from './appConstants';
import { useNativePlayerEvents } from './hooks/useNativePlayerEvents';
import { useGlobalShortcuts } from './hooks/useGlobalShortcuts';
import * as Sentry from '@sentry/react';
import { ErrorBoundary } from './components/ErrorBoundary';
import { UpdateModal, startUpdateCheck } from './components/UpdateModal';
import { HomeScreen as HomeScreenBase } from './screens/HomeScreen';
import { SearchScreen as SearchScreenBase } from './screens/SearchScreen';
import { DetailScreen as DetailScreenBase } from './screens/DetailScreen';
import { LibraryScreen as LibraryScreenBase } from './screens/LibraryScreen';
import { SettingsScreen as SettingsScreenBase } from './screens/SettingsScreen';
const HomeScreen = Sentry.withProfiler(HomeScreenBase, { name: 'HomeScreen' });
const SearchScreen = Sentry.withProfiler(SearchScreenBase, { name: 'SearchScreen' });
const DetailScreen = Sentry.withProfiler(DetailScreenBase, { name: 'DetailScreen' });
const LibraryScreen = Sentry.withProfiler(LibraryScreenBase, { name: 'LibraryScreen' });
const SettingsScreen = Sentry.withProfiler(SettingsScreenBase, { name: 'SettingsScreen' });
const DiscoverScreen = React.lazy(() => import('./screens/DiscoverScreen').then((m) => ({ default: Sentry.withProfiler(m.DiscoverScreen, { name: 'DiscoverScreen' }) })));
const CalendarScreen = React.lazy(() => import('./screens/CalendarScreen').then((m) => ({ default: Sentry.withProfiler(m.CalendarScreen, { name: 'CalendarScreen' }) })));
const ProfileSelectionScreen = React.lazy(() => import('./screens/ProfileSelectionScreen').then((m) => ({ default: m.ProfileSelectionScreen })));
const WelcomeScreen = React.lazy(() => import('./screens/WelcomeScreen').then((m) => ({ default: m.WelcomeScreen })));
import { NuvioStatusBanner } from './components/NuvioStatusBanner';
import { OfflineBanner } from './components/OfflineBanner';
import { P2PDialog } from './components/P2PDialog';
import { useNuvioConnectivity } from './hooks/useNuvioConnectivity';
import { useOnlineStatus } from './hooks/useOnlineStatus';
import { setActiveProfileId, createProfileObject, saveProfile, loadProfiles } from './core/profiles';
import { invalidateLibraryKeyCache, loadPrefs, prefsOwnerId, savePrefs } from './core/libraryOps';
import { clearEnginePlugins, hydratePluginsFromStorage } from './core/pluginsStorage';
import { storageWrite, storageRead } from './core/engine';
import { getLanguage, setLanguage } from './i18n';
import { dispatchAction } from './core/engine';
import { prefetchPlayerArtwork } from './core/mpvPlayer';
import { pumpEffects } from './core/effectRunner';
import { appPrefs, prefBool, prefString } from './core/appPrefs';
import { setRpdbApiKey } from './core/rpdb';
import { playerArtwork } from './core/playerUtils';
import { readStoredPlaybackSource } from './core/libraryStorage';
import { mergeAppState } from './core/mergeState';
import { usePlayer } from './hooks/usePlayer';
import { useAppInit } from './hooks/useAppInit';
import type { AppState, LibraryItem, Meta, Stream, Video, UserProfile } from './core/types';

export default function App() {
  const [state, setState] = useState<AppState>(DEFAULT_STATE);
  const [editProfileOpen, setEditProfileOpen] = useState(false);
  const [activeRoute, setActiveRoute] = useState<NavRoute>('home');
  const [homeScrolled, setHomeScrolled] = useState(false);
  const [detailMeta, setDetailMeta] = useState<Meta | null>(null);
  const [detailInitialEpisode, setDetailInitialEpisode] = useState<Video | null>(null);
  const [detailAutoShowStreams, setDetailAutoShowStreams] = useState(false);
  const [detailResumeAt, setDetailResumeAt] = useState<number | undefined>(undefined);
  const [detailPlaybackError, setDetailPlaybackError] = useState<string | null>(null);
  const [discoverInitialGenre, setDiscoverInitialGenre] = useState<string | null>(null);
  const [globalSearchQuery, setGlobalSearchQuery] = useState('');
  const [pendingAddonUrl, setPendingAddonUrl] = useState<string | null>(null);
  const [p2pDialog, setP2PDialog] = useState<{ mode: 'first-time' | 'disabled'; pendingPlay: () => void } | null>(null);
  const storedPrefsRef = useRef<Record<string, unknown>>({});
  const stateRef = useRef<AppState>(DEFAULT_STATE);
  const lastNonSettingsRouteRef = useRef<NavRoute>('home');
  const lastNonSearchRouteRef = useRef<NavRoute>('home');
  const episodePlaybackFailureRef = useRef<(meta: Meta, episode: Video, message: string) => Promise<void>>(async () => {});
  const artworkPrefetchRef = useRef<Promise<unknown> | null>(null);
  const handleEpisodePlaybackFailed = useCallback((meta: Meta, episode: Video, message: string) => episodePlaybackFailureRef.current(meta, episode, message), []);

  const overlayPrefs = useCallback((merged: AppState): AppState => {
    const prefs = storedPrefsRef.current;
    // Only every screen's memo comparator checks state.settings by reference — skip
    // rebuilding it when prefs haven't actually changed, or every dispatch/effect
    // completion forces every screen to re-render regardless of relevance.
    if (Object.keys(prefs).length === 0 || merged.settings.values === prefs) return merged;
    return { ...merged, settings: { ...merged.settings, values: prefs } };
  }, []);

  const updateState = useCallback((s: Partial<AppState>) => {
    const overlaid = overlayPrefs(mergeAppState(stateRef.current, s));
    stateRef.current = overlaid;
    setState(overlaid);
  }, [overlayPrefs]);

  const updateStateDeferred = useCallback((s: Partial<AppState>) => {
    const overlaid = overlayPrefs(mergeAppState(stateRef.current, s));
    stateRef.current = overlaid;
    React.startTransition(() => setState(overlaid));
  }, [overlayPrefs]);

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

  const { playerLoadingOverlay, playerUrl, playerPlaybackError, playerSubtitleWarning, dismissSubtitleWarning, playerTitle, playerEpisodeTitle, playerEpisode, playerUsesTorrent, playerPosterUrl, playerLogoUrl, playerMetaId, playerSubtitleUrl, playerStreamHeaders, playingStreamRef, playingMetaRef, handlePlay, closePlayer, notifyFirstFrame, flushProgressOnQuit } = usePlayer({
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
  }, [closePlayer]);

  useEffect(() => {
    episodePlaybackFailureRef.current = openEpisodeSourcePicker;
  }, [openEpisodeSourcePicker]);

  const { nativePlayerActive, softwareVideoActive } = useNativePlayerEvents(flushProgressOnQuit);

  const guardedPlay = useCallback(async (
    stream: Stream,
    meta: Meta,
    episode: Video | null | undefined,
    resumeAt?: number,
    totalDuration?: number,
    sourceCandidates?: Stream[],
  ) => {
    setDetailPlaybackError(null);
    const isP2P = !!(stream.isTorrent || stream.infoHash);
    if (!isP2P) {
      await handlePlay(stream, meta, episode, resumeAt, totalDuration, sourceCandidates);
      return;
    }

    const prefs = appPrefs(stateRef.current);
    const p2pEnabled = prefBool(prefs, 'p2pEnabled', true);
    const proceed = () => void handlePlay(stream, meta, episode, resumeAt, totalDuration, sourceCandidates);

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
  }, [handlePlay, stateRef]);

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
  }, [activeRoute]);

  useEffect(() => {
    const unlisten = listen<{ url?: string }>('deep-link-opened', (e) => {
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
  }, [navigateRoute]);

  const goBack = useCallback(() => {
    if (detailMeta) {
      void closePlayer();
      setDetailMeta(null);
      setDetailInitialEpisode(null);
      setDetailAutoShowStreams(false);
      setDetailResumeAt(undefined);
      return;
    }
    if (activeRoute === 'settings') { navigateRoute(lastNonSettingsRouteRef.current); return; }
    if (activeRoute === 'search') { navigateRoute(lastNonSearchRouteRef.current); }
  }, [detailMeta, activeRoute, navigateRoute, closePlayer]);

  const { searchFocusSignal, setSearchFocusSignal } = useGlobalShortcuts({ nativePlayerActive, navigateRoute, goBack });

  const dispatch = useCallback(async (actionJson: string) => {
    const result = await dispatchAction(actionJson);
    if (!result) return;
    try {
      const action = JSON.parse(actionJson) as { type?: string };
      if (action.type === 'settingsChanged') {
        const freshPrefs = await loadPrefs();
        storedPrefsRef.current = freshPrefs;
      }
    } catch {}
    updateState(result.state);
    if (result.effects.length > 0) {
      await pumpEffects(result.effects, updateStateDeferred).catch(() => undefined);
    }
  }, [updateState, updateStateDeferred]);

  const applyStoredPrefs = useCallback(async () => {
    const freshPrefs = await loadPrefs();
    storedPrefsRef.current = freshPrefs;
    setLanguage(typeof freshPrefs.language === 'string' ? freshPrefs.language : null);
    setRpdbApiKey(prefString(freshPrefs, 'rpdbApiKey', ''));
    void invoke('discord_presence_configure', { enabled: prefBool(freshPrefs, 'discordRichPresenceEnabled', true) });
    void invoke('set_diagnostic_mode', { enabled: prefBool(freshPrefs, 'diagnosticMode', false) });
    updateState({ settings: { values: freshPrefs } });
  }, [updateState]);

  const switchToNoProfile = useCallback(async () => {
    setActiveProfile(null);
    await setActiveProfileId('');
    await applyStoredPrefs();
  }, [applyStoredPrefs]);

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

  const handleNavigateDetail = useCallback((meta: Meta) => {
    setDetailInitialEpisode(null);
    setDetailAutoShowStreams(false);
    setDetailMeta(meta);
    const art = playerArtwork(meta, null);
    artworkPrefetchRef.current = prefetchPlayerArtwork(art.background, art.logo).catch(() => undefined);
    if (art.background) { const i = new Image(); i.src = art.background; }
    if (art.logo) { const i = new Image(); i.src = art.logo; }
  }, []);

  const leaveSearch = useCallback(() => {
    setGlobalSearchQuery('');
    navigateRoute(lastNonSearchRouteRef.current);
  }, [navigateRoute]);

  const handleHomePlay = useCallback((meta: Meta) => { setDetailMeta(meta); }, []);
  const handleLibraryBack = useCallback(() => { setActiveRoute('home'); }, []);
  const handleProfileUpdated = useCallback((updated: UserProfile) => { setActiveProfile(updated); }, [setActiveProfile]);
  const handleSearchQueryChange = useCallback((query: string) => { setGlobalSearchQuery(query); }, []);
  const handleDiscoverBack = useCallback(() => { setDiscoverInitialGenre(null); setActiveRoute('home'); }, []);

  const handleResumeFromContinueWatching = useCallback((meta: Meta, resumeAtOverride?: number) => {
    const item = meta as LibraryItem;
    const matchedVideo = item.lastVideoId ? meta.videos?.find((v) => v.id === item.lastVideoId) : undefined;
    const episode: Video | null = item.lastVideoId ? {
      id: item.lastVideoId,
      name: matchedVideo?.name ?? matchedVideo?.title ?? item.lastEpisodeName,
      season: matchedVideo?.season ?? item.lastEpisodeSeason,
      episode: matchedVideo?.episode ?? matchedVideo?.number ?? item.lastEpisodeNumber,
      number: matchedVideo?.episode ?? matchedVideo?.number ?? item.lastEpisodeNumber,
      thumbnail: matchedVideo?.thumbnail ?? item.lastEpisodeThumbnail,
    } : null;
    const resumeAt = resumeAtOverride ?? item.timeOffset;

    const art = playerArtwork(meta, episode);
    artworkPrefetchRef.current = prefetchPlayerArtwork(art.background, art.logo).catch(() => undefined);
    if (art.background) { const i = new Image(); i.src = art.background; }
    if (art.logo) { const i = new Image(); i.src = art.logo; }

    void (async () => {
      const stream = item.lastStream ?? await readStoredPlaybackSource(meta.id);
      const url = item.lastStreamUrl?.trim();
      const resumeStream: Stream | null = stream ?? (url
        ? { url, title: item.lastStreamTitle, name: item.lastStreamTitle }
        : null);

      if (!resumeStream) {
        setDetailInitialEpisode(episode);
        setDetailAutoShowStreams(true);
        setDetailResumeAt(resumeAt ?? undefined);
        setDetailMeta(meta);
        return;
      }
      await guardedPlay(resumeStream, meta, episode, resumeAt, item.duration);
    })();
  }, [guardedPlay]);

  const handleStartOverContinueWatching = useCallback((meta: Meta) => {
    handleResumeFromContinueWatching(meta, 0);
  }, [handleResumeFromContinueWatching]);

  const handlePlayManually = useCallback((meta: Meta) => {
    const item = meta as LibraryItem;
    const episode: Video | null = item.lastVideoId ? {
      id: item.lastVideoId,
      name: item.lastEpisodeName,
      season: item.lastEpisodeSeason,
      episode: item.lastEpisodeNumber,
      number: item.lastEpisodeNumber,
      thumbnail: item.lastEpisodeThumbnail,
    } : null;
    setDetailInitialEpisode(episode);
    setDetailAutoShowStreams(true);
    setDetailResumeAt(item.timeOffset ?? undefined);
    setDetailMeta(meta);
  }, []);

  const prefs = React.useMemo(() => appPrefs(state), [state.settings?.values]);
  const uiScale = prefString(prefs, 'uiScale', '100');
  useEffect(() => {
    const scale = (Number(uiScale) || 100) / 100;
    document.documentElement.style.fontSize = `${scale * 16}px`;
  }, [uiScale]);
  useEffect(() => {
    void (async () => {
      const flagKey = `ui_scale_auto_applied_${await prefsOwnerId()}`;
      const applied = await storageRead<boolean>(flagKey).catch(() => false);
      if (applied) return;
      const current = await loadPrefs().catch(() => ({} as Record<string, unknown>));
      if (typeof current.uiScale !== 'string') {
        const updated = { ...current, uiScale: String(computeAutoUiScale()) };
        await savePrefs(updated);
        storedPrefsRef.current = updated;
        updateState({ settings: { values: updated } });
      }
      await storageWrite(flagKey, true);
    })();
  }, [updateState]);
  const accentColor = prefString(prefs, 'accentColorArgb', '#FFFFFF');
  const rootStyle = React.useMemo(() => ({
    ...appStyles.root,
    background: nativePlayerActive ? 'transparent' : '#040508',
    ['--primary-accent-color' as string]: accentColor,
    ['--primary-accent-foreground-color' as string]: accentForegroundColor(accentColor),
  } as React.CSSProperties), [nativePlayerActive, prefs, accentColor]);

  React.useEffect(() => {
    document.documentElement.style.setProperty('--primary-accent-color', accentColor);
    document.documentElement.style.setProperty('--primary-accent-foreground-color', accentForegroundColor(accentColor));
  }, [accentColor]);

  React.useEffect(() => {
    if (detailMeta || nativePlayerActive) return;
    setBrowsingDiscordPresence(BROWSING_LABELS[activeRoute] ?? 'Browsing');
  }, [activeRoute, detailMeta, nativePlayerActive]);

  if (!ready || !profilesChecked) {
    return (
      <div style={appStyles.loading}>
        <span style={appStyles.loadingText}>fluxa</span>
      </div>
    );
  }

  if (!welcomeCompleted) {
    return (
      <React.Suspense fallback={null}>
      <WelcomeScreen
        onProfileCreated={async (profile) => {
          await storageWrite('welcome_done', true);
          const profiles = await loadProfiles();
          invalidateLibraryKeyCache();
          setAllProfiles(profiles);
          setActiveProfile(profile);
          void dispatch(JSON.stringify({ type: 'addonsRefreshRequested', forceRefresh: false }));
          setWelcomeCompleted(true);
        }}
        onContinueLocal={async () => {
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
          await storageWrite('welcome_done', true);
          const profiles = await loadProfiles();
          setAllProfiles(profiles);
          setActiveProfile(profile);
          await dispatch(JSON.stringify({ type: 'profileActivated', profile }));
          await applyStoredPrefs();
          await dispatch(JSON.stringify({ type: 'addonsRefreshRequested', forceRefresh: false }));
          void dispatch(JSON.stringify({ type: 'homeLoadRequested', force: true }));
          setWelcomeCompleted(true);
        }}
      />
      </React.Suspense>
    );
  }

  if (!activeProfile || editProfileOpen) {
    return (
      <React.Suspense fallback={null}>
      <ProfileSelectionScreen
        onProfileSelected={async (profile) => {
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
          void hydratePluginsFromStorage(updateState);
        }}
        onProfilesChanged={setAllProfiles}
      />
      </React.Suspense>
    );
  }

  const showDetail = detailMeta !== null;
  const bannerOffset = (serverDown && !dismissed) || justRecovered ? 36 : 0;
  const storedPrefs = (state.settings?.values ?? {}) as Record<string, unknown>;
  const navLayout = prefString(prefs, 'navLayout', 'sidebar');
  const rawNavBarPosition = typeof storedPrefs.navBarPosition === 'string'
    ? prefString(storedPrefs, 'navBarPosition', navLayout === 'topbar' ? 'top' : 'left')
    : (navLayout === 'topbar' ? 'top' : 'left');
  const isTopBar = navLayout === 'topbar';
  const navBarPosition = isTopBar && (rawNavBarPosition === 'left' || rawNavBarPosition === 'right')
    ? 'top'
    : rawNavBarPosition;
  const navItemsAlign = prefString(prefs, 'navItemsAlign', 'center');
  const navSidebarMode = prefString(prefs, 'navSidebarMode', 'hover');
  const sidebarAlwaysOpen = !isTopBar && navSidebarMode === 'always';
  const sidebarOffset = sidebarAlwaysOpen ? 112 : 0;
  const mirrorSearchToLeft = isTopBar && (
    navBarPosition === 'right' ||
    (navBarPosition === 'top' && navItemsAlign === 'end')
  );

  return (
    <div
      style={rootStyle}
      data-animations={prefBool(prefs, 'animationsEnabled', true) ? 'on' : 'off'}
      data-density={prefString(prefs, 'interfaceDensity', 'medium')}
      data-reduce-motion={prefBool(prefs, 'reduceMotion', false) ? 'true' : 'false'}
      data-reduced-effects={prefBool(prefs, 'reducedEffects', false) ? 'true' : 'false'}
    >
      {!nativePlayerActive && (isTopBar ? (
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
      ))}

      {!nativePlayerActive && (
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
          <GlobalSearchBar
            query={globalSearchQuery}
            onSearch={(query) => { setGlobalSearchQuery(query); navigateRoute('search'); }}
            onBack={leaveSearch}
            focusSignal={searchFocusSignal}
            state={state}
            onDispatch={dispatch}
            onNavigateDetail={handleNavigateDetail}
          />
          <div style={{ pointerEvents: 'auto', flexShrink: 0 }}>
            <ProfileChip
              profile={activeProfile}
              allProfiles={allProfiles}
              onSwitchProfile={() => void switchToNoProfile()}
              onSwitchToProfile={async (p) => {
                await setActiveProfileId(p.id);
                invalidateLibraryKeyCache();
                stateRef.current = DEFAULT_STATE;
                setState(DEFAULT_STATE);
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
      )}

      <div style={{ ...appStyles.content, top: (isTopBar && navBarPosition === 'top' && activeRoute !== 'home' && !showDetail ? 76 : 0) + bannerOffset, paddingLeft: sidebarAlwaysOpen && navBarPosition !== 'right' ? sidebarOffset : 0, paddingRight: sidebarAlwaysOpen && navBarPosition === 'right' ? sidebarOffset : 0, display: nativePlayerActive ? 'none' : undefined }}>
      <ErrorBoundary
        resetKeys={[activeRoute, detailMeta?.id]}
        onReset={() => { setDetailMeta(null); setActiveRoute('home'); }}
      >
        {showDetail && (
          <DetailScreen
            key={detailMeta!.id}
            meta={detailMeta!}
            state={state}
            onDispatch={dispatch}
            onPlay={(stream, meta, episode, resumeAt, sourceCandidates) => void guardedPlay(stream, meta, episode, resumeAt !== undefined ? resumeAt : (detailAutoShowStreams ? detailResumeAt : undefined), undefined, sourceCandidates)}
            onNavigateDetail={handleNavigateDetail}
            onNavigateGenre={(genre) => { setDiscoverInitialGenre(genre); setDetailMeta(null); navigateRoute('discover'); }}
            onBack={() => { void closePlayer(); setDetailMeta(null); setDetailInitialEpisode(null); setDetailAutoShowStreams(false); setDetailResumeAt(undefined); }}
            initialEpisode={detailInitialEpisode}
            autoShowStreams={detailAutoShowStreams}
            playbackFailure={detailPlaybackError}
          />
        )}
        <div style={{ display: !showDetail && activeRoute === 'home' ? 'contents' : 'none' }}>
          <HomeScreen
            state={state}
            onDispatch={dispatch}
            onNavigateDetail={handleNavigateDetail}
            onPlay={handleHomePlay}
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
            <CalendarScreen
              state={state}
              onDispatch={dispatch}
            />
          </React.Suspense>
        )}
        {!showDetail && activeRoute === 'discover' && (
          <React.Suspense fallback={null}>
            <DiscoverScreen
              state={state}
              onDispatch={dispatch}
              onNavigateDetail={handleNavigateDetail}
              onBack={handleDiscoverBack}
              initialGenre={discoverInitialGenre}
            />
          </React.Suspense>
        )}
        {!showDetail && activeRoute === 'library' && (
          <LibraryScreen
            state={state}
            onDispatch={dispatch}
            onNavigateDetail={handleNavigateDetail}
            onBack={handleLibraryBack}
            activeProfile={activeProfile}
            onProfileUpdated={handleProfileUpdated}
          />
        )}
        {!showDetail && activeRoute === 'search' ? (
          <SearchScreen
            state={state}
            onDispatch={dispatch}
            onNavigateDetail={handleNavigateDetail}
            query={globalSearchQuery}
            onQueryChange={handleSearchQueryChange}
            onBack={leaveSearch}
          />
        ) : !showDetail && activeRoute === 'settings' ? (
          <React.Suspense fallback={null}>
            <SettingsScreen
              state={state}
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
      </div>

      {isOnline ? (
        <NuvioStatusBanner
          serverDown={serverDown}
          justRecovered={justRecovered}
          dismissed={dismissed}
          onDismiss={dismiss}
        />
      ) : (
        <OfflineBanner online={isOnline} />
      )}
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
      {playerLoadingOverlay && (
        <PlayerLoadingOverlay
          background={playerLoadingOverlay.background}
          logo={playerLoadingOverlay.logo}
          title={playerLoadingOverlay.title}
          episodeLine={playerLoadingOverlay.episodeLine}
          status={playerLoadingOverlay.status}
          error={playerLoadingOverlay.error}
          isTorrentStream={playerUsesTorrent}
          source={playerLoadingOverlay.source}
          onBack={closePlayer}
        />
      )}
      {nativePlayerActive && (
        <ErrorBoundary>
          <ReactPlayerOverlay
            closePlayer={closePlayer}
            onFirstFrame={notifyFirstFrame}
            initialTitle={playerTitle}
            initialEpisodeTitle={playerEpisodeTitle}
            currentEpisode={playerEpisode}
            isTorrentStream={playerUsesTorrent}
            initialPosterUrl={playerPosterUrl}
            initialLogoUrl={playerLogoUrl}
            metaId={playerMetaId}
            initialSubtitleUrl={playerSubtitleUrl}
            initialStreamHeaders={playerStreamHeaders}
            streamRef={playingStreamRef}
            metaRef={playingMetaRef}
            playbackUrl={playerUrl}
            prefs={prefs}
            onDispatch={dispatch}
            playbackError={playerPlaybackError}
            subtitleWarning={playerSubtitleWarning}
            onDismissSubtitleWarning={dismissSubtitleWarning}
            softwareVideoActive={softwareVideoActive}
            bannerOffset={bannerOffset}
          />
        </ErrorBoundary>
      )}
    </div>
  );
}
