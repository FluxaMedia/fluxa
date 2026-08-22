import { useEffect, useState } from 'react';
import { RotateCcw } from 'lucide-react';
import { t } from '../../i18n';
import { SettingsSection } from './SettingsUI';
import { styles, FONT } from './settingsStyles';
import {
  GAMEPAD_ACTION_DEFS,
  loadGamepadBindingOverrides,
  resolveGamepadButton,
  saveGamepadBindingOverrides,
  type GamepadActionCategory,
  type GamepadBindingOverrides,
} from '../../core/gamepadBindings';
import { detectGamepadBrand, GamepadButtonGlyph, type GamepadBrand } from '../../platform/gamepadGlyphs';
import { getConnectedGamepads, onRawGamepadButton } from '../../platform/gamepadInput';
import type { TvAction } from '../../platform/webos/keys';

function useConnectedGamepad(): Gamepad | null {
  const [pad, setPad] = useState<Gamepad | null>(() => getConnectedGamepads()[0] ?? null);
  useEffect(() => {
    const refresh = () => setPad(getConnectedGamepads()[0] ?? null);
    window.addEventListener('gamepadconnected', refresh);
    window.addEventListener('gamepaddisconnected', refresh);
    const interval = setInterval(refresh, 1000);
    return () => {
      window.removeEventListener('gamepadconnected', refresh);
      window.removeEventListener('gamepaddisconnected', refresh);
      clearInterval(interval);
    };
  }, []);
  return pad;
}

function GamepadBindingButton({
  brand,
  index,
  recording,
  onStartRecording,
}: {
  brand: GamepadBrand;
  index: number;
  recording: boolean;
  onStartRecording: () => void;
}) {
  const [hovered, setHovered] = useState(false);
  return (
    <button
      type="button"
      onClick={onStartRecording}
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
      style={{
        minWidth: '5.5rem',
        height: '2.25rem',
        padding: '0 0.75rem',
        borderRadius: '0.5rem',
        border: recording ? '1px solid var(--primary-accent-color)' : '1px solid rgba(255,255,255,0.10)',
        background: recording ? 'rgba(255,255,255,0.08)' : hovered ? 'rgba(255,255,255,0.06)' : '#1A1A1A',
        cursor: 'pointer',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        gap: '0.375rem',
        flexShrink: 0,
      }}
    >
      {recording ? (
        <span style={{ color: 'var(--primary-accent-color)', fontSize: '0.75rem', fontWeight: 600 }}>
          {t('settings.controller_recording')}
        </span>
      ) : index >= 0 ? (
        <GamepadButtonGlyph brand={brand} index={index} size={22} />
      ) : (
        <span style={{ color: 'rgba(255,255,255,0.35)', fontSize: '0.75rem' }}>{t('settings.controller_unassigned')}</span>
      )}
    </button>
  );
}

function ControllerRow({
  title,
  brand,
  index,
  isDefault,
  recording,
  onStartRecording,
  onReset,
}: {
  title: string;
  brand: GamepadBrand;
  index: number;
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
          aria-label={t('settings.controller_reset_one')}
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
      <GamepadBindingButton brand={brand} index={index} recording={recording} onStartRecording={onStartRecording} />
    </div>
  );
}

export function ControllerSection() {
  const [overrides, setOverrides] = useState<GamepadBindingOverrides>({});
  const [recordingId, setRecordingId] = useState<TvAction | null>(null);
  const pad = useConnectedGamepad();
  const brand: GamepadBrand = pad ? detectGamepadBrand(pad.id) : 'generic';

  useEffect(() => {
    loadGamepadBindingOverrides().then(setOverrides);
  }, []);

  useEffect(() => {
    if (!recordingId) return;
    return onRawGamepadButton((index) => {
      const next: GamepadBindingOverrides = { ...overrides };
      for (const other of GAMEPAD_ACTION_DEFS) {
        if (other.id === recordingId) continue;
        if (resolveGamepadButton(other.id, next) === index) delete next[other.id];
      }
      next[recordingId] = index;
      setOverrides(next);
      void saveGamepadBindingOverrides(next);
      setRecordingId(null);
    });
  }, [recordingId, overrides]);

  useEffect(() => {
    if (!recordingId) return;
    const onKeyDown = (e: KeyboardEvent) => {
      if (e.code === 'Escape') setRecordingId(null);
    };
    window.addEventListener('keydown', onKeyDown);
    return () => window.removeEventListener('keydown', onKeyDown);
  }, [recordingId]);

  const resetOne = (id: TvAction) => {
    const next = { ...overrides };
    delete next[id];
    setOverrides(next);
    void saveGamepadBindingOverrides(next);
  };

  const resetAll = () => {
    setOverrides({});
    void saveGamepadBindingOverrides({});
  };

  const groups: { category: GamepadActionCategory; titleKey: string; subtitleKey: string }[] = [
    { category: 'global', titleKey: 'settings.controller_group_general', subtitleKey: 'settings.controller_group_general_desc' },
    { category: 'player', titleKey: 'settings.controller_group_player', subtitleKey: 'settings.controller_group_player_desc' },
  ];

  return (
    <>
      <SettingsSection title={t('settings.controller_status_title')} subtitle={pad ? pad.id : t('settings.controller_no_gamepad')}>
        <div style={{ padding: '0.75rem 1rem', display: 'flex', alignItems: 'center', gap: '0.625rem' }}>
          <span
            style={{
              width: '0.5rem',
              height: '0.5rem',
              borderRadius: '50%',
              background: pad ? '#5FA341' : 'rgba(255,255,255,0.25)',
              flexShrink: 0,
            }}
          />
          <span style={{ fontSize: '0.75rem', color: 'rgba(255,255,255,0.65)' }}>
            {pad ? t('settings.controller_connected', brand) : t('settings.controller_no_gamepad')}
          </span>
        </div>
      </SettingsSection>
      {groups.map(({ category, titleKey, subtitleKey }) => (
        <SettingsSection key={category} title={t(titleKey)} subtitle={t(subtitleKey)}>
          {GAMEPAD_ACTION_DEFS.filter((def) => def.category === category).map((def) => (
            <ControllerRow
              key={def.id}
              title={t(def.labelKey)}
              brand={brand}
              index={resolveGamepadButton(def.id, overrides)}
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
          {t('settings.controller_reset_all')}
        </button>
      </div>
    </>
  );
}
