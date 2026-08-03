import React, { useEffect, useRef, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { open as shellOpen } from '@tauri-apps/plugin-shell';
import { storageRead, storageWrite } from '../../core/engine';
import type { UserProfile } from '../../core/types';
import { t } from '../../i18n';
import type { ImportCategory } from '../../core/importCategories';
import { profileConnectionState, saveProfile } from '../../core/profiles';
import { syncExternalIntegrationNow } from '../../core/effectRunner';
import { refreshAnimeTrackingProfile } from '../../core/animeExternalSync';
import { platformFetch } from '../../core/httpClient';
import { traktHeaders } from '../../core/traktSync';
import type { Prefs, SyncMeta, TraktTokenResponse } from './settingsTypes';
import { nuvioSignIn } from '../../core/nuvioApi';
import { refreshNuvioProfiles } from '../../core/nuvioSync';
import { stremioLogin, stremioLoginWithAuthKey, stremioLogout } from '../../core/stremioApi';
import { codeChallenge, credentialAuthErrorMessage, generateCodeVerifier, type OAuthCodePayload, type OAuthService } from './accountPresentation';

async function fetchTraktUsername(token: string, clientId: string): Promise<string | undefined> {
  try {
    const res = await platformFetch('https://api.trakt.tv/users/settings', { headers: traktHeaders(token, clientId) });
    const json = await res.json() as { user?: { username?: string } };
    return json.user?.username;
  } catch {
    return undefined;
  }
}

async function fetchAnilistUsername(token: string): Promise<string | undefined> {
  try {
    const res = await platformFetch('https://graphql.anilist.co', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json', Authorization: `Bearer ${token}` },
      body: JSON.stringify({ query: '{ Viewer { name } }' }),
    });
    const json = await res.json() as { data?: { Viewer?: { name?: string } } };
    return json.data?.Viewer?.name;
  } catch {
    return undefined;
  }
}

async function fetchSimklUsername(token: string, clientId: string): Promise<string | undefined> {
  try {
    const res = await platformFetch('https://api.simkl.com/users/settings', {
      headers: { Authorization: `Bearer ${token}`, 'simkl-api-key': clientId },
    });
    const json = await res.json() as { user?: { name?: string } };
    return json.user?.name;
  } catch {
    return undefined;
  }
}

export function useIntegrationAccounts({
  prefs,
  activeProfile,
  onProfileUpdated,
  onDispatch,
  onNuvioSyncComplete,
}: {
  prefs: Prefs;
  activeProfile: UserProfile | null;
  onProfileUpdated: (profile: UserProfile) => void;
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
  const [confirmDisconnect, setConfirmDisconnect] = useState<{ title: string; onConfirm: () => void } | null>(null);

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
          const traktUsername = await fetchTraktUsername(tokens.access_token, traktClientId);
          const updated: UserProfile = { ...activeProfile, traktAccessToken: tokens.access_token, traktRefreshToken: tokens.refresh_token, traktTokenExpiresAt: tokens.created_at + tokens.expires_in, traktUsername };
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
    const updated: UserProfile = { ...activeProfile, traktAccessToken: undefined, traktRefreshToken: undefined, traktTokenExpiresAt: undefined, traktUsername: undefined };
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
          const anilistUsername = await fetchAnilistUsername(tokens.access_token);
          const updated: UserProfile = {
            ...activeProfile,
            anilistAccessToken: tokens.access_token,
            anilistRefreshToken: undefined,
            anilistTokenExpiresAt: tokens.expires_in ? Math.floor(Date.now() / 1000) + tokens.expires_in : undefined,
            anilistUsername,
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
      anilistUsername: undefined,
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
          const simklUsername = await fetchSimklUsername(tokens.access_token, simklClientId);
          const updated: UserProfile = { ...activeProfile, simklAccessToken: tokens.access_token, simklRefreshToken: tokens.refresh_token, simklTokenExpiresAt: expiresAt, simklUsername };
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
    const updated: UserProfile = { ...activeProfile, simklAccessToken: undefined, simklRefreshToken: undefined, simklTokenExpiresAt: undefined, simklUsername: undefined };
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

  return {
    traktBusy, traktError, setTraktError, traktPopoverOpen, setTraktPopoverOpen, traktRowRef, traktSyncMeta, traktConnected,
    anilistBusy, anilistError, setAnilistError, anilistPopoverOpen, setAnilistPopoverOpen, anilistRowRef, anilistSyncMeta, anilistConnected,
    simklBusy, simklError, setSimklError, simklPopoverOpen, setSimklPopoverOpen, simklRowRef, simklSyncMeta, simklConnected,
    nuvioBusy, nuvioError, setNuvioError, nuvioPopoverOpen, setNuvioPopoverOpen, nuvioRowRef, nuvioSyncMeta, nuvioConnected, nuvioFormOpen, setNuvioFormOpen,
    stremioBusy, stremioError, setStremioError, stremioPopoverOpen, setStremioPopoverOpen, stremioRowRef, stremioSyncMeta, stremioConnected,
    stremioFormOpen, setStremioFormOpen, stremioAuthKeyMode, setStremioAuthKeyMode,
    confirmDisconnect, setConfirmDisconnect,
    renderOAuthFallback,
    handleTraktConnect, handleTraktDisconnect, handleTraktSyncNow,
    handleAnilistConnect, handleAnilistDisconnect, handleAnilistSyncNow,
    handleSimklConnect, handleSimklDisconnect, handleSimklSyncNow,
    handleNuvioConnect, handleNuvioDisconnect, handleNuvioSyncNow,
    handleStremioConnect, handleStremioConnectWithAuthKey, handleStremioDisconnect, handleStremioSyncNow,
  };
}
