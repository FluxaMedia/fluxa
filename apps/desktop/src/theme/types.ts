export type ThemeColors = {
  background: string;
  backgroundElevated: string;
  surface: string;
  surfaceRaised: string;
  navigation: string;
  textPrimary: string;
  textSecondary: string;
  textMuted: string;
  border: string;
  borderStrong: string;
  accent: string;
  accentForeground: string;
  success: string;
  warning: string;
  error: string;
  info: string;
  focus: string;
  scrim: string;
};

export type ThemePack = {
  schemaVersion: 1;
  id: string;
  nameKey: string;
  name?: string;
  colors: ThemeColors;
  typography: {
    displayFont: string;
    bodyFont: string;
    titleWeight: number;
    bodyWeight: number;
  };
  shape: {
    cardRadius: number;
    controlRadius: number;
    dialogRadius: number;
  };
  spacing: {
    screenPadding: number;
    sectionGap: number;
    controlGap: number;
  };
  motion: {
    enabled: boolean;
    fastMs: number;
    normalMs: number;
    slowMs: number;
  };
  layouts: {
    home: string;
    detail: string;
    library: string;
    navigation: string;
  };
};

export type SkinLayout = {
  navigation: {
    visible: string[];
    order: string[];
  };
  home: {
    hiddenSections: string[];
    sectionOrder: string[];
  };
  detail: {
    hiddenSections: string[];
  };
  posterCard: {
    showYear: boolean;
    showRating: boolean;
    showGenres: boolean;
  };
};

export type ThemeRuntime = {
  theme: ThemePack;
  skin: SkinLayout;
};
