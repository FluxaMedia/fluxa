import { coreNormalizeAddonSubtitles, coreFindPreferredSubtitleIndex, storageRead } from './engine';
import {
  coreResourceFetchPlan,
  coreResourceParsePlan,
  coreParseAddonResourceResult,
} from './addonManifest';
import type { Stream, Meta, Video, AddonDescriptor } from './types';
import { stringValue } from './playerUtils';
import type { PlayerSubtitleSource } from './playerUtils';

export async function resolvePlaybackSubtitles(
  stream: Stream,
  meta: Meta | undefined,
  episode: Video | null | undefined,
  subtitleExtraArgs: string | undefined,
  addons: AddonDescriptor[],
): Promise<PlayerSubtitleSource[]> {
  const subtitles: PlayerSubtitleSource[] = [];
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
  if (!contentType || !id) return subtitles;

  const plan = await coreResourceFetchPlan({
    kind: 'subtitles',
    addons,
    contentType,
    id,
    extraRaw: subtitleExtraArgs ?? '',
  });
  await Promise.all((plan?.requests ?? []).map(async (request) => {
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
      pushSubtitle(normalizeSubtitle(raw, String(request.addonName ?? 'Subtitle')));
    }
  }));

  const prefs = (await storageRead<Record<string, unknown>>('prefs')) ?? {};
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

  return subtitles;
}

async function tryFetchSubtitleResource(resourceUrl: string): Promise<unknown[]> {
  let statusCode = 0;
  let body: string | null = null;
  try {
    const res = await fetch(resourceUrl);
    statusCode = res.status;
    body = await res.text();
  } catch {
    statusCode = 0;
  }

  const result = await coreParseAddonResourceResult('subtitles', resourceUrl, statusCode, body);
  if (result.kind !== 'success') return [];
  try {
    const subtitles = JSON.parse(result.valueJson) as unknown;
    return Array.isArray(subtitles) ? subtitles : [];
  } catch {
    return [];
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
  return { url, lang, label };
}
