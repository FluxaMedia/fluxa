import defaultThemeJson from '../../contracts/default-theme.json';
import type { SkinLayout, ThemePack } from './types';

export const FLUXA_DARK_THEME = defaultThemeJson as ThemePack;

export const AMOLED_THEME: ThemePack = {
  ...FLUXA_DARK_THEME,
  id: 'amoled',
  nameKey: 'theme.amoled',
  colors: {
    ...FLUXA_DARK_THEME.colors,
    background: '#000000',
    backgroundElevated: '#000000',
    surface: '#080808',
    surfaceRaised: '#141414',
    navigation: '#000000',
  },
};

export const MIDNIGHT_THEME: ThemePack = {
  ...FLUXA_DARK_THEME,
  id: 'midnight-blue',
  nameKey: 'theme.midnight_blue',
  colors: {
    ...FLUXA_DARK_THEME.colors,
    background: '#080A10',
    backgroundElevated: '#0D1220',
    surface: '#121A2A',
    surfaceRaised: '#1B263D',
    navigation: '#090E1A',
    textSecondary: '#A9B4C8',
    textMuted: '#71809A',
    accent: '#5C8DFF',
    accentForeground: '#FFFFFF',
  },
};

export const DEFAULT_SKIN: SkinLayout = {
  navigation: {
    visible: ['home', 'library', 'discover', 'calendar', 'settings'],
    order: ['home', 'library', 'discover', 'calendar', 'settings'],
  },
  home: {
    hiddenSections: [],
    sectionOrder: ['hero', 'continueWatching', 'catalogs'],
  },
  detail: {
    hiddenSections: [],
  },
  posterCard: {
    showYear: true,
    showRating: true,
    showGenres: true,
  },
};

export const BUILT_IN_THEMES: ThemePack[] = [FLUXA_DARK_THEME, AMOLED_THEME, MIDNIGHT_THEME];

export function themeById(id: string | undefined, customThemes: ThemePack[] = []): ThemePack {
  const customTheme = customThemes.find((theme) => theme.id === id);
  if (customTheme) return customTheme;
  return BUILT_IN_THEMES.find((theme) => theme.id === id) ?? FLUXA_DARK_THEME;
}
