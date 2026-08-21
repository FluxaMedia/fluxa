import { useEffect, useMemo } from 'react';
import { prefString } from '../core/appPrefs';
import { storageRead, storageWrite } from '../core/engine';
import { loadPrefs, prefsOwnerId, savePrefs } from '../core/libraryOps';
import { accentForegroundColor, appStyles, computeAutoUiScale } from '../appConstants';
import type { AppState } from '../core/types';

export function useAppLayoutPrefs({
  state,
  prefs,
  nativePlayerActive,
  updateState,
  storedPrefsRef,
}: {
  state: AppState;
  prefs: Record<string, unknown>;
  nativePlayerActive: boolean;
  updateState: (s: Partial<AppState>) => void;
  storedPrefsRef: React.MutableRefObject<Record<string, unknown>>;
}) {
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
      const current = await loadPrefs().catch(() => ({}) as Record<string, unknown>);
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
  const rootStyle = useMemo(
    () =>
      ({
        ...appStyles.root,
        background: nativePlayerActive ? 'transparent' : '#060606',
        ['--primary-accent-color' as string]: accentColor,
        ['--primary-accent-foreground-color' as string]: accentForegroundColor(accentColor),
      }) as React.CSSProperties,
    [nativePlayerActive, prefs, accentColor],
  );

  useEffect(() => {
    document.documentElement.style.setProperty('--primary-accent-color', accentColor);
    document.documentElement.style.setProperty('--primary-accent-foreground-color', accentForegroundColor(accentColor));
  }, [accentColor]);

  const navLayout = prefString(prefs, 'navLayout', 'sidebar');
  const storedPrefs = (state.settings?.values ?? {}) as Record<string, unknown>;
  const rawNavBarPosition =
    typeof storedPrefs.navBarPosition === 'string'
      ? prefString(storedPrefs, 'navBarPosition', navLayout === 'topbar' ? 'top' : 'left')
      : navLayout === 'topbar'
        ? 'top'
        : 'left';
  const isTopBar = navLayout === 'topbar';
  const navBarPosition = isTopBar && (rawNavBarPosition === 'left' || rawNavBarPosition === 'right') ? 'top' : rawNavBarPosition;
  const navItemsAlign = prefString(prefs, 'navItemsAlign', 'center');
  const navSidebarMode = prefString(prefs, 'navSidebarMode', 'hover');
  const sidebarAlwaysOpen = !isTopBar && navSidebarMode === 'always';
  const sidebarOffset = sidebarAlwaysOpen ? 112 : 0;
  const mirrorSearchToLeft = isTopBar && (navBarPosition === 'right' || (navBarPosition === 'top' && navItemsAlign === 'end'));

  return {
    rootStyle,
    accentColor,
    isTopBar,
    navBarPosition,
    navItemsAlign,
    sidebarAlwaysOpen,
    sidebarOffset,
    mirrorSearchToLeft,
  };
}
