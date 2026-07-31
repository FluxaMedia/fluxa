import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';
import type { EmbeddedMpvStatus, PlayerPositionStatus, PlayerStaticStatus } from './mpvPlayer';

type Listener = (status: EmbeddedMpvStatus) => void;

const listeners = new Set<Listener>();
let unlistens: UnlistenFn[] = [];
let listening: Promise<void> | null = null;
let sleepInhibitionEnabled = false;
let staticStatus: PlayerStaticStatus = {};
let positionStatus: PlayerPositionStatus | null = null;

function updateSleepInhibition(status: EmbeddedMpvStatus): void {
  const enabled = status.loaded && status.pause !== 'yes' && status.coreIdle !== 'yes';
  if (enabled === sleepInhibitionEnabled) return;
  sleepInhibitionEnabled = enabled;
  void invoke('player_set_sleep_inhibition', { enabled }).catch(() => {
    sleepInhibitionEnabled = false;
  });
}

function publish(): void {
  if (!positionStatus) return;
  const status = { ...staticStatus, ...positionStatus } as EmbeddedMpvStatus;
  updateSleepInhibition(status);
  listeners.forEach((listener) => listener(status));
}

function stopListening(): void {
  unlistens.forEach((stop) => stop());
  unlistens = [];
}

function ensureListening(): void {
  if (unlistens.length > 0 || listening) return;
  listening = Promise.all([
    listen<PlayerStaticStatus>('player-static-status', (event) => {
      staticStatus = { ...staticStatus, ...event.payload };
      publish();
    }),
    listen<PlayerPositionStatus>('player-position-status', (event) => {
      positionStatus = event.payload;
      publish();
    }),
  ]).then((stops) => {
    unlistens = stops;
    listening = null;
    if (listeners.size === 0) stopListening();
  }).catch(() => { listening = null; });
}

export function setPlayerStatusPositionInterval(controlsVisible: boolean): void {
  void invoke('player_set_status_interval', { intervalMs: controlsVisible ? 250 : 750 });
}

export function subscribePlayerStatus(listener: Listener): () => void {
  listeners.add(listener);
  ensureListening();
  return () => {
    listeners.delete(listener);
    if (listeners.size === 0) {
      stopListening();
      staticStatus = {};
      positionStatus = null;
      if (sleepInhibitionEnabled) {
        sleepInhibitionEnabled = false;
        void invoke('player_set_sleep_inhibition', { enabled: false });
      }
    }
  };
}
