import { useSyncExternalStore } from 'react';
import { isModifierCode } from './shortcuts';
import { onGamepadAction } from '../platform/gamepadInput';

export type InputMethod = 'keyboard' | 'mouse' | 'touch' | 'gamepad';

let current: InputMethod = 'mouse';
const listeners = new Set<() => void>();

function setCurrent(next: InputMethod) {
  if (current === next) return;
  current = next;
  for (const listener of listeners) listener();
}

function onKeyDown(e: KeyboardEvent) {
  if (isModifierCode(e.code)) return;
  setCurrent('keyboard');
}

function onPointerDown(e: PointerEvent) {
  setCurrent(e.pointerType === 'touch' ? 'touch' : 'mouse');
}

let initialized = false;
export function startInputMethodTracking(): void {
  if (initialized) return;
  initialized = true;
  window.addEventListener('keydown', onKeyDown);
  window.addEventListener('pointerdown', onPointerDown);
  onGamepadAction(() => setCurrent('gamepad'));
}

function subscribe(listener: () => void): () => void {
  listeners.add(listener);
  return () => listeners.delete(listener);
}

function getSnapshot(): InputMethod {
  return current;
}

export function useLastInputMethod(): InputMethod {
  return useSyncExternalStore(subscribe, getSnapshot, () => 'mouse');
}
