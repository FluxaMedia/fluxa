import JASSUB from 'jassub';
import workerUrl from 'jassub/dist/wasm/jassub-worker.js?url';
import wasmUrl from 'jassub/dist/wasm/jassub-worker.wasm?url';
import modernWasmUrl from 'jassub/dist/wasm/jassub-worker-modern.wasm?url';
import fallbackFontUrl from 'jassub/dist/default.woff2?url';

export interface LibassRenderer {
  destroy(): Promise<void>;
}

export function createLibassRenderer(
  video: HTMLVideoElement,
  canvas: HTMLCanvasElement,
  subContent: string,
): LibassRenderer {
  const instance = new JASSUB({
    video,
    canvas,
    subContent,
    workerUrl,
    wasmUrl,
    modernWasmUrl,
    availableFonts: { 'liberation sans': fallbackFontUrl },
  });
  return {
    destroy: () => instance.destroy(),
  };
}
