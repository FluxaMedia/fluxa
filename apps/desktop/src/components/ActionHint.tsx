import { useLastInputMethod } from '../core/inputMethod';
import { formatCombo, resolveCombo, type ShortcutOverrides } from '../core/shortcuts';
import {
  GLOBAL_ACTION_FOR_GAMEPAD,
  PLAYER_ACTION_FOR_GAMEPAD,
  resolveGamepadButton,
  type GamepadBindingOverrides,
} from '../core/gamepadBindings';
import { detectGamepadBrand, GamepadButtonGlyph, gamepadButtonLabel, type GamepadBrand } from '../platform/gamepadGlyphs';
import { getConnectedGamepads } from '../platform/gamepadInput';
import type { TvAction } from '../platform/webos/keys';

function gamepadActionForShortcut(shortcutId: string): TvAction | null {
  for (const [gamepadAction, mapped] of Object.entries(PLAYER_ACTION_FOR_GAMEPAD)) {
    if (mapped === shortcutId) return gamepadAction as TvAction;
  }
  for (const [gamepadAction, mapped] of Object.entries(GLOBAL_ACTION_FOR_GAMEPAD)) {
    if (mapped === shortcutId) return gamepadAction as TvAction;
  }
  return null;
}

export type ActionHintValue = { kind: 'keyboard'; text: string } | { kind: 'gamepad'; brand: GamepadBrand; index: number };

export function resolveActionHint(
  shortcutId: string,
  lastInput: 'keyboard' | 'mouse' | 'touch' | 'gamepad',
  connectedPads: Gamepad[],
  shortcutOverrides: ShortcutOverrides,
  gamepadOverrides: GamepadBindingOverrides,
): ActionHintValue {
  const gamepadAction = gamepadActionForShortcut(shortcutId);
  if (lastInput === 'gamepad' && gamepadAction && connectedPads.length > 0) {
    return {
      kind: 'gamepad',
      brand: detectGamepadBrand(connectedPads[0].id),
      index: resolveGamepadButton(gamepadAction, gamepadOverrides),
    };
  }
  return { kind: 'keyboard', text: formatCombo(resolveCombo(shortcutId, shortcutOverrides)) };
}

export function actionHintText(hint: ActionHintValue): string {
  return hint.kind === 'gamepad' ? gamepadButtonLabel(hint.brand, hint.index) : hint.text;
}

export function useActionHint(
  shortcutId: string,
  shortcutOverrides: ShortcutOverrides,
  gamepadOverrides: GamepadBindingOverrides,
): ActionHintValue {
  const lastInput = useLastInputMethod();
  return resolveActionHint(shortcutId, lastInput, getConnectedGamepads(), shortcutOverrides, gamepadOverrides);
}

export function useActionHintText(
  shortcutId: string,
  shortcutOverrides: ShortcutOverrides,
  gamepadOverrides: GamepadBindingOverrides,
): string {
  return actionHintText(useActionHint(shortcutId, shortcutOverrides, gamepadOverrides));
}

export function ActionHint({
  shortcutId,
  shortcutOverrides,
  gamepadOverrides,
  size = 20,
}: {
  shortcutId: string;
  shortcutOverrides: ShortcutOverrides;
  gamepadOverrides: GamepadBindingOverrides;
  size?: number;
}) {
  const hint = useActionHint(shortcutId, shortcutOverrides, gamepadOverrides);
  if (hint.kind === 'gamepad') return <GamepadButtonGlyph brand={hint.brand} index={hint.index} size={size} />;
  return <span style={{ fontFamily: 'monospace', fontWeight: 600 }}>{hint.text || '—'}</span>;
}
