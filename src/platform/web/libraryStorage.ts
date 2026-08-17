type Json = unknown;
type Table = 'progress' | 'items' | 'watched' | 'lastWatched' | 'continueWatching';
type StatusName = 'watchlist' | 'completed' | 'dropped';
type ItemRow = { status: StatusName; value: Json; updatedAt: number };
type StampedRow = { value: Json; updatedAt: number };

function tableKey(profileKey: string, table: Table): string {
  return `${profileKey}:${table}`;
}

const migrated = new Set<string>();

function migrateLegacyBlob(profileKey: string): void {
  if (migrated.has(profileKey)) return;
  migrated.add(profileKey);
  try {
    if (localStorage.getItem(tableKey(profileKey, 'progress')) !== null) return;
    const raw = localStorage.getItem(profileKey);
    if (!raw) return;
    const blob = JSON.parse(raw) as {
      progress?: Record<string, Json>;
      statuses?: Record<string, Json[]>;
      watched?: Record<string, boolean>;
      lastWatchedEpisodes?: Record<string, Json>;
      externalContinueWatching?: Json[];
    };
    const now = Date.now();
    writeTable(profileKey, 'progress', blob.progress ?? {});
    writeTable(profileKey, 'watched', blob.watched ?? {});
    const items: Record<string, ItemRow> = {};
    for (const status of ['watchlist', 'completed', 'dropped'] as StatusName[]) {
      (blob.statuses?.[status] ?? []).forEach((value, index) => {
        const id = (value as { id?: string })?.id;
        if (id) items[id] = { status, value, updatedAt: now - index };
      });
    }
    writeTable(profileKey, 'items', items);
    writeTable(profileKey, 'lastWatched', Object.fromEntries(
      Object.entries(blob.lastWatchedEpisodes ?? {}).map(([id, value]) => [id, { value, updatedAt: now }]),
    ));
    writeTable(profileKey, 'continueWatching', Object.fromEntries(
      (blob.externalContinueWatching ?? []).map((value, index) => {
        const id = (value as { id?: string })?.id ?? String(index);
        return [id, { value, updatedAt: now - index }];
      }),
    ));
  } catch {}
}

function readTable<T>(profileKey: string, table: Table): Record<string, T> {
  migrateLegacyBlob(profileKey);
  try {
    const raw = localStorage.getItem(tableKey(profileKey, table));
    if (!raw) return {};
    const parsed = JSON.parse(raw) as unknown;
    return parsed && typeof parsed === 'object' && !Array.isArray(parsed) ? parsed as Record<string, T> : {};
  } catch {
    return {};
  }
}

function writeTable(profileKey: string, table: Table, rows: Record<string, unknown>): boolean {
  try {
    localStorage.setItem(tableKey(profileKey, table), JSON.stringify(rows));
    return true;
  } catch {
    return false;
  }
}

function byNewest(rows: [string, StampedRow][]): Json[] {
  return rows.sort((a, b) => (b[1].updatedAt ?? 0) - (a[1].updatedAt ?? 0)).map(([, row]) => row.value);
}

export function progressRead(profileKey: string, mediaId: string): string | null {
  const entry = readTable<Json>(profileKey, 'progress')[mediaId];
  return entry === undefined ? null : JSON.stringify(entry);
}

export function progressList(profileKey: string): string {
  return JSON.stringify(readTable<Json>(profileKey, 'progress'));
}

export function progressUpsert(profileKey: string, mediaId: string, progressJson: string): boolean {
  const rows = readTable<Json>(profileKey, 'progress');
  rows[mediaId] = JSON.parse(progressJson) as Json;
  return writeTable(profileKey, 'progress', rows);
}

export function progressUpsertMany(profileKey: string, updatesJson: string): boolean {
  const updates = JSON.parse(updatesJson) as { mediaId: string; value: Json }[];
  const rows = readTable<Json>(profileKey, 'progress');
  for (const update of updates) {
    if (update?.mediaId) rows[update.mediaId] = update.value;
  }
  return writeTable(profileKey, 'progress', rows);
}

export function progressDelete(profileKey: string, mediaId: string): boolean {
  const rows = readTable<Json>(profileKey, 'progress');
  delete rows[mediaId];
  return writeTable(profileKey, 'progress', rows);
}

export function statusSet(profileKey: string, mediaId: string, status: string | null, itemJson: string | null): boolean {
  const rows = readTable<ItemRow>(profileKey, 'items');
  if (status === null) {
    delete rows[mediaId];
    return writeTable(profileKey, 'items', rows);
  }
  if (!['watchlist', 'completed', 'dropped'].includes(status) || itemJson == null) return false;
  rows[mediaId] = { status: status as StatusName, value: JSON.parse(itemJson) as Json, updatedAt: Date.now() };
  return writeTable(profileKey, 'items', rows);
}

export function statusList(profileKey: string): string {
  const rows = Object.entries(readTable<ItemRow>(profileKey, 'items'));
  const lists: Record<StatusName, Json[]> = { watchlist: [], completed: [], dropped: [] };
  for (const status of Object.keys(lists) as StatusName[]) {
    lists[status] = byNewest(rows.filter(([, row]) => row.status === status));
  }
  return JSON.stringify(lists);
}

export function watchedSet(profileKey: string, videoId: string, watched: boolean): boolean {
  const rows = readTable<boolean>(profileKey, 'watched');
  if (watched) rows[videoId] = true;
  else delete rows[videoId];
  return writeTable(profileKey, 'watched', rows);
}

export function watchedList(profileKey: string): string {
  return JSON.stringify(readTable<boolean>(profileKey, 'watched'));
}

export function lastWatchedList(profileKey: string): string {
  const rows = readTable<StampedRow>(profileKey, 'lastWatched');
  return JSON.stringify(Object.fromEntries(Object.entries(rows).map(([id, row]) => [id, row.value])));
}

export function lastWatchedUpsert(profileKey: string, seriesId: string, entryJson: string): boolean {
  const rows = readTable<StampedRow>(profileKey, 'lastWatched');
  rows[seriesId] = { value: JSON.parse(entryJson) as Json, updatedAt: Date.now() };
  return writeTable(profileKey, 'lastWatched', rows);
}

export function lastWatchedDelete(profileKey: string, seriesId: string): boolean {
  const rows = readTable<StampedRow>(profileKey, 'lastWatched');
  delete rows[seriesId];
  return writeTable(profileKey, 'lastWatched', rows);
}

export function continueWatchingList(profileKey: string): string {
  return JSON.stringify(byNewest(Object.entries(readTable<StampedRow>(profileKey, 'continueWatching'))));
}

export function continueWatchingUpsert(profileKey: string, mediaId: string, itemJson: string): boolean {
  const rows = readTable<StampedRow>(profileKey, 'continueWatching');
  rows[mediaId] = { value: JSON.parse(itemJson) as Json, updatedAt: Date.now() };
  return writeTable(profileKey, 'continueWatching', rows);
}

export function continueWatchingDelete(profileKey: string, mediaId: string): boolean {
  const rows = readTable<StampedRow>(profileKey, 'continueWatching');
  delete rows[mediaId];
  return writeTable(profileKey, 'continueWatching', rows);
}

export function librarySnapshot(profileKey: string): string {
  const items = Object.entries(readTable<ItemRow>(profileKey, 'items'));
  const statuses: Record<StatusName, Json[]> = { watchlist: [], completed: [], dropped: [] };
  for (const status of Object.keys(statuses) as StatusName[]) {
    statuses[status] = byNewest(items.filter(([, row]) => row.status === status));
  }
  const lastWatched = readTable<StampedRow>(profileKey, 'lastWatched');
  return JSON.stringify({
    progress: readTable<Json>(profileKey, 'progress'),
    statuses,
    watched: readTable<boolean>(profileKey, 'watched'),
    lastWatchedEpisodes: Object.fromEntries(Object.entries(lastWatched).map(([id, row]) => [id, row.value])),
    externalContinueWatching: byNewest(Object.entries(readTable<StampedRow>(profileKey, 'continueWatching'))),
  });
}
