import type { YoutubeTrailerSubtitleTrack } from './engine';
import { coreInvoke } from './engine';

export type TrailerCue = {
  start: number;
  end: number;
  text: string;
};

export async function selectTrailerSubtitle(
  tracks: YoutubeTrailerSubtitleTrack[],
  preferred?: string,
  secondary?: string,
): Promise<YoutubeTrailerSubtitleTrack | null> {
  return coreInvoke<YoutubeTrailerSubtitleTrack>(
    'trailerSubtitleSelectionPlan',
    JSON.stringify({
      tracks,
      preferred,
      secondary,
      systemLanguage: typeof navigator !== 'undefined' ? navigator.language : undefined,
    }),
  );
}

export async function normalizeTrailerSubtitleUrl(rawUrl: string): Promise<string> {
  return (await coreInvoke<string>('normalizeTrailerSubtitleUrl', JSON.stringify({ url: rawUrl }))) ?? rawUrl;
}

export async function parseTrailerSubtitleCues(input: string): Promise<TrailerCue[]> {
  return (await coreInvoke<TrailerCue[]>('parseTrailerSubtitleCues', JSON.stringify({ body: input }))) ?? [];
}
