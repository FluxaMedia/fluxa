import React from 'react';
import { t } from '../../i18n';
import { ChoiceTile, ToggleTile, SettingsSection } from './SettingsUI';
import type { Prefs } from './settingsTypes';

export function GeneralSection({ prefs, setPref }: { prefs: Prefs; setPref: <K extends keyof Prefs>(k: K, v: Prefs[K]) => void }) {
  return (
    <>
    <SettingsSection title={t('auto.app')} subtitle={t('auto.language_theme_startup')}>
      <ChoiceTile
        title={t('auto.interface_language')}
        subtitle={t('settings.language_desc')}
        options={[{ value: 'en', label: t('language.english') }, { value: 'tr', label: t('language.turkish') }]}
        selected={prefs.language}
        onSelect={(v) => setPref('language', v)}
      />
      <ChoiceTile
        title={t('auto.start_page')}
        subtitle={t('settings.start_page_desc')}
        options={[{ value: 'home', label: t('nav.home') }, { value: 'discover', label: t('nav.discover') }, { value: 'library', label: t('nav.library') }]}
        selected={prefs.startPage}
        onSelect={(v) => setPref('startPage', v)}
      />
      <ToggleTile
        title={t('auto.background_playback')}
        subtitle={t('settings.background_playback_desc')}
        checked={prefs.backgroundPlayback}
        onToggle={(v) => setPref('backgroundPlayback', v)}
      />
      <ToggleTile
        title={t('settings.automatic_updates') || 'Otomatik Güncellemeler'}
        subtitle={t('settings.automatic_updates_desc') || 'Uygulama güncellemelerini otomatik olarak indir'}
        checked={prefs.automaticUpdates}
        onToggle={(v) => setPref('automaticUpdates', v)}
      />
    </SettingsSection>
    <SettingsSection title={t('settings.notifications')} subtitle={t('settings.notifications_desc')}>
      <ToggleTile
        title={t('settings.notifications_master') || 'Bildirimleri Etkinleştir'}
        subtitle={t('settings.notifications_master_desc') || 'Tüm uygulama bildirimlerini aç/kapat'}
        checked={prefs.notificationsEnabled}
        onToggle={(v) => setPref('notificationsEnabled', v)}
      />
    </SettingsSection>
    </>
  );
}
