import type { AppState } from './types';
import { DEFAULT_PREFS } from '../components/settings/settingsTypes';

export const DEFAULT_APP_PREFS: Record<string, unknown> = DEFAULT_PREFS as unknown as Record<string, unknown>;

export function appPrefs(state: Pick<AppState, 'settings'>): Record<string, unknown> {
  return {
    ...DEFAULT_APP_PREFS,
    ...((state.settings?.values ?? {}) as Record<string, unknown>),
  };
}

export function prefString(prefs: Record<string, unknown>, key: string, fallback = ''): string {
  const value = prefs[key];
  return typeof value === 'string' && value.trim() ? value.trim() : fallback;
}

export function prefBool(prefs: Record<string, unknown>, key: string, fallback = false): boolean {
  const value = prefs[key];
  return typeof value === 'boolean' ? value : fallback;
}
