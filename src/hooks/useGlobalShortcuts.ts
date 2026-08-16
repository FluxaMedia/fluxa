import { useCallback, useEffect, useRef, useState } from 'react';
import type { NavRoute } from '../components/NavSidebar';
import { toggleWindowFullscreen, watchWindowGeometry } from '../core/windowGeometry';
import { comboFromEvent, findActionForCombo, loadShortcutOverrides, onShortcutsChanged, type ShortcutOverrides } from '../core/shortcuts';
import { focusNearestCard, isNavCard } from '../core/spatialNav';
import { isBrowserTarget } from '../platform/browser';

export function useGlobalShortcuts({
  nativePlayerActive,
  navigateRoute,
  goBack,
}: {
  nativePlayerActive: boolean;
  navigateRoute: (route: NavRoute) => void;
  goBack: () => void;
}) {
  const isWebTarget = isBrowserTarget();
  const [searchFocusSignal, setSearchFocusSignal] = useState(0);
  const [shortcutOverrides, setShortcutOverrides] = useState<ShortcutOverrides>({});
  const windowFullscreenRef = useRef(false);

  useEffect(() => {
    loadShortcutOverrides().then(setShortcutOverrides);
    return onShortcutsChanged(setShortcutOverrides);
  }, []);

  const refreshWindowFullscreen = useCallback(() => {
    if (isWebTarget) {
      windowFullscreenRef.current = Boolean(document.fullscreenElement);
      return;
    }
    void import('@tauri-apps/api/window')
      .then(({ getCurrentWindow }) => getCurrentWindow().isFullscreen())
      .then((isFullscreen) => { windowFullscreenRef.current = isFullscreen; })
      .catch(() => undefined);
  }, [isWebTarget]);

  useEffect(() => {
    if (isWebTarget) return undefined;
    return watchWindowGeometry();
  }, [isWebTarget]);

  useEffect(() => {
    if (isWebTarget) return undefined;
    let unlisten: (() => void) | null = null;
    let disposed = false;
    refreshWindowFullscreen();
    void import('@tauri-apps/api/window')
      .then(({ getCurrentWindow }) => getCurrentWindow().listen('tauri://resize', refreshWindowFullscreen))
      .then((fn) => { if (disposed) fn(); else unlisten = fn; })
      .catch(() => undefined);
    return () => { disposed = true; unlisten?.(); };
  }, [isWebTarget, refreshWindowFullscreen]);

  useEffect(() => {
    const directions: Record<string, 'up' | 'down' | 'left' | 'right'> = {
      ArrowUp: 'up', ArrowDown: 'down', ArrowLeft: 'left', ArrowRight: 'right',
    };
    const onKeyDown = (e: KeyboardEvent) => {
      if (nativePlayerActive) return;
      const direction = directions[e.key];
      if (!direction || !isNavCard(document.activeElement)) return;
      if (focusNearestCard(document.activeElement, direction)) e.preventDefault();
    };
    window.addEventListener('keydown', onKeyDown);
    return () => window.removeEventListener('keydown', onKeyDown);
  }, [nativePlayerActive]);

  useEffect(() => {
    const navRoutes: Record<string, NavRoute> = {
      nav_home: 'home', nav_library: 'library', nav_discover: 'discover', nav_calendar: 'calendar', nav_settings: 'settings',
    };
    const onKeyDown = (e: KeyboardEvent) => {
      if (nativePlayerActive) return;
      const combo = comboFromEvent(e);
      if (findActionForCombo(combo, 'global', shortcutOverrides) === 'toggle_window_fullscreen') {
        e.preventDefault();
        if (isWebTarget) {
          if (document.fullscreenElement) void document.exitFullscreen();
          else void document.documentElement.requestFullscreen();
        } else {
          windowFullscreenRef.current = !windowFullscreenRef.current;
          void toggleWindowFullscreen().finally(refreshWindowFullscreen);
        }
        return;
      }
      if (e.key === 'Escape' && windowFullscreenRef.current) {
        e.preventDefault();
        windowFullscreenRef.current = false;
        if (isWebTarget) void document.exitFullscreen().catch(() => undefined).finally(refreshWindowFullscreen);
        else void import('@tauri-apps/api/window').then(({ getCurrentWindow }) => getCurrentWindow().setFullscreen(false)).catch(() => undefined).finally(refreshWindowFullscreen);
        return;
      }
      const target = e.target as HTMLElement | null;
      if (target && (target.tagName === 'INPUT' || target.tagName === 'TEXTAREA' || target.isContentEditable)) return;
      const globalAction = findActionForCombo(combo, 'global', shortcutOverrides);
      if (globalAction === 'focus_search') {
        e.preventDefault();
        setSearchFocusSignal((n) => n + 1);
        return;
      }
      if (globalAction === 'go_back') {
        e.preventDefault();
        goBack();
        return;
      }
      const route = globalAction ? navRoutes[globalAction] : undefined;
      if (route) { navigateRoute(route); return; }
      if (e.key === '/' && !e.metaKey && !e.ctrlKey && !e.altKey) {
        e.preventDefault();
        setSearchFocusSignal((n) => n + 1);
      }
    };
    window.addEventListener('keydown', onKeyDown);
    return () => window.removeEventListener('keydown', onKeyDown);
  }, [isWebTarget, nativePlayerActive, navigateRoute, goBack, refreshWindowFullscreen, shortcutOverrides]);

  return { searchFocusSignal, setSearchFocusSignal, windowFullscreenRef, refreshWindowFullscreen };
}
