import React, { useEffect, useState } from 'react';
import type { UserProfile } from '../../core/types';
import { t } from '../../i18n';
import { ConfirmDialog } from '../ConfirmDialog';
import { AvatarPreview } from '../../screens/ProfileForm';
import { ChoiceTile, SettingsSection, SyncServicePopover, SyncServiceRow, cwRankingOptions, cwSourceOfTruthOptions, similarTitlesSourceOptions } from './SettingsUI';
import type { Prefs } from './settingsTypes';

import { AuthKeyLoginForm, CredentialLoginForm, type IntegrationService } from './accountPresentation';
import { useIntegrationAccounts } from './useIntegrationAccounts';
import { AccountIntegrationDetail } from './AccountIntegrationDetail';

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
  const [selectedIntegration, setSelectedIntegration] = useState<IntegrationService | null>(null);
  const [importDialog, setImportDialog] = useState<IntegrationService | null>(null);

  useEffect(() => {
    if (prefs.syncCwSourceOfTruth === 'most_recent') setPref('syncCwSourceOfTruth', '');
  }, [prefs.syncCwSourceOfTruth, setPref]);

  const accounts = useIntegrationAccounts({ prefs, activeProfile, onProfileUpdated, onDispatch, onNuvioSyncComplete });
  const {
    traktBusy, traktError, traktPopoverOpen, setTraktPopoverOpen, traktRowRef, traktSyncMeta, traktConnected,
    anilistBusy, anilistError, anilistPopoverOpen, setAnilistPopoverOpen, anilistRowRef, anilistSyncMeta, anilistConnected,
    simklBusy, simklError, simklPopoverOpen, setSimklPopoverOpen, simklRowRef, simklSyncMeta, simklConnected,
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
  } = accounts;

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
