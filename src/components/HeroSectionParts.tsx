import type { Meta } from '../core/types';

export function readOptionalString(meta: Meta, keys: string[]): string | null {
  const record = meta as unknown as Record<string, unknown>;
  for (const key of keys) {
    const value = record[key];
    if (typeof value === 'string' && value.trim()) return value.trim();
  }
  return null;
}
