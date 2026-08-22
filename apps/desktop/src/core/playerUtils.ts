import type { Stream, Video, Meta } from './types';
import { t } from '../i18n';

export type PlayerSubtitleSource = {
  url: string;
  label?: string;
  lang?: string;
  addonName?: string;
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
  mode?: 'direct' | 'torrent' | 'external' | 'reject';
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
    const prefix = `${t('format.season_abbrev')}${season}, ${t('format.episode_abbrev')}${episodeNumber}`;
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
    logo: stringValue(record.logo) ?? stringValue(record.logoUrl) ?? stringValue(record.titleLogo) ?? stringValue(record.titleLogoUrl),
  };
}

export function formatNextEpisodeSubtitle(ep: Video): string {
  const season = ep.season;
  const epNum = ep.episode ?? ep.number;
  const name = ep.name ?? ep.title ?? '';
  if (typeof season === 'number' && typeof epNum === 'number') {
    const code = `${t('format.season_abbrev')}${season}:${t('format.episode_abbrev')}${epNum}`;
    return name ? `${code} ${name}` : code;
  }
  return name || t('auto.next_episode');
}
