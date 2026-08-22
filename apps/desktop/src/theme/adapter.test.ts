import { describe, expect, it } from 'vitest';
import { DEFAULT_SKIN, FLUXA_DARK_THEME } from './defaults';
import { isValidThemePack, orderedVisibleRoutes, parseThemePacks, resolveTheme } from './adapter';

describe('theme adapter', () => {
  it('accepts the portable default theme contract', () => {
    expect(isValidThemePack(FLUXA_DARK_THEME)).toBe(true);
  });

  it('rejects a theme with an invalid color', () => {
    expect(isValidThemePack({ ...FLUXA_DARK_THEME, colors: { ...FLUXA_DARK_THEME.colors, accent: 'red' } })).toBe(false);
  });

  it('rejects CSS injection through font fields', () => {
    expect(isValidThemePack({ ...FLUXA_DARK_THEME, typography: { ...FLUXA_DARK_THEME.typography, bodyFont: 'url(javascript:alert(1))' } })).toBe(false);
  });

  it('falls back safely when skin configuration is malformed', () => {
    expect(resolveTheme('fluxa-dark', '{"navigation":{"visible":"home"}}').skin).toEqual(DEFAULT_SKIN);
  });

  it('keeps only visible routes in the configured order', () => {
    const runtime = resolveTheme('fluxa-dark', JSON.stringify({ navigation: { visible: ['settings', 'home'], order: ['home', 'library', 'settings'] } }));
    expect(orderedVisibleRoutes(runtime.skin)).toEqual(['home', 'settings']);
  });

  it('loads only validated custom theme packs', () => {
    const customTheme = { ...FLUXA_DARK_THEME, id: 'custom-test', name: 'Custom Test' };
    expect(parseThemePacks(JSON.stringify([customTheme, { id: 'unsafe' }]))).toEqual([customTheme]);
    expect(resolveTheme('custom-test', '', [customTheme]).theme.name).toBe('Custom Test');
  });
});
