import { useSyncExternalStore } from 'react';
import { FLUXA_UI_TOKENS } from '../theme/uiTokens.generated';

const MOBILE_QUERY = '(max-width: 820px), (orientation: landscape) and (max-height: 480px)';
const TABLET_QUERY = '(min-width: 821px) and (max-width: 1180px)';
const TOUCH_QUERY = '(pointer: coarse)';

function query(value: string): MediaQueryList | null {
  if (typeof window === 'undefined' || typeof window.matchMedia !== 'function') return null;
  return window.matchMedia(value);
}

const mobile = query(MOBILE_QUERY);
const tablet = query(TABLET_QUERY);
const touch = query(TOUCH_QUERY);

function watch(list: MediaQueryList | null, listener: () => void): () => void {
  if (!list) return () => {};
  if (typeof list.addEventListener === 'function') {
    list.addEventListener('change', listener);
    return () => list.removeEventListener('change', listener);
  }
  list.addListener(listener);
  return () => list.removeListener(listener);
}

export function isMobileLayout(): boolean {
  return mobile?.matches ?? false;
}

export type LayoutTier = 'phone' | 'tablet' | 'desktop';

export function layoutTier(): LayoutTier {
  if (isMobileLayout()) return 'phone';
  return tablet?.matches ? 'tablet' : 'desktop';
}

export type WindowClass = 'compact' | 'medium' | 'expanded';

export function windowClass(): WindowClass {
  const width = typeof window === 'undefined' ? 0 : window.innerWidth;
  const breakpoints = FLUXA_UI_TOKENS.layout.windowClasses;
  if (width < breakpoints.compact.dp.breakpointMaxWidth) return 'compact';
  if (width < breakpoints.medium.dp.breakpointMaxWidth) return 'medium';
  return 'expanded';
}

export function isTouchInput(): boolean {
  return touch?.matches ?? false;
}

function subscribe(listener: () => void): () => void {
  const stopMobile = watch(mobile, listener);
  const stopTablet = watch(tablet, listener);
  const stopTouch = watch(touch, listener);
  return () => {
    stopMobile();
    stopTablet();
    stopTouch();
  };
}

export function useIsMobile(): boolean {
  return useSyncExternalStore(subscribe, isMobileLayout, () => false);
}

export function useLayoutTier(): LayoutTier {
  return useSyncExternalStore(subscribe, layoutTier, () => 'desktop' as LayoutTier);
}

export function useIsTouch(): boolean {
  return useSyncExternalStore(subscribe, isTouchInput, () => false);
}

export function startViewportFlags(): void {
  const apply = () => {
    const root = document.documentElement;
    root.dataset.mobile = isMobileLayout() ? 'true' : 'false';
    root.dataset.tier = layoutTier();
    root.dataset.touch = isTouchInput() ? 'true' : 'false';
    root.dataset.fluxaPlatform = isMobileLayout() ? 'mobile' : 'desktop';
    root.dataset.fluxaWindowClass = windowClass();
  };
  apply();
  subscribe(apply);
}
