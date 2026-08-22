import type { HomeCategory } from './types';

const SEARCH_CACHE_MAX_ENTRIES = 24;
const searchResultsCache = new Map<string, HomeCategory[]>();

export function searchCacheKey(query: string, language: string, typeFilter: string): string {
  return `${language}::${typeFilter}::${query}`;
}

export function searchCacheGet(key: string): HomeCategory[] | null {
  const value = searchResultsCache.get(key);
  if (value === undefined) return null;
  searchResultsCache.delete(key);
  searchResultsCache.set(key, value);
  return value;
}

export function searchCacheSet(key: string, value: HomeCategory[]): void {
  searchResultsCache.delete(key);
  searchResultsCache.set(key, value);
  if (searchResultsCache.size > SEARCH_CACHE_MAX_ENTRIES) {
    const oldestKey = searchResultsCache.keys().next().value;
    if (oldestKey !== undefined) searchResultsCache.delete(oldestKey);
  }
}

export function searchCacheDelete(key: string): void {
  searchResultsCache.delete(key);
}

export function clearSearchResultsCache(): void {
  searchResultsCache.clear();
}
