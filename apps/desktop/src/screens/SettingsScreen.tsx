import React, { useEffect, useState } from 'react';
import { platformInvoke as invoke } from '../platform/invoke';
import { coreApplyPreferenceUpdate, coreInvoke, storageRead, storageWrite } from '../core/engine';
import { Gamepad2, Keyboard, Search } from 'lucide-react';
import { coreAddonCollectionMutationPlan, loadAddonManifestFromUrl, normalizeManifestUrl } from '../core/addonManifest';
import type { AddonDescriptor, AppState, PluginRepository, PluginScraper, UserProfile } from '../core/types';
import { addonKey, normalizeAddonDescriptor } from '../core/addons';
import { saveProfile } from '../core/profiles';
import { loadAddons, loadEnabledAddons, loadPrefs, saveAddons, savePrefs } from '../core/libraryOps';
import { pluginRepositoryUrlsKey, pluginScraperEnabledKey, pushPluginsToNuvio } from '../core/pluginsStorage';
import { nuvioReplaceAddons } from '../core/nuvioApi';
import { freshNuvioProfile } from '../core/nuvioSync';
import { syncStremioAddons } from '../core/stremioExternalSync';
import { setLanguage, t } from '../i18n';
import { styles } from '../components/settings/settingsStyles';
import { DEFAULT_PREFS } from '../components/settings/settingsTypes';
import type { Prefs, Tab } from '../components/settings/settingsTypes';
import {
  AccountIcon,
  ArrowBackIcon,
  CatalogsIcon,
  DeviceCapabilitiesIcon,
  DownloadIcon,
  ExtensionIcon,
  PaletteIcon,
  PlayCircleIcon,
  PluginIcon,
  RefreshIcon,
  SettingsIcon,
  SidebarDivider,
  SidebarItem,
  SettingsDetailHeader,
  VersionFooter,
} from '../components/settings/SettingsUI';
import { AccountSection } from '../components/settings/AccountSection';
import { GeneralSection } from '../components/settings/GeneralSection';
import { AppearanceSection } from '../components/settings/AppearanceSection';
import { PlaybackSection } from '../components/settings/PlaybackSection';
import { DeviceCapabilitiesSection } from '../components/settings/DeviceCapabilitiesSection';
import { ShortcutsSection } from '../components/settings/ShortcutsSection';
import { ControllerSection } from '../components/settings/ControllerSection';
import { ContentSection } from '../components/settings/ContentSection';
import { AddonsSection } from '../components/settings/AddonsSection';
import { PluginsSection } from '../components/settings/PluginsSection';
import { DownloadsSection } from '../components/settings/DownloadsSection';
import { AddonAddedDialog } from '../components/AddonAddedDialog';
import { Toast } from '../components/Toast';
import { isBrowserTarget } from '../platform/browser';

const TABS: { id: Tab; labelKey: string; subtitleKey: string; icon: React.ReactNode }[] = [
  { id: 'account', labelKey: 'auto.account_sync', subtitleKey: 'auto.account_devices_and_sync', icon: <AccountIcon /> },
  { id: 'general', labelKey: 'auto.general', subtitleKey: 'auto.language_theme_startup', icon: <SettingsIcon /> },
  { id: 'appearance', labelKey: 'auto.appearance', subtitleKey: 'auto.color_and_layout', icon: <PaletteIcon /> },
  { id: 'playback', labelKey: 'auto.playback', subtitleKey: 'auto.player_behavior_and_defaults', icon: <PlayCircleIcon /> },
  { id: 'device', labelKey: 'settings.device_capabilities', subtitleKey: 'settings.device_capabilities_desc', icon: <DeviceCapabilitiesIcon /> },
  { id: 'shortcuts', labelKey: 'settings.shortcuts_tab', subtitleKey: 'settings.shortcuts_tab_desc', icon: <Keyboard size={22} /> },
  { id: 'controller', labelKey: 'settings.controller_tab', subtitleKey: 'settings.controller_tab_desc', icon: <Gamepad2 size={22} /> },
  { id: 'content', labelKey: 'auto.catalogs', subtitleKey: 'auto.categories_sources_and_ranking', icon: <CatalogsIcon /> },
  { id: 'addons', labelKey: 'auto.add_ons', subtitleKey: 'auto.installed_add_ons_and_settings', icon: <ExtensionIcon /> },
  { id: 'plugins', labelKey: 'plugins.title', subtitleKey: 'plugins.subtitle', icon: <PluginIcon /> },
  { id: 'downloads', labelKey: 'auto.downloads', subtitleKey: 'auto.download_and_storage_settings', icon: <DownloadIcon /> },
];

const DESKTOP_ONLY_TABS = new Set<Tab>(['downloads']);

const VISIBLE_TABS = TABS.filter((entry) => !isBrowserTarget() || !DESKTOP_ONLY_TABS.has(entry.id));

const SETTINGS_SEARCH_TERMS: Record<Tab, string[]> = {
  account: ['profile', 'sync', 'trakt', 'simkl', 'anilist', 'nuvio', 'devices'],
  general: ['language', 'startup', 'start page', 'background playback', 'notifications', 'discord'],
  appearance: ['accent', 'color', 'theme', 'poster', 'layout', 'navigation', 'hero', 'continue watching', 'animations'],
  playback: [
    'player',
    'mpv',
    'pip',
    'hdr',
    'p2p',
    'speed',
    'seek',
    'subtitles',
    'audio',
    'skip intro',
    'skip outro',
    'auto skip',
    'buffer',
    'decoder',
  ],
  device: ['device', 'audio', 'video', 'codec', 'decoder', 'passthrough', 'pcm', 'hdr'],
  shortcuts: ['keyboard', 'shortcuts', 'keybindings', 'hotkeys', 'rebind'],
  controller: ['controller', 'gamepad', 'joystick', 'xbox', 'playstation', 'nintendo', 'rebind'],
  content: ['catalog', 'home', 'ranking', 'top 10', 'tmdb', 'rpdb', 'omdb', 'fanart', 'episodes'],
  addons: ['addons', 'manifest', 'install', 'remove', 'reorder', 'source'],
  plugins: ['plugins', 'scrapers', 'repository', 'manifest', 'install', 'remove', 'source'],
  downloads: ['download', 'storage', 'folder', 'subtitles'],
};

interface Props {
  state: Pick<AppState, 'addons' | 'plugins'>;
  onDispatch: (actionJson: string) => void | Promise<void>;
  activeProfile: UserProfile | null;
  onProfileUpdated: (profile: UserProfile) => void;
  onSwitchProfile: () => void;
  onBack: () => void;
  onCheckForUpdates: () => void;
  initialAddonUrl?: string | null;
}

export function SettingsScreen({
  state,
  onDispatch,
  activeProfile,
  onProfileUpdated,
  onSwitchProfile,
  onBack,
  onCheckForUpdates,
  initialAddonUrl,
}: Props) {
  const [tab, setTab] = useState<Tab>('account');
  const [prefs, setPrefs] = useState<Prefs>(DEFAULT_PREFS);
  const [addonUrl, setAddonUrl] = useState('');
  const [pluginUrl, setPluginUrl] = useState('');
  const [settingsQuery, setSettingsQuery] = useState('');
  const [installedAddons, setInstalledAddons] = useState<AddonDescriptor[]>([]);
  const [addonInstallStatus, setAddonInstallStatus] = useState<{ loading: boolean; error: string | null }>({ loading: false, error: null });
  const [addedAddonName, setAddedAddonName] = useState<string | null>(null);
  const [pluginInstallLoading, setPluginInstallLoading] = useState(false);
  const [pluginInstallError, setPluginInstallError] = useState<string | null>(null);
  const [inAppUpdatesSupported, setInAppUpdatesSupported] = useState(true);
  const [prefsLoaded, setPrefsLoaded] = useState(false);

  useEffect(() => {
    invoke<boolean>('in_app_updates_supported')
      .then(setInAppUpdatesSupported)
      .catch(() => undefined);
  }, []);

  useEffect(() => {
    loadPrefs().then((raw) => {
      const p = Object.keys(raw).length > 0 ? (raw as unknown as Prefs) : undefined;
      const legacyAnimeQuality =
        p && ['anime4k_s', 'anime4k_m', 'anime4k_l'].includes(p.animeUpscalingMode) ? p.animeUpscalingMode : undefined;
      const merged = p
        ? {
            ...DEFAULT_PREFS,
            ...p,
            ...(legacyAnimeQuality ? { animeUpscalingMode: 'auto', animeUpscalingQuality: legacyAnimeQuality } : {}),
          }
        : DEFAULT_PREFS;
      if (p) setLanguage(merged.language);
      else setLanguage(DEFAULT_PREFS.language);
      setPrefs(merged);
      if (legacyAnimeQuality) void savePrefs(merged as unknown as Record<string, unknown>);
      if (!p?.hdrDetectionCompleted) {
        void invoke<boolean>('player_hdr_supported')
          .then((supported) => {
            const detected = { ...merged, hdrEnabled: supported, hdrDetectionCompleted: true };
            setPrefs(detected);
            void savePrefs(detected);
          })
          .catch(() => undefined);
      }
      if (!isBrowserTarget()) {
        void invoke('player_set_seek_thumbnail_enabled', { enabled: merged.seekThumbnailEnabled });
      }
      setPrefsLoaded(true);
    });
    loadEnabledAddons().then((a) => setInstalledAddons(a));
  }, []);

  const reloadInstalledAddons = async () => {
    setInstalledAddons(await loadEnabledAddons());
  };

  const syncNuvioAddons = async (profile: UserProfile | null | undefined, addons: AddonDescriptor[]) => {
    if (!profile?.nuvioAccessToken || !profile.nuvioUserId) return;
    try {
      const freshProfile = await freshNuvioProfile(profile);
      if (!freshProfile.nuvioAccessToken || !freshProfile.nuvioUserId) return;
      await nuvioReplaceAddons(
        freshProfile.nuvioAccessToken,
        freshProfile.nuvioUserId,
        freshProfile.nuvioProfileIndex ?? 1,
        addons.map((addon, index) => ({
          url: addon.transportUrl,
          name: addon.manifest?.name,
          enabled: !(freshProfile.addonSettings?.disabledLocalAddons ?? freshProfile.disabledLocalAddons ?? []).includes(addonKey(addon)),
          sort_order: index,
        })),
      );
      if (freshProfile !== profile) onProfileUpdated(freshProfile);
    } catch {}
  };

  const syncStremioAddonsForProfile = async (profile: UserProfile | null | undefined, addons: AddonDescriptor[]) => {
    if (!profile?.stremioAuthKey) return;
    try {
      await syncStremioAddons(profile, addons);
    } catch {}
  };

  useEffect(() => {
    const url = initialAddonUrl?.trim();
    if (!url) return;
    setAddonUrl(url);
    setTab('addons');
  }, [initialAddonUrl]);

  const engineAddons = state.addons.installed ?? [];
  useEffect(() => {
    if (engineAddons.length > 0) {
      loadAddons().then((stored) => {
        coreAddonCollectionMutationPlan({ existing: stored, incoming: engineAddons })
          .then((plan) => (plan?.addons as AddonDescriptor[] | undefined) ?? stored)
          .then((merged) => {
            setInstalledAddons(merged);
          });
      });
    }
  }, [engineAddons]);

  const setPref = async <K extends keyof Prefs>(key: K, value: Prefs[K]) => {
    const previous = prefs;
    const optimistic = { ...prefs, [key]: value } as Prefs;
    if (key === 'language') setLanguage(String(value));
    setPrefs(optimistic);
    try {
      const planned = await coreApplyPreferenceUpdate({ existing: previous, key, value });
      const updated = { ...previous, ...(planned ?? { [key]: value }) };
      if (key === 'language') setLanguage(String(updated.language));
      setPrefs(updated as Prefs);
      await savePrefs(updated);
      onDispatch(JSON.stringify({ type: 'settingsChanged', key, value }));
      if (
        key === 'heroFeedToggles' ||
        key === 'homeFeedToggles' ||
        key === 'topTenFeedToggles' ||
        key === 'heroFeedOrder' ||
        key === 'homeFeedOrder' ||
        key === 'showHeroSection'
      ) {
        onDispatch(
          JSON.stringify({
            type: 'homeLoadRequested',
            force: true,
            language: String(updated.language ?? prefs.language),
            profile: activeProfile ?? null,
          }),
        );
      }
      if (key === 'continueWatchingSource') {
        await onDispatch(
          JSON.stringify({
            type: 'refreshContinueWatchingRequested',
            language: String(updated.language ?? prefs.language),
            source: String(value),
          }),
        );
      }
    } catch (e) {
      if (key === 'language') setLanguage(String(previous.language));
      setPrefs(previous);
    }
  };

  useEffect(() => {
    if (!prefsLoaded || !activeProfile?.nuvioAccessToken) return;
    if (prefs.continueWatchingSource !== 'nuvio') void setPref('continueWatchingSource', 'nuvio');
    if (prefs.integrationLibrarySource !== 'nuvio') void setPref('integrationLibrarySource', 'nuvio');
  }, [prefsLoaded, activeProfile?.id, activeProfile?.nuvioAccessToken]);

  const handleInstall = async () => {
    const rawUrl = addonUrl.trim();
    if (!rawUrl || addonInstallStatus.loading) return;
    setAddonInstallStatus({ loading: true, error: null });
    try {
      const addon = await loadAddonManifestFromUrl(rawUrl);
      const normalizedUrl = await normalizeManifestUrl(addon.transportUrl || rawUrl);
      const normalizedAddon = await normalizeAddonDescriptor({ ...addon, transportUrl: normalizedUrl });
      const stored = await loadAddons();
      const plan = await coreAddonCollectionMutationPlan({ existing: stored, incoming: [normalizedAddon] });
      const updated = await Promise.all(((plan?.addons as AddonDescriptor[] | undefined) ?? stored).map(normalizeAddonDescriptor));
      await saveAddons(updated);

      let syncProfile = activeProfile;
      if (activeProfile) {
        const updatedProfile =
          (await coreInvoke<UserProfile>(
            'addonProfileMutationPlan',
            JSON.stringify({ profile: activeProfile, command: 'install', addonKey: normalizedUrl }),
          )) ?? activeProfile;
        await saveProfile(updatedProfile);
        onProfileUpdated(updatedProfile);
        syncProfile = updatedProfile;
      }

      setInstalledAddons(updated);
      void syncNuvioAddons(syncProfile, updated);
      void syncStremioAddonsForProfile(syncProfile, updated);
      setAddonUrl('');
      setAddonInstallStatus({ loading: false, error: null });
      setAddedAddonName(normalizedAddon.manifest?.name || normalizedAddon.transportUrl);
      await onDispatch(JSON.stringify({ type: 'addonsRefreshRequested', forceRefresh: false, profile: activeProfile ?? null }));
      await onDispatch(
        JSON.stringify({ type: 'homeLoadRequested', force: true, language: prefs.language, profile: activeProfile ?? null }),
      );
    } catch (error) {
      setAddonInstallStatus({ loading: false, error: error instanceof Error ? error.message : String(error) });
    }
  };

  const handleRemove = async (addon: AddonDescriptor) => {
    const removeKey = addonKey(addon);
    const plan = await coreAddonCollectionMutationPlan({ existing: installedAddons, removeKey });
    const updated = (plan?.addons as AddonDescriptor[] | undefined) ?? installedAddons.filter((a) => addonKey(a) !== removeKey);
    await saveAddons(updated);
    setInstalledAddons(updated);
    if (activeProfile) {
      const updatedProfile =
        (await coreInvoke<UserProfile>(
          'addonProfileMutationPlan',
          JSON.stringify({ profile: activeProfile, command: 'remove', addonKey: removeKey }),
        )) ?? activeProfile;
      await saveProfile(updatedProfile);
      onProfileUpdated(updatedProfile);
      void syncNuvioAddons(updatedProfile, updated);
      void syncStremioAddonsForProfile(updatedProfile, updated);
    }
    await onDispatch(JSON.stringify({ type: 'addonsRefreshRequested', forceRefresh: false, profile: activeProfile ?? null }));
    await onDispatch(
      JSON.stringify({
        type: 'homeLoadRequested',
        force: true,
        language: prefs.language,
        profile: activeProfile ?? null,
      }),
    );
  };

  const handleToggleAddon = async (addon: AddonDescriptor) => {
    if (!activeProfile) return;
    const key = addonKey(addon);
    const updatedProfile =
      (await coreInvoke<UserProfile>(
        'addonProfileMutationPlan',
        JSON.stringify({ profile: activeProfile, command: 'toggle', addonKey: key }),
      )) ?? activeProfile;
    await saveProfile(updatedProfile);
    onProfileUpdated(updatedProfile);
    void syncNuvioAddons(updatedProfile, installedAddons);
    void syncStremioAddonsForProfile(updatedProfile, installedAddons);
    onDispatch(JSON.stringify({ type: 'addonsRefreshRequested' }));
    onDispatch(
      JSON.stringify({
        type: 'homeLoadRequested',
        force: true,
        language: prefs.language,
        profile: updatedProfile,
      }),
    );
  };

  const handleReorderAddon = async (addon: AddonDescriptor, direction: 'up' | 'down') => {
    const idx = installedAddons.findIndex((a) => addonKey(a) === addonKey(addon));
    if (idx < 0) return;
    const next = [...installedAddons];
    const swapIdx = direction === 'up' ? idx - 1 : idx + 1;
    if (swapIdx < 0 || swapIdx >= next.length) return;
    [next[idx], next[swapIdx]] = [next[swapIdx], next[idx]];
    await saveAddons(next);
    setInstalledAddons(next);
    void syncNuvioAddons(activeProfile, next);
    void syncStremioAddonsForProfile(activeProfile, next);
    onDispatch(JSON.stringify({ type: 'addonsRefreshRequested' }));
  };

  const pluginRepositories = state.plugins?.repositories ?? [];
  const pluginScrapers = state.plugins?.scrapers ?? [];
  const pluginStateError = typeof state.plugins?.error === 'string' ? state.plugins.error : (state.plugins?.error?.message ?? null);

  const handleInstallPlugin = async () => {
    const rawUrl = pluginUrl.trim();
    if (!rawUrl || pluginInstallLoading) return;
    setPluginInstallLoading(true);
    setPluginInstallError(null);
    try {
      const normalizedUrl = await coreInvoke<string>('normalizePluginRepositoryUrl', JSON.stringify({ url: rawUrl }));
      if (!normalizedUrl) throw new Error(t('plugins.invalid_url'));
      await onDispatch(JSON.stringify({ type: 'pluginRepositoryAddRequested', manifestUrl: normalizedUrl }));
      const key = await pluginRepositoryUrlsKey();
      const persisted = (await storageRead<string[]>(key)) ?? [];
      await storageWrite(key, [...new Set([...persisted, normalizedUrl])]);
      void pushPluginsToNuvio(activeProfile, [...pluginRepositories, { manifestUrl: normalizedUrl }]).catch(() => undefined);
      setPluginUrl('');
    } catch (error) {
      setPluginInstallError(error instanceof Error ? error.message : String(error));
    } finally {
      setPluginInstallLoading(false);
    }
  };

  const handleRemovePlugin = async (repository: PluginRepository) => {
    await onDispatch(JSON.stringify({ type: 'pluginRepositoryRemoveRequested', manifestUrl: repository.manifestUrl }));
    const key = await pluginRepositoryUrlsKey();
    const persisted = (await storageRead<string[]>(key)) ?? [];
    await storageWrite(
      key,
      persisted.filter((url) => url !== repository.manifestUrl),
    );
    void pushPluginsToNuvio(
      activeProfile,
      pluginRepositories.filter((item) => item.manifestUrl !== repository.manifestUrl),
    ).catch(() => undefined);
  };

  const handleRefreshPlugin = async (repository: PluginRepository) => {
    await onDispatch(JSON.stringify({ type: 'pluginRepositoryAddRequested', manifestUrl: repository.manifestUrl }));
  };

  const handleTogglePluginScraper = async (scraper: PluginScraper) => {
    const enabled = !scraper.enabled;
    await onDispatch(JSON.stringify({ type: 'pluginScraperToggled', scraperId: scraper.id, enabled }));
    const key = await pluginScraperEnabledKey();
    const overrides = (await storageRead<Record<string, boolean>>(key)) ?? {};
    await storageWrite(key, { ...overrides, [scraper.id]: enabled });
  };

  const disabledAddonKeys = activeProfile?.addonSettings?.disabledLocalAddons ?? activeProfile?.disabledLocalAddons ?? [];
  const normalizedSettingsQuery = settingsQuery.trim().toLowerCase();
  const searchResults = normalizedSettingsQuery
    ? VISIBLE_TABS.filter((item) => {
        const haystack = [t(item.labelKey), t(item.subtitleKey), ...SETTINGS_SEARCH_TERMS[item.id]].join(' ').toLowerCase();
        return haystack.includes(normalizedSettingsQuery);
      })
    : [];

  return (
    <div className="settings-screen" style={styles.screen}>
      <div className="settings-sidebar" style={styles.sidebar}>
        <p style={styles.sidebarTitle}>{t('nav.settings')}</p>
        <p style={styles.sidebarSubtitle}>{t('auto.general_0dbbccaf')}</p>
        <div style={settingsSearchStyles.wrap}>
          <Search size={15} style={{ color: 'rgba(255,255,255,0.35)', flexShrink: 0 }} />
          <input
            value={settingsQuery}
            onChange={(e) => setSettingsQuery(e.target.value)}
            placeholder={t('settings.search_placeholder')}
            style={settingsSearchStyles.input}
          />
        </div>

        <nav style={{ display: 'flex', flexDirection: 'column', gap: '0.125rem' }}>
          {(searchResults.length > 0 ? searchResults : VISIBLE_TABS).map((tabItem) => (
            <SidebarItem
              key={tabItem.id}
              label={t(tabItem.labelKey)}
              icon={tabItem.icon}
              selected={tab === tabItem.id}
              onClick={() => setTab(tabItem.id)}
            />
          ))}
        </nav>

        <div style={{ flex: 1 }} />
        <SidebarDivider />
        <div style={{ display: 'flex', flexDirection: 'column', gap: '0.125rem' }}>
          {inAppUpdatesSupported ? (
            <SidebarItem
              label={t('settings.check_for_updates') || 'Check for updates'}
              subtitle=""
              icon={<RefreshIcon />}
              selected={false}
              onClick={onCheckForUpdates}
            />
          ) : (
            <div style={{ padding: '0.5625rem 0.75rem', display: 'flex', alignItems: 'center', gap: '0.625rem' }}>
              <RefreshIcon />
              <span style={{ color: 'rgba(255,255,255,0.4)', fontSize: '0.8125rem' }}>
                {t('settings.updates_managed_by_package_manager')}
              </span>
            </div>
          )}
          <SidebarItem label={t('common.back')} subtitle="" icon={<ArrowBackIcon />} selected={false} onClick={onBack} />
        </div>
        <VersionFooter />
      </div>

      <div className="settings-content" style={styles.content}>
        <div style={styles.contentInner}>
          <SettingsDetailHeader title={t(TABS.find((item) => item.id === tab)?.labelKey ?? 'nav.settings')} />
          {normalizedSettingsQuery && searchResults.length === 0 && (
            <div style={settingsSearchStyles.noResults}>{t('settings.search_no_results')}</div>
          )}

          {tab === 'account' && (
            <AccountSection
              prefs={prefs}
              setPref={setPref}
              prefsLoaded={prefsLoaded}
              activeProfile={activeProfile}
              onProfileUpdated={onProfileUpdated}
              onSwitchProfile={onSwitchProfile}
              onDispatch={onDispatch}
              onNuvioSyncComplete={reloadInstalledAddons}
            />
          )}
          {tab === 'general' && <GeneralSection prefs={prefs} setPref={setPref} />}
          {tab === 'appearance' && <AppearanceSection prefs={prefs} setPref={setPref} />}
          {tab === 'playback' && <PlaybackSection prefs={prefs} setPref={setPref} />}
          {tab === 'device' && <DeviceCapabilitiesSection />}
          {tab === 'shortcuts' && <ShortcutsSection />}
          {tab === 'controller' && <ControllerSection />}
          {tab === 'content' && (
            <ContentSection prefs={prefs} setPref={setPref} installedAddons={installedAddons} disabledAddonKeys={disabledAddonKeys} />
          )}
          {tab === 'addons' && (
            <AddonsSection
              prefs={prefs}
              setPref={setPref}
              addonUrl={addonUrl}
              setAddonUrl={setAddonUrl}
              installedAddons={installedAddons}
              disabledAddonKeys={disabledAddonKeys}
              installLoading={addonInstallStatus.loading}
              installError={addonInstallStatus.error}
              onInstall={handleInstall}
              onRemove={handleRemove}
              onToggle={handleToggleAddon}
              onReorder={handleReorderAddon}
              onDispatch={onDispatch}
            />
          )}
          {tab === 'plugins' && (
            <PluginsSection
              pluginUrl={pluginUrl}
              setPluginUrl={setPluginUrl}
              repositories={pluginRepositories}
              scrapers={pluginScrapers}
              loading={pluginInstallLoading || !!state.plugins?.addingRepositoryUrl}
              error={pluginInstallError ?? pluginStateError}
              onInstall={handleInstallPlugin}
              onRemove={handleRemovePlugin}
              onRefresh={handleRefreshPlugin}
              onToggleScraper={handleTogglePluginScraper}
            />
          )}
          {tab === 'downloads' && !isBrowserTarget() && <DownloadsSection prefs={prefs} setPref={setPref} />}
        </div>
      </div>
      {addedAddonName && <AddonAddedDialog addonName={addedAddonName} onConfirm={() => setAddedAddonName(null)} />}
      {addonInstallStatus.error && (
        <div style={{ position: 'fixed', top: '1rem', right: '1rem', zIndex: 100 }}>
          <Toast
            variant="error"
            title={t('addons.install_failed_title')}
            message={t('addons.install_failed_message')}
            details={addonInstallStatus.error}
            detailsLabel={t('player.error_show_details')}
            detailsHideLabel={t('player.error_hide_details')}
            onClose={() => setAddonInstallStatus((prev) => ({ ...prev, error: null }))}
          />
        </div>
      )}
    </div>
  );
}

const settingsSearchStyles: Record<string, React.CSSProperties> = {
  wrap: {
    display: 'flex',
    alignItems: 'center',
    gap: '0.5rem',
    height: '2.25rem',
    padding: '0 0.625rem',
    margin: '0.875rem 0 1rem',
    background: 'rgba(255,255,255,0.045)',
    border: '1px solid rgba(255,255,255,0.09)',
    borderRadius: '0.5rem',
  },
  input: {
    flex: 1,
    minWidth: 0,
    background: 'none',
    border: 'none',
    outline: 'none',
    color: '#fff',
    fontSize: '0.8125rem',
    fontWeight: 600,
  },
  noResults: {
    margin: '0 1.5rem 1rem',
    padding: '0.625rem 0.75rem',
    borderRadius: '0.5rem',
    background: 'rgba(255,255,255,0.045)',
    color: 'rgba(255,255,255,0.45)',
    fontSize: '0.8125rem',
  },
};
