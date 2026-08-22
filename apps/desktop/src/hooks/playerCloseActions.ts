import { dispatchAction } from '../core/engine';
import { pumpEffects } from '../core/effectRunner';
import type { AppState } from '../core/types';

export async function applyPlayerCloseActions(
  actions: Array<Record<string, unknown> | null | undefined>,
  updateState: (state: Partial<AppState>) => void,
) {
  for (const action of actions) {
    if (!action) continue;
    const result = await dispatchAction(JSON.stringify(action)).catch(() => null);
    if (!result) continue;
    updateState(result.state);
    if (result.effects.length > 0) await pumpEffects(result.effects, updateState);
  }
}
