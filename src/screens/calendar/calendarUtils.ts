import { t } from '../../i18n';

export type CalendarItem = {
  id?: string;
  title?: string;
  name?: string;
  subtitle?: string;
  episodeTitle?: string;
  seasonNumber?: number;
  episodeNumber?: number;
  season?: number;
  episode?: number;
  number?: number;
  time?: string;
  airTime?: string;
  releaseTime?: string;
  dateIso?: string;
  poster?: string;
  seriesPoster?: string;
  episodePoster?: string;
  resolvedArtworkUrl?: string;
  contentId?: string;
  seriesId?: string;
  metaId?: string;
  metaType?: string;
};

export type CalendarCell = { day: number; dateIso: string; isCurrentMonth: boolean };

export function firstDayOfMonth(date: Date): Date {
  return new Date(date.getFullYear(), date.getMonth(), 1);
}

export function shiftMonth(date: Date, delta: number): Date {
  return new Date(date.getFullYear(), date.getMonth() + delta, 1);
}

export function monthTitle(date: Date): string {
  return date.toLocaleDateString(undefined, { month: 'long', year: 'numeric' });
}

export function weekdays(): string[] {
  return Array.from(
    { length: 7 },
    (_, index) => new Date(2024, 0, 1 + index).toLocaleDateString(undefined, { weekday: 'short' }),
  );
}

export function localDateKey(date: Date): string {
  return `${date.getFullYear()}-${String(date.getMonth() + 1).padStart(2, '0')}-${String(date.getDate()).padStart(2, '0')}`;
}

export function todayIso(): string {
  return localDateKey(new Date());
}

export function localDateKeyFromIso(dateIso: string): string {
  if (/^\d{4}-\d{2}-\d{2}$/.test(dateIso)) return dateIso;
  const date = new Date(dateIso);
  return Number.isNaN(date.getTime()) ? dateIso.slice(0, 10) : localDateKey(date);
}

export function formatLongDate(dateIso: string): string {
  const [year, month, day] = dateIso.split('-').map(Number);
  return new Date(year, month - 1, day).toLocaleDateString(undefined, {
    weekday: 'long',
    month: 'long',
    day: 'numeric',
    year: 'numeric',
  });
}

export function eventEpisodeLabel(item: CalendarItem): string {
  const episodeCode = eventEpisodeCode(item);
  const episodeTitle = item.episodeTitle?.trim();
  const subtitle = item.subtitle?.trim();
  const detail = episodeTitle || (subtitle && subtitle.toLowerCase() !== 'episode' ? subtitle : '');
  const time = item.airTime ?? item.releaseTime ?? item.time ?? '';
  return [episodeCode, detail, time].filter(Boolean).join(' • ') || t('calendar.episode');
}

export function eventEpisodeCode(item: CalendarItem): string {
  const season = item.seasonNumber ?? item.season;
  const episode = item.episodeNumber ?? item.episode ?? item.number;
  if (season != null && episode != null) return `S${season} • E${episode}`;
  if (season != null) return `S${season}`;
  if (episode != null) return `E${episode}`;
  const subtitleEpisode = item.subtitle?.match(/S\s*(\d+)\s*[:•-]?\s*E\s*(\d+)/i);
  if (subtitleEpisode) return `S${subtitleEpisode[1]} • E${subtitleEpisode[2]}`;
  return t('calendar.episode');
}

export function isReleased(item: CalendarItem): boolean {
  return !!item.dateIso && localDateKeyFromIso(item.dateIso) <= todayIso();
}

export function buildMonthCells(monthStart: Date): CalendarCell[] {
  const first = firstDayOfMonth(monthStart);
  const leading = (first.getDay() + 6) % 7;
  const last = new Date(first.getFullYear(), first.getMonth() + 1, 0);
  const trailing = (7 - ((leading + last.getDate()) % 7)) % 7;
  const start = new Date(first);
  start.setDate(first.getDate() - leading);
  return Array.from({ length: leading + last.getDate() + trailing }, (_, index) => {
    const date = new Date(start);
    date.setDate(start.getDate() + index);
    return { day: date.getDate(), dateIso: localDateKey(date), isCurrentMonth: date.getMonth() === monthStart.getMonth() };
  });
}

export function groupItemsByDate(items: CalendarItem[]): Record<string, CalendarItem[]> {
  const grouped = items.reduce<Record<string, CalendarItem[]>>((result, item) => {
    if (!item.dateIso) return result;
    const date = localDateKeyFromIso(item.dateIso);
    result[date] = [...(result[date] ?? []), item];
    return result;
  }, {});
  for (const date of Object.keys(grouped)) {
    const unique = new Map<string, CalendarItem>();
    for (const item of grouped[date]) {
      const key = calendarDisplayIdentity(item);
      const current = unique.get(key);
      if (!current || calendarDisplayDetailScore(item) > calendarDisplayDetailScore(current)) unique.set(key, item);
    }
    grouped[date] = [...unique.values()];
  }
  return grouped;
}

function calendarDisplayIdentity(item: CalendarItem): string {
  const subtitleEpisode = item.subtitle?.match(/S\s*(\d+)\s*[:•-]?\s*E\s*(\d+)/i);
  const season = item.seasonNumber ?? item.season ?? (subtitleEpisode ? Number(subtitleEpisode[1]) : 0);
  const episode = item.episodeNumber ?? item.episode ?? item.number ?? (subtitleEpisode ? Number(subtitleEpisode[2]) : 0);
  const title = item.title ?? item.name ?? item.contentId ?? item.seriesId ?? item.metaId ?? item.id ?? '';
  return `${title.trim().toLocaleLowerCase()}:${season}:${episode}`;
}

function calendarDisplayDetailScore(item: CalendarItem): number {
  const subtitleRepeatsEpisode = /^S\s*\d+\s*[:•-]?\s*E\s*\d+/i.test(item.subtitle ?? '');
  return [item.episodeTitle, item.episodePoster, item.poster, item.seriesPoster].filter(Boolean).length - (subtitleRepeatsEpisode ? 1 : 0);
}

export function calendarPoster(item: CalendarItem | undefined, resolved: Record<string, string>, seriesArtwork: Record<string, string> = {}): string | undefined {
  if (!item) return undefined;
  return [resolved[calendarArtworkKey(item)], item.resolvedArtworkUrl, item.episodePoster, item.poster, item.seriesPoster, seriesArtwork[calendarSeriesArtworkKey(item)]].find((value) => typeof value === 'string' && value.trim().length > 0);
}

export function calendarArtworkKey(item: CalendarItem): string {
  const id = item.contentId ?? item.seriesId ?? item.metaId ?? item.id ?? '';
  return `${id}:${item.dateIso ?? ''}`;
}

export function calendarSeriesArtworkKey(item: CalendarItem): string {
  return (item.title ?? item.name ?? item.contentId ?? item.seriesId ?? item.metaId ?? item.id ?? '').trim().toLocaleLowerCase();
}
