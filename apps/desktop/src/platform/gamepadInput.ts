import type { TvAction } from './webos/keys';
import {
  GAMEPAD_ACTION_DEFS,
  loadGamepadBindingOverrides,
  onGamepadBindingsChanged,
  type GamepadBindingOverrides,
} from '../core/gamepadBindings';

const DEADZONE = 0.5;
const REPEAT_INITIAL_MS = 250;
const REPEAT_MS = 130;

let bindingOverrides: GamepadBindingOverrides = {};
void loadGamepadBindingOverrides().then((overrides) => {
  bindingOverrides = overrides;
});
onGamepadBindingsChanged((overrides) => {
  bindingOverrides = overrides;
});

function buttonActions(): Record<number, TvAction> {
  const map: Record<number, TvAction> = {};
  for (const def of GAMEPAD_ACTION_DEFS) {
    const index = bindingOverrides[def.id] ?? def.default;
    map[index] = def.id;
  }
  return map;
}

function axisDirection(x: number, y: number): TvAction | null {
  if (Math.abs(x) < DEADZONE && Math.abs(y) < DEADZONE) return null;
  if (Math.abs(x) > Math.abs(y)) return x > 0 ? 'right' : 'left';
  return y > 0 ? 'down' : 'up';
}

type Listener = (action: TvAction) => void;
type RawListener = (index: number) => void;

const listeners = new Set<Listener>();
const rawListeners = new Set<RawListener>();
const heldSince = new Map<TvAction, number>();
const lastFired = new Map<TvAction, number>();
const rawPressed = new Set<number>();
let rafId: number | null = null;
let padCount = 0;

function poll(now: number) {
  const pads = navigator.getGamepads ? navigator.getGamepads() : [];
  const active = new Set<TvAction>();
  const actions = buttonActions();
  const nowPressed = new Set<number>();

  for (const pad of pads) {
    if (!pad || !pad.connected) continue;
    pad.buttons.forEach((button, index) => {
      if (!button.pressed) return;
      nowPressed.add(index);
      const action = actions[index];
      if (action) active.add(action);
      if (!rawPressed.has(index)) emitRaw(index);
    });
    const stick = axisDirection(pad.axes[0] ?? 0, pad.axes[1] ?? 0);
    if (stick) active.add(stick);
  }
  rawPressed.clear();
  for (const index of nowPressed) rawPressed.add(index);

  for (const action of active) {
    const since = heldSince.get(action);
    if (since === undefined) {
      heldSince.set(action, now);
      lastFired.set(action, now);
      emit(action);
      continue;
    }
    const held = now - since;
    const last = lastFired.get(action) ?? since;
    const interval = held < REPEAT_INITIAL_MS ? REPEAT_INITIAL_MS : REPEAT_MS;
    if (now - last >= interval) {
      lastFired.set(action, now);
      emit(action);
    }
  }
  for (const action of Array.from(heldSince.keys())) {
    if (!active.has(action)) {
      heldSince.delete(action);
      lastFired.delete(action);
    }
  }

  rafId = requestAnimationFrame(poll);
}

function emit(action: TvAction) {
  for (const listener of listeners) listener(action);
}

function emitRaw(index: number) {
  for (const listener of rawListeners) listener(index);
}

function startPolling() {
  if (rafId !== null) return;
  rafId = requestAnimationFrame(poll);
}

function stopPolling() {
  if (rafId === null) return;
  cancelAnimationFrame(rafId);
  rafId = null;
  heldSince.clear();
  lastFired.clear();
  rawPressed.clear();
}

function onGamepadConnected() {
  padCount += 1;
  startPolling();
}

function onGamepadDisconnected() {
  padCount = Math.max(0, padCount - 1);
  if (padCount === 0) stopPolling();
}

let initialized = false;
function ensureInitialized() {
  if (initialized) return;
  initialized = true;
  window.addEventListener('gamepadconnected', onGamepadConnected);
  window.addEventListener('gamepaddisconnected', onGamepadDisconnected);
  const pads = navigator.getGamepads ? navigator.getGamepads() : [];
  padCount = pads.filter((pad) => pad?.connected).length;
  if (padCount > 0) startPolling();
}

export function onGamepadAction(listener: Listener): () => void {
  ensureInitialized();
  listeners.add(listener);
  return () => listeners.delete(listener);
}

export function onRawGamepadButton(listener: RawListener): () => void {
  ensureInitialized();
  rawListeners.add(listener);
  return () => rawListeners.delete(listener);
}

export function getConnectedGamepads(): Gamepad[] {
  const pads = navigator.getGamepads ? navigator.getGamepads() : [];
  return pads.filter((pad): pad is Gamepad => !!pad && pad.connected);
}
