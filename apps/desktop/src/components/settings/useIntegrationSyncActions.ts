import { platformInvoke } from '../../platform/invoke';
import { storageWrite } from '../../core/engine';
import { syncExternalIntegrationNow } from '../../core/effectRunner';
import { refreshAnimeTrackingProfile } from '../../core/animeExternalSync';
import { t } from '../../i18n';
import type { UserProfile } from '../../core/types';
import type { ImportCategory } from '../../core/importCategories';
import type { Prefs, SyncMeta } from './settingsTypes';

type SyncResult = {
  synced?: boolean;
  error?: string;
  continueWatchingCount?: number;
  watchlistCount?: number;
  watchedCount?: number;
};

type Options = {
  prefs: Prefs;
  activeProfile: UserProfile | null;
  onProfileUpdated: (profile: UserProfile) => void;
  onDispatch: (actionJson: string) => void | Promise<void>;
  onNuvioSyncComplete?: () => void | Promise<void>;
  setTraktBusy: (value: boolean) => void;
  setTraktError: (value: string | null) => void;
  setTraktSyncMeta: (value: SyncMeta | null) => void;
  setAnilistBusy: (value: boolean) => void;
  setAnilistError: (value: string | null) => void;
  setAnilistSyncMeta: (value: SyncMeta | null) => void;
  setSimklBusy: (value: boolean) => void;
  setSimklError: (value: string | null) => void;
  setSimklSyncMeta: (value: SyncMeta | null) => void;
  setNuvioBusy: (value: boolean) => void;
  setNuvioError: (value: string | null) => void;
  setNuvioSyncMeta: (value: SyncMeta | null) => void;
  setStremioBusy: (value: boolean) => void;
  setStremioError: (value: string | null) => void;
  setStremioSyncMeta: (value: SyncMeta | null) => void;
};

export function useIntegrationSyncActions({
  prefs,
  activeProfile,
  onProfileUpdated,
  onDispatch,
  onNuvioSyncComplete,
  setTraktBusy,
  setTraktError,
  setTraktSyncMeta,
  setAnilistBusy,
  setAnilistError,
  setAnilistSyncMeta,
  setSimklBusy,
  setSimklError,
  setSimklSyncMeta,
  setNuvioBusy,
  setNuvioError,
  setNuvioSyncMeta,
  setStremioBusy,
  setStremioError,
  setStremioSyncMeta,
}: Options) {
  const reloadAfterSync = () => {
    onDispatch(JSON.stringify({ type: 'libraryHydrateRequested' }));
    onDispatch(JSON.stringify({ type: 'homeLoadRequested', force: true, language: prefs.language }));
  };

  const handleTraktSyncNow = async (categories?: ImportCategory[]) => {
    if (!activeProfile?.traktAccessToken) return;
    setTraktBusy(true);
    setTraktError(null);
    try {
      const clientId = await platformInvoke<string>('get_oauth_client_id', { service: 'trakt' });
      const result = (await syncExternalIntegrationNow({
        provider: 'trakt',
        profile: activeProfile,
        token: activeProfile.traktAccessToken,
        clientId,
        ...(categories ? { categories } : {}),
      })) as SyncResult;
      if (!result.synced) {
        setTraktError(result.error ?? t('toast.trakt_sync_failed'));
      } else {
        const meta: SyncMeta = {
          lastSyncAt: Date.now(),
          continueWatchingCount: result.continueWatchingCount ?? 0,
          watchlistCount: result.watchlistCount ?? 0,
          watchedCount: result.watchedCount ?? 0,
        };
        setTraktSyncMeta(meta);
        await storageWrite('trakt_sync_meta', meta);
      }
    } catch (error) {
      setTraktError(error instanceof Error ? error.message : String(error));
    } finally {
      setTraktBusy(false);
    }
    reloadAfterSync();
  };

  const handleSimklSyncNow = async (categories?: ImportCategory[]) => {
    if (!activeProfile?.simklAccessToken) return;
    setSimklBusy(true);
    setSimklError(null);
    try {
      const clientId = await platformInvoke<string>('get_oauth_client_id', { service: 'simkl' });
      const result = (await syncExternalIntegrationNow({
        provider: 'simkl',
        profile: activeProfile,
        token: activeProfile.simklAccessToken,
        clientId,
        ...(categories ? { categories } : {}),
      })) as SyncResult;
      if (!result.synced) {
        setSimklError(result.error ?? 'Simkl sync failed');
      } else {
        const meta: SyncMeta = {
          lastSyncAt: Date.now(),
          continueWatchingCount: result.continueWatchingCount ?? 0,
          watchlistCount: result.watchlistCount ?? 0,
          watchedCount: result.watchedCount ?? 0,
        };
        setSimklSyncMeta(meta);
        await storageWrite('simkl_sync_meta', meta);
      }
    } catch (error) {
      setSimklError(error instanceof Error ? error.message : String(error));
    } finally {
      setSimklBusy(false);
    }
    reloadAfterSync();
  };

  const handleNuvioSyncNow = async (categories?: ImportCategory[]) => {
    if (!activeProfile?.nuvioAccessToken && !activeProfile?.nuvioRefreshToken) return;
    setNuvioBusy(true);
    setNuvioError(null);
    try {
      const result = (await syncExternalIntegrationNow({
        provider: 'nuvio',
        profile: activeProfile,
        token: activeProfile.nuvioAccessToken,
        ...(categories ? { categories } : {}),
      })) as SyncResult & { profile?: UserProfile };
      if (!result.synced) {
        setNuvioError(result.error ?? 'Nuvio sync failed');
      } else {
        if (result.profile) onProfileUpdated(result.profile);
        const meta: SyncMeta = {
          lastSyncAt: Date.now(),
          continueWatchingCount: result.continueWatchingCount ?? 0,
          watchlistCount: result.watchlistCount ?? 0,
        };
        setNuvioSyncMeta(meta);
        await storageWrite('nuvio_sync_meta', meta);
        await onNuvioSyncComplete?.();
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
      const result = (await syncExternalIntegrationNow({
        provider: 'stremio',
        profile: activeProfile,
        token: activeProfile.stremioAuthKey,
        ...(categories ? { categories } : {}),
      })) as SyncResult;
      if (!result.synced) {
        setStremioError(result.error ?? 'Stremio sync failed');
      } else {
        const meta: SyncMeta = {
          lastSyncAt: Date.now(),
          continueWatchingCount: result.continueWatchingCount ?? 0,
          watchlistCount: result.watchlistCount ?? 0,
        };
        setStremioSyncMeta(meta);
        await storageWrite('stremio_sync_meta', meta);
      }
    } catch (error) {
      setStremioError(error instanceof Error ? error.message : String(error));
    } finally {
      setStremioBusy(false);
    }
    reloadAfterSync();
  };

  const handleAnilistSyncNow = async (categories?: ImportCategory[]) => {
    if (!activeProfile?.anilistAccessToken) return;
    setAnilistBusy(true);
    setAnilistError(null);
    try {
      const updated = await refreshAnimeTrackingProfile(activeProfile);
      if (updated !== activeProfile) onProfileUpdated(updated);
      const result = (await syncExternalIntegrationNow({
        provider: 'anilist',
        profile: updated,
        token: updated.anilistAccessToken,
        ...(categories ? { categories } : {}),
      })) as SyncResult;
      if (!result.synced) {
        setAnilistError(result.error ?? 'AniList sync failed');
        return;
      }
      const meta: SyncMeta = {
        lastSyncAt: Date.now(),
        continueWatchingCount: result.continueWatchingCount ?? 0,
        watchlistCount: result.watchlistCount ?? 0,
      };
      setAnilistSyncMeta(meta);
      await storageWrite('anilist_sync_meta', meta);
    } catch (error) {
      setAnilistError(error instanceof Error ? error.message : String(error));
    } finally {
      setAnilistBusy(false);
    }
    reloadAfterSync();
  };

  return {
    handleTraktSyncNow,
    handleSimklSyncNow,
    handleNuvioSyncNow,
    handleStremioSyncNow,
    handleAnilistSyncNow,
  };
}
