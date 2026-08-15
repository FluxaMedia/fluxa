import React from 'react';
import { platformInvoke } from '../../platform/invoke';
import type { UserProfile } from '../../core/types';
import { t } from '../../i18n';
import { ConfirmDialog } from '../ConfirmDialog';
import { ImportDialog } from '../ImportDialog';
import { PROVIDER_IMPORT_CATEGORIES, type ImportCategory } from '../../core/importCategories';
import { pushImportedCategoriesToDestination } from '../../core/crossProviderPush';
import { syncExternalIntegrationNow } from '../../core/effectRunner';
import { ChoiceTile, SettingsDetailHeader, SettingsSection, SyncServiceRow, ToggleTile } from './SettingsUI';
import type { Prefs } from './settingsTypes';
import { AuthKeyLoginForm, CredentialLoginForm, ProviderTokenStatus, providerIcon, type IntegrationService } from './accountPresentation';
import type { useIntegrationAccounts } from './useIntegrationAccounts';

export function AccountIntegrationDetail({
  selectedIntegration,
  setSelectedIntegration,
  importDialog,
  setImportDialog,
  activeProfile,
  prefs,
  setPref,
  accounts,
}: {
  selectedIntegration: IntegrationService;
  setSelectedIntegration: (service: IntegrationService | null) => void;
  importDialog: IntegrationService | null;
  setImportDialog: (service: IntegrationService | null) => void;
  activeProfile: UserProfile | null;
  prefs: Prefs;
  setPref: <K extends keyof Prefs>(k: K, v: Prefs[K]) => void;
  accounts: ReturnType<typeof useIntegrationAccounts>;
}) {
  const {
    traktBusy, traktError, setTraktError, traktSyncMeta, traktConnected,
    anilistBusy, anilistError, setAnilistError, anilistSyncMeta, anilistConnected,
    simklBusy, simklError, setSimklError, simklSyncMeta, simklConnected,
    nuvioBusy, nuvioError, setNuvioError, nuvioSyncMeta, nuvioConnected, nuvioFormOpen, setNuvioFormOpen,
    stremioBusy, stremioError, setStremioError, stremioSyncMeta, stremioConnected,
    stremioFormOpen, setStremioFormOpen, stremioAuthKeyMode, setStremioAuthKeyMode,
    confirmDisconnect, setConfirmDisconnect,
    renderOAuthFallback,
    handleTraktConnect, handleTraktDisconnect, handleTraktSyncNow,
    handleAnilistConnect, handleAnilistDisconnect, handleAnilistSyncNow,
    handleSimklConnect, handleSimklDisconnect, handleSimklSyncNow,
    handleNuvioConnect, handleNuvioDisconnect, handleNuvioSyncNow,
    handleStremioConnect, handleStremioConnectWithAuthKey, handleStremioDisconnect, handleStremioSyncNow,
  } = accounts;

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
        titleIcon={providerIcon(importDialog)}
        items={PROVIDER_IMPORT_CATEGORIES[importDialog].map((key) => ({ key, label: t(`settings.import_category.${key}`) }))}
        destinations={([
          ['trakt', t('brand.trakt'), traktConnected],
          ['simkl', t('brand.simkl'), simklConnected],
          ['anilist', t('brand.anilist'), anilistConnected],
          ['stremio', t('brand.stremio'), stremioConnected],
          ['nuvio', t('brand.nuvio'), nuvioConnected],
        ] as const)
          .filter(([key, , connected]) => connected && key !== importDialog)
          .map(([key, label]) => ({ key, label, icon: providerIcon(key) }))}
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
          const clientId = provider === 'trakt' ? await platformInvoke<string>('get_oauth_client_id', { service: 'trakt' })
            : provider === 'simkl' ? await platformInvoke<string>('get_oauth_client_id', { service: 'simkl' })
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
            const traktClientId = destination === 'trakt' ? await platformInvoke<string>('get_oauth_client_id', { service: 'trakt' }) : undefined;
            const simklClientId = destination === 'simkl' ? await platformInvoke<string>('get_oauth_client_id', { service: 'simkl' }) : undefined;
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
