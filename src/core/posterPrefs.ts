import type { AppState } from './types';

export type PosterLayout = 'vertical' | 'horizontal';

export interface PosterPrefs {
  layout: PosterLayout;
  width: number;
  height: number;
  radius: number;
  hideTitles: boolean;
}

export function posterPrefsFromState(state: AppState, scale = 1): PosterPrefs {
  const values = (state.settings?.values ?? {}) as Record<string, unknown>;
  const widthPreset = stringValue(values.posterWidthPreset, 'medium');
  const cornerPreset = stringValue(values.cardCornerPreset, 'soft');
  const cardLayout = stringValue(values.cardLayout, 'vertical');
  const density = stringValue(values.interfaceDensity, 'medium');
  const densityScale = density === 'small' ? 0.92 : density === 'large' ? 1.08 : 1;
  const landscape =
    values.posterLandscapeMode === true || cardLayout === 'horizontal' || cardLayout === 'episode';

  const width = landscape ? horizontalPosterWidth(widthPreset) : verticalPosterWidth(widthPreset);
  const height = landscape ? Math.round(width * 0.56) : Math.round(width * 1.5);

  return {
    layout: landscape ? 'horizontal' : 'vertical',
    width: Math.round(width * scale * densityScale),
    height: Math.round(height * scale * densityScale),
    radius: posterCornerRadius(cornerPreset),
    hideTitles: values.posterHideTitles === true,
  };
}

function stringValue(value: unknown, fallback: string): string {
  return typeof value === 'string' && value.trim() ? value : fallback;
}

function posterCornerRadius(value: string): number {
  switch (value) {
    case 'sharp':
      return 2;
    case 'classic':
    case 'small':
      return 8;
    case 'rounded':
    case 'large':
      return 18;
    case 'pill':
      return 28;
    default:
      return 12;
  }
}

function verticalPosterWidth(value: string): number {
  switch (value) {
    case 'xsmall':
      return 120;
    case 'small':
      return 138;
    case 'large':
      return 180;
    case 'xlarge':
      return 210;
    default:
      return 156;
  }
}

function horizontalPosterWidth(value: string): number {
  switch (value) {
    case 'xsmall':
      return 235;
    case 'small':
      return 247;
    case 'large':
      return 291;
    case 'xlarge':
      return 321;
    default:
      return 260;
  }
}
