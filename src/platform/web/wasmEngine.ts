let corePromise: Promise<typeof import('fluxa_core')> | null = null;
let engineHandle: number | null = null;

function loadCore() {
  if (!corePromise) {
    corePromise = import('fluxa_core').then(async (core) => {
      await core.default();
      return core;
    });
  }
  return corePromise;
}

export async function coreInvoke(method: string, argsJson: string): Promise<string> {
  const core = await loadCore();
  return core.core_invoke(method, argsJson);
}

export async function engineInit(initialJson: string): Promise<number> {
  const core = await loadCore();
  engineHandle = core.engine_init(initialJson);
  return engineHandle;
}

export async function engineDispatch(actionJson: string): Promise<string | null> {
  const core = await loadCore();
  if (engineHandle == null) throw new Error('engine_dispatch called before engine_init');
  return core.engine_dispatch(engineHandle, actionJson) ?? null;
}

export async function engineCompleteEffect(resultJson: string): Promise<string | null> {
  const core = await loadCore();
  if (engineHandle == null) throw new Error('engine_complete_effect called before engine_init');
  return core.engine_complete_effect(engineHandle, resultJson) ?? null;
}

export async function engineSnapshot(): Promise<string | null> {
  const core = await loadCore();
  if (engineHandle == null) return null;
  return core.engine_snapshot(engineHandle) ?? null;
}
