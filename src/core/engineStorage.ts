import { invoke } from "@tauri-apps/api/core";

export async function storageRead<T>(key: string): Promise<T | null> {
  const raw = await invoke<string | null>("storage_read", { key });
  if (!raw) return null;
  try {
    return JSON.parse(raw) as T;
  } catch {
    return null;
  }
}

export async function storageWrite(
  key: string,
  value: unknown,
): Promise<boolean> {
  return invoke<boolean>("storage_write", {
    key,
    value: JSON.stringify(value),
  });
}

export async function storageDelete(key: string): Promise<boolean> {
  return invoke<boolean>("storage_delete", { key });
}

export async function libraryProgressRead<T>(
  profileKey: string,
  mediaId: string,
): Promise<T | null> {
  const raw = await invoke<string | null>("library_progress_read", {
    profileKey,
    mediaId,
  });
  if (!raw) return null;
  try {
    return JSON.parse(raw) as T;
  } catch {
    return null;
  }
}

export async function libraryProgressList<T>(
  profileKey: string,
): Promise<Record<string, T>> {
  const raw = await invoke<string | null>("library_progress_list", {
    profileKey,
  });
  if (!raw) return {};
  try {
    const value = JSON.parse(raw) as unknown;
    return value && typeof value === "object" && !Array.isArray(value)
      ? value as Record<string, T>
      : {};
  } catch {
    return {};
  }
}

export async function libraryProgressUpsert(
  profileKey: string,
  mediaId: string,
  progress: unknown,
): Promise<boolean> {
  return invoke<boolean>("library_progress_upsert", {
    profileKey,
    mediaId,
    progressJson: JSON.stringify(progress),
  });
}

export async function libraryProgressUpsertMany(
  profileKey: string,
  updates: Array<{ mediaId: string; value: unknown }>,
): Promise<boolean> {
  return invoke<boolean>("library_progress_upsert_many", {
    profileKey,
    updatesJson: JSON.stringify(updates),
  });
}

export async function libraryProgressDelete(
  profileKey: string,
  mediaId: string,
): Promise<boolean> {
  return invoke<boolean>("library_progress_delete", { profileKey, mediaId });
}

export async function librarySnapshot<T>(profileKey: string): Promise<T | null> {
  const raw = await invoke<string | null>("library_snapshot", { profileKey });
  if (!raw) return null;
  try {
    return JSON.parse(raw) as T;
  } catch {
    return null;
  }
}

export async function libraryStatusSet(
  profileKey: string,
  mediaId: string,
  status: "watchlist" | "completed" | "dropped" | null,
  item?: unknown,
): Promise<boolean> {
  return invoke<boolean>("library_status_set", {
    profileKey,
    mediaId,
    status,
    itemJson: item === undefined ? null : JSON.stringify(item),
  });
}

export async function libraryStatusList(
  profileKey: string,
): Promise<Record<string, unknown[]>> {
  const raw = await invoke<string | null>("library_status_list", {
    profileKey,
  });
  try {
    return raw ? JSON.parse(raw) as Record<string, unknown[]> : {};
  } catch {
    return {};
  }
}

export async function libraryWatchedSet(
  profileKey: string,
  videoId: string,
  watched: boolean,
): Promise<boolean> {
  return invoke<boolean>("library_watched_set", {
    profileKey,
    videoId,
    watched,
  });
}

export async function libraryWatchedList(
  profileKey: string,
): Promise<Record<string, boolean>> {
  const raw = await invoke<string | null>("library_watched_list", {
    profileKey,
  });
  try {
    return raw ? JSON.parse(raw) as Record<string, boolean> : {};
  } catch {
    return {};
  }
}

export async function libraryLastWatchedList<T>(
  profileKey: string,
): Promise<Record<string, T>> {
  const raw = await invoke<string | null>("library_last_watched_list", {
    profileKey,
  });
  if (!raw) return {};
  try {
    const value = JSON.parse(raw) as unknown;
    return value && typeof value === "object" && !Array.isArray(value)
      ? value as Record<string, T>
      : {};
  } catch {
    return {};
  }
}

export async function libraryLastWatchedUpsert(
  profileKey: string,
  seriesId: string,
  entry: unknown,
): Promise<boolean> {
  return invoke<boolean>("library_last_watched_upsert", {
    profileKey,
    seriesId,
    entryJson: JSON.stringify(entry),
  });
}

export async function libraryLastWatchedDelete(
  profileKey: string,
  seriesId: string,
): Promise<boolean> {
  return invoke<boolean>("library_last_watched_delete", {
    profileKey,
    seriesId,
  });
}

export async function libraryContinueWatchingList(
  profileKey: string,
): Promise<unknown[]> {
  const raw = await invoke<string | null>("library_continue_watching_list", {
    profileKey,
  });
  if (!raw) return [];
  try {
    const value = JSON.parse(raw) as unknown;
    return Array.isArray(value) ? value : [];
  } catch {
    return [];
  }
}

export async function libraryContinueWatchingUpsert(
  profileKey: string,
  mediaId: string,
  item: unknown,
): Promise<boolean> {
  return invoke<boolean>("library_continue_watching_upsert", {
    profileKey,
    mediaId,
    itemJson: JSON.stringify(item),
  });
}

export async function libraryContinueWatchingDelete(
  profileKey: string,
  mediaId: string,
): Promise<boolean> {
  return invoke<boolean>("library_continue_watching_delete", {
    profileKey,
    mediaId,
  });
}

export async function enqueueOfflineDownload(
  request: unknown,
): Promise<unknown | null> {
  const raw = await invoke<string | null>("enqueue_offline_download", {
    requestJson: JSON.stringify(request),
  });
  return raw ? JSON.parse(raw) : null;
}

export async function streamMagnetLink(
  stream: unknown,
): Promise<string | null> {
  return invoke<string | null>("stream_magnet_link", {
    streamJson: JSON.stringify(stream),
  });
}
