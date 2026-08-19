import { useCallback, useEffect, useState } from 'react';
import {
  coreNormalizeStreamBadgeRules,
  coreParseStreamBadgeImport,
  coreRemoveStreamBadgeSource,
  coreSetActiveStreamBadgeSource,
  coreUpsertStreamBadgeImport,
  storageRead,
  storageWrite,
} from '../core/engine';
import type { StreamBadgeRules } from '../core/types';

const STORAGE_KEY = 'stream_badge_rules';
const POSITION_STORAGE_KEY = 'stream_badge_position';
const EMPTY_RULES: StreamBadgeRules = { imports: [] };

export type StreamBadgePosition = 'top' | 'bottom';

export function useStreamBadgeRules() {
  const [rules, setRules] = useState<StreamBadgeRules>(EMPTY_RULES);
  const [loaded, setLoaded] = useState(false);
  const [importing, setImporting] = useState(false);
  const [importError, setImportError] = useState<string | null>(null);
  const [badgePosition, setBadgePositionState] = useState<StreamBadgePosition>('bottom');

  useEffect(() => {
    let cancelled = false;
    void (async () => {
      const stored = (await storageRead<StreamBadgeRules>(STORAGE_KEY).catch(() => null)) ?? EMPTY_RULES;
      const normalized = (await coreNormalizeStreamBadgeRules(stored)) ?? EMPTY_RULES;
      const storedPosition = await storageRead<StreamBadgePosition>(POSITION_STORAGE_KEY).catch(() => null);
      if (!cancelled) {
        setRules(normalized);
        if (storedPosition === 'top' || storedPosition === 'bottom') setBadgePositionState(storedPosition);
        setLoaded(true);
      }
    })();
    return () => {
      cancelled = true;
    };
  }, []);

  const setBadgePosition = useCallback(async (position: StreamBadgePosition) => {
    setBadgePositionState(position);
    await storageWrite(POSITION_STORAGE_KEY, position);
  }, []);

  const persist = useCallback(async (next: StreamBadgeRules) => {
    setRules(next);
    await storageWrite(STORAGE_KEY, next);
  }, []);

  const importFromUrl = useCallback(
    async (sourceUrl: string) => {
      setImporting(true);
      setImportError(null);
      try {
        const response = await fetch(sourceUrl);
        if (!response.ok) throw new Error(`HTTP ${response.status}`);
        const payload = await response.text();
        const parsed = await coreParseStreamBadgeImport(sourceUrl, payload);
        if (!parsed) throw new Error('empty response');
        const next = await coreUpsertStreamBadgeImport(rules, parsed, true);
        if (next) await persist(next);
      } catch (error) {
        setImportError(error instanceof Error ? error.message : String(error));
      } finally {
        setImporting(false);
      }
    },
    [rules, persist],
  );

  const setActiveSource = useCallback(
    async (sourceUrl: string) => {
      const next = await coreSetActiveStreamBadgeSource(rules, sourceUrl);
      if (next) await persist(next);
    },
    [rules, persist],
  );

  const removeSource = useCallback(
    async (sourceUrl: string) => {
      const next = await coreRemoveStreamBadgeSource(rules, sourceUrl);
      if (next) await persist(next);
    },
    [rules, persist],
  );

  return {
    rules,
    loaded,
    importing,
    importError,
    importFromUrl,
    setActiveSource,
    removeSource,
    badgePosition,
    setBadgePosition,
  };
}
