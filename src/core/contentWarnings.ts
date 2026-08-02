import { coreInvoke } from './engine';
import { tryFetchJson } from './httpClient';
import { t } from '../i18n';
import type { Meta, Video } from './types';

export interface ContentWarning {
  label: string;
  severity: string;
}

function contentWarningLabels() {
  return {
    nudity: t('content_warning.nudity'),
    violence: t('content_warning.violence'),
    profanity: t('content_warning.profanity'),
    alcohol: t('content_warning.alcohol'),
    frightening: t('content_warning.frightening'),
    severe: t('content_warning.severe'),
    moderate: t('content_warning.moderate'),
    mild: t('content_warning.mild'),
  };
}

async function resolveImdbId(candidates: Array<string | undefined>): Promise<string | null> {
  for (const candidate of candidates) {
    if (!candidate) continue;
    const imdbId = await coreInvoke<string | null>('contentImdbId', JSON.stringify({ id: candidate }));
    if (imdbId) return imdbId;
  }
  return null;
}

export async function fetchContentWarnings(meta?: Meta, episode?: Video | null): Promise<ContentWarning[]> {
  const imdbId = await resolveImdbId([meta?.id, episode?.id]);
  if (!imdbId) return [];
  try {
    const url = await coreInvoke<string>('contentWarningUrl', JSON.stringify({ imdbId }));
    if (!url) return [];
    const response = await tryFetchJson(url);
    if (!response) return [];
    const result = await coreInvoke<{ warnings: ContentWarning[] }>('buildContentWarnings', JSON.stringify({
      responseJson: JSON.stringify(response),
      labels: contentWarningLabels(),
    }));
    return result?.warnings ?? [];
  } catch {
    return [];
  }
}
