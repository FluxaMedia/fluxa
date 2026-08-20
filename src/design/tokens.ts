export const color = {
  bg: '#040508',
  bgElevated: '#0A0B14',
  navBg: '#090B12',
  surface: '#12161D',
  surfaceRaised: '#1B212B',
  scrim: 'rgba(4,5,8,0.78)',

  textPrimary: '#FFFFFF',
  textBody: 'rgba(255,255,255,0.72)',
  textMuted: 'rgba(255,255,255,0.55)',
  textDim: 'rgba(255,255,255,0.38)',
  textFaint: 'rgba(255,255,255,0.24)',

  line: 'rgba(255,255,255,0.08)',
  lineStrong: 'rgba(255,255,255,0.16)',
  outline: 'rgba(255,255,255,0.42)',

  fill: 'rgba(255,255,255,0.06)',
  fillHover: 'rgba(255,255,255,0.10)',
  fillActive: 'rgba(255,255,255,0.16)',

  accent: '#E85D3F',
  accentGold: '#F0C674',
  success: '#45D483',
  error: '#FF6B6B',
  info: '#2196F3',
  imdb: '#F5C518',

  onLight: '#000000',
  light: '#FFFFFF',
  lightHover: '#E2E2E2',
} as const;

export const radius = {
  xs: '0.25rem',
  sm: '0.375rem',
  md: '0.5rem',
  lg: '0.75rem',
  xl: '1rem',
  xxl: '1.25rem',
  pill: '999px',
  circle: '50%',
} as const;

export const space = {
  0.5: '0.125rem',
  1: '0.25rem',
  1.5: '0.375rem',
  2: '0.5rem',
  2.5: '0.625rem',
  3: '0.75rem',
  4: '1rem',
  5: '1.25rem',
  6: '1.5rem',
  8: '2rem',
  10: '2.5rem',
  12: '3rem',
  15: '3.75rem',
} as const;

export const font = {
  display: "'Archivo', 'Montserrat', -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif",
  body: "'Montserrat', -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif",
  mono: "ui-monospace, SFMono-Regular, Menlo, monospace",
} as const;

export const fontSize = {
  micro: '0.625rem',
  xs: '0.6875rem',
  sm: '0.75rem',
  base: '0.8125rem',
  md: '0.875rem',
  lg: '1rem',
  xl: '1.125rem',
  xxl: '1.375rem',
  h1: '2rem',
  hero: '3.125rem',
} as const;

export const lineHeight = {
  tight: 1.05,
  snug: 1.25,
  normal: 1.45,
  relaxed: 1.6,
} as const;

export const weight = {
  regular: 400,
  medium: 500,
  semibold: 600,
  bold: 700,
  extrabold: 800,
  black: 900,
} as const;

export const stretch = {
  medium: '106%',
  semibold: '112%',
  bold: '116%',
  extrabold: '120%',
  black: '125%',
} as const;

export const tracking = {
  tight: '-0.03em',
  normal: '0',
  wide: '0.08em',
  wider: '0.12em',
} as const;

export const motion = {
  fast: '120ms',
  base: '180ms',
  slow: '300ms',
  ease: 'ease',
  easeOut: 'cubic-bezier(0.16, 1, 0.3, 1)',
} as const;

export const layout = {
  navRail: '6.5rem',
  topBar: '3.25rem',
  gutter: '2.5rem',
  detailRail: '17.5rem',
  reading: '41.25rem',
} as const;

export const z = {
  below: 0,
  content: 1,
  raised: 5,
  sticky: 20,
  nav: 40,
  topBar: 46,
  overlay: 60,
  dialog: 70,
  toast: 80,
  player: 100,
} as const;

export const heading = (level: 'hero' | 'h1' | 'h2' | 'h3') => {
  const sizes = { hero: fontSize.hero, h1: fontSize.h1, h2: fontSize.xxl, h3: fontSize.xl };
  const weights = { hero: weight.black, h1: weight.extrabold, h2: weight.bold, h3: weight.semibold };
  const stretches = { hero: stretch.black, h1: stretch.extrabold, h2: stretch.bold, h3: stretch.semibold };
  return {
    fontFamily: font.display,
    fontSize: sizes[level],
    fontWeight: weights[level],
    fontStretch: stretches[level],
    letterSpacing: tracking.tight,
    lineHeight: lineHeight.tight,
    margin: 0,
  } as const;
};
