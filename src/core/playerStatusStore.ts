import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';
import type { EmbeddedMpvStatus } from './mpvPlayer';

type Listener = (status: EmbeddedMpvStatus) => void;

const listeners = new Set<Listener>();
let unlisten: UnlistenFn | null = null;
let listening: Promise<void> | null = null;
let sleepInhibitionEnabled = false;

function updateSleepInhibition(status: EmbeddedMpvStatus): void {
  const enabled = status.loaded && status.pause !== 'yes' && status.coreIdle !== 'yes';
  if (enabled === sleepInhibitionEnabled) return;
  sleepInhibitionEnabled = enabled;
  void invoke('player_set_sleep_inhibition', { enabled }).catch(() => {
    sleepInhibitionEnabled = false;
  });
}

function publish(status: EmbeddedMpvStatus): void {
  updateSleepInhibition(status);
  listeners.forEach((listener) => listener(status));
}

function ensureListening(): void {
  if (unlisten || listening) return;
  listening = listen<EmbeddedMpvStatus>('player-status', (event) => publish(event.payload)).then((stop) => {
    unlisten = stop;
    listening = null;
    if (listeners.size === 0) {
      unlisten();
      unlisten = null;
    }
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
      unlisten?.();
      unlisten = null;
      if (sleepInhibitionEnabled) {
        sleepInhibitionEnabled = false;
        void invoke('player_set_sleep_inhibition', { enabled: false });
      }
    }
  };
}
