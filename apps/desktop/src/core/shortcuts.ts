import { storageRead, storageWrite } from './engine';

export type ShortcutCategory = 'global' | 'player';

export interface ShortcutDef {
  id: string;
  category: ShortcutCategory;
  labelKey: string;
  default: string;
}

export type ShortcutOverrides = Record<string, string>;

const SHORTCUTS_STORAGE_KEY = 'shortcuts';
const SHORTCUTS_EVENT = 'fluxa-shortcuts-updated';

export const SHORTCUT_DEFS: ShortcutDef[] = [
  { id: 'nav_home', category: 'global', labelKey: 'shortcuts.nav_home', default: 'Digit1' },
  { id: 'nav_library', category: 'global', labelKey: 'shortcuts.nav_library', default: 'Digit2' },
  { id: 'nav_discover', category: 'global', labelKey: 'shortcuts.nav_discover', default: 'Digit3' },
  { id: 'nav_calendar', category: 'global', labelKey: 'shortcuts.nav_calendar', default: 'Digit4' },
  { id: 'nav_settings', category: 'global', labelKey: 'shortcuts.nav_settings', default: 'Digit5' },
  { id: 'focus_search', category: 'global', labelKey: 'shortcuts.focus_search', default: 'Ctrl+KeyF' },
  { id: 'go_back', category: 'global', labelKey: 'shortcuts.go_back', default: 'Backspace' },
  { id: 'toggle_window_fullscreen', category: 'global', labelKey: 'shortcuts.toggle_window_fullscreen', default: 'F11' },

  { id: 'player_play_pause', category: 'player', labelKey: 'shortcuts.player_play_pause', default: 'KeyK' },
  { id: 'player_seek_back', category: 'player', labelKey: 'shortcuts.player_seek_back', default: 'ArrowLeft' },
  { id: 'player_seek_forward', category: 'player', labelKey: 'shortcuts.player_seek_forward', default: 'ArrowRight' },
  { id: 'player_seek_big_back', category: 'player', labelKey: 'shortcuts.player_seek_big_back', default: 'KeyJ' },
  { id: 'player_seek_big_forward', category: 'player', labelKey: 'shortcuts.player_seek_big_forward', default: 'KeyL' },
  { id: 'player_volume_up', category: 'player', labelKey: 'shortcuts.player_volume_up', default: 'ArrowUp' },
  { id: 'player_volume_down', category: 'player', labelKey: 'shortcuts.player_volume_down', default: 'ArrowDown' },
  { id: 'player_mute', category: 'player', labelKey: 'shortcuts.player_mute', default: 'KeyM' },
  { id: 'player_speed_decrease', category: 'player', labelKey: 'shortcuts.player_speed_decrease', default: 'BracketLeft' },
  { id: 'player_speed_increase', category: 'player', labelKey: 'shortcuts.player_speed_increase', default: 'BracketRight' },
  { id: 'player_cycle_subtitle', category: 'player', labelKey: 'shortcuts.player_cycle_subtitle', default: 'KeyC' },
  { id: 'player_cycle_audio', category: 'player', labelKey: 'shortcuts.player_cycle_audio', default: 'KeyA' },
  { id: 'player_toggle_stats', category: 'player', labelKey: 'shortcuts.player_toggle_stats', default: 'KeyI' },
  { id: 'player_frame_step_forward', category: 'player', labelKey: 'shortcuts.player_frame_step_forward', default: 'Period' },
  { id: 'player_frame_step_back', category: 'player', labelKey: 'shortcuts.player_frame_step_back', default: 'Comma' },
  { id: 'player_skip_active', category: 'player', labelKey: 'shortcuts.player_skip_active', default: 'KeyS' },
  { id: 'player_next_episode', category: 'player', labelKey: 'shortcuts.player_next_episode', default: 'Shift+KeyN' },
  { id: 'player_sub_delay_earlier', category: 'player', labelKey: 'shortcuts.player_sub_delay_earlier', default: 'KeyZ' },
  { id: 'player_sub_delay_later', category: 'player', labelKey: 'shortcuts.player_sub_delay_later', default: 'KeyX' },
  { id: 'player_fullscreen', category: 'player', labelKey: 'shortcuts.player_fullscreen', default: 'KeyF' },
  { id: 'player_toggle_shortcuts_help', category: 'player', labelKey: 'shortcuts.player_toggle_shortcuts_help', default: 'Shift+Slash' },
  { id: 'player_toggle_pip', category: 'player', labelKey: 'shortcuts.player_toggle_pip', default: 'KeyP' },
  { id: 'player_open_cast', category: 'player', labelKey: 'shortcuts.player_open_cast', default: 'KeyT' },
  { id: 'player_ab_loop', category: 'player', labelKey: 'shortcuts.player_ab_loop', default: 'KeyR' },
  { id: 'player_screenshot', category: 'player', labelKey: 'shortcuts.player_screenshot', default: 'KeyG' },
  { id: 'player_seek_start', category: 'player', labelKey: 'shortcuts.player_seek_start', default: 'Home' },
  { id: 'player_seek_end', category: 'player', labelKey: 'shortcuts.player_seek_end', default: 'End' },
  { id: 'player_anime4k_mode_next', category: 'player', labelKey: 'shortcuts.player_anime4k_mode_next', default: 'Alt+BracketRight' },
  { id: 'player_anime4k_mode_prev', category: 'player', labelKey: 'shortcuts.player_anime4k_mode_prev', default: 'Alt+BracketLeft' },
];

const CODE_LABELS: Record<string, string> = {
  ArrowLeft: '←',
  ArrowRight: '→',
  ArrowUp: '↑',
  ArrowDown: '↓',
  Space: 'Space',
  Escape: 'Esc',
  Enter: 'Enter',
  Backspace: 'Backspace',
  Period: '.',
  Comma: ',',
  Slash: '/',
  BracketLeft: '[',
  BracketRight: ']',
};

function codeLabel(code: string): string {
  if (CODE_LABELS[code]) return CODE_LABELS[code];
  if (code.startsWith('Key')) return code.slice(3);
  if (code.startsWith('Digit')) return code.slice(5);
  return code;
}

export function comboFromEvent(e: KeyboardEvent): string {
  const parts: string[] = [];
  if (e.ctrlKey || e.metaKey) parts.push('Ctrl');
  if (e.shiftKey) parts.push('Shift');
  if (e.altKey) parts.push('Alt');
  parts.push(e.code);
  return parts.join('+');
}

export function isModifierCode(code: string): boolean {
  return code.startsWith('Control') || code.startsWith('Shift') || code.startsWith('Alt') || code.startsWith('Meta');
}

export function formatCombo(combo: string): string {
  if (!combo) return '';
  return combo
    .split('+')
    .map((part) => (['Ctrl', 'Shift', 'Alt'].includes(part) ? part : codeLabel(part)))
    .join(' + ');
}

export function resolveCombo(id: string, overrides: ShortcutOverrides): string {
  const def = SHORTCUT_DEFS.find((d) => d.id === id);
  if (!def) return '';
  const override = overrides[id];
  return override !== undefined ? override : def.default;
}

export function findActionForCombo(combo: string, category: ShortcutCategory, overrides: ShortcutOverrides): string | null {
  for (const def of SHORTCUT_DEFS) {
    if (def.category !== category) continue;
    if (resolveCombo(def.id, overrides) === combo) return def.id;
  }
  return null;
}

export async function loadShortcutOverrides(): Promise<ShortcutOverrides> {
  return (await storageRead<ShortcutOverrides>(SHORTCUTS_STORAGE_KEY)) ?? {};
}

export async function saveShortcutOverrides(overrides: ShortcutOverrides): Promise<void> {
  await storageWrite(SHORTCUTS_STORAGE_KEY, overrides);
  window.dispatchEvent(new CustomEvent(SHORTCUTS_EVENT, { detail: overrides }));
}

export function onShortcutsChanged(cb: (overrides: ShortcutOverrides) => void): () => void {
  const handler = (e: Event) => cb((e as CustomEvent<ShortcutOverrides>).detail);
  window.addEventListener(SHORTCUTS_EVENT, handler);
  return () => window.removeEventListener(SHORTCUTS_EVENT, handler);
}
