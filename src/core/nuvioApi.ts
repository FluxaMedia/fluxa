import { platformInvoke } from '../platform/invoke';
import { coreInvoke } from './engine';

export interface NuvioSession {
  access_token: string;
  refresh_token: string;
  expires_in: number;
  token_type: string;
  user: { id: string; email: string; created_at: string };
}

export interface NuvioProfile {
  id: string;
  user_id: string;
  profile_index: number;
  name: string;
  avatar_color_hex: string | null;
  uses_primary_addons: boolean;
  uses_primary_plugins: boolean;
  avatar_id: string | null;
  avatar_url: string | null;
  pin_enabled: boolean;
  pin_locked_until: string | null;
  created_at: string;
  updated_at: string;
}

export interface NuvioAddon {
  id: string;
  user_id: string;
  profile_id: number;
  url: string;
  name: string | null;
  enabled: boolean;
  sort_order: number;
  created_at: string;
  updated_at: string;
}

export interface NuvioPlugin {
  id?: string;
  user_id?: string;
  profile_id?: number;
  url: string;
  name?: string | null;
  enabled?: boolean;
  sort_order?: number;
}

export interface NuvioLibraryItem {
  id?: string;
  user_id?: string;
  profile_id?: number;
  content_id: string;
  content_type: string;
  name: string;
  poster: string | null;
  poster_shape: string;
  background: string | null;
  description: string | null;
  release_info: string | null;
  imdb_rating: number | null;
  genres: string[];
  addon_base_url: string | null;
  added_at: number;
}

export interface NuvioWatchProgress {
  id?: string;
  user_id?: string;
  profile_id?: number;
  content_id: string;
  content_type: string;
  video_id: string;
  season: number | null;
  episode: number | null;
  progress_key: string;
  position: number;
  duration: number;
  last_watched: number;
}

export interface NuvioLibraryDeltaEvent extends NuvioLibraryItem {
  event_id: number;
  operation: 'upsert' | 'delete';
}

export interface NuvioWatchedItem {
  id?: string;
  user_id?: string;
  profile_id?: number;
  content_id: string;
  content_type: string;
  title: string;
  season: number | null;
  episode: number | null;
  watched_at: number;
}

export interface NuvioWatchProgressDeltaEvent extends NuvioWatchProgress {
  event_id: number;
  operation: 'upsert' | 'delete';
}

export interface NuvioWatchedItemDeltaEvent extends NuvioWatchedItem {
  event_id: number;
  operation: 'upsert' | 'delete';
}

export interface NuvioAvatar {
  id: string;
  display_name: string;
  storage_path: string;
  category: string;
  sort_order: number;
  is_active: boolean;
  bg_color: string | null;
}

export interface NuvioCollectionRow {
  profile_id: number;
  collections_json: unknown[];
  updated_at: string;
}

const NUVIO_CLIENT_MAX_PROFILES = 6;

export class NuvioApiError extends Error {
  status?: number;
  code?: string;
  rawBody?: string;

  constructor(message: string, status?: number, code?: string, rawBody?: string) {
    super(message);
    this.name = 'NuvioApiError';
    this.status = status;
    this.code = code;
    this.rawBody = rawBody;
  }
}

function nuvioProgressKey(contentId: string, season?: number, episode?: number): string {
  return season != null && episode != null ? `${contentId}_s${season}e${episode}` : contentId;
}

function extractNuvioError(status: number, text: string): { message: string; code?: string } {
  if (!text) return { message: `Nuvio API ${status}` };
  try {
    const json = JSON.parse(text) as Record<string, unknown>;
    const message = [
      json.error_description,
      json.msg,
      json.message,
      json.error,
    ].find((value) => typeof value === 'string' && value.trim().length > 0);
    const code = [
      json.code,
      json.error_code,
      json.error,
    ].find((value) => typeof value === 'string' && value.trim().length > 0);
    if (message) return { message: String(message), code: code ? String(code) : undefined };
  } catch {}
  return { message: text };
}

export type NuvioAuthErrorKind =
  | 'invalid_credentials'
  | 'account_exists'
  | 'email_not_confirmed'
  | 'rate_limited'
  | 'server'
  | 'network'
  | 'unknown';

export function nuvioAuthErrorKind(error: unknown): NuvioAuthErrorKind {
  const status = error instanceof NuvioApiError ? error.status : undefined;
  const code = error instanceof NuvioApiError ? error.code ?? '' : '';
  const message = error instanceof Error ? error.message : String(error);
  const combined = `${code} ${message}`;

  if (/invalid login|invalid_grant|invalid credentials|wrong password|user not found/i.test(combined)) {
    return 'invalid_credentials';
  }
  if (/already registered|already exists|user_already_exists|email_exists/i.test(combined)) {
    return 'account_exists';
  }
  if (/email.*not.*confirm|confirm.*email|email_not_confirmed/i.test(combined)) {
    return 'email_not_confirmed';
  }
  if (status === 400 || status === 401) return 'invalid_credentials';
  if (status === 429 || /rate limit|too many requests/i.test(combined)) return 'rate_limited';
  if (status != null && status >= 500) return 'server';
  if (/failed to fetch|networkerror|load failed|connection|timed out|timeout|dns|error sending request/i.test(combined)) {
    return 'network';
  }
  return 'unknown';
}

async function rawNuvioRequest(
  method: 'GET' | 'POST' | 'PATCH' | 'DELETE',
  path: string,
  body?: unknown,
  token?: string,
): Promise<[number, string]> {
  return platformInvoke<[number, string]>('nuvio_request', {
    method,
    path,
    body: body !== undefined ? JSON.stringify(body) : null,
    token: token ?? null,
  });
}

async function nuvioRequest<T>(
  method: 'GET' | 'POST' | 'PATCH' | 'DELETE',
  path: string,
  body?: unknown,
  token?: string,
): Promise<T> {
  let status: number;
  let text: string;
  try {
    [status, text] = await rawNuvioRequest(method, path, body, token);
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    throw new NuvioApiError(message || 'Nuvio request failed');
  }
  if (status < 200 || status >= 300) {
    const parsed = extractNuvioError(status, text);
    throw new NuvioApiError(parsed.message, status, parsed.code, text);
  }
  return text ? (JSON.parse(text) as T) : (null as T);
}

async function post<T>(path: string, body: unknown, token?: string): Promise<T> {
  return nuvioRequest<T>('POST', path, body, token);
}

async function patch<T>(path: string, body: unknown, token: string): Promise<T> {
  return nuvioRequest<T>('PATCH', path, body, token);
}

async function get<T>(path: string, token: string): Promise<T> {
  return nuvioRequest<T>('GET', path, undefined, token);
}

export async function nuvioSignUp(email: string, password: string): Promise<NuvioSession> {
  return post<NuvioSession>('/auth/v1/signup', { email, password });
}

export async function nuvioSignIn(email: string, password: string): Promise<NuvioSession> {
  return post<NuvioSession>('/auth/v1/token?grant_type=password', { email, password });
}

export async function nuvioRefreshToken(refresh_token: string): Promise<NuvioSession> {
  return post<NuvioSession>('/auth/v1/token?grant_type=refresh_token', { refresh_token });
}

export async function nuvioSignOut(token: string): Promise<void> {
  await rawNuvioRequest('POST', '/auth/v1/logout', undefined, token);
}

export async function nuvioGetUser(token: string): Promise<{ id: string; email: string }> {
  return get(`/auth/v1/user`, token);
}

export async function nuvioPullProfiles(token: string): Promise<NuvioProfile[]> {
  const profiles = await post<NuvioProfile[]>('/rest/v1/rpc/sync_pull_profiles', {}, token);
  if (profiles.length > 0) return profiles;
  return get<NuvioProfile[]>('/rest/v1/profiles?select=*&order=profile_index', token);
}

export interface NuvioPinVerifyResult {
  unlocked: boolean;
  retry_after_seconds?: number;
  message?: string | null;
}

export async function nuvioVerifyPin(token: string, profileId: number, pin: string): Promise<NuvioPinVerifyResult> {
  return post<NuvioPinVerifyResult>('/rest/v1/rpc/verify_profile_pin', {
    p_profile_id: profileId,
    p_pin: pin,
  }, token);
}

export async function nuvioSetPin(token: string, profileId: number, pin: string, currentPin?: string): Promise<void> {
  await post('/rest/v1/rpc/set_profile_pin', {
    p_profile_id: profileId,
    p_pin: pin,
    ...(currentPin ? { p_current_pin: currentPin } : {}),
  }, token);
}

export async function nuvioClearPin(token: string, profileId: number, currentPin?: string): Promise<void> {
  await post('/rest/v1/rpc/clear_profile_pin', {
    p_profile_id: profileId,
    ...(currentPin ? { p_current_pin: currentPin } : {}),
  }, token);
}

export async function nuvioClearPinWithPassword(token: string, profileId: number, accountPassword: string): Promise<void> {
  await post('/rest/v1/rpc/clear_profile_pin_with_account_password', {
    p_profile_id: profileId,
    p_account_password: accountPassword,
  }, token);
}

export async function nuvioPullProfileLocks(token: string): Promise<Array<Pick<NuvioProfile, 'profile_index' | 'pin_enabled' | 'pin_locked_until'>>> {
  return post('/rest/v1/rpc/sync_pull_profile_locks', {}, token);
}

export async function nuvioPushProfiles(
  token: string,
  profiles: Array<{
    profile_index: number;
    name: string;
    avatar_color_hex?: string | null;
    uses_primary_addons?: boolean;
    uses_primary_plugins?: boolean;
    avatar_id?: string | null;
    avatar_url?: string | null;
  }>,
): Promise<void> {
  return post('/rest/v1/rpc/sync_push_profiles', {
    p_client_max_profiles: NUVIO_CLIENT_MAX_PROFILES,
    p_profiles: profiles,
  }, token);
}

export async function nuvioDeleteProfileData(token: string, profileId: number): Promise<void> {
  await post('/rest/v1/rpc/sync_delete_profile_data', { p_profile_id: profileId }, token);
}

export async function nuvioPullAddons(token: string, profileId: number): Promise<NuvioAddon[]> {
  return get<NuvioAddon[]>(
    `/rest/v1/addons?select=*&profile_id=eq.${profileId}&order=sort_order`,
    token,
  );
}

export async function nuvioPushAddons(
  token: string,
  profileId: number,
  addons: Array<{ url: string; name?: string; enabled?: boolean; sort_order?: number }>,
): Promise<void> {
  return post('/rest/v1/rpc/sync_push_addons', { p_profile_id: profileId, p_addons: addons }, token);
}

export async function nuvioReplaceAddons(
  token: string,
  userId: string,
  profileId: number,
  addons: Array<{ url: string; name?: string; enabled?: boolean; sort_order?: number }>,
): Promise<void> {
  const current = await nuvioPullAddons(token, profileId);
  const plan = await coreInvoke<{
    deleteIds: string[];
    updates: Array<{ id: string; payload: Record<string, unknown> }>;
    creates: Record<string, unknown>[];
  }>('nuvioAddonReconciliationPlan', JSON.stringify({ current, desired: addons, userId, profileId }));
  if (!plan) return;

  await Promise.all(plan.deleteIds.map((id) => nuvioRequest<void>(
      'DELETE',
      `/rest/v1/addons?id=eq.${encodeURIComponent(id)}&profile_id=eq.${profileId}`,
      undefined,
      token,
    )));
  await Promise.all(plan.updates.map(({ id, payload }) => patch<void>(
    `/rest/v1/addons?id=eq.${encodeURIComponent(id)}&profile_id=eq.${profileId}`,
    payload,
    token,
  )));
  await Promise.all(plan.creates.map((payload) => post<void>('/rest/v1/addons', payload, token)));
}

export async function nuvioPullPlugins(token: string, profileId: number): Promise<NuvioPlugin[]> {
  return get<NuvioPlugin[]>(
    `/rest/v1/plugins?select=*&profile_id=eq.${profileId}&order=sort_order`,
    token,
  );
}

export async function nuvioPushPlugins(
  token: string,
  profileId: number,
  plugins: Array<{ url: string; name?: string; enabled?: boolean; sort_order?: number }>,
): Promise<void> {
  await post('/rest/v1/rpc/sync_push_plugins', { p_profile_id: profileId, p_plugins: plugins }, token);
}

export async function nuvioPullLibrary(
  token: string,
  profileId: number,
  limit = 500,
  offset = 0,
): Promise<NuvioLibraryItem[]> {
  return post<NuvioLibraryItem[]>('/rest/v1/rpc/sync_pull_library', {
    p_profile_id: profileId,
    p_limit: limit,
    p_offset: offset,
  }, token);
}

export async function nuvioPullLibraryDelta(
  token: string,
  profileId: number,
  sinceEventId: number,
  limit = 1_000,
): Promise<NuvioLibraryDeltaEvent[]> {
  return post<NuvioLibraryDeltaEvent[]>('/rest/v1/rpc/sync_pull_library_delta', {
    p_profile_id: profileId,
    p_since_event_id: Math.max(0, Math.trunc(sinceEventId)),
    p_limit: Math.min(1_000, Math.max(1, Math.trunc(limit))),
  }, token);
}

export async function nuvioGetLibraryDeltaCursor(token: string, profileId: number): Promise<number> {
  return post<number>('/rest/v1/rpc/sync_get_library_delta_cursor', { p_profile_id: profileId }, token);
}

export async function nuvioPushLibrary(
  token: string,
  profileId: number,
  items: Array<{
    content_id: string;
    content_type: string;
    name?: string;
    poster?: string | null;
    poster_shape?: string;
    background?: string | null;
    description?: string | null;
    release_info?: string | null;
    imdb_rating?: number | null;
    genres?: string[];
    addon_base_url?: string | null;
    added_at?: number;
  }>,
): Promise<void> {
  return post('/rest/v1/rpc/sync_push_library', { p_profile_id: profileId, p_items: items }, token);
}

export async function nuvioPullWatchProgress(
  token: string,
  profileId: number,
  limit = 1_000,
  sinceLastWatched?: number,
): Promise<NuvioWatchProgress[]> {
  const body: Record<string, unknown> = {
    p_profile_id: profileId,
    p_limit: Math.min(1_000, Math.max(1, Math.trunc(limit))),
  };
  if (typeof sinceLastWatched === 'number' && Number.isFinite(sinceLastWatched) && sinceLastWatched > 0) {
    body.p_since_last_watched = Math.trunc(sinceLastWatched);
  }
  return post<NuvioWatchProgress[]>('/rest/v1/rpc/sync_pull_watch_progress', {
    ...body,
  }, token);
}

export async function nuvioPullWatchProgressDelta(
  token: string,
  profileId: number,
  sinceEventId: number,
  limit = 1_000,
): Promise<NuvioWatchProgressDeltaEvent[]> {
  return post<NuvioWatchProgressDeltaEvent[]>('/rest/v1/rpc/sync_pull_watch_progress_delta', {
    p_profile_id: profileId,
    p_since_event_id: Math.max(0, Math.trunc(sinceEventId)),
    p_limit: Math.min(1_000, Math.max(1, Math.trunc(limit))),
  }, token);
}

export async function nuvioGetWatchProgressDeltaCursor(token: string, profileId: number): Promise<number> {
  return post<number>('/rest/v1/rpc/sync_get_watch_progress_delta_cursor', { p_profile_id: profileId }, token);
}

export async function nuvioPushWatchProgress(
  token: string,
  profileId: number,
  entries: Array<{
    content_id: string;
    content_type: string;
    video_id: string;
    position: number;
    duration: number;
    last_watched: number;
    season?: number;
    episode?: number;
  }>,
): Promise<void> {
  return post('/rest/v1/rpc/sync_push_watch_progress', { p_profile_id: profileId, p_entries: entries }, token);
}

export async function nuvioDeleteWatchProgress(
  token: string,
  profileId: number,
  contentId: string,
  season?: number,
  episode?: number,
): Promise<void> {
  return post('/rest/v1/rpc/sync_delete_watch_progress', {
    p_profile_id: profileId,
    p_progress_key: nuvioProgressKey(contentId, season, episode),
  }, token);
}

export async function nuvioPullWatchHistory(
  token: string,
  profileId: number,
  pageSize = 500,
  page = 1,
): Promise<NuvioWatchedItem[]> {
  return post<NuvioWatchedItem[]>('/rest/v1/rpc/sync_pull_watched_items', {
    p_profile_id: profileId,
    p_page: page,
    p_page_size: pageSize,
  }, token);
}

export async function nuvioPullWatchHistoryDelta(
  token: string,
  profileId: number,
  sinceEventId: number,
  limit = 1_000,
): Promise<NuvioWatchedItemDeltaEvent[]> {
  return post<NuvioWatchedItemDeltaEvent[]>('/rest/v1/rpc/sync_pull_watched_items_delta', {
    p_profile_id: profileId,
    p_since_event_id: Math.max(0, Math.trunc(sinceEventId)),
    p_limit: Math.min(1_000, Math.max(1, Math.trunc(limit))),
  }, token);
}

export async function nuvioGetWatchHistoryDeltaCursor(token: string, profileId: number): Promise<number> {
  return post<number>('/rest/v1/rpc/sync_get_watched_items_delta_cursor', { p_profile_id: profileId }, token);
}

export async function nuvioPushWatchHistory(
  token: string,
  profileId: number,
  items: Array<{
    content_id: string;
    content_type: string;
    title?: string;
    season?: number;
    episode?: number;
    watched_at: number;
  }>,
): Promise<void> {
  return post('/rest/v1/rpc/sync_push_watched_items', { p_profile_id: profileId, p_items: items }, token);
}

export async function nuvioDeleteWatchHistory(
  token: string,
  profileId: number,
  keys: Array<{ content_id: string; season?: number; episode?: number }>,
): Promise<void> {
  await post('/rest/v1/rpc/sync_delete_watched_items', {
    p_profile_id: profileId,
    p_keys: keys,
  }, token);
}

export async function nuvioPullCollections(
  token: string,
  profileId: number,
): Promise<NuvioCollectionRow[]> {
  return post<NuvioCollectionRow[]>('/rest/v1/rpc/sync_pull_collections', {
    p_profile_id: profileId,
  }, token);
}

export async function nuvioPushCollections(
  token: string,
  profileId: number,
  collectionsJson: unknown[],
): Promise<void> {
  return post('/rest/v1/rpc/sync_push_collections', {
    p_profile_id: profileId,
    p_collections_json: collectionsJson,
  }, token);
}

export async function nuvioListAvatars(): Promise<NuvioAvatar[]> {
  try {
    return await nuvioRequest<NuvioAvatar[]>('POST', '/rest/v1/rpc/get_avatar_catalog', {});
  } catch {
    return [];
  }
}

export async function nuvioGetSyncOverview(token: string): Promise<unknown> {
  return post('/rest/v1/rpc/get_sync_overview', {}, token);
}

export async function nuvioHealthCheck(): Promise<{ status: string; database: string; latency_ms: number }> {
  const [, text] = await rawNuvioRequest('GET', '/functions/v1/health-check');
  return JSON.parse(text);
}
