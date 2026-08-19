import type React from 'react';
import type { NavRoute } from './components/NavSidebar';
import { isMobileLayout } from './platform/viewport';
import type { AppState } from './core/types';

export const OFFICIAL_FLUXA_SYNC_URL: string = '';

export const OFFICIAL_WATCH_TOGETHER_URL: string = '';

export const BROWSING_LABELS: Record<NavRoute, string> = {
  home: 'Browsing Home',
  search: 'Searching',
  library: 'Browsing Library',
  discover: 'Browsing Discover',
  calendar: 'Browsing Calendar',
  settings: 'In Settings',
};

export const DEFAULT_STATE: AppState = {
  navigation: { route: 'home', params: null },
  home: {},
  detail: {},
  search: {},
  player: {},
  library: {},
  discover: {},
  calendar: {},
  addons: { installed: [] },
  plugins: { repositories: [], scrapers: [] },
  settings: {},
  profile: {},
  pendingEffects: [],
};

export function computeAutoUiScale(): number {
  if (isMobileLayout()) return 100;
  const width = window.screen.width || 1920;
  const raw = Math.round(((width / 1920) * 100) / 5) * 5;
  return Math.min(150, Math.max(75, raw));
}

export function accentForegroundColor(hex: string): string {
  const c = hex.replace('#', '');
  const r = parseInt(c.substring(0, 2), 16) / 255;
  const g = parseInt(c.substring(2, 4), 16) / 255;
  const b = parseInt(c.substring(4, 6), 16) / 255;
  const luminance = 0.2126 * r + 0.7152 * g + 0.0722 * b;
  return luminance > 0.45 ? '#000000' : '#FFFFFF';
}

export const appStyles: Record<string, React.CSSProperties> = {
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
    fontSize: '2.5rem',
    fontWeight: 900,
    fontFamily: "'Montserrat', sans-serif",
    letterSpacing: 0,
  },
};
