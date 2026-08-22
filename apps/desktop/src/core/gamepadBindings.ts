import { storageRead, storageWrite } from './engine';
import type { TvAction } from '../platform/webos/keys';

export type GamepadActionCategory = 'global' | 'player';

export interface GamepadActionDef {
  id: TvAction;
  category: GamepadActionCategory;
  labelKey: string;
  default: number;
}

export type GamepadBindingOverrides = Partial<Record<TvAction, number>>;

const BINDINGS_STORAGE_KEY = 'gamepadBindings';
const BINDINGS_EVENT = 'fluxa-gamepad-bindings-updated';

export const GAMEPAD_ACTION_DEFS: GamepadActionDef[] = [
  { id: 'enter', category: 'global', labelKey: 'settings.controller_action_enter', default: 0 },
  { id: 'back', category: 'global', labelKey: 'settings.controller_action_back', default: 1 },
  { id: 'up', category: 'global', labelKey: 'settings.controller_action_up', default: 12 },
  { id: 'down', category: 'global', labelKey: 'settings.controller_action_down', default: 13 },
  { id: 'left', category: 'global', labelKey: 'settings.controller_action_left', default: 14 },
  { id: 'right', category: 'global', labelKey: 'settings.controller_action_right', default: 15 },
  { id: 'previous', category: 'player', labelKey: 'settings.controller_action_previous', default: 4 },
  { id: 'next', category: 'player', labelKey: 'settings.controller_action_next', default: 5 },
  { id: 'rewind', category: 'player', labelKey: 'settings.controller_action_rewind', default: 6 },
  { id: 'fastForward', category: 'player', labelKey: 'settings.controller_action_fastForward', default: 7 },
];

export const PLAYER_ACTION_FOR_GAMEPAD: Partial<Record<TvAction, string>> = {
  left: 'player_seek_back',
  right: 'player_seek_forward',
  up: 'player_volume_up',
  down: 'player_volume_down',
  enter: 'player_play_pause',
  previous: 'player_seek_big_back',
  next: 'player_seek_big_forward',
  rewind: 'player_frame_step_back',
  fastForward: 'player_frame_step_forward',
};

export const GLOBAL_ACTION_FOR_GAMEPAD: Partial<Record<TvAction, string>> = {
  back: 'go_back',
};

export function resolveGamepadButton(id: TvAction, overrides: GamepadBindingOverrides): number {
  const def = GAMEPAD_ACTION_DEFS.find((d) => d.id === id);
  if (!def) return -1;
  const override = overrides[id];
  return override !== undefined ? override : def.default;
}

export function findActionForButton(index: number, overrides: GamepadBindingOverrides): TvAction | null {
  for (const def of GAMEPAD_ACTION_DEFS) {
    if (resolveGamepadButton(def.id, overrides) === index) return def.id;
  }
  return null;
}

export async function loadGamepadBindingOverrides(): Promise<GamepadBindingOverrides> {
  return (await storageRead<GamepadBindingOverrides>(BINDINGS_STORAGE_KEY)) ?? {};
}

export async function saveGamepadBindingOverrides(overrides: GamepadBindingOverrides): Promise<void> {
  await storageWrite(BINDINGS_STORAGE_KEY, overrides);
  window.dispatchEvent(new CustomEvent(BINDINGS_EVENT, { detail: overrides }));
}

export function onGamepadBindingsChanged(cb: (overrides: GamepadBindingOverrides) => void): () => void {
  const handler = (e: Event) => cb((e as CustomEvent<GamepadBindingOverrides>).detail);
  window.addEventListener(BINDINGS_EVENT, handler);
  return () => window.removeEventListener(BINDINGS_EVENT, handler);
}
