import { OFFICIAL_FLUXA_SYNC_URL } from '../appConstants';

export interface FluxaSession {
  instanceUrl: string;
  accessToken: string;
  refreshToken: string;
  expiresAt: number;
  user: { id: string; email: string };
}

export interface FluxaProfile {
  id: string;
  name: string;
  avatar: string | null;
  settings: Record<string, unknown>;
  updated_at: string;
}

export type FluxaEntityType = 'library' | 'watch_progress' | 'watched_history' | 'collections' | 'addons' | 'plugins' | 'settings';

export interface FluxaChange {
  entity_type: FluxaEntityType;
  key: string;
  payload: unknown;
  deleted?: boolean;
  expected_revision?: number;
}

export interface FluxaDocument {
  entity_type: FluxaEntityType;
  key: string;
  payload: unknown;
  deleted: boolean;
  revision: number;
}

export interface FluxaPullResult {
  cursor: number;
  minimumAvailableRevision: number | null;
  resetRequired: boolean;
  changes: FluxaDocument[];
}

export interface FluxaPushResult {
  cursor: number;
  applied: Array<{ entity_type: FluxaEntityType; key: string; revision: number; deleted: boolean }>;
  conflicts: Array<{ entity_type: FluxaEntityType; key: string; expected_revision: number; actual_revision: number }>;
}

interface SessionResponse {
  access_token: string | null;
  refresh_token: string | null;
  expires_in: number | null;
  user: { id: string; email: string } | null;
}

interface RawDocument {
  entity_type?: FluxaEntityType;
  document_type?: FluxaEntityType;
  document_key: string;
  payload: unknown;
  deleted: boolean;
  revision: number;
}

export class FluxaApiError extends Error {
  status?: number;
  rawBody?: string;

  constructor(message: string, status?: number, rawBody?: string) {
    super(message);
    this.name = 'FluxaApiError';
    this.status = status;
    this.rawBody = rawBody;
  }
}

export type FluxaAuthErrorKind =
  'invalid_credentials' | 'account_exists' | 'email_not_confirmed' | 'rate_limited' | 'no_instance' | 'unreachable' | 'server' | 'unknown';

export function fluxaAuthErrorKind(error: unknown): FluxaAuthErrorKind {
  const status = error instanceof FluxaApiError ? error.status : undefined;
  const message = error instanceof Error ? error.message : String(error);

  if (/no instance configured/i.test(message)) return 'no_instance';
  if (/invalid login|invalid credentials|unauthorized/i.test(message)) return 'invalid_credentials';
  if (/already registered|already exists|duplicate key/i.test(message)) return 'account_exists';
  if (/email.*not.*confirm|confirm.*email/i.test(message)) return 'email_not_confirmed';
  if (status === 401) return 'invalid_credentials';
  if (status === 429 || /rate limit|too many requests/i.test(message)) return 'rate_limited';
  if (status != null && status >= 500) return 'server';
  if (/failed to fetch|networkerror|load failed|connection|timed out|timeout|dns/i.test(message)) return 'unreachable';
  return 'unknown';
}

export function resolveInstanceBase(input: string): string {
  const trimmed = input.trim().replace(/\/+$/, '');
  if (!trimmed) {
    if (!OFFICIAL_FLUXA_SYNC_URL) throw new FluxaApiError('no instance configured');
    return OFFICIAL_FLUXA_SYNC_URL.replace(/\/+$/, '');
  }
  const absolute = /^https?:\/\//.test(trimmed) ? trimmed : `https://${trimmed}`;
  let url: URL;
  try {
    url = new URL(absolute);
  } catch {
    throw new FluxaApiError('instance address is not a valid URL');
  }
  if (url.pathname.endsWith('/api/v1') || url.pathname.includes('/functions/v1/')) return absolute;
  if (url.hostname.endsWith('.supabase.co')) return `${absolute}/functions/v1/fluxa-sync`;
  return `${absolute}/api/v1`;
}

async function request<T>(
  base: string,
  method: 'GET' | 'POST' | 'PATCH' | 'DELETE',
  path: string,
  body?: unknown,
  token?: string,
): Promise<T> {
  const headers: Record<string, string> = {};
  if (body !== undefined) headers['content-type'] = 'application/json';
  if (token) headers.authorization = `Bearer ${token}`;
  let response: Response;
  try {
    response = await fetch(`${base}${path}`, {
      method,
      headers,
      body: body === undefined ? undefined : JSON.stringify(body),
    });
  } catch (error) {
    throw new FluxaApiError(error instanceof Error ? error.message : 'request failed');
  }
  const text = await response.text();
  if (!response.ok) {
    let message = text || `Fluxa Sync ${response.status}`;
    try {
      const parsed = JSON.parse(text) as { error?: string; message?: string };
      message = parsed.error ?? parsed.message ?? message;
    } catch {}
    throw new FluxaApiError(message, response.status, text);
  }
  return text ? (JSON.parse(text) as T) : (null as T);
}

function toSession(instanceUrl: string, payload: SessionResponse): FluxaSession {
  if (!payload?.access_token || !payload.refresh_token) {
    throw new FluxaApiError('confirm your email address before signing in');
  }
  if (!payload.user) throw new FluxaApiError('instance returned no account');
  const lifetime = typeof payload.expires_in === 'number' && payload.expires_in > 0 ? payload.expires_in : 3600;
  return {
    instanceUrl,
    accessToken: payload.access_token,
    refreshToken: payload.refresh_token,
    expiresAt: Date.now() + lifetime * 1000,
    user: { id: payload.user.id, email: payload.user.email },
  };
}

function toDocument(raw: RawDocument): FluxaDocument {
  return {
    entity_type: (raw.entity_type ?? raw.document_type) as FluxaEntityType,
    key: raw.document_key,
    payload: raw.payload,
    deleted: raw.deleted === true,
    revision: raw.revision,
  };
}

export async function fluxaSignUp(instanceUrl: string, email: string, password: string): Promise<FluxaSession> {
  const base = resolveInstanceBase(instanceUrl);
  return toSession(base, await request<SessionResponse>(base, 'POST', '/auth/register', { email, password }));
}

export async function fluxaSignIn(instanceUrl: string, email: string, password: string): Promise<FluxaSession> {
  const base = resolveInstanceBase(instanceUrl);
  return toSession(base, await request<SessionResponse>(base, 'POST', '/auth/login', { email, password }));
}

export async function fluxaRefresh(base: string, refreshToken: string): Promise<FluxaSession> {
  return toSession(base, await request<SessionResponse>(base, 'POST', '/auth/refresh', { refresh_token: refreshToken }));
}

export async function fluxaSignOut(base: string, token: string, refreshToken: string): Promise<void> {
  await request(base, 'POST', '/auth/logout', { refresh_token: refreshToken }, token);
}

export async function fluxaProfiles(base: string, token: string): Promise<FluxaProfile[]> {
  return (await request<FluxaProfile[]>(base, 'GET', '/profiles', undefined, token)) ?? [];
}

export async function fluxaCreateProfile(
  base: string,
  token: string,
  profile: { name: string; avatar?: string | null; settings?: Record<string, unknown> },
): Promise<FluxaProfile> {
  return request<FluxaProfile>(base, 'POST', '/profiles', profile, token);
}

export async function fluxaUpdateProfile(
  base: string,
  token: string,
  profileId: string,
  profile: { name?: string; avatar?: string | null; settings?: Record<string, unknown> },
): Promise<FluxaProfile> {
  return request<FluxaProfile>(base, 'PATCH', `/profiles/${profileId}`, profile, token);
}

export async function fluxaDeleteProfile(base: string, token: string, profileId: string): Promise<void> {
  await request(base, 'DELETE', `/profiles/${profileId}`, undefined, token);
}

export async function fluxaSnapshot(base: string, token: string, profileId: string): Promise<FluxaPullResult> {
  const body = await request<{ cursor: number; documents: RawDocument[] }>(
    base,
    'GET',
    `/sync/snapshot?profile_id=${encodeURIComponent(profileId)}`,
    undefined,
    token,
  );
  return {
    cursor: body.cursor,
    minimumAvailableRevision: null,
    resetRequired: false,
    changes: (body.documents ?? []).map(toDocument),
  };
}

export async function fluxaPull(base: string, token: string, profileId: string, since: number): Promise<FluxaPullResult> {
  const body = await request<{
    cursor: number;
    minimum_available_revision: number | null;
    reset_required: boolean;
    changes: RawDocument[];
  }>(base, 'GET', `/sync/pull?profile_id=${encodeURIComponent(profileId)}&since=${since}`, undefined, token);
  return {
    cursor: body.cursor,
    minimumAvailableRevision: body.minimum_available_revision,
    resetRequired: body.reset_required === true,
    changes: (body.changes ?? []).map(toDocument),
  };
}

export async function fluxaPush(base: string, token: string, profileId: string, changes: FluxaChange[]): Promise<FluxaPushResult> {
  return request<FluxaPushResult>(base, 'POST', '/sync/push', { profile_id: profileId, changes }, token);
}
