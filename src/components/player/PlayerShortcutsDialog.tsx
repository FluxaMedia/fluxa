import { t } from '../../i18n';
import { type ShortcutOverrides } from '../../core/shortcuts';
import type { GamepadBindingOverrides } from '../../core/gamepadBindings';
import { actionHintText, resolveActionHint } from '../ActionHint';
import { useLastInputMethod } from '../../core/inputMethod';
import { getConnectedGamepads } from '../../platform/gamepadInput';

interface Props {
  overrides: ShortcutOverrides;
  gamepadOverrides: GamepadBindingOverrides;
  onClose: () => void;
}

export function PlayerShortcutsDialog({ overrides, gamepadOverrides, onClose }: Props) {
  const lastInput = useLastInputMethod();
  const pads = getConnectedGamepads();
  const combo = (action: string) => actionHintText(resolveActionHint(action, lastInput, pads, overrides, gamepadOverrides));
  const groups: { heading: string; rows: [string, string][] }[] = [
    {
      heading: t('player.shortcut_group_playback'),
      rows: [
        ['Space', t('player.shortcut_play_pause_hold')],
        [combo('player_play_pause'), t('player.shortcut_play_pause')],
        [combo('player_mute'), t('player.shortcut_mute')],
      ],
    },
    {
      heading: t('player.shortcut_group_seek'),
      rows: [
        [`${combo('player_seek_back')}  ${combo('player_seek_forward')}`, t('player.shortcut_seek_10')],
        [`${combo('player_seek_big_back')}  ${combo('player_seek_big_forward')}`, t('player.shortcut_seek_60')],
        ['0 – 9', t('player.shortcut_percent_seek')],
        ['Shift + 1 – 9', t('player.shortcut_chapter_jump')],
        [`${combo('player_seek_start')} / ${combo('player_seek_end')}`, t('player.shortcut_seek_start_end')],
      ],
    },
    {
      heading: t('player.shortcut_group_speed'),
      rows: [[`${combo('player_speed_decrease')} ${combo('player_speed_increase')}`, t('player.shortcut_speed_step')]],
    },
    {
      heading: t('player.shortcut_group_volume'),
      rows: [[`${combo('player_volume_up')} ${combo('player_volume_down')}`, t('player.shortcut_volume')]],
    },
    {
      heading: t('player.shortcut_group_frame'),
      rows: [
        [`${combo('player_frame_step_back')} ${combo('player_frame_step_forward')}`, t('player.shortcut_frame_step')],
        [`${combo('player_sub_delay_earlier')}  ${combo('player_sub_delay_later')}`, t('player.shortcut_sub_delay')],
      ],
    },
    {
      heading: t('player.shortcut_group_tracks'),
      rows: [
        [combo('player_cycle_subtitle'), t('player.shortcut_cycle_sub')],
        [combo('player_cycle_audio'), t('player.shortcut_cycle_audio')],
      ],
    },
    {
      heading: t('player.shortcut_group_interface'),
      rows: [
        [`${combo('player_fullscreen')} / F11`, t('player.shortcut_fullscreen')],
        [`${combo('player_skip_active')} / Enter`, t('player.shortcut_skip')],
        [combo('player_next_episode'), t('player.shortcut_next_ep')],
        [combo('player_toggle_stats'), t('player.shortcut_stats')],
        [combo('player_toggle_shortcuts_help'), t('player.shortcut_this_help')],
        ['Backspace', t('player.shortcut_close')],
      ],
    },
    {
      heading: t('player.shortcut_group_extras'),
      rows: [
        [combo('player_toggle_pip'), t('player.shortcut_pip')],
        [combo('player_open_cast'), t('player.shortcut_cast')],
        [combo('player_ab_loop'), t('player.shortcut_ab_loop')],
        [combo('player_screenshot'), t('player.shortcut_screenshot')],
      ],
    },
  ];
  return (
    <div
      style={{
        position: 'fixed',
        inset: 0,
        zIndex: 30,
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        background: 'rgba(0,0,0,0.55)',
      }}
      onClick={onClose}
    >
      <div
        style={{
          background: 'rgba(14,16,22,0.97)',
          border: '1px solid rgba(255,255,255,0.1)',
          borderRadius: '0.75rem',
          padding: '1.25rem 1.5rem',
          maxWidth: '36.25rem',
          width: '90vw',
          maxHeight: '80vh',
          overflowY: 'auto',
        }}
        onClick={(event) => event.stopPropagation()}
      >
        <div
          style={{
            fontSize: '0.8125rem',
            fontWeight: 700,
            color: 'rgba(255,255,255,0.5)',
            letterSpacing: '0.08em',
            textTransform: 'uppercase',
            marginBottom: '1rem',
          }}
        >
          {t('player.shortcuts_help')}
        </div>
        <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '0 2rem' }}>
          {groups.map(({ heading, rows }) => (
            <div key={heading} style={{ marginBottom: '1rem' }}>
              <div
                style={{
                  fontSize: '0.625rem',
                  fontWeight: 700,
                  color: 'rgba(255,255,255,0.35)',
                  letterSpacing: '0.1em',
                  textTransform: 'uppercase',
                  marginBottom: '0.375rem',
                }}
              >
                {heading}
              </div>
              {rows.map(([key, description]) => (
                <div key={key} style={{ display: 'flex', alignItems: 'baseline', gap: '0.625rem', marginBottom: '0.25rem' }}>
                  <span
                    style={{
                      fontFamily: 'monospace',
                      fontSize: '0.75rem',
                      color: '#fff',
                      background: 'rgba(255,255,255,0.08)',
                      borderRadius: '0.25rem',
                      padding: '1px 0.375rem',
                      whiteSpace: 'nowrap',
                      flexShrink: 0,
                      minWidth: '3.25rem',
                      textAlign: 'center',
                    }}
                  >
                    {key}
                  </span>
                  <span style={{ fontSize: '0.75rem', color: 'rgba(255,255,255,0.65)', lineHeight: 1.4 }}>{description}</span>
                </div>
              ))}
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}
