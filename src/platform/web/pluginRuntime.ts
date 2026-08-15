import Worker from './pluginWorker?worker';

type Pending = { resolve: (value: string) => void; reject: (reason: unknown) => void };

const worker = new Worker();
const pending = new Map<number, Pending>();
let requestId = 0;
const companionConfigured = Boolean(import.meta.env.VITE_FLUXA_COMPANION_URL);

worker.onmessage = (event: MessageEvent<{ id: number; result?: string; error?: string }>) => {
  const item = pending.get(event.data.id);
  if (!item) return;
  pending.delete(event.data.id);
  if (event.data.error) item.reject(new Error(event.data.error));
  else item.resolve(event.data.result || '[]');
};

export function runWebPluginScraper(
  code: string,
  scraperId: string,
  settingsJson: string,
  tmdbId: string,
  mediaType: string,
  season: number | null,
  episode: number | null,
): Promise<string> {
  if (!companionConfigured) return Promise.resolve('[]');
  const id = requestId++;
  return new Promise((resolve, reject) => {
    pending.set(id, { resolve, reject });
    worker.postMessage({ id, code, scraperId, settingsJson, tmdbId, mediaType, season, episode });
  });
}
