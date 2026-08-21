import React, { useRef, useState } from 'react';
import { platformInvoke as invoke } from '../../platform/invoke';
import { platformSaveDialog } from '../../platform/browser';
import { FileDown } from 'lucide-react';
import { t } from '../../i18n';
import { ChoiceTile, ToggleTile, SettingsSection, ActionTile } from './SettingsUI';
import type { Prefs } from './settingsTypes';
import { initDiagnosticsSentry } from '../../core/sentryRuntime';

function applyDiscordPresenceConfig(enabled: boolean) {
  void invoke('discord_presence_configure', { enabled });
}

function applyDiagnosticMode(enabled: boolean) {
  void invoke('set_diagnostic_mode', { enabled }).catch(() => undefined);
  if (enabled && import.meta.env.PROD) void initDiagnosticsSentry();
}

export function GeneralSection({ prefs, setPref }: { prefs: Prefs; setPref: <K extends keyof Prefs>(k: K, v: Prefs[K]) => void }) {
  const [diagExportStatus, setDiagExportStatus] = useState<'idle' | 'saved' | 'failed'>('idle');
  const diagStatusTimer = useRef<ReturnType<typeof setTimeout> | null>(null);

  const exportDiagnostics = async () => {
    try {
      const destination = await platformSaveDialog({
        defaultPath: `fluxa-diagnostics-${new Date().toISOString().slice(0, 10)}.log`,
        filters: [{ name: 'Log', extensions: ['log'] }],
      });
      if (!destination) return;
      await invoke('export_diagnostic_log', { destination });
      setDiagExportStatus('saved');
    } catch {
      setDiagExportStatus('failed');
    }
    if (diagStatusTimer.current) clearTimeout(diagStatusTimer.current);
    diagStatusTimer.current = setTimeout(() => setDiagExportStatus('idle'), 4000);
  };

  return (
    <>
      <SettingsSection title={t('auto.app')} subtitle={t('auto.language_theme_startup')}>
        <ChoiceTile
          title={t('auto.interface_language')}
          subtitle={t('settings.language_desc')}
          options={[
            { value: 'en', label: t('language.english') },
            { value: 'tr', label: t('language.turkish') },
          ]}
          selected={prefs.language}
          onSelect={(v) => setPref('language', v)}
        />
        <ChoiceTile
          title={t('auto.start_page')}
          subtitle={t('settings.start_page_desc')}
          options={[
            { value: 'home', label: t('nav.home') },
            { value: 'discover', label: t('nav.discover') },
            { value: 'library', label: t('nav.library') },
          ]}
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
      <SettingsSection title={t('settings.search')} subtitle={t('settings.search_desc')}>
        <ToggleTile
          title={t('settings.search_suggestions_open_detail')}
          subtitle={t('settings.search_suggestions_open_detail_desc')}
          checked={prefs.searchSuggestionsOpenDetail}
          onToggle={(v) => setPref('searchSuggestionsOpenDetail', v)}
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
      <SettingsSection title={t('settings.discord_rich_presence')} subtitle={t('settings.discord_rich_presence_desc')}>
        <ToggleTile
          title={t('settings.discord_rich_presence_enable')}
          subtitle={t('settings.discord_rich_presence_enable_desc')}
          checked={prefs.discordRichPresenceEnabled}
          onToggle={(v) => {
            void setPref('discordRichPresenceEnabled', v);
            applyDiscordPresenceConfig(v);
          }}
        />
      </SettingsSection>
      <SettingsSection title={t('settings.diagnostics')} subtitle={t('settings.diagnostics_desc')}>
        <ToggleTile
          title={t('settings.diagnostic_mode')}
          subtitle={t('settings.diagnostic_mode_desc')}
          checked={prefs.diagnosticMode}
          onToggle={(v) => {
            void setPref('diagnosticMode', v);
            applyDiagnosticMode(v);
          }}
        />
        <ActionTile
          title={t('settings.diagnostic_export')}
          subtitle={
            diagExportStatus === 'saved'
              ? t('settings.diagnostic_export_saved')
              : diagExportStatus === 'failed'
                ? t('settings.diagnostic_export_failed')
                : t('settings.diagnostic_export_desc')
          }
          icon={<FileDown size={20} />}
          accent={diagExportStatus === 'failed' ? '#ff8a8a' : diagExportStatus === 'saved' ? '#9be89b' : '#FFFFFF'}
          onClick={() => void exportDiagnostics()}
        />
      </SettingsSection>
    </>
  );
}
