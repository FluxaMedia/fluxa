import { describe, expect, it } from 'vitest';
import { buildSoftLimiterCurve } from './audioProcessing';

describe('buildSoftLimiterCurve', () => {
  it('keeps both polarities inside the configured headroom', () => {
    const curve = buildSoftLimiterCurve(2049, 0.98);

    expect(curve[0]).toBeCloseTo(-0.98, 3);
    expect(curve[curve.length - 1]).toBeCloseTo(0.98, 3);
    expect(Math.max(...curve)).toBeLessThanOrEqual(0.980001);
    expect(Math.min(...curve)).toBeGreaterThanOrEqual(-0.980001);
  });

  it('remains monotonic through the soft-knee region', () => {
    const curve = buildSoftLimiterCurve(257, 0.98);

    for (let index = 1; index < curve.length; index += 1) {
      expect(curve[index]).toBeGreaterThanOrEqual(curve[index - 1]);
    }
  });
});
