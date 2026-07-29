import React, { useEffect, useRef, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { open as shellOpen } from '@tauri-apps/plugin-shell';
import { storageRead, storageWrite } from '../../core/engine';
import type { UserProfile } from '../../core/types';
import { t } from '../../i18n';
import { ConfirmDialog } from '../ConfirmDialog';
import { ImportDialog } from '../ImportDialog';
import { PROVIDER_IMPORT_CATEGORIES, type ImportCategory } from '../../core/importCategories';
import { pushImportedCategoriesToDestination } from '../../core/crossProviderPush';
import { profileConnectionState, profileColor, saveProfile } from '../../core/profiles';
import { AvatarPreview } from '../../screens/ProfileForm';
import { syncExternalIntegrationNow } from '../../core/effectRunner';
import { refreshAnimeTrackingProfile } from '../../core/animeExternalSync';
import { ChoiceTile, SettingsDetailHeader, SettingsSection, SyncServicePopover, SyncServiceRow, ToggleTile, cwRankingOptions, cwSourceOfTruthOptions, similarTitlesSourceOptions } from './SettingsUI';
import type { Prefs, SyncMeta, TraktTokenResponse } from './settingsTypes';
import { nuvioAuthErrorKind, nuvioSignIn } from '../../core/nuvioApi';
import { refreshNuvioProfiles } from '../../core/nuvioSync';
import { stremioLogin, stremioLoginWithAuthKey, stremioLogout } from '../../core/stremioApi';

function generateCodeVerifier(): string {
  const array = new Uint8Array(48);
  crypto.getRandomValues(array);
  return btoa(String.fromCharCode(...array))
    .replace(/\+/g, '-')
    .replace(/\//g, '_')
    .replace(/=/g, '')
    .slice(0, 64);
}

async function codeChallenge(verifier: string): Promise<string> {
  const digest = await crypto.subtle.digest('SHA-256', new TextEncoder().encode(verifier));
  return btoa(String.fromCharCode(...new Uint8Array(digest)))
    .replace(/\+/g, '-')
    .replace(/\//g, '_')
    .replace(/=/g, '');
}

interface OAuthCodePayload {
  code: string;
  state: string | null;
}

type OAuthService = 'trakt' | 'anilist' | 'simkl';
type IntegrationService = OAuthService | 'nuvio' | 'stremio';

function tokenRefreshCountdown(seconds: number): string {
  const total = Math.max(0, Math.floor(seconds));
  const hours = Math.floor(total / 3600);
  const minutes = Math.floor((total % 3600) / 60);
  const remainingSeconds = total % 60;
  return hours > 0
    ? t('format.duration_hours_minutes_seconds', hours, minutes, remainingSeconds)
    : t('format.duration_minutes_seconds', minutes, remainingSeconds);
}

function ProviderTokenStatus({ expiresAt, verified, refreshScheduled }: { expiresAt?: number; verified: boolean; refreshScheduled: boolean }) {
  const [now, setNow] = useState(() => Math.floor(Date.now() / 1000));

  useEffect(() => {
    const timer = window.setInterval(() => setNow(Math.floor(Date.now() / 1000)), 1000);
    return () => window.clearInterval(timer);
  }, []);

  const refreshAt = expiresAt ? expiresAt - 60 : undefined;
  const expired = expiresAt != null && expiresAt <= now;
  const status = expired
    ? t('settings.token_status_expired')
    : verified
    ? t('settings.token_status_verified')
    : t('sync.device.connected');
  const refresh = !refreshScheduled || refreshAt == null
    ? t('settings.token_refresh_not_scheduled')
    : refreshAt <= now
    ? t('settings.token_refresh_due')
    : t('settings.token_refresh_in', tokenRefreshCountdown(refreshAt - now));

  return (
    <div style={{ display: 'grid', gap: '0.25rem', padding: '0 1.125rem 0.75rem', color: 'rgba(255,255,255,0.66)', fontSize: '0.75rem' }}>
      <span>{t('settings.token_status', status)}</span>
      <span>{refresh}</span>
    </div>
  );
}

function credentialAuthErrorMessage(err: unknown): string {
  switch (nuvioAuthErrorKind(err)) {
    case 'invalid_credentials':
      return t('auth.error.invalid_credentials');
    case 'email_not_confirmed':
      return t('auth.error.email_not_confirmed');
    case 'rate_limited':
      return t('auth.error.rate_limited');
    case 'server':
      return t('auth.error.server');
    case 'network': {
      const detail = err instanceof Error && err.message ? err.message : '';
      return detail ? `${t('auth.error.network')} (${detail})` : t('auth.error.network');
    }
    default:
      return err instanceof Error && err.message ? err.message : t('auth.error.network');
  }
}

function CredentialLoginForm({
  busy,
  onSubmit,
  onCancel,
}: {
  busy: boolean;
  onSubmit: (email: string, password: string) => void;
  onCancel: () => void;
}) {
  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');
  const canSubmit = !busy && email.trim().length > 0 && password.length > 0;
  const input: React.CSSProperties = {
    height: '1.875rem',
    borderRadius: '0.4375rem',
    border: '1px solid rgba(255,255,255,0.12)',
    background: 'rgba(255,255,255,0.05)',
    color: '#fff',
    fontSize: '0.7813rem',
    padding: '0 0.625rem',
    outline: 'none',
    flex: 1,
    minWidth: 0,
  };
  const btn: React.CSSProperties = {
    height: '1.875rem',
    borderRadius: '0.4375rem',
    border: '1px solid rgba(255,255,255,0.12)',
    background: 'rgba(255,255,255,0.06)',
    color: '#fff',
    fontSize: '0.75rem',
    fontWeight: 500,
    cursor: 'pointer',
    padding: '0 0.75rem',
    whiteSpace: 'nowrap',
  };
  return (
    <div style={{ padding: '0 1.125rem 0.75rem', borderBottom: '1px solid rgba(255,255,255,0.055)', display: 'flex', gap: '0.5rem', alignItems: 'center' }}>
      <input
        type="email"
        placeholder={t('auth.placeholder.email')}
        value={email}
        onChange={(e) => setEmail(e.target.value)}
        style={input}
        autoFocus
      />
      <input
        type="password"
        placeholder={t('auth.placeholder.password_login')}
        value={password}
        onChange={(e) => setPassword(e.target.value)}
        onKeyDown={(e) => { if (e.key === 'Enter' && canSubmit) onSubmit(email.trim(), password); }}
        style={input}
      />
      <button disabled={!canSubmit} onClick={() => onSubmit(email.trim(), password)} style={{ ...btn, opacity: canSubmit ? 1 : 0.5 }}>
        {busy ? t('auth.signing_in') : t('auth.sign_in')}
      </button>
      <button onClick={onCancel} disabled={busy} style={btn}>{t('common.cancel')}</button>
    </div>
  );
}

function AuthKeyLoginForm({
  busy,
  onSubmit,
  onCancel,
}: {
  busy: boolean;
  onSubmit: (authKey: string) => void;
  onCancel: () => void;
}) {
  const [authKey, setAuthKey] = useState('');
  const canSubmit = !busy && authKey.trim().length > 0;
  const input: React.CSSProperties = {
    height: '1.875rem',
    borderRadius: '0.4375rem',
    border: '1px solid rgba(255,255,255,0.12)',
    background: 'rgba(255,255,255,0.05)',
    color: '#fff',
    fontSize: '0.7813rem',
    padding: '0 0.625rem',
    outline: 'none',
    flex: 1,
    minWidth: 0,
  };
  const btn: React.CSSProperties = {
    height: '1.875rem',
    borderRadius: '0.4375rem',
    border: '1px solid rgba(255,255,255,0.12)',
    background: 'rgba(255,255,255,0.06)',
    color: '#fff',
    fontSize: '0.75rem',
    fontWeight: 500,
    cursor: 'pointer',
    padding: '0 0.75rem',
    whiteSpace: 'nowrap',
  };
  return (
    <div style={{ padding: '0 1.125rem 0.75rem', borderBottom: '1px solid rgba(255,255,255,0.055)', display: 'flex', flexDirection: 'column', gap: '0.5rem' }}>
      <p style={{ margin: 0, fontSize: '0.6875rem', color: 'rgba(255,255,255,0.55)' }}>{t('auth.stremio.authkey_hint')}</p>
      <div style={{ display: 'flex', gap: '0.5rem', alignItems: 'center' }}>
        <input
          type="text"
          placeholder={t('auth.placeholder.stremio_authkey')}
          value={authKey}
          onChange={(e) => setAuthKey(e.target.value)}
          onKeyDown={(e) => { if (e.key === 'Enter' && canSubmit) onSubmit(authKey.trim()); }}
          style={input}
          autoFocus
        />
        <button disabled={!canSubmit} onClick={() => onSubmit(authKey.trim())} style={{ ...btn, opacity: canSubmit ? 1 : 0.5 }}>
          {busy ? t('auth.signing_in') : t('auth.sign_in')}
        </button>
        <button onClick={onCancel} disabled={busy} style={btn}>{t('common.cancel')}</button>
      </div>
    </div>
  );
}

export function AccountSection({
  prefs,
  setPref,
  activeProfile,
  onProfileUpdated,
  onSwitchProfile,
  onDispatch,
  onNuvioSyncComplete,
}: {
  prefs: Prefs;
  setPref: <K extends keyof Prefs>(k: K, v: Prefs[K]) => void;
  activeProfile: UserProfile | null;
  onProfileUpdated: (profile: UserProfile) => void;
  onSwitchProfile: () => void;
  onDispatch: (actionJson: string) => void | Promise<void>;
  onNuvioSyncComplete?: () => void | Promise<void>;
}) {
  const [traktBusy, setTraktBusy] = useState(false);
  const [traktError, setTraktError] = useState<string | null>(null);
  const [traktPopoverOpen, setTraktPopoverOpen] = useState(false);
  const traktRowRef = useRef<HTMLDivElement>(null);
  const [traktSyncMeta, setTraktSyncMeta] = useState<SyncMeta | null>(null);
  const traktStateRef = useRef<string | null>(null);
  const anilistStateRef = useRef<string | null>(null);
  const simklStateRef = useRef<string | null>(null);
  const simklCodeVerifierRef = useRef<string | null>(null);
  const [anilistBusy, setAnilistBusy] = useState(false);
  const [anilistError, setAnilistError] = useState<string | null>(null);
  const [anilistPopoverOpen, setAnilistPopoverOpen] = useState(false);
  const anilistRowRef = useRef<HTMLDivElement>(null);
  const [anilistSyncMeta, setAnilistSyncMeta] = useState<SyncMeta | null>(null);
  const [simklBusy, setSimklBusy] = useState(false);
  const [simklError, setSimklError] = useState<string | null>(null);
  const [simklPopoverOpen, setSimklPopoverOpen] = useState(false);
  const simklRowRef = useRef<HTMLDivElement>(null);
  const [simklSyncMeta, setSimklSyncMeta] = useState<SyncMeta | null>(null);
  const [nuvioBusy, setNuvioBusy] = useState(false);
  const [nuvioError, setNuvioError] = useState<string | null>(null);
  const [nuvioPopoverOpen, setNuvioPopoverOpen] = useState(false);
  const nuvioRowRef = useRef<HTMLDivElement>(null);
  const [nuvioSyncMeta, setNuvioSyncMeta] = useState<SyncMeta | null>(null);
  const [nuvioFormOpen, setNuvioFormOpen] = useState(false);
  const [stremioBusy, setStremioBusy] = useState(false);
  const [stremioError, setStremioError] = useState<string | null>(null);
  const [stremioPopoverOpen, setStremioPopoverOpen] = useState(false);
  const stremioRowRef = useRef<HTMLDivElement>(null);
  const [stremioSyncMeta, setStremioSyncMeta] = useState<SyncMeta | null>(null);
  const [stremioFormOpen, setStremioFormOpen] = useState(false);
  const [stremioAuthKeyMode, setStremioAuthKeyMode] = useState(false);
  const [authUrls, setAuthUrls] = useState<Partial<Record<OAuthService, string>>>({});
  const [selectedIntegration, setSelectedIntegration] = useState<IntegrationService | null>(null);
  const [confirmDisconnect, setConfirmDisconnect] = useState<{ title: string; onConfirm: () => void } | null>(null);
  const [importDialog, setImportDialog] = useState<IntegrationService | null>(null);

  useEffect(() => {
    if (prefs.syncCwSourceOfTruth === 'most_recent') setPref('syncCwSourceOfTruth', '');
  }, [prefs.syncCwSourceOfTruth, setPref]);

  useEffect(() => {
    storageRead<SyncMeta>('trakt_sync_meta').then((m) => { if (m) setTraktSyncMeta(m); });
    storageRead<SyncMeta>('anilist_sync_meta').then((m) => { if (m) setAnilistSyncMeta(m); });
    storageRead<SyncMeta>('simkl_sync_meta').then((m) => { if (m) setSimklSyncMeta(m); });
    storageRead<SyncMeta>('nuvio_sync_meta').then((m) => { if (m) setNuvioSyncMeta(m); });
    storageRead<SyncMeta>('stremio_sync_meta').then((m) => { if (m) setStremioSyncMeta(m); });
  }, []);

  const [traktConnected, setTraktConnected] = useState(false);
  const anilistConnected = Boolean(activeProfile?.anilistAccessToken);
  const simklConnected = Boolean(activeProfile?.simklAccessToken);
  const nuvioConnected = Boolean(activeProfile?.nuvioAccessToken || activeProfile?.nuvioRefreshToken);
  const stremioConnected = Boolean(activeProfile?.stremioAuthKey);

  useEffect(() => {
    let cancelled = false;
    profileConnectionState(activeProfile).then((state) => {
      if (!cancelled) setTraktConnected(state.trakt);
    });
    return () => {
      cancelled = true;
    };
  }, [activeProfile]);

  useEffect(() => { if (traktConnected) setTraktBusy(false); }, [traktConnected]);
  useEffect(() => { if (anilistConnected) setAnilistBusy(false); }, [anilistConnected]);
  useEffect(() => { if (simklConnected) setSimklBusy(false); }, [simklConnected]);

  const setAuthUrl = (service: OAuthService, url?: string) => {
    setAuthUrls((current) => ({ ...current, [service]: url }));
  };

  const copyAuthUrl = async (service: OAuthService) => {
    const url = authUrls[service];
    if (!url) return;
    await navigator.clipboard.writeText(url).catch(() => undefined);
  };

  const renderOAuthFallback = (service: OAuthService) => {
    const url = authUrls[service];
    if (!url) return null;
    return (
      <div style={{ padding: '0 1.125rem 0.625rem', borderBottom: '1px solid rgba(255,255,255,0.055)', display: 'flex', gap: '0.5rem', alignItems: 'center' }}>
        <p style={{ color: 'rgba(255,255,255,0.44)', fontSize: '0.75rem', margin: 0, flex: 1, fontFamily: '-apple-system, BlinkMacSystemFont, "Segoe UI", "Ubuntu", "Noto Sans", sans-serif' }}>
          {t('settings.oauth_waiting_browser')}
        </p>
        <button
          onClick={() => void shellOpen(url)}
          style={{ height: '1.75rem', borderRadius: '0.4375rem', border: '1px solid rgba(255,255,255,0.12)', background: 'rgba(255,255,255,0.06)', color: '#fff', fontSize: '0.75rem', fontWeight: 500, cursor: 'pointer' }}
        >
          {t('settings.oauth_reopen')}
        </button>
        <button
          onClick={() => void copyAuthUrl(service)}
          style={{ height: '1.75rem', borderRadius: '0.4375rem', border: '1px solid rgba(255,255,255,0.12)', background: 'rgba(255,255,255,0.06)', color: '#fff', fontSize: '0.75rem', fontWeight: 500, cursor: 'pointer' }}
        >
          {t('settings.oauth_copy_link')}
        </button>
      </div>
    );
  };

  const handleTraktConnect = async () => {
    if (!activeProfile || traktBusy) return;
    setTraktBusy(true);
    setTraktError(null);
    try {
      const traktClientId = await invoke<string>('get_oauth_client_id', { service: 'trakt' });
      const state = generateCodeVerifier();
      traktStateRef.current = state;
      const authUrl = `https://trakt.tv/oauth/authorize?response_type=code&client_id=${traktClientId}&redirect_uri=${encodeURIComponent('fluxa://oauth/trakt')}&state=${state}`;
      setAuthUrl('trakt', authUrl);
      let unlisten: (() => void) | undefined;
      const consumeCallback = async () => {
        const payload = await invoke<OAuthCodePayload | null>('take_oauth_callback', { service: 'trakt' });
        if (!payload) return;
        unlisten?.();
        if (payload.state !== traktStateRef.current) {
          setTraktError(t('settings.oauth_state_mismatch'));
          setAuthUrl('trakt');
          setTraktBusy(false);
          return;
        }
        traktStateRef.current = null;
        setAuthUrl('trakt');
        try {
          const tokenJson = await invoke<string>('trakt_oauth_exchange', { code: payload.code });
          const tokens = JSON.parse(tokenJson) as TraktTokenResponse;
          const updated: UserProfile = { ...activeProfile, traktAccessToken: tokens.access_token, traktRefreshToken: tokens.refresh_token, traktTokenExpiresAt: tokens.created_at + tokens.expires_in };
          await saveProfile(updated);
          onProfileUpdated(updated);
        } catch (err) {
          setTraktError(err instanceof Error ? err.message : String(err));
        } finally {
          setTraktBusy(false);
        }
      };
      unlisten = await listen<OAuthCodePayload>('trakt-oauth-code', () => { void consumeCallback(); });
      await shellOpen(authUrl);
      void consumeCallback();
    } catch (err) {
      setTraktError(err instanceof Error ? err.message : String(err));
      setAuthUrl('trakt');
      setTraktBusy(false);
    }
  };

  const handleTraktDisconnect = async () => {
    if (!activeProfile) return;
    const updated: UserProfile = { ...activeProfile, traktAccessToken: undefined, traktRefreshToken: undefined, traktTokenExpiresAt: undefined };
    await saveProfile(updated);
    onProfileUpdated(updated);
  };

  const handleAnilistConnect = async () => {
    if (!activeProfile || anilistBusy) return;
    setAnilistBusy(true);
    setAnilistError(null);
    try {
      const anilistClientId = await invoke<string>('get_oauth_client_id', { service: 'anilist' });
      if (!anilistClientId) {
        setAnilistError('FLUXA_ANILIST_CLIENT_ID is not set.');
        setAnilistBusy(false);
        return;
      }
      const state = generateCodeVerifier();
      anilistStateRef.current = state;
      const authUrl = `https://anilist.co/api/v2/oauth/authorize?response_type=code&client_id=${anilistClientId}&redirect_uri=${encodeURIComponent('fluxa://oauth/anilist')}&state=${state}`;
      setAuthUrl('anilist', authUrl);
      let unlisten: (() => void) | undefined;
      const consumeCallback = async () => {
        const payload = await invoke<OAuthCodePayload | null>('take_oauth_callback', { service: 'anilist' });
        if (!payload) return;
        unlisten?.();
        if (payload.state !== anilistStateRef.current) {
          setAnilistError(t('settings.oauth_state_mismatch'));
          setAuthUrl('anilist');
          setAnilistBusy(false);
          return;
        }
        anilistStateRef.current = null;
        setAuthUrl('anilist');
        try {
          const tokenJson = await invoke<string>('anilist_oauth_exchange', { code: payload.code });
          const tokens = JSON.parse(tokenJson) as { access_token: string; expires_in?: number };
          const updated: UserProfile = {
            ...activeProfile,
            anilistAccessToken: tokens.access_token,
            anilistRefreshToken: undefined,
            anilistTokenExpiresAt: tokens.expires_in ? Math.floor(Date.now() / 1000) + tokens.expires_in : undefined,
          };
          await saveProfile(updated);
          onProfileUpdated(updated);
        } catch (err) {
          setAnilistError(err instanceof Error ? err.message : String(err));
        } finally {
          setAnilistBusy(false);
        }
      };
      unlisten = await listen<OAuthCodePayload>('anilist-oauth-code', () => { void consumeCallback(); });
      await shellOpen(authUrl);
      void consumeCallback();
    } catch (err) {
      setAnilistError(err instanceof Error ? err.message : String(err));
      setAuthUrl('anilist');
      setAnilistBusy(false);
    }
  };

  const handleAnilistDisconnect = async () => {
    if (!activeProfile) return;
    setAnilistPopoverOpen(false);
    const updated: UserProfile = {
      ...activeProfile,
      anilistAccessToken: undefined,
      anilistRefreshToken: undefined,
      anilistTokenExpiresAt: undefined,
    };
    await saveProfile(updated);
    onProfileUpdated(updated);
  };

  const handleSimklConnect = async () => {
    if (!activeProfile || simklBusy) return;
    setSimklBusy(true);
    setSimklError(null);
    try {
      const simklClientId = await invoke<string>('get_oauth_client_id', { service: 'simkl' });
      if (!simklClientId) {
        setSimklError('FLUXA_SIMKL_CLIENT_ID is not set.');
        setSimklBusy(false);
        return;
      }
      const state = generateCodeVerifier();
      const verifier = generateCodeVerifier();
      simklStateRef.current = state;
      simklCodeVerifierRef.current = verifier;
      const challenge = await codeChallenge(verifier);
      const authUrl = `https://simkl.com/oauth/authorize?response_type=code&client_id=${simklClientId}&redirect_uri=${encodeURIComponent('fluxa://oauth/simkl')}&state=${state}&code_challenge=${challenge}&code_challenge_method=S256&app-name=fluxa&app-version=1`;
      setAuthUrl('simkl', authUrl);
      let unlisten: (() => void) | undefined;
      const consumeCallback = async () => {
        const payload = await invoke<OAuthCodePayload | null>('take_oauth_callback', { service: 'simkl' });
        if (!payload) return;
        unlisten?.();
        if (payload.state !== simklStateRef.current) {
          setSimklError(t('settings.oauth_state_mismatch'));
          simklCodeVerifierRef.current = null;
          setAuthUrl('simkl');
          setSimklBusy(false);
          return;
        }
        simklStateRef.current = null;
        setAuthUrl('simkl');
        try {
          const codeVerifier = simklCodeVerifierRef.current;
          if (!codeVerifier) throw new Error(t('settings.oauth_state_mismatch'));
          const tokenJson = await invoke<string>('simkl_oauth_exchange', { code: payload.code, codeVerifier });
          const tokens = JSON.parse(tokenJson) as { access_token: string; refresh_token?: string; created_at?: number; expires_in?: number };
          const expiresAt = tokens.expires_in
            ? (tokens.created_at ?? Math.floor(Date.now() / 1000)) + tokens.expires_in
            : undefined;
          const updated: UserProfile = { ...activeProfile, simklAccessToken: tokens.access_token, simklRefreshToken: tokens.refresh_token, simklTokenExpiresAt: expiresAt };
          await saveProfile(updated);
          onProfileUpdated(updated);
        } catch (err) {
          setSimklError(err instanceof Error ? err.message : String(err));
        } finally {
          simklCodeVerifierRef.current = null;
          setSimklBusy(false);
        }
      };
      unlisten = await listen<OAuthCodePayload>('simkl-oauth-code', () => { void consumeCallback(); });
      await shellOpen(authUrl);
      void consumeCallback();
    } catch (err) {
      setSimklError(err instanceof Error ? err.message : String(err));
      setAuthUrl('simkl');
      setSimklBusy(false);
    }
  };

  const handleSimklDisconnect = async () => {
    if (!activeProfile) return;
    setSimklPopoverOpen(false);
    const updated: UserProfile = { ...activeProfile, simklAccessToken: undefined, simklRefreshToken: undefined, simklTokenExpiresAt: undefined };
    await saveProfile(updated);
    onProfileUpdated(updated);
  };

  const handleNuvioConnect = async (email: string, password: string) => {
    if (!activeProfile || nuvioBusy) return;
    setNuvioBusy(true);
    setNuvioError(null);
    try {
      const session = await nuvioSignIn(email, password);
      const updated: UserProfile = {
        ...activeProfile,
        nuvioAccessToken: session.access_token,
        nuvioRefreshToken: session.refresh_token,
        nuvioTokenExpiresAt: Math.floor(Date.now() / 1000) + (session.expires_in ?? 3600),
        nuvioUserId: session.user?.id,
        nuvioEmail: email,
        nuvioProfileIndex: activeProfile.nuvioProfileIndex ?? 1,
      };
      await saveProfile(updated);
      const importedProfile = await refreshNuvioProfiles(updated);
      onProfileUpdated(importedProfile);
      setNuvioFormOpen(false);
    } catch (err) {
      setNuvioError(credentialAuthErrorMessage(err));
    } finally {
      setNuvioBusy(false);
    }
  };

  const handleNuvioDisconnect = async () => {
    if (!activeProfile) return;
    setNuvioPopoverOpen(false);
    const updated: UserProfile = {
      ...activeProfile,
      nuvioAccessToken: undefined,
      nuvioRefreshToken: undefined,
      nuvioTokenExpiresAt: undefined,
      nuvioUserId: undefined,
      nuvioEmail: undefined,
    };
    await saveProfile(updated);
    onProfileUpdated(updated);
  };

  const handleStremioConnect = async (email: string, password: string) => {
    if (!activeProfile || stremioBusy) return;
    setStremioBusy(true);
    setStremioError(null);
    try {
      const auth = await stremioLogin(email, password);
      const updated: UserProfile = {
        ...activeProfile,
        stremioAuthKey: auth.authKey,
        stremioEmail: auth.user.email ?? email,
      };
      await saveProfile(updated);
      onProfileUpdated(updated);
      setStremioFormOpen(false);
    } catch (err) {
      setStremioError(credentialAuthErrorMessage(err));
    } finally {
      setStremioBusy(false);
    }
  };

  const handleStremioConnectWithAuthKey = async (authKey: string) => {
    if (!activeProfile || stremioBusy) return;
    setStremioBusy(true);
    setStremioError(null);
    try {
      const auth = await stremioLoginWithAuthKey(authKey);
      const updated: UserProfile = {
        ...activeProfile,
        stremioAuthKey: auth.authKey,
        stremioEmail: auth.user.email,
      };
      await saveProfile(updated);
      onProfileUpdated(updated);
      setStremioFormOpen(false);
      setStremioAuthKeyMode(false);
    } catch (err) {
      setStremioError(credentialAuthErrorMessage(err));
    } finally {
      setStremioBusy(false);
    }
  };

  const handleStremioDisconnect = async () => {
    if (!activeProfile) return;
    setStremioPopoverOpen(false);
    if (activeProfile.stremioAuthKey) void stremioLogout(activeProfile.stremioAuthKey);
    const updated: UserProfile = { ...activeProfile, stremioAuthKey: undefined, stremioEmail: undefined };
    await saveProfile(updated);
    onProfileUpdated(updated);
  };

  const handleTraktSyncNow = async (categories?: ImportCategory[]) => {
    if (!activeProfile?.traktAccessToken) return;
    setTraktBusy(true);
    setTraktError(null);
    try {
      const traktClientId = await invoke<string>('get_oauth_client_id', { service: 'trakt' });
      const result = await syncExternalIntegrationNow({
        provider: 'trakt',
        profile: activeProfile,
        token: activeProfile.traktAccessToken,
        clientId: traktClientId,
        ...(categories ? { categories } : {}),
      }) as { synced?: boolean; error?: string; continueWatchingCount?: number; watchlistCount?: number; watchedCount?: number };
      if (!result.synced) {
        setTraktError(result.error ?? t('toast.trakt_sync_failed'));
      } else {
        const meta: SyncMeta = { lastSyncAt: Date.now(), continueWatchingCount: result.continueWatchingCount ?? 0, watchlistCount: result.watchlistCount ?? 0, watchedCount: result.watchedCount ?? 0 };
        setTraktSyncMeta(meta);
        await storageWrite('trakt_sync_meta', meta);
      }
    } catch (error) {
      setTraktError(error instanceof Error ? error.message : String(error));
    } finally {
      setTraktBusy(false);
    }
    onDispatch(JSON.stringify({ type: 'libraryHydrateRequested' }));
    onDispatch(JSON.stringify({ type: 'homeLoadRequested', force: true, language: prefs.language }));
  };

  const handleSimklSyncNow = async (categories?: ImportCategory[]) => {
    if (!activeProfile?.simklAccessToken) return;
    setSimklBusy(true);
    setSimklError(null);
    try {
      const simklClientId = await invoke<string>('get_oauth_client_id', { service: 'simkl' });
      const result = await syncExternalIntegrationNow({
        provider: 'simkl',
        profile: activeProfile,
        token: activeProfile.simklAccessToken,
        clientId: simklClientId,
        ...(categories ? { categories } : {}),
      }) as { synced?: boolean; error?: string; continueWatchingCount?: number; watchlistCount?: number; watchedCount?: number };
      if (!result.synced) {
        setSimklError(result.error ?? 'Simkl sync failed');
      } else {
        const meta: SyncMeta = { lastSyncAt: Date.now(), continueWatchingCount: result.continueWatchingCount ?? 0, watchlistCount: result.watchlistCount ?? 0, watchedCount: result.watchedCount ?? 0 };
        setSimklSyncMeta(meta);
        await storageWrite('simkl_sync_meta', meta);
      }
    } catch (error) {
      setSimklError(error instanceof Error ? error.message : String(error));
    } finally {
      setSimklBusy(false);
    }
    onDispatch(JSON.stringify({ type: 'libraryHydrateRequested' }));
    onDispatch(JSON.stringify({ type: 'homeLoadRequested', force: true, language: prefs.language }));
  };

  const handleNuvioSyncNow = async (categories?: ImportCategory[]) => {
    if (!activeProfile?.nuvioAccessToken && !activeProfile?.nuvioRefreshToken) return;
    setNuvioBusy(true);
    setNuvioError(null);
    try {
      const result = await syncExternalIntegrationNow({
        provider: 'nuvio',
        profile: activeProfile,
        ...(categories ? { categories } : {}),
      }) as { synced?: boolean; error?: string };
      const meta: SyncMeta = { lastSyncAt: Date.now(), continueWatchingCount: 0, watchlistCount: 0, error: result.synced ? undefined : (result.error ?? 'Nuvio sync failed') };
      setNuvioSyncMeta(meta);
      await storageWrite('nuvio_sync_meta', meta);
      if (!result.synced) {
        setNuvioError(meta.error!);
      } else {
        const updatedProfile = await refreshNuvioProfiles(activeProfile);
        onProfileUpdated(updatedProfile);
        await onNuvioSyncComplete?.();
        await onDispatch(JSON.stringify({ type: 'addonsRefreshRequested', forceRefresh: false, profile: activeProfile }));
      }
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      setNuvioError(message);
      const meta: SyncMeta = { lastSyncAt: Date.now(), continueWatchingCount: 0, watchlistCount: 0, error: message };
      setNuvioSyncMeta(meta);
      await storageWrite('nuvio_sync_meta', meta);
    } finally {
      setNuvioBusy(false);
    }
    await onDispatch(JSON.stringify({ type: 'libraryHydrateRequested' }));
    await onDispatch(JSON.stringify({ type: 'homeLoadRequested', force: true, language: prefs.language }));
  };

  const handleStremioSyncNow = async (categories?: ImportCategory[]) => {
    if (!activeProfile?.stremioAuthKey) return;
    setStremioBusy(true);
    setStremioError(null);
    try {
      const result = await syncExternalIntegrationNow({
        provider: 'stremio',
        profile: activeProfile,
        token: activeProfile.stremioAuthKey,
        ...(categories ? { categories } : {}),
      }) as { synced?: boolean; error?: string; continueWatchingCount?: number; watchlistCount?: number };
      if (!result.synced) {
        setStremioError(result.error ?? 'Stremio sync failed');
      } else {
        const meta: SyncMeta = { lastSyncAt: Date.now(), continueWatchingCount: result.continueWatchingCount ?? 0, watchlistCount: result.watchlistCount ?? 0 };
        setStremioSyncMeta(meta);
        await storageWrite('stremio_sync_meta', meta);
      }
    } catch (error) {
      setStremioError(error instanceof Error ? error.message : String(error));
    } finally {
      setStremioBusy(false);
    }
    onDispatch(JSON.stringify({ type: 'libraryHydrateRequested' }));
    onDispatch(JSON.stringify({ type: 'homeLoadRequested', force: true, language: prefs.language }));
  };

  const handleAnilistSyncNow = async (categories?: ImportCategory[]) => {
    if (!activeProfile?.anilistAccessToken) return;
    setAnilistBusy(true);
    setAnilistError(null);
    try {
      const updated = await refreshAnimeTrackingProfile(activeProfile);
      if (updated !== activeProfile) onProfileUpdated(updated);
      const result = await syncExternalIntegrationNow({
        provider: 'anilist',
        profile: updated,
        token: updated.anilistAccessToken,
        ...(categories ? { categories } : {}),
      }) as { synced?: boolean; error?: string; continueWatchingCount?: number; watchlistCount?: number };
      if (!result.synced) {
        setAnilistError(result.error ?? 'AniList sync failed');
        return;
      }
      const meta: SyncMeta = { lastSyncAt: Date.now(), continueWatchingCount: result.continueWatchingCount ?? 0, watchlistCount: result.watchlistCount ?? 0 };
      setAnilistSyncMeta(meta);
      await storageWrite('anilist_sync_meta', meta);
    } catch (error) {
      setAnilistError(error instanceof Error ? error.message : String(error));
    } finally {
      setAnilistBusy(false);
    }
    onDispatch(JSON.stringify({ type: 'libraryHydrateRequested' }));
    onDispatch(JSON.stringify({ type: 'homeLoadRequested', force: true, language: prefs.language }));
  };

  if (selectedIntegration) {
    const tokenExpiresAt = selectedIntegration === 'trakt'
      ? activeProfile?.traktTokenExpiresAt
      : selectedIntegration === 'simkl'
      ? activeProfile?.simklTokenExpiresAt
      : undefined;
    const page = selectedIntegration === 'trakt'
      ? { title: t('brand.trakt'), connected: traktConnected, busy: traktBusy, meta: traktSyncMeta, error: traktError, connect: () => void handleTraktConnect(), sync: () => void handleTraktSyncNow(), disconnect: () => void handleTraktDisconnect() }
      : selectedIntegration === 'anilist'
        ? { title: t('brand.anilist'), connected: anilistConnected, busy: anilistBusy, meta: anilistSyncMeta, error: anilistError, connect: () => void handleAnilistConnect(), sync: () => void handleAnilistSyncNow(), disconnect: () => void handleAnilistDisconnect() }
        : selectedIntegration === 'simkl'
          ? { title: t('brand.simkl'), connected: simklConnected, busy: simklBusy, meta: simklSyncMeta, error: simklError, connect: () => void handleSimklConnect(), sync: () => void handleSimklSyncNow(), disconnect: () => void handleSimklDisconnect() }
          : selectedIntegration === 'nuvio'
            ? { title: t('brand.nuvio'), connected: nuvioConnected, busy: nuvioBusy, meta: nuvioSyncMeta, error: nuvioError, connect: () => setNuvioFormOpen(true), sync: () => void handleNuvioSyncNow(), disconnect: () => void handleNuvioDisconnect() }
            : { title: t('brand.stremio'), connected: stremioConnected, busy: stremioBusy, meta: stremioSyncMeta, error: stremioError, connect: () => setStremioFormOpen(true), sync: () => void handleStremioSyncNow(), disconnect: () => void handleStremioDisconnect() };
    const oauthService = selectedIntegration === 'trakt' || selectedIntegration === 'anilist' || selectedIntegration === 'simkl' ? selectedIntegration : null;

    return <>
      <div style={{ display: 'flex', alignItems: 'center' }}>
        <button type="button" aria-label={t('common.back')} onClick={() => setSelectedIntegration(null)} style={{ width: '2.25rem', height: '2.25rem', border: 'none', background: 'transparent', color: 'rgba(255,255,255,0.78)', cursor: 'pointer' }}><svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><path d="m15 18-6-6 6-6" /></svg></button>
        <div style={{ flex: 1 }}><SettingsDetailHeader title={page.title} /></div>
      </div>
      <SettingsSection title={t('settings.sync_with')} subtitle={t('settings.sync_with_desc')}>
        <SyncServiceRow icon={null} title={page.title} value={page.connected ? t('sync.device.connected') : t('settings.connect_account')} valueColor={page.connected ? '#54D17A' : undefined} onClick={page.connected ? undefined : page.connect} busy={page.busy} />
        {page.connected && (selectedIntegration === 'trakt' || selectedIntegration === 'simkl') && <ProviderTokenStatus expiresAt={tokenExpiresAt} verified={Boolean(page.meta && !page.meta.error)} refreshScheduled={selectedIntegration === 'trakt'} />}
        {!page.connected && oauthService && renderOAuthFallback(oauthService)}
        {!page.connected && selectedIntegration === 'nuvio' && nuvioFormOpen && <CredentialLoginForm busy={nuvioBusy} onSubmit={(email, password) => void handleNuvioConnect(email, password)} onCancel={() => setNuvioFormOpen(false)} />}
        {!page.connected && selectedIntegration === 'stremio' && stremioFormOpen && !stremioAuthKeyMode && <CredentialLoginForm busy={stremioBusy} onSubmit={(email, password) => void handleStremioConnect(email, password)} onCancel={() => setStremioFormOpen(false)} />}
        {!page.connected && selectedIntegration === 'stremio' && stremioFormOpen && stremioAuthKeyMode && <AuthKeyLoginForm busy={stremioBusy} onSubmit={(authKey) => void handleStremioConnectWithAuthKey(authKey)} onCancel={() => setStremioFormOpen(false)} />}
        {page.error && <div style={{ padding: '0 1.125rem 0.625rem' }}><p style={{ color: '#FF5D5D', fontSize: '0.75rem', margin: 0 }}>{t('common.error')}: {page.error}</p></div>}
      </SettingsSection>
      {page.connected && <SettingsSection title={page.title} subtitle={t('settings.sync_with_desc')}>
        <SyncServiceRow icon={null} title={t('settings.sync_now')} value={page.meta ? new Date(page.meta.lastSyncAt).toLocaleString() : ''} onClick={page.sync} busy={page.busy} />
        <SyncServiceRow icon={null} title={t('settings.import')} value="" onClick={() => setImportDialog(selectedIntegration)} busy={page.busy} />
        <SyncServiceRow icon={null} title={t('auto.disconnect')} value="" onClick={() => setConfirmDisconnect({ title: page.title, onConfirm: page.disconnect })} destructive />
      </SettingsSection>}
      {page.connected && <SettingsSection title={t('settings.provider_library')} subtitle={page.title}>
        <SyncServiceRow icon={null} title={t('settings.continue_watching_count', page.meta?.continueWatchingCount ?? 0)} value="" />
        <SyncServiceRow icon={null} title={t('settings.plan_to_watch_count', page.meta?.watchlistCount ?? 0)} value="" />
        <SyncServiceRow icon={null} title={t('settings.watched_count', page.meta?.watchedCount ?? 0)} value="" />
      </SettingsSection>}
      {page.connected && selectedIntegration === 'trakt' && <SettingsSection title={t('brand.trakt')} subtitle={t('settings.sync_with_desc')}>
        <ChoiceTile title={t('settings.continue_watching_window')} subtitle={t('settings.continue_watching_window_desc')} options={[{ value: '0', label: t('settings.continue_watching_window_all') }, { value: '7', label: '7' }, { value: '30', label: '30' }, { value: '90', label: '90' }, { value: '365', label: '365' }]} selected={prefs.continueWatchingDays} onSelect={(value) => setPref('continueWatchingDays', value)} />
        <ToggleTile title={t('settings.trakt_comments')} subtitle={t('settings.trakt_comments_desc')} checked={prefs.traktCommentsEnabled} onToggle={(value) => setPref('traktCommentsEnabled', value)} />
      </SettingsSection>}
      {confirmDisconnect && (
        <ConfirmDialog
          title={t('settings.disconnect_confirm_title', confirmDisconnect.title)}
          body={t('settings.disconnect_confirm_body', confirmDisconnect.title)}
          confirmLabel={t('auto.disconnect')}
          cancelLabel={t('common.cancel')}
          destructive
          onCancel={() => setConfirmDisconnect(null)}
          onConfirm={() => { const { onConfirm } = confirmDisconnect; setConfirmDisconnect(null); onConfirm(); }}
        />
      )}
      {importDialog && (
        <ImportDialog
          title={t('settings.import_title', page.title)}
          items={PROVIDER_IMPORT_CATEGORIES[importDialog].map((key) => ({ key, label: t(`settings.import_category.${key}`) }))}
          destinations={([
            ['trakt', t('brand.trakt'), traktConnected],
            ['simkl', t('brand.simkl'), simklConnected],
            ['anilist', t('brand.anilist'), anilistConnected],
            ['stremio', t('brand.stremio'), stremioConnected],
            ['nuvio', t('brand.nuvio'), nuvioConnected],
          ] as const)
            .filter(([key, , connected]) => connected && key !== importDialog)
            .map(([key, label]) => ({ key, label }))}
          destinationLabel={t('settings.import_destination_label')}
          localOnlyLabel={t('settings.import_destination_local')}
          scanLabel={t('settings.import_scan')}
          scanningLabel={t('settings.import_scanning')}
          backLabel={t('settings.import_back')}
          continueLabel={t('settings.import_continue')}
          confirmLabel={t('settings.import')}
          cancelLabel={t('common.cancel')}
          onCancel={() => setImportDialog(null)}
          onScan={async (selected) => {
            if (!importDialog || !activeProfile) return { counts: {} };
            const provider = importDialog;
            const token = provider === 'trakt' ? activeProfile.traktAccessToken
              : provider === 'simkl' ? activeProfile.simklAccessToken
              : provider === 'anilist' ? activeProfile.anilistAccessToken
              : provider === 'stremio' ? activeProfile.stremioAuthKey
              : activeProfile.nuvioAccessToken;
            const clientId = provider === 'trakt' ? await invoke<string>('get_oauth_client_id', { service: 'trakt' })
              : provider === 'simkl' ? await invoke<string>('get_oauth_client_id', { service: 'simkl' })
              : undefined;
            const result = await syncExternalIntegrationNow({
              provider, profile: activeProfile, token, clientId, categories: selected, dryRun: true,
            }) as {
              synced?: boolean; error?: string;
              watchlistCount?: number; continueWatchingCount?: number; watchedCount?: number;
              completedCount?: number; droppedCount?: number; collectionsCount?: number; addonCount?: number;
            };
            if (!result.synced) return { counts: {}, error: result.error ?? t('common.error') };
            const counts: Partial<Record<ImportCategory, number>> = {};
            if (selected.includes('watchlist')) counts.watchlist = result.watchlistCount ?? 0;
            if (selected.includes('continueWatching')) counts.continueWatching = result.continueWatchingCount ?? 0;
            if (selected.includes('watched')) counts.watched = result.watchedCount ?? ((result.completedCount ?? 0) + (result.droppedCount ?? 0));
            if (selected.includes('collections')) counts.collections = result.collectionsCount ?? 0;
            if (selected.includes('addons')) counts.addons = result.addonCount ?? 0;
            return { counts };
          }}
          onConfirm={(selected, destination) => {
            const source = importDialog;
            setImportDialog(null);
            void (async () => {
              if (source === 'trakt') await handleTraktSyncNow(selected);
              else if (source === 'simkl') await handleSimklSyncNow(selected);
              else if (source === 'anilist') await handleAnilistSyncNow(selected);
              else if (source === 'stremio') await handleStremioSyncNow(selected);
              else if (source === 'nuvio') await handleNuvioSyncNow(selected);

              if (!destination || !activeProfile) return;
              const traktClientId = destination === 'trakt' ? await invoke<string>('get_oauth_client_id', { service: 'trakt' }) : undefined;
              const simklClientId = destination === 'simkl' ? await invoke<string>('get_oauth_client_id', { service: 'simkl' }) : undefined;
              const { errors } = await pushImportedCategoriesToDestination({
                destination,
                categories: selected,
                profile: activeProfile,
                traktClientId,
                simklClientId,
              });
              const message = Object.values(errors).filter(Boolean).join('; ');
              if (!message) return;
              if (source === 'trakt') setTraktError(message);
              else if (source === 'simkl') setSimklError(message);
              else if (source === 'anilist') setAnilistError(message);
              else if (source === 'stremio') setStremioError(message);
              else if (source === 'nuvio') setNuvioError(message);
            })();
          }}
        />
      )}
    </>;
  }

  return (
    <>
      {activeProfile && (
        <SettingsSection title={t('profiles.active_profile')} subtitle={t('profiles.active_profile_desc')}>
          <SyncServiceRow
            icon={<AvatarPreview profile={activeProfile} size={36} circular />}
            title={activeProfile.name ?? t('auto.profile')}
            value={t('settings.switch_profiles_desc')}
            onClick={onSwitchProfile}
          />
        </SettingsSection>
      )}

      <SettingsSection title={t('settings.sync_with')} subtitle={t('settings.sync_with_desc')}>
        {/* Trakt */}
        {!traktConnected && (
          <SyncServiceRow
            icon={<div style={{ width: '2.125rem', height: '2.125rem', borderRadius: '0.5625rem', background: 'rgba(237,28,36,0.12)', display: 'flex', alignItems: 'center', justifyContent: 'center', overflow: 'hidden' }}><img src="/trakt.svg" alt="Trakt" style={{ width: '1.625rem', height: '1.625rem', objectFit: 'contain' }} /></div>}
            title="Trakt.tv"
            value={traktBusy ? t('trakt.device.waiting') : t('auto.connect_trakt_tv_account')}
            onClick={() => setSelectedIntegration('trakt')}
            busy={traktBusy}
          />
        )}
        {!traktConnected && renderOAuthFallback('trakt')}
        {traktError && (
          <div style={{ padding: '0 1.125rem 0.625rem', borderBottom: '1px solid rgba(255,255,255,0.055)' }}>
            <p style={{ color: '#FF5D5D', fontSize: '0.75rem', margin: 0, fontFamily: '-apple-system, BlinkMacSystemFont, "Segoe UI", "Ubuntu", "Noto Sans", sans-serif' }}>{t('common.error')}: {traktError}</p>
          </div>
        )}
        {traktConnected && (
          <div ref={traktRowRef} style={{ position: 'relative' }}>
            <SyncServiceRow
              icon={<div style={{ width: '2.125rem', height: '2.125rem', borderRadius: '0.5625rem', background: 'rgba(237,28,36,0.12)', display: 'flex', alignItems: 'center', justifyContent: 'center', overflow: 'hidden' }}><img src="/trakt.svg" alt="Trakt" style={{ width: '1.625rem', height: '1.625rem', objectFit: 'contain' }} /></div>}
              title="Trakt.tv"
              value={traktBusy ? t('sync.device.syncing') : t('sync.device.connected')}
              valueColor="#54D17A"
              onClick={() => setSelectedIntegration('trakt')}
              busy={traktBusy}
              expanded={traktPopoverOpen}
            />
            <SyncServicePopover
              open={traktPopoverOpen}
              anchorRef={traktRowRef}
              serviceName="Trakt.tv"
              meta={traktSyncMeta}
              busy={traktBusy}
              onSyncNow={() => void handleTraktSyncNow()}
              onDisconnect={() => setConfirmDisconnect({ title: 'Trakt.tv', onConfirm: () => void handleTraktDisconnect() })}
              onClose={() => setTraktPopoverOpen(false)}
            />
          </div>
        )}

        {/* AniList */}
        {!anilistConnected && (
          <SyncServiceRow
            icon={<div style={{ width: '2.125rem', height: '2.125rem', borderRadius: '0.5625rem', background: 'rgba(2,169,255,0.12)', display: 'flex', alignItems: 'center', justifyContent: 'center', overflow: 'hidden' }}><img src="/anilist.svg" alt="AniList" style={{ width: '1.625rem', height: '1.625rem', objectFit: 'contain' }} /></div>}
            title="AniList"
            value={anilistBusy ? t('trakt.device.waiting') : t('auto.connect_anilist_account')}
            onClick={() => setSelectedIntegration('anilist')}
            busy={anilistBusy}
          />
        )}
        {!anilistConnected && renderOAuthFallback('anilist')}
        {anilistError && (
          <div style={{ padding: '0 1.125rem 0.625rem', borderBottom: '1px solid rgba(255,255,255,0.055)' }}>
            <p style={{ color: '#FF5D5D', fontSize: '0.75rem', margin: 0, fontFamily: '-apple-system, BlinkMacSystemFont, "Segoe UI", "Ubuntu", "Noto Sans", sans-serif' }}>{t('common.error')}: {anilistError}</p>
          </div>
        )}
        {anilistConnected && (
          <div ref={anilistRowRef} style={{ position: 'relative' }}>
            <SyncServiceRow
              icon={<div style={{ width: '2.125rem', height: '2.125rem', borderRadius: '0.5625rem', background: 'rgba(2,169,255,0.12)', display: 'flex', alignItems: 'center', justifyContent: 'center', overflow: 'hidden' }}><img src="/anilist.svg" alt="AniList" style={{ width: '1.625rem', height: '1.625rem', objectFit: 'contain' }} /></div>}
              title="AniList"
              value={anilistBusy ? t('sync.device.syncing') : t('settings.anime_tracking_enabled')}
              valueColor="#54D17A"
              onClick={() => setSelectedIntegration('anilist')}
              busy={anilistBusy}
              expanded={anilistPopoverOpen}
            />
            <SyncServicePopover
              open={anilistPopoverOpen}
              anchorRef={anilistRowRef}
              serviceName="AniList"
              meta={anilistSyncMeta}
              busy={anilistBusy}
              statusLabel={anilistSyncMeta ? `${t('settings.anime_tracking_enabled')} · ${new Date(anilistSyncMeta.lastSyncAt).toLocaleString()}` : t('settings.anime_tracking_enabled')}
              statusColor="#54D17A"
              syncLabel={t('settings.sync_now')}
              onSyncNow={() => void handleAnilistSyncNow()}
              onDisconnect={() => setConfirmDisconnect({ title: 'AniList', onConfirm: () => void handleAnilistDisconnect() })}
              onClose={() => setAnilistPopoverOpen(false)}
            />
          </div>
        )}

        {/* Simkl */}
        {!simklConnected && (
          <SyncServiceRow
            icon={<div style={{ width: '2.125rem', height: '2.125rem', borderRadius: '0.5625rem', background: 'rgba(28,177,74,0.12)', display: 'flex', alignItems: 'center', justifyContent: 'center', overflow: 'hidden' }}><img src="/simkl.svg" alt="Simkl" style={{ width: '1.625rem', height: '1.625rem', objectFit: 'contain' }} /></div>}
            title="Simkl"
            value={simklBusy ? t('trakt.device.waiting') : t('auto.connect_simkl_account')}
            onClick={() => setSelectedIntegration('simkl')}
            busy={simklBusy}
          />
        )}
        {!simklConnected && renderOAuthFallback('simkl')}
        {simklError && (
          <div style={{ padding: '0 1.125rem 0.625rem', borderBottom: '1px solid rgba(255,255,255,0.055)' }}>
            <p style={{ color: '#FF5D5D', fontSize: '0.75rem', margin: 0, fontFamily: '-apple-system, BlinkMacSystemFont, "Segoe UI", "Ubuntu", "Noto Sans", sans-serif' }}>{t('common.error')}: {simklError}</p>
          </div>
        )}
        {simklConnected && (
          <div ref={simklRowRef} style={{ position: 'relative' }}>
            <SyncServiceRow
              icon={<div style={{ width: '2.125rem', height: '2.125rem', borderRadius: '0.5625rem', background: 'rgba(28,177,74,0.12)', display: 'flex', alignItems: 'center', justifyContent: 'center', overflow: 'hidden' }}><img src="/simkl.svg" alt="Simkl" style={{ width: '1.625rem', height: '1.625rem', objectFit: 'contain' }} /></div>}
              title="Simkl"
              value={simklBusy ? t('sync.device.syncing') : t('sync.device.connected')}
              valueColor="#54D17A"
              onClick={() => setSelectedIntegration('simkl')}
              busy={simklBusy}
              expanded={simklPopoverOpen}
            />
            <SyncServicePopover
              open={simklPopoverOpen}
              anchorRef={simklRowRef}
              serviceName="Simkl"
              meta={simklSyncMeta}
              busy={simklBusy}
              onSyncNow={() => void handleSimklSyncNow()}
              onDisconnect={() => setConfirmDisconnect({ title: 'Simkl', onConfirm: () => void handleSimklDisconnect() })}
              onClose={() => setSimklPopoverOpen(false)}
            />
          </div>
        )}

        {/* Nuvio */}
        {!nuvioConnected && (
          <SyncServiceRow
            icon={<div style={{ width: '2.125rem', height: '2.125rem', borderRadius: '0.5625rem', background: 'rgba(255,255,255,0.06)', display: 'flex', alignItems: 'center', justifyContent: 'center', overflow: 'hidden' }}><img src="https://nuvio.tv//assets/Logo_1080x1080.png" alt="Nuvio" style={{ width: '1.5rem', height: '1.5rem', objectFit: 'contain' }} /></div>}
            title="Nuvio"
            value={nuvioBusy ? t('auth.signing_in') : t('settings.connect_nuvio_account')}
            onClick={() => setSelectedIntegration('nuvio')}
            busy={nuvioBusy}
            expanded={nuvioFormOpen}
          />
        )}
        {!nuvioConnected && nuvioFormOpen && (
          <CredentialLoginForm
            busy={nuvioBusy}
            onSubmit={(email, password) => void handleNuvioConnect(email, password)}
            onCancel={() => { setNuvioFormOpen(false); setNuvioError(null); }}
          />
        )}
        {nuvioError && (
          <div style={{ padding: '0 1.125rem 0.625rem', borderBottom: '1px solid rgba(255,255,255,0.055)' }}>
            <p style={{ color: '#FF5D5D', fontSize: '0.75rem', margin: 0, fontFamily: '-apple-system, BlinkMacSystemFont, "Segoe UI", "Ubuntu", "Noto Sans", sans-serif' }}>{t('common.error')}: {nuvioError}</p>
          </div>
        )}
        {nuvioConnected && (
          <div ref={nuvioRowRef} style={{ position: 'relative' }}>
            <SyncServiceRow
              icon={<div style={{ width: '2.125rem', height: '2.125rem', borderRadius: '0.5625rem', background: 'rgba(255,255,255,0.06)', display: 'flex', alignItems: 'center', justifyContent: 'center', overflow: 'hidden' }}><img src="https://nuvio.tv//assets/Logo_1080x1080.png" alt="Nuvio" style={{ width: '1.5rem', height: '1.5rem', objectFit: 'contain' }} /></div>}
              title="Nuvio"
              value={nuvioBusy ? t('sync.device.syncing') : (activeProfile?.nuvioEmail ?? t('sync.device.connected'))}
              valueColor="#54D17A"
              onClick={() => setSelectedIntegration('nuvio')}
              busy={nuvioBusy}
              expanded={nuvioPopoverOpen}
            />
            <SyncServicePopover
              open={nuvioPopoverOpen}
              anchorRef={nuvioRowRef}
              serviceName="Nuvio"
              meta={nuvioSyncMeta}
              busy={nuvioBusy}
              onSyncNow={() => void handleNuvioSyncNow()}
              onDisconnect={() => setConfirmDisconnect({ title: 'Nuvio', onConfirm: () => void handleNuvioDisconnect() })}
              onClose={() => setNuvioPopoverOpen(false)}
            />
          </div>
        )}

        {/* Stremio */}
        {!stremioConnected && (
          <SyncServiceRow
            icon={<div style={{ width: '2.125rem', height: '2.125rem', borderRadius: '0.5625rem', background: 'rgba(123,91,245,0.12)', display: 'flex', alignItems: 'center', justifyContent: 'center', overflow: 'hidden' }}><img src="/stremio.svg" alt="Stremio" style={{ width: '1.5rem', height: '1.5rem', objectFit: 'contain' }} /></div>}
            title="Stremio"
            value={stremioBusy ? t('auth.signing_in') : t('settings.connect_stremio_account')}
            onClick={() => setSelectedIntegration('stremio')}
            busy={stremioBusy}
            expanded={stremioFormOpen}
          />
        )}
        {!stremioConnected && stremioFormOpen && !stremioAuthKeyMode && (
          <CredentialLoginForm
            busy={stremioBusy}
            onSubmit={(email, password) => void handleStremioConnect(email, password)}
            onCancel={() => { setStremioFormOpen(false); setStremioError(null); }}
          />
        )}
        {!stremioConnected && stremioFormOpen && (
          <div style={{ padding: stremioAuthKeyMode ? '0 1.125rem' : '0.5rem 1.125rem 0', borderBottom: stremioAuthKeyMode ? undefined : '1px solid rgba(255,255,255,0.055)' }}>
            <button
              onClick={() => { setStremioAuthKeyMode((m) => !m); setStremioError(null); }}
              disabled={stremioBusy}
              style={{ background: 'none', border: 'none', color: 'rgba(255,255,255,0.5)', fontSize: '0.6875rem', cursor: 'pointer', padding: 0, marginBottom: stremioAuthKeyMode ? 0 : '0.5rem' }}
            >
              {stremioAuthKeyMode ? t('auth.stremio.use_password_instead') : t('auth.stremio.use_authkey_instead')}
            </button>
          </div>
        )}
        {!stremioConnected && stremioFormOpen && stremioAuthKeyMode && (
          <AuthKeyLoginForm
            busy={stremioBusy}
            onSubmit={(authKey) => void handleStremioConnectWithAuthKey(authKey)}
            onCancel={() => { setStremioFormOpen(false); setStremioAuthKeyMode(false); setStremioError(null); }}
          />
        )}
        {stremioError && (
          <div style={{ padding: '0 1.125rem 0.625rem', borderBottom: '1px solid rgba(255,255,255,0.055)' }}>
            <p style={{ color: '#FF5D5D', fontSize: '0.75rem', margin: 0, fontFamily: '-apple-system, BlinkMacSystemFont, "Segoe UI", "Ubuntu", "Noto Sans", sans-serif' }}>{t('common.error')}: {stremioError}</p>
          </div>
        )}
        {stremioConnected && (
          <div ref={stremioRowRef} style={{ position: 'relative' }}>
            <SyncServiceRow
              icon={<div style={{ width: '2.125rem', height: '2.125rem', borderRadius: '0.5625rem', background: 'rgba(123,91,245,0.12)', display: 'flex', alignItems: 'center', justifyContent: 'center', overflow: 'hidden' }}><img src="/stremio.svg" alt="Stremio" style={{ width: '1.5rem', height: '1.5rem', objectFit: 'contain' }} /></div>}
              title="Stremio"
              value={stremioBusy ? t('sync.device.syncing') : (activeProfile?.stremioEmail ?? t('sync.device.connected'))}
              valueColor="#54D17A"
              onClick={() => setSelectedIntegration('stremio')}
              busy={stremioBusy}
              expanded={stremioPopoverOpen}
            />
            <SyncServicePopover
              open={stremioPopoverOpen}
              anchorRef={stremioRowRef}
              serviceName="Stremio"
              meta={stremioSyncMeta}
              busy={stremioBusy}
              onSyncNow={() => void handleStremioSyncNow()}
              onDisconnect={() => setConfirmDisconnect({ title: 'Stremio', onConfirm: () => void handleStremioDisconnect() })}
              onClose={() => setStremioPopoverOpen(false)}
            />
          </div>
        )}
      </SettingsSection>

      <SettingsSection title={t('settings.cw_conflict_resolution')} subtitle={t('settings.cw_conflict_resolution_desc')}>
        <ChoiceTile title={t('settings.cw_source_of_truth')} subtitle={t('settings.cw_source_of_truth_desc')} options={cwSourceOfTruthOptions()} selected={prefs.syncCwSourceOfTruth} onSelect={(value) => setPref('syncCwSourceOfTruth', value)} />
        <ChoiceTile title={t('settings.cw_ranking')} subtitle={t('settings.cw_ranking_desc')} options={cwRankingOptions()} selected={prefs.syncCwRanking} onSelect={(value) => setPref('syncCwRanking', value)} />
        <ChoiceTile title={t('settings.similar_titles_source')} subtitle={t('settings.similar_titles_source_desc')} options={similarTitlesSourceOptions()} selected={prefs.similarTitlesSource} onSelect={(value) => setPref('similarTitlesSource', value)} />
      </SettingsSection>

      {confirmDisconnect && (
        <ConfirmDialog
          title={t('settings.disconnect_confirm_title', confirmDisconnect.title)}
          body={t('settings.disconnect_confirm_body', confirmDisconnect.title)}
          confirmLabel={t('auto.disconnect')}
          cancelLabel={t('common.cancel')}
          destructive
          onCancel={() => setConfirmDisconnect(null)}
          onConfirm={() => { const { onConfirm } = confirmDisconnect; setConfirmDisconnect(null); onConfirm(); }}
        />
      )}
    </>
  );
}
