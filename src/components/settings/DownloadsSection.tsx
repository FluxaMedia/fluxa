import React, { useEffect, useState } from 'react';
import { platformInvoke as invoke } from '../../platform/invoke';
import { platformListen as listen } from '../../platform/browser';
import { platformOpenDialog } from '../../platform/browser';
import { t } from '../../i18n';
import { ConfirmDialog } from '../ConfirmDialog';
import { ChoiceTile, InputTile, SettingsSection, streamSourceOptions } from './SettingsUI';
import { styles, FONT } from './settingsStyles';
import type { Prefs } from './settingsTypes';

interface OfflineDownloadItem {
  id: string;
  videoFileName: string;
  path?: string;
  sizeBytes?: number;
  title?: string;
  downloadedBytes?: number;
  totalBytes?: number;
  status: string;
  error?: string;
}

interface DownloadProgressEvent {
  id: string;
  downloadedBytes: number;
  totalBytes: number | null;
  status: string;
  error: string | null;
}

function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(2)} GB`;
}

function TrashIcon() {
  return (
    <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
      <path d="M6 19c0 1.1.9 2 2 2h8c1.1 0 2-.9 2-2V7H6v12zM19 4h-3.5l-1-1h-5l-1 1H5v2h14V4z" />
    </svg>
  );
}

function PauseIcon() {
  return (
    <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
      <path d="M6 5h4v14H6zm8 0h4v14h-4z" />
    </svg>
  );
}

function ResumeIcon() {
  return (
    <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
      <path d="M8 5v14l11-7z" />
    </svg>
  );
}

function ActionBtn({ onClick, title, children }: { onClick: () => void; title: string; children: React.ReactNode }) {
  const [hovered, setHovered] = useState(false);
  return (
    <button
      style={{
        background: hovered ? 'rgba(255,255,255,0.10)' : 'transparent',
        border: 'none',
        color: hovered ? '#FFFFFF' : 'rgba(255,255,255,0.45)',
        cursor: 'pointer',
        padding: '0.4375rem',
        borderRadius: '0.4375rem',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        flexShrink: 0,
        transition: 'background 0.12s, color 0.12s',
      }}
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
      onClick={onClick}
      aria-label={title}
      title={title}
    >
      {children}
    </button>
  );
}

function DownloadItemRow({
  item,
  onDelete,
  onPause,
  onResume,
}: {
  item: OfflineDownloadItem;
  onDelete: () => void;
  onPause: () => void;
  onResume: () => void;
}) {
  const [hovered, setHovered] = useState(false);
  const [deleteHovered, setDeleteHovered] = useState(false);
  const inProgress = item.status === 'downloading' || item.status === 'paused' || item.status === 'failed';
  const progressPct = item.totalBytes ? Math.min(100, Math.round(((item.downloadedBytes ?? 0) / item.totalBytes) * 100)) : null;

  return (
    <div
      style={{
        width: '100%',
        borderBottom: '1px solid rgba(255,255,255,0.055)',
        background: hovered ? 'rgba(255,255,255,0.03)' : 'transparent',
        display: 'flex',
        alignItems: 'center',
        padding: '0.8125rem 1rem',
        boxSizing: 'border-box',
        gap: '0.75rem',
        transition: 'background 0.15s',
      }}
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
    >
      <div style={{ flex: 1, minWidth: 0 }}>
        <p style={{ ...styles.rowTitle, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>
          {item.title ?? item.videoFileName}
        </p>
        {inProgress ? (
          <>
            <p style={styles.rowSubtitle}>
              {item.status === 'failed'
                ? t('downloads.status_failed')
                : item.status === 'paused'
                  ? t('downloads.status_paused')
                  : t('downloads.status_downloading', progressPct ?? 0)}
              {item.totalBytes ? ` · ${formatBytes(item.downloadedBytes ?? 0)} / ${formatBytes(item.totalBytes)}` : ''}
            </p>
            <div
              style={{
                height: '0.1875rem',
                borderRadius: '0.125rem',
                background: 'rgba(255,255,255,0.08)',
                marginTop: '0.375rem',
                overflow: 'hidden',
              }}
            >
              <div
                style={{ height: '100%', width: `${progressPct ?? 0}%`, background: 'rgba(255,255,255,0.5)', transition: 'width 0.2s' }}
              />
            </div>
          </>
        ) : (
          <p style={styles.rowSubtitle}>{formatBytes(item.sizeBytes ?? 0)}</p>
        )}
      </div>
      {item.status === 'downloading' && (
        <ActionBtn onClick={onPause} title={t('downloads.pause')}>
          <PauseIcon />
        </ActionBtn>
      )}
      {(item.status === 'paused' || item.status === 'failed') && (
        <ActionBtn onClick={onResume} title={t('downloads.resume')}>
          <ResumeIcon />
        </ActionBtn>
      )}
      <button
        style={{
          background: deleteHovered ? 'rgba(255,80,80,0.12)' : 'transparent',
          border: 'none',
          color: deleteHovered ? 'rgba(255,120,120,0.9)' : 'rgba(255,255,255,0.25)',
          cursor: 'pointer',
          padding: '0.4375rem',
          borderRadius: '0.4375rem',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          flexShrink: 0,
          transition: 'background 0.12s, color 0.12s',
          opacity: hovered || inProgress ? 1 : 0,
        }}
        onMouseEnter={() => setDeleteHovered(true)}
        onMouseLeave={() => setDeleteHovered(false)}
        onClick={onDelete}
        aria-label={t('common.delete')}
      >
        <TrashIcon />
      </button>
    </div>
  );
}

function RefreshButton({ onClick }: { onClick: () => void }) {
  const [hovered, setHovered] = useState(false);
  return (
    <button
      onClick={onClick}
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
      aria-label={t('common.refresh')}
      style={{
        background: hovered ? 'rgba(255,255,255,0.08)' : 'transparent',
        border: 'none',
        color: hovered ? '#FFFFFF' : 'rgba(255,255,255,0.35)',
        cursor: 'pointer',
        padding: '0.375rem',
        borderRadius: '0.4375rem',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        transition: 'background 0.12s, color 0.12s',
      }}
    >
      <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
        <path d="M17.65 6.35C16.2 4.9 14.21 4 12 4c-4.42 0-7.99 3.58-7.99 8s3.57 8 7.99 8c3.73 0 6.84-2.55 7.73-6h-2.08c-.82 2.33-3.04 4-5.65 4-3.31 0-6-2.69-6-6s2.69-6 6-6c1.66 0 3.14.69 4.22 1.78L13 11h7V4l-2.35 2.35z" />
      </svg>
    </button>
  );
}

function DownloadDirRow({ value, onChange }: { value: string; onChange: (v: string) => void }) {
  const [hovered, setHovered] = useState(false);
  const [btnHovered, setBtnHovered] = useState(false);
  const [defaultDir, setDefaultDir] = useState('');

  useEffect(() => {
    invoke<string | null>('get_data_dir')
      .then((d) => {
        if (d) setDefaultDir(`${d}/offline`);
      })
      .catch(() => {});
  }, []);

  const pick = async () => {
    const selected = await platformOpenDialog({ directory: true, multiple: false });
    if (typeof selected === 'string' && selected) onChange(selected);
  };

  const effectivePath = value || defaultDir;
  const displayPath = effectivePath || t('settings.download_dir_default');

  return (
    <div
      style={{
        width: '100%',
        borderBottom: '1px solid rgba(255,255,255,0.055)',
        background: hovered ? 'rgba(255,255,255,0.03)' : 'transparent',
        display: 'flex',
        alignItems: 'center',
        padding: '0.75rem 1rem',
        boxSizing: 'border-box',
        gap: '1rem',
        transition: 'background 0.12s',
      }}
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
    >
      <div style={{ flex: 1, minWidth: 0 }}>
        <p style={styles.rowTitle}>{t('settings.download_dir')}</p>
        <p style={styles.rowSubtitle}>{t('settings.download_dir_desc')}</p>
      </div>
      <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem', flexShrink: 0 }}>
        <span
          style={{
            color: 'rgba(255,255,255,0.38)',
            fontSize: '0.75rem',
            fontFamily: FONT,
            maxWidth: '21.25rem',
            overflow: 'hidden',
            textOverflow: 'ellipsis',
            whiteSpace: 'nowrap',
          }}
          title={effectivePath || undefined}
        >
          {displayPath}
        </span>
        <button
          onClick={pick}
          onMouseEnter={() => setBtnHovered(true)}
          onMouseLeave={() => setBtnHovered(false)}
          style={{
            height: '2rem',
            padding: '0 0.875rem',
            background: btnHovered ? 'rgba(255,255,255,0.12)' : 'rgba(255,255,255,0.07)',
            border: 'none',
            outline: 'none',
            borderRadius: '0.5rem',
            color: '#FFFFFF',
            fontSize: '0.75rem',
            fontWeight: 500,
            fontFamily: FONT,
            cursor: 'pointer',
            transition: 'background 0.12s',
            whiteSpace: 'nowrap',
          }}
        >
          {t('settings.download_dir_browse')}
        </button>
        {value && (
          <button
            onClick={() => onChange('')}
            title={t('common.delete')}
            style={{
              background: 'transparent',
              border: 'none',
              color: 'rgba(255,255,255,0.28)',
              cursor: 'pointer',
              padding: '0.25rem',
              display: 'flex',
              alignItems: 'center',
              borderRadius: '0.3125rem',
            }}
          >
            <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
              <path d="M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z" />
            </svg>
          </button>
        )}
      </div>
    </div>
  );
}

export function DownloadsSection({ prefs, setPref }: { prefs: Prefs; setPref: <K extends keyof Prefs>(k: K, v: Prefs[K]) => void }) {
  const [downloads, setDownloads] = useState<OfflineDownloadItem[]>([]);
  const [confirmDelete, setConfirmDelete] = useState<OfflineDownloadItem | null>(null);

  useEffect(() => {
    invoke('set_download_dir', { path: prefs.downloadDir || null }).catch(() => {});
  }, [prefs.downloadDir]);

  const refreshDownloads = () => {
    invoke<OfflineDownloadItem[]>('list_offline_downloads')
      .then(setDownloads)
      .catch(() => setDownloads([]));
  };

  useEffect(() => {
    refreshDownloads();
  }, []);

  useEffect(() => {
    const unlisten = listen<DownloadProgressEvent>('download-progress', (e) => {
      setDownloads((prev) => {
        const idx = prev.findIndex((d) => d.id === e.payload.id);
        if (idx < 0) return prev;
        const next = [...prev];
        next[idx] = {
          ...next[idx],
          status: e.payload.status,
          downloadedBytes: e.payload.downloadedBytes,
          totalBytes: e.payload.totalBytes ?? next[idx].totalBytes,
          error: e.payload.error ?? undefined,
        };
        return next;
      });
      if (e.payload.status === 'downloaded') refreshDownloads();
    });
    return () => {
      void unlisten.then((fn) => fn());
    };
  }, []);

  const handleDelete = async (item: OfflineDownloadItem) => {
    try {
      await invoke('cancel_offline_download', { id: item.id });
      await invoke('delete_offline_download', { fileName: item.videoFileName });
      refreshDownloads();
    } catch {
      /* ignore */
    }
  };

  const handlePause = async (item: OfflineDownloadItem) => {
    try {
      await invoke('pause_offline_download', { id: item.id });
    } catch {
      /* ignore */
    }
  };

  const handleResume = async (item: OfflineDownloadItem) => {
    try {
      await invoke('resume_offline_download', { id: item.id });
      refreshDownloads();
    } catch {
      /* ignore */
    }
  };

  const totalSize = downloads.reduce((acc, d) => acc + (d.sizeBytes ?? 0), 0);

  return (
    <>
      <SettingsSection title={t('auto.downloads')} subtitle={t('settings.downloads_desc')}>
        <DownloadDirRow value={prefs.downloadDir} onChange={(v) => setPref('downloadDir', v)} />
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
          options={[
            { value: 'off', label: t('settings.download_subtitle_off') },
            { value: 'preferred', label: t('settings.download_subtitle_preferred') },
          ]}
          selected={prefs.downloadSubtitleLanguage}
          onSelect={(v) => setPref('downloadSubtitleLanguage', v)}
        />
      </SettingsSection>

      <div style={{ display: 'flex', flexDirection: 'column', gap: '0.375rem', marginBottom: '1.5rem' }}>
        <div style={{ paddingLeft: '0.25rem', display: 'flex', alignItems: 'flex-start', justifyContent: 'space-between' }}>
          <div>
            <p style={styles.groupTitle}>{t('settings.downloads_list_desc')}</p>
            {downloads.length > 0 && (
              <p style={styles.groupSubtitle}>{t('downloads.storage_summary', downloads.length, formatBytes(totalSize))}</p>
            )}
          </div>
          <RefreshButton onClick={refreshDownloads} />
        </div>

        <div style={styles.settingsCard}>
          {downloads.length === 0 ? (
            <div style={{ padding: '1.75rem 1rem', textAlign: 'center' }}>
              <p style={{ color: 'rgba(255,255,255,0.30)', fontSize: '0.8125rem', margin: 0, fontFamily: FONT }}>{t('downloads.empty')}</p>
            </div>
          ) : (
            downloads.map((item) => (
              <DownloadItemRow
                key={item.id}
                item={item}
                onDelete={() => setConfirmDelete(item)}
                onPause={() => void handlePause(item)}
                onResume={() => void handleResume(item)}
              />
            ))
          )}
        </div>
      </div>

      {confirmDelete && (
        <ConfirmDialog
          title={t('downloads.delete_confirm_title')}
          body={t('downloads.delete_confirm_message')}
          confirmLabel={t('downloads.delete')}
          cancelLabel={t('common.cancel')}
          destructive
          onCancel={() => setConfirmDelete(null)}
          onConfirm={() => {
            const item = confirmDelete;
            setConfirmDelete(null);
            void handleDelete(item);
          }}
        />
      )}
    </>
  );
}
