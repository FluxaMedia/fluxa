export type WebAudioProcessingMode = 'reference' | 'balanced' | 'night';

type AudioContextConstructor = new () => AudioContext;

type ReusableMediaAudioGraph = {
  context: AudioContext;
  source: MediaElementAudioSourceNode;
};

// The Web Audio specification permits only one MediaElementAudioSourceNode
// per media element. Reuse it when a new URL or processing mode is loaded.
const reusableMediaAudioGraphs = new WeakMap<HTMLMediaElement, ReusableMediaAudioGraph>();

export function buildSoftLimiterCurve(length = 2049, ceiling = 0.98): Float32Array<ArrayBuffer> {
  const curveLength = Math.max(3, length);
  const curve: Float32Array<ArrayBuffer> = new Float32Array(
    new ArrayBuffer(curveLength * Float32Array.BYTES_PER_ELEMENT),
  );
  const safeCeiling = Math.max(0, Math.min(1, ceiling));
  const normalization = Math.tanh(1.15);
  for (let index = 0; index < curve.length; index += 1) {
    const input = (index / (curve.length - 1)) * 2 - 1;
    curve[index] = safeCeiling * (Math.tanh(input * 1.15) / normalization);
  }
  return curve;
}

/**
 * Applies optional PCM-only processing to an HTML media element.
 * Reference mode deliberately bypasses Web Audio so browser/TV spatial and
 * bitstream handling remains owned by the platform. A failed Web Audio setup
 * is non-fatal: the caller keeps native playback.
 */
export async function attachWebAudioProcessing(
  video: HTMLMediaElement,
  mode: string | undefined,
): Promise<(() => void) | null> {
  const normalized = mode === 'balanced' || mode === 'night' ? mode : 'reference';
  if (normalized === 'reference' || typeof window === 'undefined') return null;

  // A MediaElementAudioSourceNode can become silent for a cross-origin media
  // element without a matching CORS header. Native playback is preferable to
  // risking a silent track, so remote sources keep the platform PCM path.
  try {
    const mediaUrl = new URL(video.currentSrc || video.src, window.location.href);
    if (mediaUrl.origin !== window.location.origin && mediaUrl.protocol !== 'blob:') return null;
  } catch {
    return null;
  }

  const AudioContextImpl = (window.AudioContext ??
    (window as Window & { webkitAudioContext?: AudioContextConstructor }).webkitAudioContext) as
    | AudioContextConstructor
    | undefined;
  if (!AudioContextImpl) return null;

  let context: AudioContext | null = null;
  let source: MediaElementAudioSourceNode | null = null;
  try {
    const reusable = reusableMediaAudioGraphs.get(video);
    if (reusable && reusable.context.state !== 'closed') {
      context = reusable.context;
      source = reusable.source;
    } else {
      context = new AudioContextImpl();
      source = context.createMediaElementSource(video);
      reusableMediaAudioGraphs.set(video, { context, source });
    }
    const compressor = context.createDynamicsCompressor();
    compressor.threshold.value = normalized === 'night' ? -25 : -14;
    compressor.knee.value = normalized === 'night' ? 18 : 12;
    compressor.ratio.value = normalized === 'night' ? 3 : 1.5;
    compressor.attack.value = normalized === 'night' ? 0.02 : 0.03;
    compressor.release.value = normalized === 'night' ? 0.25 : 0.3;

    const gain = context.createGain();
    gain.gain.value = normalized === 'night' ? 1.08 : 1.0;
    const limiter = context.createWaveShaper();
    limiter.curve = buildSoftLimiterCurve();
    limiter.oversample = '4x';
    const disconnectGraph = () => {
      try {
        source?.disconnect();
        compressor.disconnect();
        gain.disconnect();
        limiter.disconnect();
      } catch {
        /* already disconnected */
      }
    };
    source.connect(compressor).connect(gain).connect(limiter).connect(context.destination);
    const resumed = await context
      .resume()
      .then(() => context?.state === 'running')
      .catch(() => false);
    if (!resumed) {
      // A rejected autoplay/user-gesture policy must never leave the media
      // element connected to a suspended graph: that would turn a recoverable
      // processing failure into silent playback. Closing the graph makes the
      // next attempt create a fresh context, while the caller keeps native
      // media-element output for this attempt.
      disconnectGraph();
      await context.close().catch(() => undefined);
      context = null;
      source = null;
      return null;
    }

    return () => {
      disconnectGraph();
      void context?.suspend().catch(() => undefined);
      context = null;
      source = null;
    };
  } catch {
    if (source) {
      try {
        source.disconnect();
      } catch {
        /* graph was not connected */
      }
    }
    if (context) void context.suspend().catch(() => undefined);
    return null;
  }
}
