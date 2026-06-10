import { prefBool, prefString } from './appPrefs';
import type { Stream, Video, Meta } from './types';

export type PlayerSubtitleSource = {
  url: string;
  label?: string;
  lang?: string;
};

export type PlayerDisplayTitle = {
  contentTitle: string;
  episodeLine?: string;
};

export type PlayerArtwork = {
  background?: string | null;
  logo?: string | null;
};

export type PlaybackPreparePlan = {
  mode?: 'direct' | 'torrent' | 'reject';
  url?: string;
  rejectReason?: string;
  subtitleExtraArgs?: string;
  title?: PlayerDisplayTitle;
  artwork?: PlayerArtwork;
};

export function stringValue(value: unknown): string | undefined {
  return typeof value === 'string' && value.trim() ? value.trim() : undefined;
}

export function withCloseTimeout<T>(promise: Promise<T>, timeoutMs: number): Promise<T> {
  return Promise.race([
    promise,
    new Promise<T>((_, reject) => {
      window.setTimeout(() => reject(new Error('player close timed out')), timeoutMs);
    }),
  ]);
}

export function playerDisplayTitle(meta?: Meta, episode?: Video | null, stream?: Stream): PlayerDisplayTitle {
  const contentTitle = meta?.name ?? stream?.title ?? stream?.name ?? 'Fluxa';
  const season = episode?.season;
  const episodeNumber = episode?.episode ?? episode?.number;
  const episodeName = episode?.name ?? episode?.title;
  if (typeof season === 'number' && typeof episodeNumber === 'number') {
    const prefix = `S${season}, E${episodeNumber}`;
    return {
      contentTitle,
      episodeLine: episodeName?.trim() ? `${prefix}: ${episodeName.trim()}` : prefix,
    };
  }
  return { contentTitle };
}

export function playerArtwork(meta?: Meta, episode?: Video | null): PlayerArtwork {
  const record = (meta ?? {}) as Record<string, unknown>;
  return {
    background:
      stringValue(record.background) ??
      stringValue(record.backgroundUrl) ??
      stringValue(record.backdrop) ??
      stringValue(record.backdropUrl) ??
      episode?.thumbnail ??
      meta?.poster,
    logo:
      stringValue(record.logo) ??
      stringValue(record.logoUrl) ??
      stringValue(record.titleLogo) ??
      stringValue(record.titleLogoUrl),
  };
}

export function formatNextEpisodeSubtitle(ep: Video): string {
  const season = ep.season;
  const epNum = ep.episode ?? ep.number;
  const name = ep.name ?? ep.title ?? '';
  if (typeof season === 'number' && typeof epNum === 'number') {
    return name ? `S${season}:E${epNum} ${name}` : `S${season}:E${epNum}`;
  }
  return name || 'Next Episode';
}

export function resolveNextEpisode(videos: Video[], current: Video | null | undefined, releasedOnly = false): Video | null {
  if (!videos.length || !current) return null;
  const currentSeason = current.season ?? 0;
  const currentEpisode = current.episode ?? current.number ?? 0;
  return [...videos]
    .filter((video) => {
      const season = video.season ?? 0;
      const episode = video.episode ?? video.number ?? 0;
      if (season < currentSeason || (season === currentSeason && episode <= currentEpisode)) return false;
      return !releasedOnly || isEpisodeReleasedForPlayback(video);
    })
    .sort(compareVideosByEpisode)[0] ?? null;
}

export function isEpisodeReleasedForPlayback(video: Video): boolean {
  if (!video.released) return true;
  const releasedAt = Date.parse(video.released);
  return Number.isNaN(releasedAt) || releasedAt <= Date.now();
}

function compareVideosByEpisode(a: Video, b: Video): number {
  const seasonA = a.season ?? 0;
  const seasonB = b.season ?? 0;
  if (seasonA !== seasonB) return seasonA - seasonB;
  return (a.episode ?? a.number ?? 0) - (b.episode ?? b.number ?? 0);
}

export function streamText(s: Stream): string {
  return [s.name, s.title, s.description, s.url, s.playableUrl, s.infoHash]
    .filter(Boolean).join(' ');
}

export function canPrefetchNextEpisode(prefs: Record<string, unknown>, currentStream: Stream): boolean {
  const tryBinge = prefBool(prefs, 'tryBingeGroup', false);
  const mode = prefString(prefs, 'streamSourceSelectionMode', 'manual');
  return (tryBinge && !!currentStream.behaviorHints?.bingeGroup) || mode !== 'manual';
}

export function selectNextEpisodeStream(
  streams: Stream[],
  currentStream: Stream,
  prefs: Record<string, unknown>,
): Stream | null {
  const tryBinge = prefBool(prefs, 'tryBingeGroup', false);
  const mode = prefString(prefs, 'streamSourceSelectionMode', 'manual');
  const regexPat = prefString(prefs, 'streamSourceRegexPattern', '');
  const curBinge = currentStream.behaviorHints?.bingeGroup;

  if (tryBinge && curBinge) {
    const match = streams.find((s) => s.behaviorHints?.bingeGroup === curBinge);
    if (match) return match;
  }

  if (mode === 'regex' && regexPat) {
    try {
      const re = new RegExp(regexPat, 'i');
      const match = streams.find((s) => re.test(streamText(s)));
      if (match) return match;
    } catch { /* invalid regex */ }
  }

  return streams[0] ?? null;
}
