import React, { useEffect, useState } from 'react';
import { coreApplyPreferenceUpdate, httpFetchText, storageRead, storageWrite } from '../core/engine';
import {
  coreAddonCollectionMutationPlan,
  manifestFetchPlan,
  normalizeManifestUrl,
  parseManifest,
  resolveManifestAssets,
} from '../core/addonManifest';
import type { AddonDescriptor, AppState, UserProfile } from '../core/types';
import { addonKey, normalizeAddonDescriptor } from '../core/addons';
import { saveProfile } from '../core/profiles';
import { setLanguage, t } from '../i18n';
import { styles } from '../components/settings/settingsStyles';
import { DEFAULT_PREFS } from '../components/settings/settingsTypes';
import type { Prefs, Tab } from '../components/settings/settingsTypes';
import {
  AccountIcon,
  ArrowBackIcon,
  DownloadIcon,
  ExtensionIcon,
  PaletteIcon,
  PlayCircleIcon,
  RefreshIcon,
  SettingsIcon,
  SidebarItem,
  SettingsDetailHeader,
  StorageIcon,
  VersionFooter,
} from '../components/settings/SettingsUI';
import { AccountSection } from '../components/settings/AccountSection';
import { GeneralSection } from '../components/settings/GeneralSection';
import { AppearanceSection } from '../components/settings/AppearanceSection';
import { PlaybackSection } from '../components/settings/PlaybackSection';
import { ContentSection } from '../components/settings/ContentSection';
import { AddonsSection } from '../components/settings/AddonsSection';
import { DownloadsSection } from '../components/settings/DownloadsSection';

function mergeAddons(existing: AddonDescriptor[], incoming: AddonDescriptor[]): AddonDescriptor[] {
  const merged = new Map<string, AddonDescriptor>();
  for (const addon of existing.map(normalizeAddonDescriptor)) merged.set(addonKey(addon), addon);
  for (const addon of incoming.map(normalizeAddonDescriptor)) merged.set(addonKey(addon), addon);
  return [...merged.values()];
}

function addonUrlIdentity(url: string): string {
  return url
    .trim()
    .replace(/\/+$/, '')
    .replace(/^https?:\/\//i, '')
    .toLowerCase();
}

function profileLocalAddons(profile: UserProfile | null): string[] {
  return profile?.addonSettings?.localAddons ?? profile?.localAddons ?? [];
}

function withInstalledLocalAddon(profile: UserProfile, normalizedUrl: string): UserProfile {
  const existing = profileLocalAddons(profile);
  const next = existing.some((url) => addonUrlIdentity(url) === addonUrlIdentity(normalizedUrl))
    ? existing
    : [...existing, normalizedUrl];
  return {
    ...profile,
    localAddons: next,
    addonSettings: {
      ...(profile.addonSettings ?? {}),
      localAddons: next,
      disabledLocalAddons: profile.addonSettings?.disabledLocalAddons ?? profile.disabledLocalAddons ?? [],
    },
  };
}

async function settingsFetchJson(url: string): Promise<unknown> {
  const response = await httpFetchText(url);
  if (response.statusCode < 200 || response.statusCode > 299) {
    throw new Error(`Request failed (${response.statusCode})`);
  }
  return JSON.parse(response.body);
}

async function loadAddonManifestFromUrl(rawUrl: string): Promise<AddonDescriptor> {
  const plan = await manifestFetchPlan(rawUrl);
  const candidateUrls = plan?.candidateUrls?.length ? plan.candidateUrls : [rawUrl];
  let lastError: unknown = null;

  for (const candidateUrl of candidateUrls) {
    try {
      const manifest = await settingsFetchJson(candidateUrl);
      const parsed = await parseManifest(JSON.stringify(manifest), candidateUrl);
      if (!parsed) throw new Error('Manifest parse returned empty result');
      const resolved = await resolveManifestAssets(parsed);
      return normalizeAddonDescriptor((resolved ?? parsed) as AddonDescriptor);
    } catch (error) {
      lastError = error;
    }
  }

  throw lastError instanceof Error ? lastError : new Error(`Unable to fetch addon manifest: ${rawUrl}`);
}

const TABS: { id: Tab; labelKey: string; subtitleKey: string; icon: React.ReactNode }[] = [
  { id: 'account',    labelKey: 'auto.account_sync',          subtitleKey: 'auto.account_devices_and_sync',        icon: <AccountIcon /> },
  { id: 'general',    labelKey: 'auto.general',               subtitleKey: 'auto.language_theme_startup',          icon: <SettingsIcon /> },
  { id: 'appearance', labelKey: 'auto.appearance',            subtitleKey: 'auto.color_and_layout',                icon: <PaletteIcon /> },
  { id: 'playback',   labelKey: 'auto.playback',              subtitleKey: 'auto.player_behavior_and_defaults',    icon: <PlayCircleIcon /> },
  { id: 'content',    labelKey: 'auto.catalogs',              subtitleKey: 'auto.categories_sources_and_ranking',  icon: <StorageIcon /> },
  { id: 'addons',     labelKey: 'auto.add_ons',               subtitleKey: 'auto.installed_add_ons_and_settings',  icon: <ExtensionIcon /> },
  { id: 'downloads',  labelKey: 'auto.downloads',             subtitleKey: 'auto.download_and_storage_settings',   icon: <DownloadIcon /> },
];

interface Props {
  state: AppState;
  onDispatch: (actionJson: string) => void;
  activeProfile: UserProfile | null;
  onProfileUpdated: (profile: UserProfile) => void;
  onSwitchProfile: () => void;
  onBack: () => void;
  onCheckForUpdates: () => void;
}

export function SettingsScreen({ state, onDispatch, activeProfile, onProfileUpdated, onSwitchProfile, onBack, onCheckForUpdates }: Props) {
  const [tab, setTab] = useState<Tab>('account');
  const [prefs, setPrefs] = useState<Prefs>(DEFAULT_PREFS);
  const [addonUrl, setAddonUrl] = useState('');
  const [installedAddons, setInstalledAddons] = useState<AddonDescriptor[]>([]);
  const [addonInstallStatus, setAddonInstallStatus] = useState<{ loading: boolean; error: string | null }>({ loading: false, error: null });

  useEffect(() => {
    storageRead<Prefs>('prefs').then((p) => {
      const merged = p ? { ...DEFAULT_PREFS, ...p } : DEFAULT_PREFS;
      if (p) setLanguage(merged.language);
      else setLanguage(DEFAULT_PREFS.language);
      setPrefs(merged);
      void import('@tauri-apps/api/core').then(({ invoke }) =>
        invoke('player_set_seek_thumbnail_enabled', { enabled: merged.seekThumbnailEnabled })
      );
    });
    storageRead<AddonDescriptor[]>('addons').then((a) => setInstalledAddons((a ?? []).map(normalizeAddonDescriptor)));
  }, []);

  const engineAddons = state.addons.installed ?? [];
  useEffect(() => {
    if (engineAddons.length > 0) {
      storageRead<AddonDescriptor[]>('addons').then((stored) => {
        coreAddonCollectionMutationPlan({ existing: stored ?? [], incoming: engineAddons })
          .then((plan) => ((plan?.addons as AddonDescriptor[] | undefined) ?? mergeAddons(stored ?? [], engineAddons)))
          .then((merged) => {
            storageWrite('addons', merged).then(() => setInstalledAddons(merged));
          });
      });
    }
  }, [engineAddons]);

  const setPref = async <K extends keyof Prefs>(key: K, value: Prefs[K]) => {
    const planned = await coreApplyPreferenceUpdate({ existing: prefs, key, value });
    const updated = { ...prefs, ...(planned ?? { [key]: value }) };
    if (key === 'language') setLanguage(String(updated.language));
    setPrefs(updated as Prefs);
    await storageWrite('prefs', updated);
    onDispatch(JSON.stringify({ type: 'settingsChanged', key, value }));
  };

  const handleInstall = async () => {
    const rawUrl = addonUrl.trim();
    if (!rawUrl || addonInstallStatus.loading) return;
    setAddonInstallStatus({ loading: true, error: null });
    try {
      const addon = await loadAddonManifestFromUrl(rawUrl);
      const normalizedUrl = await normalizeManifestUrl(addon.transportUrl || rawUrl);
      const normalizedAddon = normalizeAddonDescriptor({ ...addon, transportUrl: normalizedUrl });
      const stored = (await storageRead<AddonDescriptor[]>('addons')) ?? [];
      const plan = await coreAddonCollectionMutationPlan({ existing: stored, incoming: [normalizedAddon] });
      const updated = ((plan?.addons as AddonDescriptor[] | undefined) ?? mergeAddons(stored, [normalizedAddon])).map(normalizeAddonDescriptor);
      await storageWrite('addons', updated);

      if (activeProfile) {
        const updatedProfile = withInstalledLocalAddon(activeProfile, normalizedUrl);
        await saveProfile(updatedProfile);
        onProfileUpdated(updatedProfile);
      }

      setInstalledAddons(updated);
      setAddonUrl('');
      setAddonInstallStatus({ loading: false, error: null });
      onDispatch(JSON.stringify({ type: 'addonsRefreshRequested', forceRefresh: false, profile: activeProfile ?? null }));
      onDispatch(JSON.stringify({ type: 'homeLoadRequested', force: true, language: prefs.language, profile: activeProfile ?? null }));
    } catch (error) {
      setAddonInstallStatus({ loading: false, error: error instanceof Error ? error.message : String(error) });
    }
  };

  const handleRemove = async (addon: AddonDescriptor) => {
    const removeKey = addonKey(addon);
    const plan = await coreAddonCollectionMutationPlan({ existing: installedAddons, removeKey });
    const updated = (plan?.addons as AddonDescriptor[] | undefined) ?? installedAddons.filter((a) => addonKey(a) !== removeKey);
    await storageWrite('addons', updated);
    setInstalledAddons(updated);
    if (activeProfile) {
      const nextUrls = profileLocalAddons(activeProfile).filter((url) => addonUrlIdentity(url) !== addonUrlIdentity(removeKey));
      const updatedProfile: UserProfile = {
        ...activeProfile,
        localAddons: nextUrls,
        addonSettings: {
          ...(activeProfile.addonSettings ?? {}),
          localAddons: nextUrls,
          disabledLocalAddons: activeProfile.addonSettings?.disabledLocalAddons ?? activeProfile.disabledLocalAddons ?? [],
        },
      };
      await saveProfile(updatedProfile);
      onProfileUpdated(updatedProfile);
    }
    onDispatch(JSON.stringify({ type: 'addonsRefreshRequested' }));
  };

  const handleToggleAddon = async (addon: AddonDescriptor) => {
    if (!activeProfile) return;
    const key = addonKey(addon);
    const disabled = activeProfile.addonSettings?.disabledLocalAddons ?? activeProfile.disabledLocalAddons ?? [];
    const isDisabled = disabled.includes(key);
    const nextDisabled = isDisabled ? disabled.filter((k) => k !== key) : [...disabled, key];
    const updatedProfile: UserProfile = {
      ...activeProfile,
      addonSettings: {
        ...(activeProfile.addonSettings ?? {}),
        localAddons: profileLocalAddons(activeProfile),
        disabledLocalAddons: nextDisabled,
      },
    };
    await saveProfile(updatedProfile);
    onProfileUpdated(updatedProfile);
    onDispatch(JSON.stringify({ type: 'addonsRefreshRequested' }));
  };

  const handleReorderAddon = async (addon: AddonDescriptor, direction: 'up' | 'down') => {
    const idx = installedAddons.findIndex((a) => addonKey(a) === addonKey(addon));
    if (idx < 0) return;
    const next = [...installedAddons];
    const swapIdx = direction === 'up' ? idx - 1 : idx + 1;
    if (swapIdx < 0 || swapIdx >= next.length) return;
    [next[idx], next[swapIdx]] = [next[swapIdx], next[idx]];
    await storageWrite('addons', next);
    setInstalledAddons(next);
    onDispatch(JSON.stringify({ type: 'addonsRefreshRequested' }));
  };

  const disabledAddonKeys = activeProfile?.addonSettings?.disabledLocalAddons ?? activeProfile?.disabledLocalAddons ?? [];

  return (
    <div style={styles.screen}>
      <div style={styles.sidebar}>
        <p style={styles.sidebarTitle}>{t('nav.settings')}</p>
        <p style={styles.sidebarSubtitle}>{t('auto.general_0dbbccaf')}</p>
        <div style={{ height: 16 }} />

        <div style={styles.categoryCard}>
          {TABS.map((tabItem, index) => (
            <React.Fragment key={tabItem.id}>
              <SidebarItem
                label={t(tabItem.labelKey)}
                subtitle={t(tabItem.subtitleKey)}
                icon={tabItem.icon}
                selected={tab === tabItem.id}
                onClick={() => setTab(tabItem.id)}
              />
              {index !== TABS.length - 1 && <div style={styles.mobileDivider} />}
            </React.Fragment>
          ))}
        </div>

        <div style={{ flex: 1 }} />
        <div style={styles.categoryCard}>
          <SidebarItem label={t('settings.check_for_updates') || 'Check for updates'} subtitle="" icon={<RefreshIcon />} selected={false} onClick={onCheckForUpdates} />
          <div style={styles.mobileDivider} />
          <SidebarItem label={t('common.back')} subtitle="" icon={<ArrowBackIcon />} selected={false} onClick={onBack} />
        </div>
        <VersionFooter />
      </div>

      <div style={styles.content}>
        <SettingsDetailHeader title={t(TABS.find((item) => item.id === tab)?.labelKey ?? 'nav.settings')} />

        {tab === 'account' && (
          <AccountSection
            prefs={prefs}
            setPref={setPref}
            activeProfile={activeProfile}
            onProfileUpdated={onProfileUpdated}
            onSwitchProfile={onSwitchProfile}
            onDispatch={onDispatch}
          />
        )}
        {tab === 'general'    && <GeneralSection prefs={prefs} setPref={setPref} />}
        {tab === 'appearance' && <AppearanceSection prefs={prefs} setPref={setPref} />}
        {tab === 'playback'   && <PlaybackSection prefs={prefs} setPref={setPref} />}
        {tab === 'content'    && <ContentSection prefs={prefs} setPref={setPref} installedAddons={installedAddons} />}
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
        {tab === 'downloads' && <DownloadsSection prefs={prefs} setPref={setPref} />}
      </div>
    </div>
  );
}
