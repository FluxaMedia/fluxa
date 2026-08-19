import { coreInvoke, storageRead, storageWrite } from './engine';
import type { Meta } from './types';

const RECENT_SEARCHES_KEY = 'recent_searches';

export interface RecentSearch {
  query: string;
  meta?: Meta;
}

export async function normalizeRecentSearches(value: unknown): Promise<RecentSearch[]> {
  return (await coreInvoke<RecentSearch[]>('recentSearchesPlan', JSON.stringify({ operation: 'normalize', items: value }))) ?? [];
}

export async function loadRecentSearches(): Promise<RecentSearch[]> {
  try {
    const items = await storageRead<unknown[]>(RECENT_SEARCHES_KEY);
    return await normalizeRecentSearches(items);
  } catch {
    return [];
  }
}

export async function addRecentSearch(query: string, current: RecentSearch[], meta?: Meta): Promise<RecentSearch[]> {
  const next =
    (await coreInvoke<RecentSearch[]>('recentSearchesPlan', JSON.stringify({ operation: 'add', items: current, query, meta }))) ?? current;
  void storageWrite(RECENT_SEARCHES_KEY, next);
  return next;
}

export async function removeRecentSearch(query: string, current: RecentSearch[]): Promise<RecentSearch[]> {
  const next =
    (await coreInvoke<RecentSearch[]>('recentSearchesPlan', JSON.stringify({ operation: 'remove', items: current, query }))) ?? current;
  void storageWrite(RECENT_SEARCHES_KEY, next);
  return next;
}

export async function clearRecentSearches(): Promise<RecentSearch[]> {
  const next = (await coreInvoke<RecentSearch[]>('recentSearchesPlan', JSON.stringify({ operation: 'clear', items: [] }))) ?? [];
  void storageWrite(RECENT_SEARCHES_KEY, next);
  return next;
}
