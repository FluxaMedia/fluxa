import React, { useCallback, useEffect, useRef, useState } from 'react';
import { NavSidebar, TopBar, type NavRoute } from './components/NavSidebar';
import { ProfileChip } from './components/ProfileChip';
import { GlobalSearchBar } from './components/GlobalSearchBar';
import { PlayerLoadingOverlay } from './components/PlayerLoadingOverlay';
import { ReactPlayerOverlay } from './components/ReactPlayerOverlay';
import { listen } from '@tauri-apps/api/event';
import { ErrorBoundary } from './components/ErrorBoundary';
import { UpdateModal, startUpdateCheck } from './components/UpdateModal';
import { HomeScreen } from './screens/HomeScreen';
import { SearchScreen } from './screens/SearchScreen';
import { DetailScreen } from './screens/DetailScreen';
import { LibraryScreen } from './screens/LibraryScreen';
import { DiscoverScreen } from './screens/DiscoverScreen';
import { CalendarScreen } from './screens/CalendarScreen';
import { SettingsScreen } from './screens/SettingsScreen';
import { ProfileSelectionScreen } from './screens/ProfileSelectionScreen';
import { setActiveProfileId } from './core/profiles';
import { dispatchAction } from './core/engine';
import { prefetchPlayerArtwork } from './core/mpvPlayer';
import { pumpEffects } from './core/effectRunner';
import { appPrefs, prefBool, prefString } from './core/appPrefs';
import { playerArtwork } from './core/playerUtils';
import { readStoredPlaybackSource } from './core/libraryStorage';
import { usePlayer } from './hooks/usePlayer';
import { useAppInit } from './hooks/useAppInit';
import type { AppState, LibraryItem, Meta, Stream, Video, UserProfile } from './core/types';

function accentForegroundColor(hex: string): string {
  const c = hex.replace('#', '');
  const r = parseInt(c.substring(0, 2), 16) / 255;
  const g = parseInt(c.substring(2, 4), 16) / 255;
  const b = parseInt(c.substring(4, 6), 16) / 255;
  const luminance = 0.2126 * r + 0.7152 * g + 0.0722 * b;
  return luminance > 0.45 ? '#000000' : '#FFFFFF';
}

const DEFAULT_STATE: AppState = {
  navigation: { route: 'home', params: null },
  home: {},
  detail: {},
  search: {},
  player: {},
  library: {},
  discover: {},
  calendar: {},
  addons: { installed: [] },
  settings: {},
  profile: {},
  pendingEffects: [],
};

export default function App() {
  const [state, setState] = useState<AppState>(DEFAULT_STATE);
  const [editProfileOpen, setEditProfileOpen] = useState(false);
  const [activeRoute, setActiveRoute] = useState<NavRoute>('home');
  const [detailMeta, setDetailMeta] = useState<Meta | null>(null);
  const [detailInitialEpisode, setDetailInitialEpisode] = useState<Video | null>(null);
  const [detailAutoShowStreams, setDetailAutoShowStreams] = useState(false);
  const [detailResumeAt, setDetailResumeAt] = useState<number | undefined>(undefined);
  const [discoverInitialGenre, setDiscoverInitialGenre] = useState<string | null>(null);
  const [globalSearchQuery, setGlobalSearchQuery] = useState('');
  const [nativePlayerActive, setNativePlayerActive] = useState(false);
  const storedPrefsRef = useRef<Record<string, unknown>>({});
  const stateRef = useRef<AppState>(DEFAULT_STATE);
  const lastNonSettingsRouteRef = useRef<NavRoute>('home');
  const lastNonSearchRouteRef = useRef<NavRoute>('home');
  const artworkPrefetchRef = useRef<Promise<unknown> | null>(null);

  const updateState = useCallback((s: AppState) => {
    const prefs = storedPrefsRef.current;
    const overlaid = Object.keys(prefs).length > 0
      ? { ...s, settings: { ...s.settings, values: prefs } }
      : s;
    stateRef.current = overlaid;
    setState(overlaid);
  }, []);

  useEffect(() => {
    const unlisteners: Array<() => void> = [];
    let cancelled = false;

    listen('native-player-show', () => {
      setNativePlayerActive(true);
      document.documentElement.setAttribute('data-native-player-active', 'true');
    }).then((fn) => { if (cancelled) fn(); else unlisteners.push(fn); }).catch(() => undefined);

    listen('native-player-hide', () => {
      setNativePlayerActive(false);
      document.documentElement.removeAttribute('data-native-player-active');
    }).then((fn) => { if (cancelled) fn(); else unlisteners.push(fn); }).catch(() => undefined);

    return () => { cancelled = true; unlisteners.forEach((fn) => fn()); };
  }, []);

  const {
    ready,
    profilesChecked,
    activeProfile,
    allProfiles,
    updateModalState,
    setActiveProfile,
    setUpdateModalState,
  } = useAppInit(updateState, setActiveRoute, storedPrefsRef);

  const { playerLoadingOverlay, handlePlay, closePlayer } = usePlayer({
    stateRef,
    activeProfile,
    updateState,
  });

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
    setActiveRoute(route);
    setDetailMeta(null);
  }, [activeRoute]);

  const dispatch = useCallback(async (actionJson: string) => {
    const result = await dispatchAction(actionJson);
    if (!result) return;
    try {
      const action = JSON.parse(actionJson) as { type?: string };
      if (action.type === 'settingsChanged') {
        const { storageRead } = await import('./core/engine');
        const freshPrefs = (await storageRead<Record<string, unknown>>('prefs')) ?? {};
        storedPrefsRef.current = freshPrefs;
      }
    } catch {}
    updateState(result.state);
    if (result.effects.length > 0) {
      await pumpEffects(result.effects, updateState).catch(() => undefined);
    }
  }, [updateState]);

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

  const handleResumeFromContinueWatching = useCallback((meta: Meta) => {
    const item = meta as LibraryItem;
    const episode: Video | null = item.lastVideoId ? {
      id: item.lastVideoId,
      name: item.lastEpisodeName,
      season: item.lastEpisodeSeason,
      episode: item.lastEpisodeNumber,
      number: item.lastEpisodeNumber,
      thumbnail: item.lastEpisodeThumbnail,
    } : null;

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
        setDetailResumeAt(item.timeOffset ?? undefined);
        setDetailMeta(meta);
        return;
      }
      await handlePlay(resumeStream, meta, episode, item.timeOffset, item.duration);
    })();
  }, [handlePlay]);

  if (!ready || !profilesChecked) {
    return (
      <div style={appStyles.loading}>
        <span style={appStyles.loadingText}>fluxa</span>
      </div>
    );
  }

  if (!activeProfile || editProfileOpen) {
    return (
      <ProfileSelectionScreen
        onProfileSelected={(profile) => {
          setActiveProfile(profile);
          setEditProfileOpen(false);
        }}
      />
    );
  }

  const showDetail = detailMeta !== null;
  const prefs = appPrefs(state);
  const storedPrefs = (state.settings?.values ?? {}) as Record<string, unknown>;
  const navLayout = prefString(prefs, 'navLayout', 'sidebar');
  const navBarPosition = typeof storedPrefs.navBarPosition === 'string'
    ? prefString(storedPrefs, 'navBarPosition', navLayout === 'topbar' ? 'top' : 'left')
    : (navLayout === 'topbar' ? 'top' : 'left');
  const navItemsAlign = prefString(prefs, 'navItemsAlign', 'center');
  const isTopBar = navLayout === 'topbar';
  const mirrorSearchToLeft = isTopBar && (
    navBarPosition === 'right' ||
    (navBarPosition === 'top' && navItemsAlign === 'end')
  );
  const accentColor = prefString(prefs, 'accentColorArgb', '#FFFFFF');
  const rootStyle = {
    ...appStyles.root,
    background: nativePlayerActive ? 'transparent' : (prefBool(prefs, 'amoledMode') ? '#000000' : '#040508'),
    ['--primary-accent-color' as string]: accentColor,
    ['--primary-accent-foreground-color' as string]: accentForegroundColor(accentColor),
  } as React.CSSProperties;

  return (
    <div
      style={rootStyle}
      data-animations={prefBool(prefs, 'animationsEnabled', true) ? 'on' : 'off'}
      data-density={prefString(prefs, 'interfaceDensity', 'medium')}
      data-reduce-motion={prefBool(prefs, 'reduceMotion', false) ? 'true' : 'false'}
    >
      {!nativePlayerActive && (isTopBar ? (
        <TopBar
          activeRoute={activeRoute}
          onNavigate={navigateRoute}
          transparent={activeRoute === 'home' && !showDetail}
          position={navBarPosition}
          itemsAlign={navItemsAlign}
        />
      ) : (
        <NavSidebar
          activeRoute={activeRoute}
          onNavigate={navigateRoute}
          position={navBarPosition}
          itemsAlign={navItemsAlign}
        />
      ))}

      {!nativePlayerActive && (
        <div
          style={{
            position: 'fixed',
            top: 18,
            left: mirrorSearchToLeft ? 20 : undefined,
            right: mirrorSearchToLeft ? undefined : 20,
            zIndex: 46,
            display: 'flex',
            alignItems: 'center',
            gap: 8,
            pointerEvents: 'none',
          }}
        >
          <GlobalSearchBar
            query={globalSearchQuery}
            onSearch={(query) => { setGlobalSearchQuery(query); navigateRoute('search'); }}
            onBack={leaveSearch}
          />
          <div style={{ pointerEvents: 'auto', flexShrink: 0 }}>
            <ProfileChip
              profile={activeProfile}
              allProfiles={allProfiles}
              onSwitchProfile={() => setActiveProfile(null)}
              onSwitchToProfile={async (p) => { await setActiveProfileId(p.id); setActiveProfile(p); }}
              onOpenSettings={() => navigateRoute('settings')}
              onEditProfile={() => setEditProfileOpen(true)}
            />
          </div>
        </div>
      )}

      <div style={{ ...appStyles.content, top: isTopBar && navBarPosition === 'top' && activeRoute !== 'home' && !showDetail ? 76 : 0, display: nativePlayerActive ? 'none' : undefined }}>
        {showDetail && (
          <DetailScreen
            key={detailMeta!.id}
            meta={detailMeta!}
            state={state}
            onDispatch={dispatch}
            onPlay={(stream, meta, episode) => handlePlay(stream, meta, episode, detailAutoShowStreams ? detailResumeAt : undefined)}
            onNavigateDetail={handleNavigateDetail}
            onNavigateGenre={(genre) => { setDiscoverInitialGenre(genre); setDetailMeta(null); navigateRoute('discover'); }}
            onBack={() => { void closePlayer(); setDetailMeta(null); setDetailInitialEpisode(null); setDetailAutoShowStreams(false); setDetailResumeAt(undefined); }}
            initialEpisode={detailInitialEpisode}
            autoShowStreams={detailAutoShowStreams}
          />
        )}
        <div style={{ display: !showDetail && activeRoute === 'home' ? 'contents' : 'none' }}>
          <HomeScreen
            state={state}
            onDispatch={dispatch}
            onNavigateDetail={handleNavigateDetail}
            onPlay={(meta) => setDetailMeta(meta)}
            onResume={handleResumeFromContinueWatching}
          />
        </div>
        {!showDetail && activeRoute === 'search' ? (
          <SearchScreen
            state={state}
            onDispatch={dispatch}
            onNavigateDetail={handleNavigateDetail}
            query={globalSearchQuery}
            onQueryChange={(query) => setGlobalSearchQuery(query)}
            onBack={leaveSearch}
          />
        ) : !showDetail && activeRoute === 'library' ? (
          <LibraryScreen
            state={state}
            onDispatch={dispatch}
            onNavigateDetail={handleNavigateDetail}
            onBack={() => setActiveRoute('home')}
            activeProfile={activeProfile}
            onProfileUpdated={(updated) => setActiveProfile(updated)}
          />
        ) : !showDetail && activeRoute === 'discover' ? (
          <DiscoverScreen
            state={state}
            onDispatch={dispatch}
            onNavigateDetail={handleNavigateDetail}
            onBack={() => { setDiscoverInitialGenre(null); setActiveRoute('home'); }}
            initialGenre={discoverInitialGenre}
          />
        ) : !showDetail && activeRoute === 'calendar' ? (
          <CalendarScreen
            state={state}
            onDispatch={dispatch}
          />
        ) : !showDetail && activeRoute === 'settings' ? (
          <SettingsScreen
            state={state}
            onDispatch={dispatch}
            activeProfile={activeProfile}
            onProfileUpdated={(updated) => setActiveProfile(updated)}
            onSwitchProfile={() => setActiveProfile(null)}
            onBack={() => navigateRoute(lastNonSettingsRouteRef.current)}
            onCheckForUpdates={() => void startUpdateCheck(setUpdateModalState)}
          />
        ) : null}
      </div>

      <UpdateModal state={updateModalState} onClose={() => setUpdateModalState({ phase: 'idle' })} />
      {playerLoadingOverlay && (
        <PlayerLoadingOverlay
          background={playerLoadingOverlay.background}
          logo={playerLoadingOverlay.logo}
          title={playerLoadingOverlay.title}
          episodeLine={playerLoadingOverlay.episodeLine}
        />
      )}
      {nativePlayerActive && (
        <ReactPlayerOverlay closePlayer={closePlayer} />
      )}
    </div>
  );
}

const appStyles: Record<string, React.CSSProperties> = {
  root: {
    position: 'relative',
    width: '100vw',
    height: '100vh',
    background: '#040508',
    overflow: 'hidden',
  },
  content: {
    position: 'absolute',
    top: 0,
    left: 0,
    right: 0,
    bottom: 0,
    overflow: 'hidden',
  },
  loading: {
    width: '100vw',
    height: '100vh',
    background: '#040508',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
  },
  loadingText: {
    color: '#FFFFFF',
    fontSize: 40,
    fontWeight: 900,
    fontFamily: "'Montserrat', sans-serif",
    letterSpacing: 0,
  },
};
