import { afterEach, describe, expect, it } from 'vitest';
import { isNavCard } from './spatialNav';

describe('spatial navigation targets', () => {
  afterEach(() => {
    document.body.innerHTML = '';
  });

  it('includes native controls and excludes disabled controls', () => {
    document.body.innerHTML = '<button id="enabled">Enabled</button><button id="disabled" disabled>Disabled</button>';

    expect(isNavCard(document.getElementById('enabled'))).toBe(true);
    expect(isNavCard(document.getElementById('disabled'))).toBe(false);
  });
});
