import React, { useEffect, useRef, useState } from 'react';
import { platformInvoke as invoke } from '../../platform/invoke';
import { platformSaveDialog } from '../../platform/browser';
import { FileDown } from 'lucide-react';
import { t } from '../../i18n';
import { ChoiceTile, ToggleTile, InputTile, SettingsSection, ActionTile } from './SettingsUI';
import type { Prefs } from './settingsTypes';
import { setRpdbApiKey, validateRpdbApiKey } from '../../core/rpdb';
import { initDiagnosticsSentry } from '../../core/sentryRuntime';

function applyDiscordPresenceConfig(enabled: boolean) {
  void invoke('discord_presence_configure', { enabled });
}

function applyDiagnosticMode(enabled: boolean) {
  void invoke('set_diagnostic_mode', { enabled }).catch(() => undefined);
  if (enabled && import.meta.env.PROD) void initDiagnosticsSentry();
}

export function GeneralSection({ prefs, setPref }: { prefs: Prefs; setPref: <K extends keyof Prefs>(k: K, v: Prefs[K]) => void }) {
  const [rpdbKeyStatus, setRpdbKeyStatus] = useState<'idle' | 'checking' | 'valid' | 'invalid'>('idle');
  const rpdbCheckTimer = useRef<ReturnType<typeof setTimeout> | null>(null);
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

  useEffect(() => {
    if (!prefs.rpdbApiKey) {
      setRpdbKeyStatus('idle');
      return;
    }
    setRpdbKeyStatus('checking');
    if (rpdbCheckTimer.current) clearTimeout(rpdbCheckTimer.current);
    rpdbCheckTimer.current = setTimeout(() => {
      void validateRpdbApiKey(prefs.rpdbApiKey).then((valid) => setRpdbKeyStatus(valid ? 'valid' : 'invalid'));
    }, 500);
    return () => {
      if (rpdbCheckTimer.current) clearTimeout(rpdbCheckTimer.current);
    };
  }, [prefs.rpdbApiKey]);

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
      <SettingsSection title={t('settings.integrations')} subtitle={t('settings.integrations_desc')}>
        <InputTile
          title={t('settings.tmdb_api_key')}
          subtitle={t('settings.tmdb_api_key_desc')}
          value={prefs.tmdbApiKey}
          placeholder={t('settings.api_key_placeholder')}
          onChange={(v) => setPref('tmdbApiKey', v)}
          status={
            <p style={{ fontSize: '0.75rem', marginTop: '0.375rem', color: prefs.tmdbApiKey ? '#4ADE80' : 'rgba(255,255,255,0.45)' }}>
              {prefs.tmdbApiKey ? t('settings.tmdb_metadata_source_active') : t('settings.tmdb_metadata_source_inactive')}
            </p>
          }
        />
        {prefs.tmdbApiKey && (
          <ToggleTile
            title={t('settings.tmdb_prefer_over_addons')}
            subtitle={t('settings.tmdb_prefer_over_addons_desc')}
            checked={prefs.tmdbPreferOverAddons}
            onToggle={(v) => setPref('tmdbPreferOverAddons', v)}
          />
        )}
        <InputTile
          title={t('settings.rpdb_api_key')}
          subtitle={t('settings.rpdb_api_key_desc')}
          value={prefs.rpdbApiKey}
          placeholder={t('settings.api_key_placeholder')}
          onChange={(v) => {
            void setPref('rpdbApiKey', v);
            setRpdbApiKey(v);
          }}
          status={
            rpdbKeyStatus !== 'idle' && (
              <p
                style={{
                  fontSize: '0.75rem',
                  marginTop: '0.375rem',
                  color: rpdbKeyStatus === 'invalid' ? 'var(--primary-accent-color)' : 'rgba(255,255,255,0.45)',
                }}
              >
                {rpdbKeyStatus === 'checking'
                  ? t('settings.rpdb_key_checking')
                  : rpdbKeyStatus === 'valid'
                    ? t('settings.rpdb_key_valid')
                    : t('settings.rpdb_key_invalid')}
              </p>
            )
          }
        />
        <InputTile
          title={t('settings.omdb_api_key')}
          subtitle={t('settings.omdb_api_key_desc')}
          value={prefs.omdbApiKey}
          placeholder={t('settings.api_key_placeholder')}
          onChange={(v) => setPref('omdbApiKey', v)}
        />
        <InputTile
          title={t('settings.mdblist_api_key')}
          subtitle={t('settings.mdblist_api_key_desc')}
          value={prefs.mdblistApiKey}
          placeholder={t('settings.api_key_placeholder')}
          onChange={(v) => setPref('mdblistApiKey', v)}
        />
        <InputTile
          title={t('settings.fanart_api_key')}
          subtitle={t('settings.fanart_api_key_desc')}
          value={prefs.fanartApiKey}
          placeholder={t('settings.api_key_placeholder')}
          onChange={(v) => setPref('fanartApiKey', v)}
        />
      </SettingsSection>
      {prefs.tmdbApiKey && (
        <SettingsSection title={t('settings.tmdb_enrichment')} subtitle={t('settings.tmdb_enrichment_desc')}>
          <ToggleTile
            title={t('settings.tmdb_enrich_artwork')}
            subtitle={t('settings.tmdb_enrich_artwork_desc')}
            checked={prefs.tmdbEnrichArtworkEnabled}
            onToggle={(v) => setPref('tmdbEnrichArtworkEnabled', v)}
          />
          <ToggleTile
            title={t('settings.tmdb_enrich_description')}
            subtitle={t('settings.tmdb_enrich_description_desc')}
            checked={prefs.tmdbEnrichDescriptionEnabled}
            onToggle={(v) => setPref('tmdbEnrichDescriptionEnabled', v)}
          />
          <ToggleTile
            title={t('settings.tmdb_enrich_genres_keywords')}
            subtitle={t('settings.tmdb_enrich_genres_keywords_desc')}
            checked={prefs.tmdbEnrichGenresKeywordsEnabled}
            onToggle={(v) => setPref('tmdbEnrichGenresKeywordsEnabled', v)}
          />
          <ToggleTile
            title={t('settings.tmdb_enrich_cast_crew')}
            subtitle={t('settings.tmdb_enrich_cast_crew_desc')}
            checked={prefs.tmdbEnrichCastCrewEnabled}
            onToggle={(v) => setPref('tmdbEnrichCastCrewEnabled', v)}
          />
          <ToggleTile
            title={t('settings.tmdb_enrich_network')}
            subtitle={t('settings.tmdb_enrich_network_desc')}
            checked={prefs.tmdbEnrichNetworkEnabled}
            onToggle={(v) => setPref('tmdbEnrichNetworkEnabled', v)}
          />
          <ToggleTile
            title={t('settings.tmdb_enrich_ratings')}
            subtitle={t('settings.tmdb_enrich_ratings_desc')}
            checked={prefs.tmdbRatingsEnabled}
            onToggle={(v) => setPref('tmdbRatingsEnabled', v)}
          />
          <ToggleTile
            title={t('settings.tmdb_enrich_collection')}
            subtitle={t('settings.tmdb_enrich_collection_desc')}
            checked={prefs.tmdbCollectionInfoEnabled}
            onToggle={(v) => setPref('tmdbCollectionInfoEnabled', v)}
          />
          <ToggleTile
            title={t('settings.tmdb_enrich_status_schedule')}
            subtitle={t('settings.tmdb_enrich_status_schedule_desc')}
            checked={prefs.tmdbEnrichStatusScheduleEnabled}
            onToggle={(v) => setPref('tmdbEnrichStatusScheduleEnabled', v)}
          />
          <ToggleTile
            title={t('settings.tmdb_enrich_origin_titles')}
            subtitle={t('settings.tmdb_enrich_origin_titles_desc')}
            checked={prefs.tmdbEnrichOriginTitlesEnabled}
            onToggle={(v) => setPref('tmdbEnrichOriginTitlesEnabled', v)}
          />
          <ToggleTile
            title={t('settings.tmdb_enrich_watch_providers')}
            subtitle={t('settings.tmdb_enrich_watch_providers_desc')}
            checked={prefs.tmdbEnrichWatchProvidersEnabled}
            onToggle={(v) => setPref('tmdbEnrichWatchProvidersEnabled', v)}
          />
          <ToggleTile
            title={t('settings.tmdb_enrich_episode_stills')}
            subtitle={t('settings.tmdb_enrich_episode_stills_desc')}
            checked={prefs.tmdbEpisodeImagesEnabled}
            onToggle={(v) => setPref('tmdbEpisodeImagesEnabled', v)}
          />
        </SettingsSection>
      )}
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
