import React, { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { t } from '../../i18n';
import { ActionTile, ChoiceTile, DownloadIcon, InfoTile, InputTile, RefreshIcon, SettingsSection, streamSourceOptions } from './SettingsUI';
import { styles } from './settingsStyles';
import type { Prefs } from './settingsTypes';

interface OfflineDownloadItem {
  id: string;
  videoFileName: string;
  path: string;
  sizeBytes: number;
  status: string;
}

function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(2)} GB`;
}

function DownloadItemRow({ item, onDelete }: { item: OfflineDownloadItem; onDelete: () => void }) {
  const [hovered, setHovered] = useState(false);
  const [deleteHovered, setDeleteHovered] = useState(false);
  return (
    <div
      style={{ width: '100%', borderBottom: '1px solid rgba(255,255,255,0.055)', background: hovered ? 'rgba(255,255,255,0.03)' : 'transparent', display: 'flex', alignItems: 'center', padding: '13px 18px', boxSizing: 'border-box', gap: 14, transition: 'background 0.15s' }}
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
    >
      <span style={{ ...styles.rowIcon, color: 'var(--primary-accent-color)', flexShrink: 0 }}><DownloadIcon /></span>
      <div style={{ flex: 1, minWidth: 0 }}>
        <p style={{ ...styles.rowTitle, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>{item.videoFileName}</p>
        <p style={styles.rowSubtitle}>{formatBytes(item.sizeBytes)}</p>
      </div>
      <button
        style={{ background: deleteHovered ? 'rgba(255,80,80,0.15)' : 'rgba(255,80,80,0.08)', border: '1px solid rgba(255,80,80,0.18)', color: 'rgba(255,140,140,0.9)', fontSize: 12, fontWeight: 700, cursor: 'pointer', padding: '6px 14px', borderRadius: 999, fontFamily: 'sans-serif', flexShrink: 0, transition: 'background 0.15s' }}
        onMouseEnter={() => setDeleteHovered(true)}
        onMouseLeave={() => setDeleteHovered(false)}
        onClick={onDelete}
      >
        {t('common.delete') || 'Sil'}
      </button>
    </div>
  );
}

export function DownloadsSection({ prefs, setPref }: { prefs: Prefs; setPref: <K extends keyof Prefs>(k: K, v: Prefs[K]) => void }) {
  const [downloads, setDownloads] = useState<OfflineDownloadItem[]>([]);

  const refreshDownloads = () => {
    invoke<OfflineDownloadItem[]>('list_offline_downloads')
      .then(setDownloads)
      .catch(() => setDownloads([]));
  };

  useEffect(() => { refreshDownloads(); }, []);

  const handleDelete = async (item: OfflineDownloadItem) => {
    try {
      await invoke('delete_offline_download', { fileName: item.videoFileName });
      refreshDownloads();
    } catch { /* ignore */ }
  };

  return (
    <>
    <SettingsSection title={t('auto.downloads')} subtitle={t('settings.downloads_desc')}>
      <ChoiceTile
        title={t('settings.download_source_selection')}
        subtitle={t('settings.download_source_selection_desc')}
        options={streamSourceOptions()}
        selected={prefs.downloadSourceSelectionMode}
        onSelect={(v) => setPref('downloadSourceSelectionMode', v)}
      />
      {prefs.downloadSourceSelectionMode === 'regex' && (
        <InputTile
          title={t('settings.regex_pattern')}
          subtitle={t('settings.regex_pattern_desc')}
          value={prefs.downloadSourceRegexPattern}
          placeholder={t('settings.regex_pattern_placeholder')}
          onChange={(v) => setPref('downloadSourceRegexPattern', v)}
        />
      )}
      <ChoiceTile
        title={t('settings.download_subtitle')}
        subtitle={t('settings.download_subtitle_desc')}
        options={[{ value: 'off', label: t('settings.download_subtitle_off') }, { value: 'preferred', label: t('settings.download_subtitle_preferred') }, { value: 'tr', label: t('language.turkish') }, { value: 'en', label: t('language.english') }, { value: 'ja', label: t('language.japanese') }, { value: 'es', label: t('language.spanish') }, { value: 'fr', label: t('language.french') }, { value: 'de', label: t('language.german') }]}
        selected={prefs.downloadSubtitleLanguage}
        onSelect={(v) => setPref('downloadSubtitleLanguage', v)}
      />
    </SettingsSection>
    <SettingsSection
      title={`${t('auto.downloads')} (${downloads.length})`}
      subtitle={t('settings.downloads_list_desc') || 'İndirilen dosyalar'}
    >
      {downloads.length === 0 ? (
        <InfoTile title={t('downloads.empty') || 'İndirme yok'} value={t('settings.downloads_empty_desc') || 'Henüz tamamlanmış indirme bulunmuyor'} icon={<DownloadIcon />} />
      ) : (
        downloads.map((item) => (
          <DownloadItemRow key={item.id} item={item} onDelete={() => void handleDelete(item)} />
        ))
      )}
      <ActionTile
        title={t('common.refresh') || 'Yenile'}
        subtitle={t('settings.downloads_refresh_desc') || 'İndirme listesini yenile'}
        icon={<RefreshIcon />}
        onClick={refreshDownloads}
      />
    </SettingsSection>
    </>
  );
}
