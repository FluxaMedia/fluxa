import { useEffect, useMemo, useRef, useState } from 'react';
import { coreInvoke } from '../core/engine';
import { prewarmYoutubeTrailerConfig } from '../core/effectRunner';
import { fetchContentLogo, fetchTmdbTrailers } from '../core/detailEffects';
import { fetchHeroDescription } from '../core/homeEffects';
import { getLanguage } from '../i18n';
import { prefString } from '../core/appPrefs';
import type { HomeState, HomeCategory, Meta, Trailer } from '../core/types';

export type HomeHeroPlan = {
  categories: HomeCategory[];
  billboard: Meta | null;
  slides: Meta[];
  trailerTargets: Meta[];
  logoTargets: Meta[];
  showHero: boolean;
  autoplayTrailer: boolean;
};

export function useHomeHeroData({
  home,
  prefs,
}: {
  home: HomeState;
  prefs: Record<string, unknown>;
}) {
  const [heroTrailers, setHeroTrailers] = useState<Record<string, Trailer[]>>({});
  const [fetchedHeroTrailerIds, setFetchedHeroTrailerIds] = useState<string[]>([]);
  const [heroLogos, setHeroLogos] = useState<Record<string, string>>({});
  const [fetchedHeroLogoIds, setFetchedHeroLogoIds] = useState<string[]>([]);
  const [heroDescriptions, setHeroDescriptions] = useState<Record<string, string>>({});
  const [homePlan, setHomePlan] = useState<HomeHeroPlan>({
    categories: [],
    billboard: null,
    slides: [],
    trailerTargets: [],
    logoTargets: [],
    showHero: true,
    autoplayTrailer: false,
  });

  useEffect(() => {
    let active = true;
    void coreInvoke<HomeHeroPlan>(
      'homeHeroPlan',
      JSON.stringify({
        categories: home.categories ?? [],
        billboard: home.billboard ?? null,
        prefs,
        fetchedTrailers: heroTrailers,
        fetchedIds: fetchedHeroTrailerIds,
        fetchedLogos: heroLogos,
        fetchedLogoIds: fetchedHeroLogoIds,
      }),
    ).then((plan) => {
      if (active && plan) setHomePlan(plan);
    });
    return () => {
      active = false;
    };
  }, [fetchedHeroLogoIds, fetchedHeroTrailerIds, heroLogos, heroTrailers, home.billboard, home.categories, prefs]);

  const resolvedHomePlan = useMemo(() => {
    const resolve = (item: Meta | null): Meta | null => {
      if (!item) return null;
      const trailers = heroTrailers[item.id];
      const logo = heroLogos[item.id];
      return trailers || logo ? { ...item, ...(trailers ? { trailers } : {}), ...(logo ? { logo } : {}) } : item;
    };
    return {
      ...homePlan,
      billboard: resolve(homePlan.billboard),
      slides: homePlan.slides.map((item) => resolve(item) ?? item),
    };
  }, [heroLogos, heroTrailers, homePlan]);

  useEffect(() => {
    if (!resolvedHomePlan.autoplayTrailer) return;
    prewarmYoutubeTrailerConfig().catch((error) => console.error('prewarmYoutubeTrailerConfig failed', error));
  }, [resolvedHomePlan.autoplayTrailer]);

  useEffect(() => {
    const apiKey = prefString(prefs, 'tmdbApiKey');
    const targets = resolvedHomePlan.trailerTargets;
    if (!targets.length) return;
    let cancelled = false;
    const language = getLanguage();
    Promise.all(
      targets.map(async (item) => {
        const trailers = (await fetchTmdbTrailers({ contentType: item.type, id: item.id, language, apiKey })) as Trailer[];
        return [item.id, trailers] as const;
      }),
    )
      .then((results) => {
        if (cancelled) return;
        setFetchedHeroTrailerIds((current) => Array.from(new Set([...current, ...targets.map((item) => item.id)])));
        const found = results.filter(([, trailers]) => trailers.length);
        if (found.length) setHeroTrailers((previous) => ({ ...previous, ...Object.fromEntries(found) }));
      })
      .catch((error) => console.error('hero trailer fetch failed', error));
    return () => {
      cancelled = true;
    };
  }, [prefs, resolvedHomePlan.trailerTargets]);

  useEffect(() => {
    const apiKey = prefString(prefs, 'tmdbApiKey');
    const fanartApiKey = prefString(prefs, 'fanartApiKey');
    const targets = resolvedHomePlan.logoTargets;
    if (!targets.length) return;
    let cancelled = false;
    const language = getLanguage();
    Promise.all(
      targets.map(async (item) => {
        const logo = await fetchContentLogo(item.id, item.type, language, apiKey, fanartApiKey).catch(() => undefined);
        return [item.id, logo ?? null] as const;
      }),
    )
      .then((results) => {
        if (cancelled) return;
        setFetchedHeroLogoIds((current) => Array.from(new Set([...current, ...targets.map((item) => item.id)])));
        const found = results.filter((entry): entry is [string, string] => !!entry[1]);
        if (found.length) setHeroLogos((previous) => ({ ...previous, ...Object.fromEntries(found) }));
      })
      .catch((error) => console.error('hero logo fetch failed', error));
    return () => {
      cancelled = true;
    };
  }, [prefs, resolvedHomePlan.logoTargets]);

  const heroDescriptionRequestedRef = useRef<Set<string>>(new Set());
  const billboard = resolvedHomePlan.billboard;
  const heroSlides = resolvedHomePlan.slides;
  const heroItemsSignature = `${billboard?.id ?? ''}|${heroSlides.map((slide) => slide.id).join(',')}`;
  const heroDescriptionTargets = useMemo(() => {
    const seen = new Set<string>();
    const targets: Meta[] = [];
    for (const item of [billboard, ...heroSlides]) {
      if (!item || seen.has(item.id) || item.description) continue;
      seen.add(item.id);
      if (!heroDescriptionRequestedRef.current.has(item.id)) targets.push(item);
    }
    return targets;
  }, [heroItemsSignature]);

  useEffect(() => {
    const targets = heroDescriptionTargets;
    if (!targets.length) return;
    targets.forEach((item) => heroDescriptionRequestedRef.current.add(item.id));
    let cancelled = false;
    Promise.all(
      targets.map(async (item) => [item.id, await fetchHeroDescription(item).catch(() => null)] as const),
    )
      .then((results) => {
        if (cancelled) return;
        const found = results.filter((entry): entry is [string, string] => !!entry[1]);
        if (found.length) setHeroDescriptions((previous) => ({ ...previous, ...Object.fromEntries(found) }));
      })
      .catch((error) => console.error('hero description fetch failed', error));
    return () => {
      cancelled = true;
    };
  }, [heroDescriptionTargets]);

  return {
    categories: resolvedHomePlan.categories,
    billboard: billboard && !billboard.description && heroDescriptions[billboard.id]
      ? { ...billboard, description: heroDescriptions[billboard.id] }
      : billboard,
    heroSlides: heroSlides.map((item) =>
      !item.description && heroDescriptions[item.id] ? { ...item, description: heroDescriptions[item.id] } : item,
    ),
    heroPendingLogoIds: new Set(
      resolvedHomePlan.logoTargets.map((item) => item.id).filter((id) => !fetchedHeroLogoIds.includes(id)),
    ),
    showHero: resolvedHomePlan.showHero,
    autoplayTrailerEnabled: resolvedHomePlan.autoplayTrailer,
  };
}
