import type { EffectType } from './generated/effectTypes';
import type { AppActionType } from './generated/actionTypes';

export type { EffectType } from './generated/effectTypes';
export type { AppActionType } from './generated/actionTypes';

export interface AppAction<TPayload extends Record<string, unknown> = Record<string, unknown>> {
  type: AppActionType;
  [key: string]: unknown;
  payload?: TPayload;
}

export interface Effect {
  id: string;
  type: EffectType;
  generation: number;
  payload: Record<string, unknown>;
  groupId?: string;
  priority?: number;
  dedupeKey?: string;
  cachePolicy?: string;
  timeoutMs?: number;
}

export interface EffectResult {
  effectId: string;
  status: 'ok' | 'error' | 'cancelled';
  value?: unknown;
  error?: unknown;
}
