import type { Meta, Stream, Video } from './types';
import { coreStreamShellPlan } from './engine';

export const streamShellPlan = coreStreamShellPlan;

export function buildOfflineDownloadRequest(meta: Meta, stream: Stream, video?: Video | null) {
  return {
    downloadId: crypto.randomUUID(),
    meta,
    video: video ?? undefined,
    stream,
  };
}
