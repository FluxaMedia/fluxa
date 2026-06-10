import React, { useEffect, useRef, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { open as shellOpen } from '@tauri-apps/plugin-shell';
import { storageRead, storageWrite } from '../../core/engine';
import type { UserProfile } from '../../core/types';
import { t } from '../../i18n';
import { isTraktConnected, profileColor, profileInitials, saveProfile } from '../../core/profiles';
import { syncExternalIntegrationNow } from '../../core/effectRunner';
import { SettingsSection, SyncServicePopover, SyncServiceRow } from './SettingsUI';
import type { Prefs, SyncMeta, TraktTokenResponse } from './settingsTypes';

function generateCodeVerifier(): string {
  const array = new Uint8Array(48);
  crypto.getRandomValues(array);
  return btoa(String.fromCharCode(...array))
    .replace(/\+/g, '-')
    .replace(/\//g, '_')
    .replace(/=/g, '')
    .slice(0, 64);
}

export function AccountSection({
  prefs,
  setPref: _setPref,
  activeProfile,
  onProfileUpdated,
  onSwitchProfile,
  onDispatch,
}: {
  prefs: Prefs;
  setPref: <K extends keyof Prefs>(k: K, v: Prefs[K]) => void;
  activeProfile: UserProfile | null;
  onProfileUpdated: (profile: UserProfile) => void;
  onSwitchProfile: () => void;
  onDispatch: (actionJson: string) => void;
}) {
  const [traktBusy, setTraktBusy] = useState(false);
  const [traktError, setTraktError] = useState<string | null>(null);
  const [traktPopoverOpen, setTraktPopoverOpen] = useState(false);
  const [traktSyncMeta, setTraktSyncMeta] = useState<SyncMeta | null>(null);
  const [malBusy, setMalBusy] = useState(false);
  const [malError, setMalError] = useState<string | null>(null);
  const malVerifierRef = useRef<string | null>(null);
  const [simklBusy, setSimklBusy] = useState(false);
  const [simklError, setSimklError] = useState<string | null>(null);
  const [simklPopoverOpen, setSimklPopoverOpen] = useState(false);
  const [simklSyncMeta, setSimklSyncMeta] = useState<SyncMeta | null>(null);

  useEffect(() => {
    storageRead<SyncMeta>('trakt_sync_meta').then((m) => { if (m) setTraktSyncMeta(m); });
    storageRead<SyncMeta>('simkl_sync_meta').then((m) => { if (m) setSimklSyncMeta(m); });
  }, []);

  const traktConnected = isTraktConnected(activeProfile);
  const malConnected = Boolean(activeProfile?.malAccessToken);
  const simklConnected = Boolean(activeProfile?.simklAccessToken);

  useEffect(() => { if (traktConnected) setTraktBusy(false); }, [traktConnected]);
  useEffect(() => { if (malConnected) setMalBusy(false); }, [malConnected]);
  useEffect(() => { if (simklConnected) setSimklBusy(false); }, [simklConnected]);

  const handleTraktConnect = async () => {
    if (!activeProfile || traktBusy) return;
    setTraktBusy(true);
    setTraktError(null);
    try {
      const traktClientId = await invoke<string>('get_oauth_client_id', { service: 'trakt' });
      const authUrl = `https://trakt.tv/oauth/authorize?response_type=code&client_id=${traktClientId}&redirect_uri=${encodeURIComponent('fluxa://oauth/trakt')}`;
      await shellOpen(authUrl);

      const unlisten = await listen<string>('trakt-oauth-code', async (event) => {
        unlisten();
        try {
          const tokenJson = await invoke<string>('trakt_oauth_exchange', { code: event.payload });
          const tokens = JSON.parse(tokenJson) as TraktTokenResponse;
          const updated: UserProfile = { ...activeProfile, traktAccessToken: tokens.access_token, traktRefreshToken: tokens.refresh_token, traktTokenExpiresAt: tokens.created_at + tokens.expires_in };
          await saveProfile(updated);
          onProfileUpdated(updated);
        } catch (err) {
          setTraktError(err instanceof Error ? err.message : String(err));
        } finally {
          setTraktBusy(false);
        }
      });
    } catch (err) {
      setTraktError(err instanceof Error ? err.message : String(err));
      setTraktBusy(false);
    }
  };

  const handleTraktDisconnect = async () => {
    if (!activeProfile) return;
    const updated: UserProfile = { ...activeProfile, traktAccessToken: undefined, traktRefreshToken: undefined, traktTokenExpiresAt: undefined };
    await saveProfile(updated);
    onProfileUpdated(updated);
  };

  const handleMalConnect = async () => {
    if (!activeProfile || malBusy) return;
    setMalBusy(true);
    setMalError(null);
    try {
      const malClientId = await invoke<string>('get_oauth_client_id', { service: 'mal' });
      if (!malClientId) {
        setMalError('FLUXA_MAL_CLIENT_ID ortam değişkeni ayarlanmamış.');
        setMalBusy(false);
        return;
      }
      const verifier = generateCodeVerifier();
      malVerifierRef.current = verifier;
      const authUrl = `https://myanimelist.net/v1/oauth2/authorize?response_type=code&client_id=${malClientId}&redirect_uri=${encodeURIComponent('fluxa://oauth/mal')}&code_challenge=${verifier}&code_challenge_method=plain`;
      await shellOpen(authUrl);

      const unlisten = await listen<string>('mal-oauth-code', async (event) => {
        unlisten();
        const storedVerifier = malVerifierRef.current;
        if (!storedVerifier) { setMalBusy(false); return; }
        try {
          const tokenJson = await invoke<string>('mal_oauth_exchange', { code: event.payload, codeVerifier: storedVerifier });
          const tokens = JSON.parse(tokenJson) as { access_token: string; refresh_token?: string; expires_in?: number };
          const updated: UserProfile = {
            ...activeProfile,
            malAccessToken: tokens.access_token,
            malRefreshToken: tokens.refresh_token,
            malTokenExpiresAt: tokens.expires_in ? Math.floor(Date.now() / 1000) + tokens.expires_in : undefined,
          };
          await saveProfile(updated);
          onProfileUpdated(updated);
        } catch (err) {
          setMalError(err instanceof Error ? err.message : String(err));
        } finally {
          setMalBusy(false);
        }
      });
    } catch (err) {
      setMalError(err instanceof Error ? err.message : String(err));
      setMalBusy(false);
    }
  };

  const handleMalDisconnect = async () => {
    if (!activeProfile) return;
    const updated: UserProfile = { ...activeProfile, malAccessToken: undefined, malRefreshToken: undefined, malTokenExpiresAt: undefined };
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
      const authUrl = `https://simkl.com/oauth/authorize?response_type=code&client_id=${simklClientId}&redirect_uri=${encodeURIComponent('fluxa://oauth/simkl')}`;
      await shellOpen(authUrl);

      const unlisten = await listen<string>('simkl-oauth-code', async (event) => {
        unlisten();
        try {
          const tokenJson = await invoke<string>('simkl_oauth_exchange', { code: event.payload });
          const tokens = JSON.parse(tokenJson) as { access_token: string; refresh_token?: string };
          const updated: UserProfile = { ...activeProfile, simklAccessToken: tokens.access_token, simklRefreshToken: tokens.refresh_token };
          await saveProfile(updated);
          onProfileUpdated(updated);
        } catch (err) {
          setSimklError(err instanceof Error ? err.message : String(err));
        } finally {
          setSimklBusy(false);
        }
      });
    } catch (err) {
      setSimklError(err instanceof Error ? err.message : String(err));
      setSimklBusy(false);
    }
  };

  const handleSimklDisconnect = async () => {
    if (!activeProfile) return;
    setSimklPopoverOpen(false);
    const updated: UserProfile = { ...activeProfile, simklAccessToken: undefined, simklRefreshToken: undefined };
    await saveProfile(updated);
    onProfileUpdated(updated);
  };

  const handleTraktSyncNow = async () => {
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
      }) as { synced?: boolean; error?: string; continueWatchingCount?: number; watchlistCount?: number };
      if (!result.synced) {
        setTraktError(result.error ?? t('toast.trakt_sync_failed'));
      } else {
        const meta: SyncMeta = { lastSyncAt: Date.now(), continueWatchingCount: result.continueWatchingCount ?? 0, watchlistCount: result.watchlistCount ?? 0 };
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

  const handleSimklSyncNow = async () => {
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
      }) as { synced?: boolean; error?: string; continueWatchingCount?: number; watchlistCount?: number };
      if (!result.synced) {
        setSimklError(result.error ?? 'Simkl sync failed');
      } else {
        const meta: SyncMeta = { lastSyncAt: Date.now(), continueWatchingCount: result.continueWatchingCount ?? 0, watchlistCount: result.watchlistCount ?? 0 };
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

  return (
    <>
      {activeProfile && (
        <SettingsSection title={t('profiles.active_profile')} subtitle={t('profiles.active_profile_desc')}>
          <SyncServiceRow
            icon={
              <div style={{ width: 36, height: 36, borderRadius: '50%', background: profileColor(activeProfile), display: 'flex', alignItems: 'center', justifyContent: 'center' }}>
                {activeProfile.avatarUrl ? (
                  <img src={activeProfile.avatarUrl} alt="" style={{ width: '100%', height: '100%', borderRadius: '50%', objectFit: 'cover' }} />
                ) : (
                  <span style={{ color: '#fff', fontSize: 15, fontWeight: 900, fontFamily: 'sans-serif' }}>{profileInitials(activeProfile)}</span>
                )}
              </div>
            }
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
            icon={<div style={{ width: 34, height: 34, borderRadius: 9, background: 'rgba(237,28,36,0.12)', display: 'flex', alignItems: 'center', justifyContent: 'center', overflow: 'hidden' }}><img src="/trakt.png" alt="Trakt" style={{ width: 26, height: 26, objectFit: 'contain' }} /></div>}
            title="Trakt.tv"
            value={traktBusy ? t('trakt.device.waiting') : t('auto.connect_trakt_tv_account')}
            onClick={() => void handleTraktConnect()}
            busy={traktBusy}
          />
        )}
        {traktError && (
          <div style={{ padding: '0 18px 10px', borderBottom: '1px solid rgba(255,255,255,0.055)' }}>
            <p style={{ color: '#FF5D5D', fontSize: 12, margin: 0, fontFamily: 'sans-serif' }}>{t('common.error')}: {traktError}</p>
          </div>
        )}
        {traktConnected && (
          <div style={{ position: 'relative' }}>
            <SyncServiceRow
              icon={<div style={{ width: 34, height: 34, borderRadius: 9, background: 'rgba(237,28,36,0.12)', display: 'flex', alignItems: 'center', justifyContent: 'center', overflow: 'hidden' }}><img src="/trakt.png" alt="Trakt" style={{ width: 26, height: 26, objectFit: 'contain' }} /></div>}
              title="Trakt.tv"
              value={traktBusy ? t('trakt.device.syncing') : t('trakt.device.connected')}
              valueColor="#54D17A"
              onClick={() => setTraktPopoverOpen((o) => !o)}
              busy={traktBusy}
            />
            {traktPopoverOpen && (
              <SyncServicePopover
                logoSrc="/trakt.png"
                serviceName="Trakt.tv"
                meta={traktSyncMeta}
                busy={traktBusy}
                onSyncNow={() => void handleTraktSyncNow()}
                onDisconnect={() => void handleTraktDisconnect()}
                onClose={() => setTraktPopoverOpen(false)}
              />
            )}
          </div>
        )}

        {/* MAL */}
        {!malConnected && (
          <SyncServiceRow
            icon={<div style={{ width: 34, height: 34, borderRadius: 9, background: 'rgba(42,133,255,0.12)', display: 'flex', alignItems: 'center', justifyContent: 'center', overflow: 'hidden' }}><img src="/mal.png" alt="MAL" style={{ width: 26, height: 26, objectFit: 'contain' }} /></div>}
            title="MyAnimeList"
            value={malBusy ? t('trakt.device.waiting') : t('auto.connect_myanimelist_account') || 'Connect MyAnimeList Account'}
            onClick={() => void handleMalConnect()}
            busy={malBusy}
          />
        )}
        {malError && (
          <div style={{ padding: '0 18px 10px', borderBottom: '1px solid rgba(255,255,255,0.055)' }}>
            <p style={{ color: '#FF5D5D', fontSize: 12, margin: 0, fontFamily: 'sans-serif' }}>{t('common.error')}: {malError}</p>
          </div>
        )}
        {malConnected && (
          <SyncServiceRow
            icon={<div style={{ width: 34, height: 34, borderRadius: 9, background: 'rgba(42,133,255,0.12)', display: 'flex', alignItems: 'center', justifyContent: 'center', overflow: 'hidden' }}><img src="/mal.png" alt="MAL" style={{ width: 26, height: 26, objectFit: 'contain' }} /></div>}
            title="MyAnimeList"
            value={t('trakt.device.connected')}
            valueColor="#54D17A"
            onClick={() => void handleMalDisconnect()}
          />
        )}

        {/* Simkl */}
        {!simklConnected && (
          <SyncServiceRow
            icon={<div style={{ width: 34, height: 34, borderRadius: 9, background: 'rgba(28,177,74,0.12)', display: 'flex', alignItems: 'center', justifyContent: 'center', overflow: 'hidden' }}><img src="/simkl.png" alt="Simkl" style={{ width: 26, height: 26, objectFit: 'contain' }} /></div>}
            title="Simkl"
            value={simklBusy ? t('trakt.device.waiting') : 'Connect Simkl Account'}
            onClick={() => void handleSimklConnect()}
            busy={simklBusy}
          />
        )}
        {simklError && (
          <div style={{ padding: '0 18px 10px', borderBottom: '1px solid rgba(255,255,255,0.055)' }}>
            <p style={{ color: '#FF5D5D', fontSize: 12, margin: 0, fontFamily: 'sans-serif' }}>{t('common.error')}: {simklError}</p>
          </div>
        )}
        {simklConnected && (
          <div style={{ position: 'relative' }}>
            <SyncServiceRow
              icon={<div style={{ width: 34, height: 34, borderRadius: 9, background: 'rgba(28,177,74,0.12)', display: 'flex', alignItems: 'center', justifyContent: 'center', overflow: 'hidden' }}><img src="/simkl.png" alt="Simkl" style={{ width: 26, height: 26, objectFit: 'contain' }} /></div>}
              title="Simkl"
              value={simklBusy ? t('trakt.device.syncing') : t('trakt.device.connected')}
              valueColor="#54D17A"
              onClick={() => setSimklPopoverOpen((o) => !o)}
              busy={simklBusy}
            />
            {simklPopoverOpen && (
              <SyncServicePopover
                logoSrc="/simkl.png"
                serviceName="Simkl"
                meta={simklSyncMeta}
                busy={simklBusy}
                onSyncNow={() => void handleSimklSyncNow()}
                onDisconnect={() => void handleSimklDisconnect()}
                onClose={() => setSimklPopoverOpen(false)}
              />
            )}
          </div>
        )}
      </SettingsSection>
    </>
  );
}
