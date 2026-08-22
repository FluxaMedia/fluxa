import React, { useEffect, useState } from 'react';
import { RotateCcw } from 'lucide-react';
import { t } from '../../i18n';
import { SettingsSection } from './SettingsUI';
import { styles, FONT } from './settingsStyles';
import {
  SHORTCUT_DEFS,
  comboFromEvent,
  formatCombo,
  isModifierCode,
  loadShortcutOverrides,
  resolveCombo,
  saveShortcutOverrides,
  type ShortcutCategory,
  type ShortcutOverrides,
} from '../../core/shortcuts';

function ShortcutKeyButton({ combo, recording, onStartRecording }: { combo: string; recording: boolean; onStartRecording: () => void }) {
  const [hovered, setHovered] = useState(false);
  return (
    <button
      type="button"
      onClick={onStartRecording}
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
      style={{
        minWidth: '8.5rem',
        height: '2.25rem',
        padding: '0 0.75rem',
        borderRadius: '0.5rem',
        border: recording ? '1px solid var(--primary-accent-color)' : '1px solid rgba(255,255,255,0.10)',
        background: recording ? 'rgba(255,255,255,0.08)' : hovered ? 'rgba(255,255,255,0.06)' : '#1A1A1A',
        color: recording ? 'var(--primary-accent-color)' : combo ? '#FFFFFF' : 'rgba(255,255,255,0.35)',
        fontSize: '0.75rem',
        fontWeight: 600,
        fontFamily: 'monospace',
        cursor: 'pointer',
        textAlign: 'center',
        flexShrink: 0,
      }}
    >
      {recording ? t('settings.shortcuts_recording') : combo ? formatCombo(combo) : t('settings.shortcuts_unassigned')}
    </button>
  );
}

function ShortcutRow({
  title,
  combo,
  isDefault,
  recording,
  onStartRecording,
  onReset,
}: {
  title: string;
  combo: string;
  isDefault: boolean;
  recording: boolean;
  onStartRecording: () => void;
  onReset: () => void;
}) {
  const [hovered, setHovered] = useState(false);
  return (
    <div
      style={{
        width: '100%',
        minHeight: '3.25rem',
        borderBottom: '1px solid rgba(255,255,255,0.055)',
        display: 'flex',
        alignItems: 'center',
        padding: '0.625rem 1rem',
        boxSizing: 'border-box',
        gap: '0.75rem',
        background: hovered ? 'rgba(255,255,255,0.03)' : 'transparent',
      }}
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
    >
      <p style={{ ...styles.rowTitle, flex: 1, minWidth: 0 }}>{title}</p>
      {!isDefault && (
        <button
          type="button"
          aria-label={t('settings.shortcuts_reset_one')}
          onClick={onReset}
          style={{
            background: 'none',
            border: 'none',
            color: 'rgba(255,255,255,0.35)',
            cursor: 'pointer',
            display: 'flex',
            padding: '0.25rem',
          }}
        >
          <RotateCcw size={14} />
        </button>
      )}
      <ShortcutKeyButton combo={combo} recording={recording} onStartRecording={onStartRecording} />
    </div>
  );
}

export function ShortcutsSection() {
  const [overrides, setOverrides] = useState<ShortcutOverrides>({});
  const [recordingId, setRecordingId] = useState<string | null>(null);

  useEffect(() => {
    loadShortcutOverrides().then(setOverrides);
  }, []);

  useEffect(() => {
    if (!recordingId) return;
    const onKeyDown = (e: KeyboardEvent) => {
      e.preventDefault();
      e.stopPropagation();
      if (isModifierCode(e.code)) return;
      if (e.code === 'Escape') {
        setRecordingId(null);
        return;
      }

      const combo = comboFromEvent(e);
      const def = SHORTCUT_DEFS.find((d) => d.id === recordingId);
      if (!def) {
        setRecordingId(null);
        return;
      }

      const next: ShortcutOverrides = { ...overrides };
      for (const other of SHORTCUT_DEFS) {
        if (other.id === def.id || other.category !== def.category) continue;
        if (resolveCombo(other.id, next) === combo) next[other.id] = '';
      }
      next[def.id] = combo;
      setOverrides(next);
      void saveShortcutOverrides(next);
      setRecordingId(null);
    };
    window.addEventListener('keydown', onKeyDown, true);
    return () => window.removeEventListener('keydown', onKeyDown, true);
  }, [recordingId, overrides]);

  const resetOne = (id: string) => {
    const next = { ...overrides };
    delete next[id];
    setOverrides(next);
    void saveShortcutOverrides(next);
  };

  const resetAll = () => {
    setOverrides({});
    void saveShortcutOverrides({});
  };

  const groups: { category: ShortcutCategory; titleKey: string; subtitleKey: string }[] = [
    { category: 'global', titleKey: 'settings.shortcuts_group_general', subtitleKey: 'settings.shortcuts_group_general_desc' },
    { category: 'player', titleKey: 'settings.shortcuts_group_player', subtitleKey: 'settings.shortcuts_group_player_desc' },
  ];

  return (
    <>
      {groups.map(({ category, titleKey, subtitleKey }) => (
        <SettingsSection key={category} title={t(titleKey)} subtitle={t(subtitleKey)}>
          {SHORTCUT_DEFS.filter((def) => def.category === category).map((def) => (
            <ShortcutRow
              key={def.id}
              title={t(def.labelKey)}
              combo={resolveCombo(def.id, overrides)}
              isDefault={overrides[def.id] === undefined}
              recording={recordingId === def.id}
              onStartRecording={() => setRecordingId(def.id)}
              onReset={() => resetOne(def.id)}
            />
          ))}
        </SettingsSection>
      ))}
      <div style={{ display: 'flex', justifyContent: 'flex-end', marginTop: '-0.5rem', marginBottom: '1.5rem' }}>
        <button
          type="button"
          onClick={resetAll}
          style={{
            background: 'none',
            border: '1px solid rgba(255,255,255,0.14)',
            borderRadius: '0.5rem',
            color: 'rgba(255,255,255,0.65)',
            fontFamily: FONT,
            fontSize: '0.75rem',
            fontWeight: 600,
            padding: '0.5rem 0.875rem',
            cursor: 'pointer',
          }}
        >
          {t('settings.shortcuts_reset_all')}
        </button>
      </div>
    </>
  );
}
