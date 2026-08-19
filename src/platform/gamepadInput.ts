import type { TvAction } from './webosKeys';

const DEADZONE = 0.5;
const REPEAT_INITIAL_MS = 250;
const REPEAT_MS = 130;

const BUTTON_ACTIONS: Record<number, TvAction> = {
  0: 'enter',
  1: 'back',
  4: 'previous',
  5: 'next',
  6: 'rewind',
  7: 'fastForward',
  12: 'up',
  13: 'down',
  14: 'left',
  15: 'right',
};

function axisDirection(x: number, y: number): TvAction | null {
  if (Math.abs(x) < DEADZONE && Math.abs(y) < DEADZONE) return null;
  if (Math.abs(x) > Math.abs(y)) return x > 0 ? 'right' : 'left';
  return y > 0 ? 'down' : 'up';
}

type Listener = (action: TvAction) => void;

const listeners = new Set<Listener>();
const heldSince = new Map<TvAction, number>();
const lastFired = new Map<TvAction, number>();
let rafId: number | null = null;
let padCount = 0;

function poll(now: number) {
  const pads = navigator.getGamepads ? navigator.getGamepads() : [];
  const active = new Set<TvAction>();

  for (const pad of pads) {
    if (!pad || !pad.connected) continue;
    for (const [index, action] of Object.entries(BUTTON_ACTIONS)) {
      if (pad.buttons[Number(index)]?.pressed) active.add(action);
    }
    const stick = axisDirection(pad.axes[0] ?? 0, pad.axes[1] ?? 0);
    if (stick) active.add(stick);
  }

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
