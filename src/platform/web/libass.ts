import JASSUB from 'jassub';
import workerUrl from 'jassub/dist/wasm/jassub-worker.js?url';
import wasmUrl from 'jassub/dist/wasm/jassub-worker.wasm?url';
import modernWasmUrl from 'jassub/dist/wasm/jassub-worker-modern.wasm?url';
import fallbackFontUrl from 'jassub/dist/default.woff2?url';
import serifFontUrl from '../../assets/subtitle-fonts/LiberationSerif-Regular.woff2?url';
import monoFontUrl from '../../assets/subtitle-fonts/LiberationMono-Regular.woff2?url';

const FONT_ALIASES: Record<string, string> = {
  'liberation sans': fallbackFontUrl,
  arial: fallbackFontUrl,
  helvetica: fallbackFontUrl,
  'helvetica neue': fallbackFontUrl,
  verdana: fallbackFontUrl,
  tahoma: fallbackFontUrl,
  'trebuchet ms': fallbackFontUrl,
  'segoe ui': fallbackFontUrl,
  roboto: fallbackFontUrl,
  'liberation serif': serifFontUrl,
  'times new roman': serifFontUrl,
  times: serifFontUrl,
  georgia: serifFontUrl,
  garamond: serifFontUrl,
  'book antiqua': serifFontUrl,
  'liberation mono': monoFontUrl,
  'courier new': monoFontUrl,
  courier: monoFontUrl,
  consolas: monoFontUrl,
};

export interface LibassRenderer {
  destroy(): Promise<void>;
}

export function createLibassRenderer(
  video: HTMLVideoElement,
  canvas: HTMLCanvasElement,
  subContent: string,
  extraFonts: Record<string, string> = {},
): LibassRenderer {
  const instance = new JASSUB({
    video,
    canvas,
    subContent,
    workerUrl,
    wasmUrl,
    modernWasmUrl,
    availableFonts: { ...FONT_ALIASES, ...extraFonts },
  });
  return {
    destroy: () => instance.destroy(),
  };
}
