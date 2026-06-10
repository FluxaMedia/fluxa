import englishUsJson from './i18n/english_us.json';
import trTrJson from './i18n/tr_tr.json';

type LanguageMap = Record<string, string>;

const englishUs = englishUsJson as LanguageMap;
const trTr = trTrJson as LanguageMap;

const dictionaries: Record<string, LanguageMap> = {
  english_us: englishUs,
  en: englishUs,
  en_us: englishUs,
  tr: trTr,
  tr_tr: trTr,
};

let currentLanguage = 'english_us';

export function normalizeLanguage(language?: string | null): string {
  const normalized = language?.trim().toLowerCase().replace('-', '_') ?? '';
  if (!normalized || normalized === 'en' || normalized === 'en_us' || normalized === 'english_us') return 'english_us';
  if (normalized === 'tr' || normalized === 'tr_tr') return 'tr_tr';
  return normalized.endsWith('.json') ? normalized.slice(0, -5) : normalized;
}

export function setLanguage(language?: string | null): void {
  currentLanguage = normalizeLanguage(language);
}

export function getLanguage(): string {
  return currentLanguage;
}

export function t(key: string, ...args: Array<string | number | boolean | null | undefined>): string {
  const values = dictionaries[currentLanguage] ?? englishUs;
  let value = values[key] ?? englishUs[key] ?? key;
  for (const arg of args) {
    value = value.replace('%s', String(arg ?? ''));
  }
  return value;
}

export function list(key: string): string[] {
  return t(key)
    .split('|')
    .map((item) => item.trim())
    .filter(Boolean);
}
