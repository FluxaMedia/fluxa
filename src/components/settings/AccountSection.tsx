import React, { useEffect, useState } from 'react';
import type { UserProfile } from '../../core/types';
import { t } from '../../i18n';
import { ConfirmDialog } from '../ConfirmDialog';
import { AvatarPreview } from '../../screens/ProfileForm';
import {
  ChoiceTile,
  SettingsSection,
  SyncServicePopover,
  SyncServiceRow,
  cwSourceOfTruthOptions,
  librarySourceOfTruthOptions,
  similarTitlesSourceOptions,
} from './SettingsUI';
import type { Prefs } from './settingsTypes';

import { AuthKeyLoginForm, CredentialLoginForm, type IntegrationService } from './accountPresentation';
import { useIntegrationAccounts } from './useIntegrationAccounts';
import { AccountIntegrationDetail } from './AccountIntegrationDetail';
import { assetUrl } from '../../platform/assets';

export function AccountSection({
  prefs,
  setPref,
  prefsLoaded,
  activeProfile,
  onProfileUpdated,
  onSwitchProfile,
  onDispatch,
  onNuvioSyncComplete,
}: {
  prefs: Prefs;
  setPref: <K extends keyof Prefs>(k: K, v: Prefs[K]) => void;
  prefsLoaded: boolean;
  activeProfile: UserProfile | null;
  onProfileUpdated: (profile: UserProfile) => void;
  onSwitchProfile: () => void;
  onDispatch: (actionJson: string) => void | Promise<void>;
  onNuvioSyncComplete?: () => void | Promise<void>;
}) {
  const [selectedIntegration, setSelectedIntegration] = useState<IntegrationService | null>(null);
  const [importDialog, setImportDialog] = useState<IntegrationService | null>(null);

  const accounts = useIntegrationAccounts({ prefs, activeProfile, onProfileUpdated, onDispatch, onNuvioSyncComplete });
  const {
    traktBusy,
    traktError,
    traktPopoverOpen,
    setTraktPopoverOpen,
    traktRowRef,
    traktSyncMeta,
    traktConnected,
    anilistBusy,
    anilistError,
    anilistPopoverOpen,
    setAnilistPopoverOpen,
    anilistRowRef,
    anilistSyncMeta,
    anilistConnected,
    simklBusy,
    simklError,
    simklPopoverOpen,
    setSimklPopoverOpen,
    simklRowRef,
    simklSyncMeta,
    simklConnected,
    nuvioBusy,
    nuvioError,
    setNuvioError,
    nuvioPopoverOpen,
    setNuvioPopoverOpen,
    nuvioRowRef,
    nuvioSyncMeta,
    nuvioConnected,
    nuvioFormOpen,
    setNuvioFormOpen,
    stremioBusy,
    stremioError,
    setStremioError,
    stremioPopoverOpen,
    setStremioPopoverOpen,
    stremioRowRef,
    stremioSyncMeta,
    stremioConnected,
    stremioFormOpen,
    setStremioFormOpen,
    stremioAuthKeyMode,
    setStremioAuthKeyMode,
    confirmDisconnect,
    setConfirmDisconnect,
    renderOAuthFallback,
    handleTraktConnect,
    handleTraktDisconnect,
    handleTraktSyncNow,
    handleAnilistConnect,
    handleAnilistDisconnect,
    handleAnilistSyncNow,
    handleSimklConnect,
    handleSimklDisconnect,
    handleSimklSyncNow,
    handleNuvioConnect,
    handleNuvioDisconnect,
    handleNuvioSyncNow,
    handleStremioConnect,
    handleStremioConnectWithAuthKey,
    handleStremioDisconnect,
    handleStremioSyncNow,
  } = accounts;

  const connectedSources = {
    nuvio: nuvioConnected,
    trakt: traktConnected,
    simkl: simklConnected,
    anilist: anilistConnected,
    stremio: stremioConnected,
  };
  const cwOptions = cwSourceOfTruthOptions(connectedSources);
  const libraryOptions = librarySourceOfTruthOptions(connectedSources);
  const preferredConnectedSource = nuvioConnected
    ? 'nuvio'
    : traktConnected
      ? 'trakt'
      : simklConnected
        ? 'simkl'
        : anilistConnected
          ? 'anilist'
          : stremioConnected
            ? 'stremio'
            : 'local';
  const isConnectedSource = (source: string) =>
    source === 'local' ? preferredConnectedSource === 'local' : Boolean(connectedSources[source as keyof typeof connectedSources]);
  const validSource = (source: string) => (isConnectedSource(source) ? source : preferredConnectedSource);

  useEffect(() => {
    if (!prefsLoaded) return;
    if (!isConnectedSource(prefs.continueWatchingSource)) void setPref('continueWatchingSource', preferredConnectedSource);
    if (!isConnectedSource(prefs.integrationLibrarySource)) void setPref('integrationLibrarySource', preferredConnectedSource);
  }, [
    prefsLoaded,
    prefs.continueWatchingSource,
    prefs.integrationLibrarySource,
    nuvioConnected,
    traktConnected,
    simklConnected,
    anilistConnected,
    stremioConnected,
    setPref,
  ]);

  if (selectedIntegration) {
    return (
      <AccountIntegrationDetail
        selectedIntegration={selectedIntegration}
        setSelectedIntegration={setSelectedIntegration}
        importDialog={importDialog}
        setImportDialog={setImportDialog}
        activeProfile={activeProfile}
        prefs={prefs}
        setPref={setPref}
        accounts={accounts}
      />
    );
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

          {/* Nuvio */}
          {!nuvioConnected && !stremioConnected && (
            <SyncServiceRow
              icon={
                <div
                  style={{
                    width: '2.75rem',
                    height: '2.75rem',
                    borderRadius: '0.75rem',
                    background: 'rgba(255,255,255,0.06)',
                    display: 'flex',
                    alignItems: 'center',
                    justifyContent: 'center',
                    overflow: 'hidden',
                  }}
                >
                  <img src={assetUrl('nuvio.png')} alt="Nuvio" style={{ width: '2rem', height: '2rem', objectFit: 'contain' }} />
                </div>
              }
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
              onCancel={() => {
                setNuvioFormOpen(false);
                setNuvioError(null);
              }}
            />
          )}
          {nuvioError && (
            <div style={{ padding: '0 1.125rem 0.625rem', borderBottom: '1px solid rgba(255,255,255,0.055)' }}>
              <p
                style={{
                  color: '#FF5D5D',
                  fontSize: '0.75rem',
                  margin: 0,
                  fontFamily: '-apple-system, BlinkMacSystemFont, "Segoe UI", "Ubuntu", "Noto Sans", sans-serif',
                }}
              >
                {t('common.error')}: {nuvioError}
              </p>
            </div>
          )}
          {nuvioConnected && (
            <div ref={nuvioRowRef} style={{ position: 'relative' }}>
              <SyncServiceRow
                icon={
                  <div
                    style={{
                      width: '2.75rem',
                      height: '2.75rem',
                      borderRadius: '0.75rem',
                      background: 'rgba(255,255,255,0.06)',
                      display: 'flex',
                      alignItems: 'center',
                      justifyContent: 'center',
                      overflow: 'hidden',
                    }}
                  >
                    <img src={assetUrl('nuvio.png')} alt="Nuvio" style={{ width: '2rem', height: '2rem', objectFit: 'contain' }} />
                  </div>
                }
                title={t('settings.signed_in_as', 'Nuvio')}
                value={
                  nuvioBusy
                    ? t('sync.device.syncing')
                    : activeProfile?.nuvioEmail
                      ? activeProfile.nuvioEmail
                      : t('sync.device.connected')
                }
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
          {!stremioConnected && !nuvioConnected && (
            <SyncServiceRow
              icon={
                <div
                  style={{
                    width: '2.75rem',
                    height: '2.75rem',
                    borderRadius: '0.75rem',
                    background: 'rgba(123,91,245,0.12)',
                    display: 'flex',
                    alignItems: 'center',
                    justifyContent: 'center',
                    overflow: 'hidden',
                  }}
                >
                  <img src={assetUrl('stremio.svg')} alt="Stremio" style={{ width: '2rem', height: '2rem', objectFit: 'contain' }} />
                </div>
              }
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
              onCancel={() => {
                setStremioFormOpen(false);
                setStremioError(null);
              }}
            />
          )}
          {!stremioConnected && stremioFormOpen && (
            <div
              style={{
                padding: stremioAuthKeyMode ? '0 1.125rem' : '0.5rem 1.125rem 0',
                borderBottom: stremioAuthKeyMode ? undefined : '1px solid rgba(255,255,255,0.055)',
              }}
            >
              <button
                onClick={() => {
                  setStremioAuthKeyMode((m) => !m);
                  setStremioError(null);
                }}
                disabled={stremioBusy}
                style={{
                  background: 'none',
                  border: 'none',
                  color: 'rgba(255,255,255,0.5)',
                  fontSize: '0.6875rem',
                  cursor: 'pointer',
                  padding: 0,
                  marginBottom: stremioAuthKeyMode ? 0 : '0.5rem',
                }}
              >
                {stremioAuthKeyMode ? t('auth.stremio.use_password_instead') : t('auth.stremio.use_authkey_instead')}
              </button>
            </div>
          )}
          {!stremioConnected && stremioFormOpen && stremioAuthKeyMode && (
            <AuthKeyLoginForm
              busy={stremioBusy}
              onSubmit={(authKey) => void handleStremioConnectWithAuthKey(authKey)}
              onCancel={() => {
                setStremioFormOpen(false);
                setStremioAuthKeyMode(false);
                setStremioError(null);
              }}
            />
          )}
          {stremioError && (
            <div style={{ padding: '0 1.125rem 0.625rem', borderBottom: '1px solid rgba(255,255,255,0.055)' }}>
              <p
                style={{
                  color: '#FF5D5D',
                  fontSize: '0.75rem',
                  margin: 0,
                  fontFamily: '-apple-system, BlinkMacSystemFont, "Segoe UI", "Ubuntu", "Noto Sans", sans-serif',
                }}
              >
                {t('common.error')}: {stremioError}
              </p>
            </div>
          )}
          {stremioConnected && (
            <div ref={stremioRowRef} style={{ position: 'relative' }}>
              <SyncServiceRow
                icon={
                  <div
                    style={{
                      width: '2.75rem',
                      height: '2.75rem',
                      borderRadius: '0.75rem',
                      background: 'rgba(123,91,245,0.12)',
                      display: 'flex',
                      alignItems: 'center',
                      justifyContent: 'center',
                      overflow: 'hidden',
                    }}
                  >
                    <img src={assetUrl('stremio.svg')} alt="Stremio" style={{ width: '2rem', height: '2rem', objectFit: 'contain' }} />
                  </div>
                }
                title={t('settings.signed_in_as', 'Stremio')}
                value={
                  stremioBusy
                    ? t('sync.device.syncing')
                    : activeProfile?.stremioEmail
                      ? activeProfile.stremioEmail
                      : t('sync.device.connected')
                }
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
      )}

      <SettingsSection title={t('settings.sync_with')} subtitle={t('settings.sync_with_desc')}>
        {/* Trakt */}
        {!traktConnected && (
          <SyncServiceRow
            icon={
              <div
                style={{
                  width: '2.75rem',
                  height: '2.75rem',
                  borderRadius: '0.75rem',
                  background: 'rgba(237,28,36,0.12)',
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'center',
                  overflow: 'hidden',
                }}
              >
                <img src={assetUrl('trakt.svg')} alt="Trakt" style={{ width: '2.125rem', height: '2.125rem', objectFit: 'contain' }} />
              </div>
            }
            title="Trakt.tv"
            value={traktBusy ? t('trakt.device.waiting') : t('auto.connect_trakt_tv_account')}
            onClick={() => setSelectedIntegration('trakt')}
            busy={traktBusy}
          />
        )}
        {!traktConnected && renderOAuthFallback('trakt')}
        {traktError && (
          <div style={{ padding: '0 1.125rem 0.625rem', borderBottom: '1px solid rgba(255,255,255,0.055)' }}>
            <p
              style={{
                color: '#FF5D5D',
                fontSize: '0.75rem',
                margin: 0,
                fontFamily: '-apple-system, BlinkMacSystemFont, "Segoe UI", "Ubuntu", "Noto Sans", sans-serif',
              }}
            >
              {t('common.error')}: {traktError}
            </p>
          </div>
        )}
        {traktConnected && (
          <div ref={traktRowRef} style={{ position: 'relative' }}>
            <SyncServiceRow
              icon={
                <div
                  style={{
                    width: '2.75rem',
                    height: '2.75rem',
                    borderRadius: '0.75rem',
                    background: 'rgba(237,28,36,0.12)',
                    display: 'flex',
                    alignItems: 'center',
                    justifyContent: 'center',
                    overflow: 'hidden',
                  }}
                >
                  <img src={assetUrl('trakt.svg')} alt="Trakt" style={{ width: '2.125rem', height: '2.125rem', objectFit: 'contain' }} />
                </div>
              }
              title="Trakt.tv"
              value={
                traktBusy
                  ? t('sync.device.syncing')
                  : activeProfile?.traktUsername
                    ? t('settings.connected_as', activeProfile.traktUsername)
                    : t('sync.device.connected')
              }
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
            icon={
              <div
                style={{
                  width: '2.75rem',
                  height: '2.75rem',
                  borderRadius: '0.75rem',
                  background: 'rgba(2,169,255,0.12)',
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'center',
                  overflow: 'hidden',
                }}
              >
                <img src={assetUrl('anilist.svg')} alt="AniList" style={{ width: '2.125rem', height: '2.125rem', objectFit: 'contain' }} />
              </div>
            }
            title="AniList"
            value={anilistBusy ? t('trakt.device.waiting') : t('auto.connect_anilist_account')}
            onClick={() => setSelectedIntegration('anilist')}
            busy={anilistBusy}
          />
        )}
        {!anilistConnected && renderOAuthFallback('anilist')}
        {anilistError && (
          <div style={{ padding: '0 1.125rem 0.625rem', borderBottom: '1px solid rgba(255,255,255,0.055)' }}>
            <p
              style={{
                color: '#FF5D5D',
                fontSize: '0.75rem',
                margin: 0,
                fontFamily: '-apple-system, BlinkMacSystemFont, "Segoe UI", "Ubuntu", "Noto Sans", sans-serif',
              }}
            >
              {t('common.error')}: {anilistError}
            </p>
          </div>
        )}
        {anilistConnected && (
          <div ref={anilistRowRef} style={{ position: 'relative' }}>
            <SyncServiceRow
              icon={
                <div
                  style={{
                    width: '2.75rem',
                    height: '2.75rem',
                    borderRadius: '0.75rem',
                    background: 'rgba(2,169,255,0.12)',
                    display: 'flex',
                    alignItems: 'center',
                    justifyContent: 'center',
                    overflow: 'hidden',
                  }}
                >
                  <img
                    src={assetUrl('anilist.svg')}
                    alt="AniList"
                    style={{ width: '2.125rem', height: '2.125rem', objectFit: 'contain' }}
                  />
                </div>
              }
              title="AniList"
              value={
                anilistBusy
                  ? t('sync.device.syncing')
                  : activeProfile?.anilistUsername
                    ? t('settings.connected_as', activeProfile.anilistUsername)
                    : t('settings.anime_tracking_enabled')
              }
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
              statusLabel={
                anilistSyncMeta
                  ? `${t('settings.anime_tracking_enabled')} · ${new Date(anilistSyncMeta.lastSyncAt).toLocaleString()}`
                  : t('settings.anime_tracking_enabled')
              }
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
            icon={
              <div
                style={{
                  width: '2.75rem',
                  height: '2.75rem',
                  borderRadius: '0.75rem',
                  background: 'rgba(255,255,255,0.06)',
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'center',
                  overflow: 'hidden',
                }}
              >
                <img src={assetUrl('simkl.svg')} alt="Simkl" style={{ width: '2.125rem', height: '2.125rem', objectFit: 'contain' }} />
              </div>
            }
            title="Simkl"
            value={simklBusy ? t('trakt.device.waiting') : t('auto.connect_simkl_account')}
            onClick={() => setSelectedIntegration('simkl')}
            busy={simklBusy}
          />
        )}
        {!simklConnected && renderOAuthFallback('simkl')}
        {simklError && (
          <div style={{ padding: '0 1.125rem 0.625rem', borderBottom: '1px solid rgba(255,255,255,0.055)' }}>
            <p
              style={{
                color: '#FF5D5D',
                fontSize: '0.75rem',
                margin: 0,
                fontFamily: '-apple-system, BlinkMacSystemFont, "Segoe UI", "Ubuntu", "Noto Sans", sans-serif',
              }}
            >
              {t('common.error')}: {simklError}
            </p>
          </div>
        )}
        {simklConnected && (
          <div ref={simklRowRef} style={{ position: 'relative' }}>
            <SyncServiceRow
              icon={
                <div
                  style={{
                    width: '2.75rem',
                    height: '2.75rem',
                    borderRadius: '0.75rem',
                    background: 'rgba(255,255,255,0.06)',
                    display: 'flex',
                    alignItems: 'center',
                    justifyContent: 'center',
                    overflow: 'hidden',
                  }}
                >
                  <img src={assetUrl('simkl.svg')} alt="Simkl" style={{ width: '2.125rem', height: '2.125rem', objectFit: 'contain' }} />
                </div>
              }
              title="Simkl"
              value={
                simklBusy
                  ? t('sync.device.syncing')
                  : activeProfile?.simklUsername
                    ? t('settings.connected_as', activeProfile.simklUsername)
                    : t('sync.device.connected')
              }
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
      </SettingsSection>

      <SettingsSection title={t('settings.cw_conflict_resolution')} subtitle={t('settings.cw_conflict_resolution_desc')}>
        <ChoiceTile
          title={t('settings.cw_source_of_truth')}
          subtitle={t('settings.cw_source_of_truth_desc')}
          options={cwOptions}
          selected={validSource(prefs.continueWatchingSource)}
          onSelect={(value) => setPref('continueWatchingSource', value)}
        />
        <ChoiceTile
          title={t('settings.library_source_of_truth')}
          subtitle={t('settings.library_source_of_truth_desc')}
          options={libraryOptions}
          selected={validSource(prefs.integrationLibrarySource)}
          onSelect={(value) => setPref('integrationLibrarySource', value)}
        />
        <ChoiceTile
          title={t('settings.similar_titles_source')}
          subtitle={t('settings.similar_titles_source_desc')}
          options={similarTitlesSourceOptions()}
          selected={prefs.similarTitlesSource}
          onSelect={(value) => setPref('similarTitlesSource', value)}
        />
      </SettingsSection>

      {confirmDisconnect && (
        <ConfirmDialog
          title={t('settings.disconnect_confirm_title', confirmDisconnect.title)}
          body={t('settings.disconnect_confirm_body', confirmDisconnect.title)}
          confirmLabel={t('auto.disconnect')}
          cancelLabel={t('common.cancel')}
          destructive
          onCancel={() => setConfirmDisconnect(null)}
          onConfirm={() => {
            const { onConfirm } = confirmDisconnect;
            setConfirmDisconnect(null);
            onConfirm();
          }}
        />
      )}
    </>
  );
}
