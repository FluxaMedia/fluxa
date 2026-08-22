import { t } from '../../i18n';

export function timeAgo(ts: number): string {
  const s = Math.floor((Date.now() - ts) / 1000);
  if (s < 60) return `${s}s ago`;
  const m = Math.floor(s / 60);
  if (m < 60) return `${m} minute${m !== 1 ? 's' : ''} ago`;
  const h = Math.floor(m / 60);
  if (h < 24) return `${h} hour${h !== 1 ? 's' : ''} ago`;
  const d = Math.floor(h / 24);
  if (d < 7) return `${d} day${d !== 1 ? 's' : ''} ago`;
  return `${Math.floor(d / 7)} week${Math.floor(d / 7) !== 1 ? 's' : ''} ago`;
}

export function langOptions() {
  return [
    { value: 'none', label: t('settings.none') },
    { value: 'tr', label: t('language.turkish') },
    { value: 'en', label: t('language.english') },
    { value: 'ja', label: t('language.japanese') },
    { value: 'ko', label: t('language.korean') },
    { value: 'zh', label: t('language.chinese') },
    { value: 'de', label: t('language.german') },
    { value: 'fr', label: t('language.french') },
    { value: 'es', label: t('language.spanish') },
    { value: 'it', label: t('language.italian') },
    { value: 'pt', label: t('language.portuguese') },
    { value: 'ru', label: t('language.russian') },
    { value: 'ar', label: t('language.arabic') },
    { value: 'hi', label: t('language.hindi') },
  ];
}

export function subtitleFontOptions(customFontFamilies: string[] = []) {
  return [
    { value: 'default', label: t('settings.subtitle_font_default') },
    { value: 'Arial', label: 'Arial' },
    { value: 'Verdana', label: 'Verdana' },
    { value: 'Tahoma', label: 'Tahoma' },
    { value: 'Trebuchet MS', label: 'Trebuchet MS' },
    { value: 'Georgia', label: 'Georgia' },
    { value: 'Times New Roman', label: 'Times New Roman' },
    { value: 'Courier New', label: 'Courier New' },
    { value: 'Comic Sans MS', label: 'Comic Sans MS' },
    ...customFontFamilies.map((family) => ({ value: family, label: family })),
  ];
}

export function streamSourceOptions() {
  return [
    { value: 'first', label: t('settings.stream_source_first_available') },
    { value: 'manual', label: t('settings.stream_source_manual') },
    { value: 'regex', label: t('settings.stream_source_regex_short') },
  ];
}

export type ConnectedSourceState = Partial<Record<'nuvio' | 'trakt' | 'simkl' | 'anilist' | 'stremio', boolean>>;

function connectedSourceOptions(connected: ConnectedSourceState = {}) {
  const remote = [
    ...(connected.nuvio ? [{ value: 'nuvio', label: t('settings.cw_source_of_truth_nuvio') }] : []),
    ...(connected.trakt ? [{ value: 'trakt', label: t('settings.cw_source_of_truth_trakt') }] : []),
    ...(connected.simkl ? [{ value: 'simkl', label: t('settings.cw_source_of_truth_simkl') }] : []),
    ...(connected.anilist ? [{ value: 'anilist', label: t('settings.cw_source_of_truth_anilist') }] : []),
    ...(connected.stremio ? [{ value: 'stremio', label: t('settings.cw_source_of_truth_stremio') }] : []),
  ];
  return remote.length > 0 ? remote : [{ value: 'local', label: t('settings.cw_source_of_truth_local') }];
}

export function cwSourceOfTruthOptions(connected: ConnectedSourceState = {}) {
  return connectedSourceOptions(connected);
}

export function librarySourceOfTruthOptions(connected: ConnectedSourceState = {}) {
  return connectedSourceOptions(connected);
}

export function cwRankingOptions() {
  return [
    { value: 'last_watched', label: t('settings.cw_ranking_last_watched') },
    { value: 'most_recent_episode', label: t('settings.cw_ranking_most_recent_episode') },
  ];
}

export function similarTitlesSourceOptions() {
  return [
    { value: 'auto', label: t('settings.similar_titles_source_auto') },
    { value: 'trakt', label: t('settings.similar_titles_source_trakt') },
    { value: 'simkl', label: t('settings.similar_titles_source_simkl') },
    { value: 'tmdb', label: t('settings.similar_titles_source_tmdb') },
  ];
}

export function isFeedEnabled(selected: string[], key: string): boolean {
  return selected.length === 0 || selected.includes(key);
}
