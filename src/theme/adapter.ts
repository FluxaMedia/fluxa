import type React from 'react';
import { accentForegroundColor } from '../appConstants';
import { BUILT_IN_THEMES, DEFAULT_SKIN, themeById } from './defaults';
import type { SkinLayout, ThemePack, ThemeRuntime } from './types';

const HEX_COLOR = /^#[0-9a-f]{6}(?:[0-9a-f]{2})?$/i;
const THEME_ID = /^[a-z0-9][a-z0-9._-]{1,63}$/;
const THEME_NAME_KEY = /^[a-z0-9._-]+$/;
const FONT_FAMILY = /^[a-zA-Z0-9\s,'._-]{1,120}$/;
const COLOR_KEYS = ['background', 'backgroundElevated', 'surface', 'surfaceRaised', 'navigation', 'textPrimary', 'textSecondary', 'textMuted', 'border', 'borderStrong', 'accent', 'accentForeground', 'success', 'warning', 'error', 'info', 'focus', 'scrim'];
const SHAPE_KEYS = ['cardRadius', 'controlRadius', 'dialogRadius'];
const SPACING_KEYS = ['screenPadding', 'sectionGap', 'controlGap'];
const LAYOUT_KEYS = ['home', 'detail', 'library', 'navigation'];

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null && !Array.isArray(value);
}

function validStringArray(value: unknown): value is string[] {
  return Array.isArray(value) && value.every((item) => typeof item === 'string');
}

export function isValidThemePack(value: unknown): value is ThemePack {
  if (!isRecord(value)) return false;
  if (value.schemaVersion !== 1 || typeof value.id !== 'string' || !THEME_ID.test(value.id) || typeof value.nameKey !== 'string' || !THEME_NAME_KEY.test(value.nameKey)) return false;
  if ('name' in value && (typeof value.name !== 'string' || value.name.length > 80)) return false;
  const colors = value.colors;
  if (!isRecord(colors) || !COLOR_KEYS.every((key) => key in colors) || !Object.values(colors).every((color) => typeof color === 'string' && HEX_COLOR.test(color))) return false;
  if (!isRecord(value.typography) || typeof value.typography.displayFont !== 'string' || !FONT_FAMILY.test(value.typography.displayFont) || typeof value.typography.bodyFont !== 'string' || !FONT_FAMILY.test(value.typography.bodyFont)) return false;
  if (typeof value.typography.titleWeight !== 'number' || typeof value.typography.bodyWeight !== 'number') return false;
  const shape = value.shape;
  if (!isRecord(shape) || !SHAPE_KEYS.every((key) => key in shape) || !Object.values(shape).every((size) => typeof size === 'number' && size >= 0)) return false;
  const spacing = value.spacing;
  if (!isRecord(spacing) || !SPACING_KEYS.every((key) => key in spacing) || !Object.values(spacing).every((size) => typeof size === 'number' && size >= 0)) return false;
  if (
    !isRecord(value.motion) ||
    typeof value.motion.enabled !== 'boolean' ||
    typeof value.motion.fastMs !== 'number' ||
    typeof value.motion.normalMs !== 'number' ||
    typeof value.motion.slowMs !== 'number' ||
    ![value.motion.fastMs, value.motion.normalMs, value.motion.slowMs].every((duration) => duration >= 0)
  ) return false;
  const layouts = value.layouts;
  if (!isRecord(layouts) || !LAYOUT_KEYS.every((key) => key in layouts) || !Object.values(layouts).every((layout) => typeof layout === 'string')) return false;
  return true;
}

export function parseThemePacks(rawCustomThemes: string | undefined): ThemePack[] {
  if (!rawCustomThemes) return [];
  try {
    const parsed: unknown = JSON.parse(rawCustomThemes);
    if (!Array.isArray(parsed)) return [];
    return parsed.filter(isValidThemePack).filter((theme) => !BUILT_IN_THEMES.some((builtIn) => builtIn.id === theme.id)).slice(0, 24);
  } catch {
    return [];
  }
}

function parseSkinConfig(rawSkinConfig: string | undefined): Partial<SkinLayout> {
  if (!rawSkinConfig) return {};
  try {
    const parsed: unknown = JSON.parse(rawSkinConfig);
    if (!isRecord(parsed)) return {};
    const skin: Partial<SkinLayout> = {};
    if (isRecord(parsed.navigation)) {
      skin.navigation = {
        visible: validStringArray(parsed.navigation.visible) ? parsed.navigation.visible : DEFAULT_SKIN.navigation.visible,
        order: validStringArray(parsed.navigation.order) ? parsed.navigation.order : DEFAULT_SKIN.navigation.order,
      };
    }
    if (isRecord(parsed.home)) {
      skin.home = {
        hiddenSections: validStringArray(parsed.home.hiddenSections) ? parsed.home.hiddenSections : DEFAULT_SKIN.home.hiddenSections,
        sectionOrder: validStringArray(parsed.home.sectionOrder) ? parsed.home.sectionOrder : DEFAULT_SKIN.home.sectionOrder,
      };
    }
    if (isRecord(parsed.detail) && validStringArray(parsed.detail.hiddenSections)) skin.detail = { hiddenSections: parsed.detail.hiddenSections };
    if (isRecord(parsed.posterCard)) {
      skin.posterCard = {
        showYear: typeof parsed.posterCard.showYear === 'boolean' ? parsed.posterCard.showYear : DEFAULT_SKIN.posterCard.showYear,
        showRating: typeof parsed.posterCard.showRating === 'boolean' ? parsed.posterCard.showRating : DEFAULT_SKIN.posterCard.showRating,
        showGenres: typeof parsed.posterCard.showGenres === 'boolean' ? parsed.posterCard.showGenres : DEFAULT_SKIN.posterCard.showGenres,
      };
    }
    return skin;
  } catch {
    return {};
  }
}

function cssName(value: string): string {
  return `--fluxa-${value.replace(/[A-Z]/g, (letter) => `-${letter.toLowerCase()}`)}`;
}

function withAlpha(color: string, alpha: number): string {
  const value = color.slice(1);
  if (value.length !== 6 && value.length !== 8) return color;
  const red = Number.parseInt(value.slice(0, 2), 16);
  const green = Number.parseInt(value.slice(2, 4), 16);
  const blue = Number.parseInt(value.slice(4, 6), 16);
  return `rgba(${red}, ${green}, ${blue}, ${alpha})`;
}

export function resolveTheme(themeId: string | undefined, rawSkinConfig?: string, customThemes?: ThemePack[]): ThemeRuntime {
  const parsed = parseSkinConfig(rawSkinConfig);
  return {
    theme: customThemes?.find((theme) => theme.id === themeId) ?? themeById(themeId),
    skin: {
      ...DEFAULT_SKIN,
      ...parsed,
      navigation: { ...DEFAULT_SKIN.navigation, ...(parsed.navigation ?? {}) },
      home: { ...DEFAULT_SKIN.home, ...(parsed.home ?? {}) },
      detail: { ...DEFAULT_SKIN.detail, ...(parsed.detail ?? {}) },
      posterCard: { ...DEFAULT_SKIN.posterCard, ...(parsed.posterCard ?? {}) },
    },
  };
}

export function themeCssVariables(theme: ThemePack): Record<string, string> {
  const variables: Record<string, string> = {};
  for (const [key, value] of Object.entries(theme.colors)) variables[cssName(key)] = value;
  variables['--fluxa-text-strong'] = withAlpha(theme.colors.textPrimary, 0.88);
  variables['--fluxa-text-dim'] = withAlpha(theme.colors.textMuted, 0.72);
  variables['--fluxa-text-faint'] = withAlpha(theme.colors.textMuted, 0.48);
  variables['--fluxa-fill-active'] = withAlpha(theme.colors.textPrimary, 0.22);
  variables['--fluxa-fill-strong'] = withAlpha(theme.colors.textPrimary, 0.3);
  variables['--fluxa-accent-soft'] = withAlpha(theme.colors.accent, 0.2);
  variables['--fluxa-accent-shadow'] = withAlpha(theme.colors.accent, 0.65);
  variables['--fluxa-display-font'] = theme.typography.displayFont;
  variables['--fluxa-body-font'] = theme.typography.bodyFont;
  variables['--fluxa-title-weight'] = String(theme.typography.titleWeight);
  variables['--fluxa-body-weight'] = String(theme.typography.bodyWeight);
  variables['--fluxa-card-radius'] = `${theme.shape.cardRadius}px`;
  variables['--fluxa-control-radius'] = `${theme.shape.controlRadius}px`;
  variables['--fluxa-dialog-radius'] = `${theme.shape.dialogRadius}px`;
  variables['--fluxa-screen-padding'] = `${theme.spacing.screenPadding}px`;
  variables['--fluxa-section-gap'] = `${theme.spacing.sectionGap}px`;
  variables['--fluxa-control-gap'] = `${theme.spacing.controlGap}px`;
  variables['--fluxa-motion-enabled'] = theme.motion.enabled ? '1' : '0';
  variables['--fluxa-motion-fast'] = `${theme.motion.fastMs}ms`;
  variables['--fluxa-motion-normal'] = `${theme.motion.normalMs}ms`;
  variables['--fluxa-motion-slow'] = `${theme.motion.slowMs}ms`;
  variables['--primary-accent-color'] = theme.colors.accent;
  variables['--primary-accent-foreground-color'] = theme.colors.accentForeground || accentForegroundColor(theme.colors.accent);
  return variables;
}

export function applyThemeToDocument(theme: ThemePack): void {
  const root = document.documentElement;
  for (const [name, value] of Object.entries(themeCssVariables(theme))) root.style.setProperty(name, value);
  root.dataset.fluxaTheme = theme.id;
}

export function themeRootStyle(theme: ThemePack, nativePlayerActive: boolean): React.CSSProperties {
  return {
    background: nativePlayerActive ? 'transparent' : 'var(--fluxa-background)',
    color: 'var(--fluxa-text-primary)',
  };
}

export function orderedVisibleRoutes(skin: SkinLayout): string[] {
  const visible = new Set(skin.navigation.visible);
  return skin.navigation.order.filter((route) => visible.has(route));
}
