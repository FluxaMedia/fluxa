let corePromise: Promise<typeof import('fluxa_core')> | null = null;

function loadCore() {
  if (!corePromise) {
    corePromise = import('fluxa_core').then(async (core) => {
      await core.default();
      return core;
    });
  }
  return corePromise;
}

type StartMessage = { id: number; type: 'start'; url: string };
type OutMessage =
  | { id: number; type: 'segment'; webmBytes: ArrayBuffer }
  | { id: number; type: 'done' }
  | { id: number; type: 'error'; error: string };

function post(message: OutMessage, transfer: Transferable[] = []) {
  (self as unknown as Worker).postMessage(message, transfer);
}

self.onmessage = async (event: MessageEvent<StartMessage>) => {
  const { id, url } = event.data;
  try {
    const core = await loadCore();
    const remuxer = new core.IncrementalMkvRemuxer();
    const response = await fetch(url);
    if (!response.ok || !response.body) throw new Error(`fetch failed: ${response.status}`);

    const reader = response.body.getReader();
    for (;;) {
      const { done, value } = await reader.read();
      if (done) break;
      const segment = remuxer.push(value);
      if (segment.length > 0) {
        const buffer = segment.buffer.slice(segment.byteOffset, segment.byteOffset + segment.byteLength) as ArrayBuffer;
        post({ id, type: 'segment', webmBytes: buffer }, [buffer]);
      }
    }
    const tail = remuxer.finish();
    if (tail.length > 0) {
      const buffer = tail.buffer.slice(tail.byteOffset, tail.byteOffset + tail.byteLength) as ArrayBuffer;
      post({ id, type: 'segment', webmBytes: buffer }, [buffer]);
    }
    post({ id, type: 'done' });
  } catch (error) {
    post({ id, type: 'error', error: error instanceof Error ? error.message : String(error) });
  }
};
