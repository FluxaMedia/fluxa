import { useEffect, useRef, useState } from 'react';
import { fetchMetaDetail } from '../../core/detailEffects';
import type { Video } from '../../core/types';
import { calendarArtworkKey, localDateKeyFromIso, type CalendarItem } from './calendarUtils';

const RESOLVE_LIMIT = 4;

export function useCalendarArtwork(items: CalendarItem[]) {
  const [resolvedArtwork, setResolvedArtwork] = useState<Record<string, string>>({});
  const [resolvedEpisodes, setResolvedEpisodes] = useState<Record<string, Partial<CalendarItem>>>({});
  const attemptedArtwork = useRef(new Set<string>());
  const [generation, setGeneration] = useState(0);

  useEffect(() => {
    let active = true;
    const unresolved = items
      .filter((item) => {
        const id = item.contentId ?? item.seriesId;
        const key = calendarArtworkKey(item);
        return id && !resolvedArtwork[key] && !item.episodePoster && !attemptedArtwork.current.has(key);
      })
      .slice(0, RESOLVE_LIMIT);
    if (!unresolved.length) return;
    for (const item of unresolved) attemptedArtwork.current.add(calendarArtworkKey(item));
    const itemsBySeries = new Map<string, CalendarItem[]>();
    for (const item of unresolved) {
      const id = item.contentId ?? item.seriesId ?? '';
      itemsBySeries.set(id, [...(itemsBySeries.get(id) ?? []), item]);
    }
    void Promise.all(
      [...itemsBySeries].map(async ([id, seriesItems]) => {
        let meta: { poster?: string; videos?: Video[] } | null = null;
        try {
          meta = (await fetchMetaDetail({ id, contentType: seriesItems[0]?.metaType === 'movie' ? 'movie' : 'series' })) as {
            poster?: string;
            videos?: Video[];
          } | null;
        } catch {
          return [];
        }
        return seriesItems.map((item) => {
          const season = item.seasonNumber ?? item.season;
          const episode = item.episodeNumber ?? item.episode ?? item.number;
          const matchedEpisode = meta?.videos?.find((video) =>
            season != null && episode != null
              ? video.season === season && (video.episode ?? video.number) === episode
              : !!item.dateIso && !!video.released && localDateKeyFromIso(video.released) === localDateKeyFromIso(item.dateIso),
          );
          const episodePoster = matchedEpisode?.thumbnail;
          return {
            key: calendarArtworkKey(item),
            artwork: episodePoster ?? meta?.poster,
            episode: matchedEpisode
              ? {
                  seasonNumber: matchedEpisode.season,
                  episodeNumber: matchedEpisode.episode ?? matchedEpisode.number,
                  episodeTitle: matchedEpisode.title ?? matchedEpisode.name,
                  episodePoster,
                }
              : null,
          };
        });
      }),
    ).then((entries) => {
      if (!active) return;
      const resolutions = entries.flat();
      const posters = resolutions.flatMap((entry) => (entry.artwork ? [[entry.key, entry.artwork] as const] : []));
      if (posters.length) setResolvedArtwork((current) => ({ ...current, ...Object.fromEntries(posters) }));
      const episodes = resolutions.flatMap((entry) => (entry.episode ? [[entry.key, entry.episode] as const] : []));
      if (episodes.length) setResolvedEpisodes((current) => ({ ...current, ...Object.fromEntries(episodes) }));
      setGeneration((value) => value + 1);
    });
    return () => {
      active = false;
    };
  }, [generation, items, resolvedArtwork]);

  return { resolvedArtwork, resolvedEpisodes };
}
