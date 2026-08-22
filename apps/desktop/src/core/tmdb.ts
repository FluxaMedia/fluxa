import type { Meta, MetaLink } from './types';
import { coreTmdbCreditsUrlFromFind, coreTmdbPeopleImagesFromCredits, coreTmdbPeopleRequestPlan } from './engine';

export async function fetchTmdbPeopleImages({
  meta,
  links,
  apiKey,
  language,
}: {
  meta: Meta;
  links: MetaLink[];
  apiKey: string;
  language: string;
}): Promise<Record<string, string>> {
  const trimmedKey = apiKey.trim();
  if (!trimmedKey || links.length === 0) return {};

  const plan = await coreTmdbPeopleRequestPlan(meta, trimmedKey, language);
  let creditsUrl = plan?.creditsUrl ?? null;
  if (!creditsUrl && plan?.findUrl) {
    const found = await tryFetchJson(plan.findUrl);
    if (found) creditsUrl = await coreTmdbCreditsUrlFromFind(found, meta, trimmedKey, language);
  }
  if (!creditsUrl) return {};

  const credits = await tryFetchJson(creditsUrl);
  if (!credits) return {};
  return coreTmdbPeopleImagesFromCredits(credits, links);
}

async function tryFetchJson(url: string): Promise<unknown | null> {
  try {
    const response = await fetch(url);
    if (!response.ok) return null;
    return response.json();
  } catch {
    return null;
  }
}
