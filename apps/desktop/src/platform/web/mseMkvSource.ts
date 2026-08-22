import DemuxWorker from './demuxWorker?worker';
import { codecString } from './stream';

type Session = {
  mediaSource: MediaSource;
  sourceBuffer: SourceBuffer | null;
  queue: ArrayBuffer[];
  appending: boolean;
  streamDone: boolean;
  resolveReady: (() => void) | null;
  rejectReady: ((reason: unknown) => void) | null;
};

let worker: Worker | null = null;
const sessions = new Map<number, Session>();
let requestId = 0;

function getWorker(): Worker {
  if (!worker) {
    worker = new DemuxWorker();
    worker.onmessage = (
      event: MessageEvent<
        | { id: number; type: 'segment'; webmBytes: ArrayBuffer }
        | { id: number; type: 'done' }
        | { id: number; type: 'error'; error: string }
      >,
    ) => {
      const session = sessions.get(event.data.id);
      if (!session) return;
      if (event.data.type === 'segment') {
        session.queue.push(event.data.webmBytes);
        pump(session);
      } else if (event.data.type === 'done') {
        session.streamDone = true;
        pump(session);
      } else {
        session.rejectReady?.(new Error(event.data.error));
        sessions.delete(event.data.id);
      }
    };
  }
  return worker;
}

function pump(session: Session) {
  if (session.appending || !session.sourceBuffer || session.sourceBuffer.updating) return;
  const next = session.queue.shift();
  if (!next) {
    if (session.streamDone && session.mediaSource.readyState === 'open') {
      session.mediaSource.endOfStream();
    }
    return;
  }
  session.appending = true;
  const onUpdateEnd = () => {
    session.sourceBuffer?.removeEventListener('updateend', onUpdateEnd);
    session.appending = false;
    session.resolveReady?.();
    session.resolveReady = null;
    session.rejectReady = null;
    pump(session);
  };
  session.sourceBuffer.addEventListener('updateend', onUpdateEnd, { once: true });
  session.sourceBuffer.appendBuffer(next);
}

// Streams the source through fetch()'s ReadableStream so the WASM remuxer
// can start producing WebM segments (and playback can start) well before
// the whole file has downloaded. Requires the source to allow cross-origin
// byte reads (CORS) — unlike plain <video src>, reading raw bytes via
// fetch() is subject to CORS, and there's no way around that from pure
// client-side JS. Callers should fall back (e.g. to the companion-server
// transcode path) if this rejects.
export async function attachMseRemuxSource(
  video: HTMLVideoElement,
  sourceUrl: string,
  videoCodec: string | null,
  audioCodec: string | null,
): Promise<() => void> {
  const id = requestId++;
  const mediaSource = new MediaSource();
  const objectUrl = URL.createObjectURL(mediaSource);
  video.src = objectUrl;

  const session: Session = {
    mediaSource,
    sourceBuffer: null,
    queue: [],
    appending: false,
    streamDone: false,
    resolveReady: null,
    rejectReady: null,
  };
  sessions.set(id, session);

  const ready = new Promise<void>((resolve, reject) => {
    session.resolveReady = resolve;
    session.rejectReady = reject;
    mediaSource.addEventListener(
      'sourceopen',
      () => {
        try {
          const mimeType = `video/webm; codecs="${codecString(videoCodec, audioCodec)}"`;
          session.sourceBuffer = mediaSource.addSourceBuffer(mimeType);
          getWorker().postMessage({ id, type: 'start', url: sourceUrl });
        } catch (error) {
          reject(error instanceof Error ? error : new Error(String(error)));
        }
      },
      { once: true },
    );
  });

  try {
    await ready;
  } catch (error) {
    sessions.delete(id);
    URL.revokeObjectURL(objectUrl);
    throw error;
  }

  return () => {
    sessions.delete(id);
    URL.revokeObjectURL(objectUrl);
  };
}
