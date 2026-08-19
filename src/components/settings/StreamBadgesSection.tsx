import React, { useState } from 'react';
import { t } from '../../i18n';
import { useStreamBadgeRules } from '../../hooks/useStreamBadgeRules';
import { SettingsSection } from './SettingsUI';
import { FONT } from './settingsStyles';

export function StreamBadgesSection() {
  const { rules, importing, importError, importFromUrl, setActiveSource, removeSource, badgePosition, setBadgePosition } =
    useStreamBadgeRules();
  const [url, setUrl] = useState('');

  const submit = () => {
    const trimmed = url.trim();
    if (!trimmed || importing) return;
    void importFromUrl(trimmed).then(() => setUrl(''));
  };

  return (
    <SettingsSection title={t('settings.stream_badges')} subtitle={t('settings.stream_badges_desc')}>
      <div style={styles.positionRow}>
        <span style={styles.positionLabel}>{t('settings.stream_badges_position')}</span>
        <div style={styles.positionToggle}>
          {(['top', 'bottom'] as const).map((position) => (
            <button
              key={position}
              onClick={() => setBadgePosition(position)}
              style={{
                ...styles.positionOption,
                ...(badgePosition === position ? styles.positionOptionActive : null),
              }}
            >
              {position === 'top' ? t('settings.stream_badges_position_top') : t('settings.stream_badges_position_bottom')}
            </button>
          ))}
        </div>
      </div>
      <div style={styles.importRow}>
        <input
          value={url}
          onChange={(e) => setUrl(e.target.value)}
          onKeyDown={(e) => e.key === 'Enter' && submit()}
          placeholder={t('settings.stream_badges_url_placeholder')}
          style={styles.input}
        />
        <button onClick={submit} disabled={importing || !url.trim()} style={styles.importButton}>
          {importing ? t('settings.stream_badges_importing') : t('settings.stream_badges_import')}
        </button>
      </div>
      {importError && <p style={styles.error}>{importError}</p>}

      {rules.imports.length === 0 ? (
        <p style={styles.empty}>{t('settings.stream_badges_empty')}</p>
      ) : (
        <div style={styles.list}>
          {rules.imports.map((imp) => (
            <div key={imp.sourceUrl} style={styles.row}>
              <button
                onClick={() => !imp.isActive && setActiveSource(imp.sourceUrl)}
                style={{ ...styles.radio, background: imp.isActive ? '#FFFFFF' : 'transparent' }}
                title={imp.isActive ? t('settings.stream_badges_active') : t('settings.stream_badges_set_active')}
              />
              <div style={styles.rowInfo}>
                <p style={styles.rowUrl}>{imp.sourceUrl}</p>
                <p style={styles.rowMeta}>{t('settings.stream_badges_filter_count', imp.filters.length)}</p>
              </div>
              <button onClick={() => removeSource(imp.sourceUrl)} style={styles.removeButton}>
                {t('auto.remove')}
              </button>
            </div>
          ))}
        </div>
      )}
    </SettingsSection>
  );
}

const styles: Record<string, React.CSSProperties> = {
  positionRow: {
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'space-between',
    marginBottom: '0.75rem',
  },
  positionLabel: {
    color: 'rgba(255,255,255,0.6)',
    fontSize: '0.8125rem',
    fontFamily: FONT,
  },
  positionToggle: {
    display: 'flex',
    background: 'rgba(255,255,255,0.06)',
    borderRadius: '0.5rem',
    padding: '0.1875rem',
    gap: '0.1875rem',
  },
  positionOption: {
    background: 'transparent',
    border: 'none',
    borderRadius: '0.375rem',
    padding: '0.375rem 0.75rem',
    color: 'rgba(255,255,255,0.6)',
    fontSize: '0.75rem',
    fontWeight: 600,
    fontFamily: FONT,
    cursor: 'pointer',
  },
  positionOptionActive: {
    background: 'rgba(255,255,255,0.14)',
    color: '#FFFFFF',
  },
  importRow: {
    display: 'flex',
    gap: '0.5rem',
  },
  input: {
    flex: 1,
    background: 'rgba(255,255,255,0.06)',
    border: '0.0625rem solid rgba(255,255,255,0.1)',
    borderRadius: '0.5rem',
    padding: '0.625rem 0.75rem',
    color: '#FFFFFF',
    fontSize: '0.875rem',
    fontFamily: FONT,
    outline: 'none',
  },
  importButton: {
    background: 'rgba(255,255,255,0.1)',
    border: '0.0625rem solid rgba(255,255,255,0.1)',
    borderRadius: '0.5rem',
    padding: '0 1rem',
    color: '#FFFFFF',
    fontSize: '0.875rem',
    fontWeight: 600,
    fontFamily: FONT,
    cursor: 'pointer',
  },
  error: {
    color: 'rgba(255,120,120,0.9)',
    fontSize: '0.75rem',
    margin: '0.5rem 0 0',
  },
  empty: {
    color: 'rgba(255,255,255,0.4)',
    fontSize: '0.8125rem',
    margin: '0.75rem 0 0',
  },
  list: {
    display: 'flex',
    flexDirection: 'column',
    gap: '0.375rem',
    marginTop: '0.75rem',
  },
  row: {
    display: 'flex',
    alignItems: 'center',
    gap: '0.75rem',
    padding: '0.625rem 0.75rem',
    background: 'rgba(255,255,255,0.04)',
    borderRadius: '0.5rem',
  },
  radio: {
    width: '0.75rem',
    height: '0.75rem',
    borderRadius: '50%',
    border: '0.0625rem solid rgba(255,255,255,0.4)',
    padding: 0,
    cursor: 'pointer',
    flexShrink: 0,
  },
  rowInfo: {
    flex: 1,
    minWidth: 0,
  },
  rowUrl: {
    color: '#FFFFFF',
    fontSize: '0.8125rem',
    margin: 0,
    overflow: 'hidden',
    textOverflow: 'ellipsis',
    whiteSpace: 'nowrap',
  },
  rowMeta: {
    color: 'rgba(255,255,255,0.45)',
    fontSize: '0.6875rem',
    margin: '0.125rem 0 0',
  },
  removeButton: {
    background: 'transparent',
    border: 'none',
    color: 'rgba(255,255,255,0.5)',
    fontSize: '0.75rem',
    fontFamily: FONT,
    cursor: 'pointer',
    flexShrink: 0,
  },
};
