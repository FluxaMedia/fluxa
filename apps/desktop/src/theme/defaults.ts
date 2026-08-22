import builtInThemesJson from '../../contracts/built-in-themes.json';
import type { SkinLayout, ThemePack } from './types';

export const FLUXA_DARK_THEME = builtInThemesJson[0] as ThemePack;

export const AMOLED_THEME = builtInThemesJson[1] as ThemePack;
export const MIDNIGHT_THEME = builtInThemesJson[2] as ThemePack;

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

export const BUILT_IN_THEMES: ThemePack[] = builtInThemesJson as ThemePack[];

export function themeById(id: string | undefined, customThemes: ThemePack[] = []): ThemePack {
  const customTheme = customThemes.find((theme) => theme.id === id);
  if (customTheme) return customTheme;
  return BUILT_IN_THEMES.find((theme) => theme.id === id) ?? FLUXA_DARK_THEME;
}
