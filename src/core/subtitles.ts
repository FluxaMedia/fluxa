import { coreNormalizeAddonSubtitles, coreFindPreferredSubtitleIndex, coreSubtitleLanguageDedupKeepIndices } from './engine';
import { invoke } from '@tauri-apps/api/core';
import {
  coreResourceFetchPlan,
  coreResourceParsePlan,
  coreParseAddonResourceResult,
} from './addonManifest';
import { loadPrefs } from './libraryOps';
import type { Stream, Meta, Video, AddonDescriptor } from './types';
import { stringValue } from './playerUtils';
import type { PlayerSubtitleSource } from './playerUtils';

export type ResolvedSubtitles = {
  subtitles: PlayerSubtitleSource[];
  failedAddons: string[];
};
function subtitleTrace(message: string): void {
  void invoke('debug_log', { msg: `subtitles: ${message}` }).catch(() => undefined);
}


export async function resolvePlaybackSubtitles(
  stream: Stream,
  meta: Meta | undefined,
  episode: Video | null | undefined,
  subtitleExtraArgs: string | undefined,
  addons: AddonDescriptor[],
): Promise<ResolvedSubtitles> {
  const subtitles: PlayerSubtitleSource[] = [];
  const failedAddons: string[] = [];
  const seen = new Set<string>();

  const pushSubtitle = (subtitle: PlayerSubtitleSource | null | undefined) => {
    if (!subtitle?.url) return;
    const key = subtitle.url.trim();
    if (!key || seen.has(key)) return;
    seen.add(key);
    subtitles.push(subtitle);
  };

  for (const subtitle of stream.subtitles ?? []) {
    pushSubtitle({
      url: subtitle.url,
      label: subtitle.label ?? subtitle.lang ?? 'Stream subtitle',
      lang: subtitle.lang,
    });
  }

  const contentType = meta?.type;
  const id = episode?.id ?? meta?.id;
  if (!contentType || !id) return { subtitles, failedAddons };

  const plan = await coreResourceFetchPlan({
    kind: 'subtitles',
    addons,
    contentType,
    id,
    extraRaw: subtitleExtraArgs ?? '',
  });
  subtitleTrace(`plan contentType=${contentType}, id=${id}, addons=${addons.length}, requests=${plan?.requests?.length ?? 0}`);
  await Promise.all((plan?.requests ?? []).map(async (request) => {
    const addonName = String(request.addonName ?? 'Subtitle');
    try {
      const resourceUrl = typeof request.url === 'string' ? request.url : '';
      if (!resourceUrl) return;
      const data = await tryFetchSubtitleResource(resourceUrl);
      const parsed = await coreResourceParsePlan({
        kind: 'subtitles',
        response: { subtitles: data },
        addonName: request.addonName,
      });
      const rawSubtitles = ((parsed?.subtitles as unknown[] | undefined) ?? data);
      const normalized = await coreNormalizeAddonSubtitles(rawSubtitles, resourceUrl);
      for (const raw of normalized) {
        pushSubtitle(normalizeSubtitle(raw, addonName));
      }
    } catch {
      failedAddons.push(addonName);
    }
  }));

  const prefs = await loadPrefs();
  const preferredIndex = await coreFindPreferredSubtitleIndex(
    subtitles.map((subtitle, index) => ({
      id: subtitle.url || String(index),
      label: subtitle.label ?? subtitle.lang ?? '',
      language: subtitle.lang ?? null,
    })),
    null,
    stringValue(prefs.preferredSubtitleLanguage),
    stringValue(prefs.secondarySubtitleLanguage),
  ).catch(() => -1);

  if (preferredIndex > 0 && preferredIndex < subtitles.length) {
    const [preferred] = subtitles.splice(preferredIndex, 1);
    subtitles.unshift(preferred);
  }

  const keepIndices = new Set(
    await coreSubtitleLanguageDedupKeepIndices(subtitles.map((subtitle) => subtitle.lang)),
  );
  const limitedSubtitles = subtitles.filter((_, index) => keepIndices.has(index));
  return { subtitles: limitedSubtitles, failedAddons };
}

async function tryFetchSubtitleResource(resourceUrl: string): Promise<unknown[]> {
  let statusCode: number;
  let body: string;
  try {
    const response = await fetch(resourceUrl);
    statusCode = response.status;
    body = await response.text();
  } catch (error) {
    console.warn('[subtitles] addon request failed', resourceUrl, error);
    throw error;
  }

  const result = await coreParseAddonResourceResult('subtitles', resourceUrl, statusCode, body);
  if (result.kind !== 'success') {
    console.warn('[subtitles] addon response rejected', resourceUrl, result);
    throw new Error(result.error ?? `subtitle addon returned ${statusCode}`);
  }
  try {
    const subtitles = JSON.parse(result.valueJson) as unknown;
    const entries = Array.isArray(subtitles)
      ? subtitles
      : subtitles && typeof subtitles === 'object' && Array.isArray((subtitles as { subtitles?: unknown[] }).subtitles)
        ? (subtitles as { subtitles: unknown[] }).subtitles
        : [];
    console.info('[subtitles] addon results', resourceUrl, entries.length);
    return entries;
  } catch (error) {
    console.warn('[subtitles] addon response JSON failed', resourceUrl, error);
    throw error;
  }
}

function normalizeSubtitle(raw: unknown, addonName: string): PlayerSubtitleSource | null {
  if (!raw || typeof raw !== 'object') return null;
  const object = raw as {
    url?: unknown;
    lang?: unknown;
    label?: unknown;
    name?: unknown;
    attributes?: { url?: unknown; language?: unknown; lang?: unknown; name?: unknown };
  };
  const url = stringValue(object.url) ?? stringValue(object.attributes?.url) ?? null;
  if (!url) return null;
  const lang =
    stringValue(object.lang) ??
    stringValue(object.attributes?.language) ??
    stringValue(object.attributes?.lang);
  const fallbackLabel = lang ?? (addonName || 'Subtitle');
  const label =
    stringValue(object.label) ??
    stringValue(object.name) ??
    stringValue(object.attributes?.name) ??
    fallbackLabel;
  return { url, lang, label, addonName: addonName || undefined };
}
