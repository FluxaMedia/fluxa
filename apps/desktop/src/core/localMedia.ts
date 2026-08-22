import { platformInvoke } from '../platform/invoke';
import { coreInvoke } from './engine';
import { runSearch } from './catalogEffects';
import { fetchMetaDetail } from './detailEffects';
import type { Meta, Stream, Video } from './types';

export interface LocalMediaFile {
  path: string;
  fileName: string;
  relativePath: string;
  sizeBytes: number;
  modifiedAtMs?: number;
}

interface ParsedName {
  title: string;
  year?: number;
  season?: number;
  episode?: number;
  absoluteEpisode?: number;
  explicitId?: string;
}

export interface LocalMediaItem extends Meta {
  localFiles: LocalMediaFile[];
  localEpisodes: Record<string, Video>;
}

export function localFileUrl(path: string): string {
  const normalized = path.replaceAll('\\', '/');
  return normalized.startsWith('/') ? `file://${encodeURI(normalized)}` : `file:///${encodeURI(normalized)}`;
}

export async function scanLocalMedia(root: string): Promise<LocalMediaFile[]> {
  return platformInvoke<LocalMediaFile[]>('local_media_scan', { root });
}

function parsedFor(file: LocalMediaFile, kind: 'movies' | 'tvShows'): Promise<ParsedName | null> {
  return coreInvoke<ParsedName>(
    'localMediaParseFilename',
    JSON.stringify({
      fileName: file.fileName,
      parentHints: file.relativePath.split(/[\\/]/).slice(0, -1),
      kind,
    }),
  );
}

function streamFor(file: LocalMediaFile): Stream {
  const url = localFileUrl(file.path);
  return {
    url,
    playableUrl: url,
    name: 'Local file',
    title: file.fileName,
    addonName: 'Local media',
    behaviorHints: { filename: file.fileName, videoSize: file.sizeBytes },
    extra: { localPath: file.path, localRelativePath: file.relativePath },
  };
}

function toMeta(value: unknown): Meta | null {
  if (!value || typeof value !== 'object') return null;
  const record = value as Record<string, unknown>;
  if (typeof record.id !== 'string' || typeof record.name !== 'string' || typeof record.type !== 'string') return null;
  return record as unknown as Meta;
}

export async function resolveLocalMedia(root: string): Promise<LocalMediaItem[]> {
  const files = await scanLocalMedia(root);
  const groups = new Map<string, { file: LocalMediaFile; parsed: ParsedName; kind: 'movies' | 'tvShows' }[]>();
  for (const file of files) {
    const kind = /(?:s\d{1,3}e\d{1,4}|\d{1,2}x\d{1,4}|episode|ep\d+)/i.test(file.fileName) ? 'tvShows' : 'movies';
    const parsed = await parsedFor(file, kind);
    if (!parsed?.title) continue;
    const key = `${kind}|${parsed.explicitId ?? parsed.title.toLowerCase()}|${parsed.year ?? ''}`;
    const group = groups.get(key) ?? [];
    group.push({ file, parsed, kind });
    groups.set(key, group);
  }

  const searchCache = new Map<string, Promise<unknown>>();
  const items: LocalMediaItem[] = [];
  for (const group of groups.values()) {
    const first = group[0];
    const query = first.parsed.explicitId ?? first.parsed.title;
    const cacheKey = `${first.kind}|${query}`;
    const direct = first.parsed.explicitId
      ? fetchMetaDetail({ id: first.parsed.explicitId, contentType: first.kind === 'movies' ? 'movie' : 'series' }).catch(() => null)
      : Promise.resolve(null);
    const search = searchCache.get(cacheKey) ?? runSearch({ query, language: 'en' });
    searchCache.set(cacheKey, search);
    const result = (await search) as { results?: unknown[] } | null;
    const directMeta = toMeta(await direct);
    const candidates = directMeta ? [directMeta, ...(result?.results ?? [])] : (result?.results ?? []);
    let winner: Meta | null = null;
    let winnerScore = 0.62;
    for (const candidate of candidates) {
      const meta = toMeta(candidate);
      if (!meta) continue;
      const score = await coreInvoke<number>('localMediaScoreCandidate', JSON.stringify({ parsed: first.parsed, meta, kind: first.kind }));
      if (score != null && score > winnerScore) {
        winner = meta;
        winnerScore = score;
      }
    }
    if (!winner) continue;
    const detailed = await fetchMetaDetail({
      id: winner.id,
      contentType: winner.type,
      sourceAddonTransportUrl: winner.sourceAddonTransportUrl,
    }).catch(() => null);
    const meta = toMeta(detailed) ?? winner;
    const localFiles = group.map(({ file }) => file);
    const videos = (meta.videos ?? []) as Video[];
    const localEpisodes: Record<string, Video> = {};
    await Promise.all(
      group.map(async ({ file, parsed }) => {
        const video = await coreInvoke<Video>('localMediaResolveVideo', JSON.stringify({ parsed, videos })).catch(() => null);
        if (video) localEpisodes[file.path] = video;
      }),
    );
    items.push({ ...meta, localFiles, localEpisodes, videos });
  }
  return items.sort((a, b) => a.name.localeCompare(b.name));
}

export function localStream(item: LocalMediaItem, file: LocalMediaFile): { stream: Stream; episode?: Video } {
  const stream = streamFor(file);
  const video = item.localEpisodes[file.path];
  return { stream, episode: video };
}
