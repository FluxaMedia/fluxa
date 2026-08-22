import { useCallback, type MutableRefObject } from 'react';
import { coreInvoke } from '../core/engine';
import { appPrefs, prefBool } from '../core/appPrefs';
import type { AppState, Stream } from '../core/types';

type Options = {
  stateRef: MutableRefObject<AppState>;
  sourceCandidatesRef: MutableRefObject<Stream[]>;
  attemptedSourceKeysRef: MutableRefObject<Set<string>>;
};

export function usePlayerRetry({ stateRef, sourceCandidatesRef, attemptedSourceKeysRef }: Options) {
  return useCallback(
    async (currentStream: Stream | null, force = false): Promise<Stream | null> => {
      if (!currentStream) return null;
      const prefs = appPrefs(stateRef.current);
      const plan = await coreInvoke<{ stream: Stream | null; attemptedKeys: string[] }>(
        'nextRetrySourcePlan',
        JSON.stringify({
          currentStream,
          candidates: sourceCandidatesRef.current,
          attemptedKeys: [...attemptedSourceKeysRef.current],
          autoRetry: prefBool(prefs, 'autoRetryNextSource', false),
          force,
          tryBingeGroup: prefBool(prefs, 'tryBingeGroup', false),
          p2pEnabled: prefBool(prefs, 'p2pEnabled', true),
        }),
      );
      attemptedSourceKeysRef.current = new Set(plan?.attemptedKeys ?? []);
      return plan?.stream ?? null;
    },
    [attemptedSourceKeysRef, sourceCandidatesRef, stateRef],
  );
}
