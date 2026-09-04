import { Check } from 'lucide-react';
import { t } from '../../i18n';
import type { SubtitleCaptureCue, SubtitleStyleKey, SubtitleStylePage } from './TrackPopover.shared';
import { CHARACTER_EDGES, ColorOption, SUBTITLE_COLORS, SUBTITLE_OPACITIES, rowBtn, styleBtn, SubtitleStyleNavigationRow } from './TrackPopover.shared';

interface SubtitleStylePanelProps {
  stylePage: SubtitleStylePage | null;
  setStylePage: (page: SubtitleStylePage | null) => void;
  subtitleDelay: number;
  subtitlePosition: number;
  subtitleColor: string;
  subtitleTextOpacity: string;
  subtitleSize: number;
  subtitleFont: string;
  subtitleCharacterEdge: string;
  subtitleOutlineColor: string;
  subtitleOutlineOpacity: string;
  subtitleBackgroundColor: string;
  subtitleBackgroundOpacity: string;
  subtitleForceStyle: boolean;
  subtitleShadow: boolean;
  onAdjustSubtitleDelay?: (delta: number) => void;
  onChooseSubtitlePosition?: (position: number) => void;
  onResetSubtitleDelay?: () => void;
  autoSyncing: boolean;
  onAutoSyncSubtitles?: () => void;
  subtitleCaptureCues: SubtitleCaptureCue[];
  onApplySubtitleCapture?: (cueStart: number) => void;
  fontOptions: string[];
  onChooseSubtitleFont?: (font: string) => void;
  onChooseSubtitleSize?: (size: number) => void;
  onAdjustSubtitleSize?: (delta: number) => void;
  onChooseSubtitleColor?: (color: string) => void;
  onChooseSubtitleStyle?: (key: SubtitleStyleKey, value: string | boolean) => void;
}
export function SubtitleStylePanel({
  stylePage,
  setStylePage,
  subtitleDelay,
  subtitlePosition,
  subtitleColor,
  subtitleTextOpacity,
  subtitleSize,
  subtitleFont,
  subtitleCharacterEdge,
  subtitleOutlineColor,
  subtitleOutlineOpacity,
  subtitleBackgroundColor,
  subtitleBackgroundOpacity,
  subtitleForceStyle,
  subtitleShadow,
  onAdjustSubtitleDelay,
  onChooseSubtitlePosition,
  onResetSubtitleDelay,
  autoSyncing,
  onAutoSyncSubtitles,
  subtitleCaptureCues,
  onApplySubtitleCapture,
  fontOptions,
  onChooseSubtitleFont,
  onChooseSubtitleSize,
  onAdjustSubtitleSize,
  onChooseSubtitleColor,
  onChooseSubtitleStyle,
}: SubtitleStylePanelProps) {
  return (
    <div style={{ padding: '0 0.875rem 0.625rem' }}>
      {!stylePage ? (
        <>
          <SubtitleStyleNavigationRow
            label={t('player.subtitle_delay')}
            value={`${subtitleDelay > 0 ? '+' : ''}${subtitleDelay.toFixed(1)}s`}
            onClick={() => setStylePage('delay')}
          />
          <SubtitleStyleNavigationRow
            label={t('settings.subtitle_position')}
            value={
              subtitlePosition === 100
                ? t('settings.subtitle_position_bottom')
                : subtitlePosition === 90
                  ? t('settings.subtitle_position_low')
                  : subtitlePosition === 80
                    ? t('settings.subtitle_position_middle')
                    : t('settings.subtitle_position_high')
            }
            onClick={() => setStylePage('position')}
          />
          <SubtitleStyleNavigationRow
            label={t('player.subtitle_color')}
            value={<ColorOption color={subtitleColor} />}
            onClick={() => setStylePage('textColor')}
          />
          <SubtitleStyleNavigationRow
            label={t('auto.text_transparency')}
            value={`${Math.round(Number(subtitleTextOpacity) * 100)}%`}
            onClick={() => setStylePage('textOpacity')}
          />
          <SubtitleStyleNavigationRow
            label={t('player.subtitle_size')}
            value={`${subtitleSize}%`}
            onClick={() => setStylePage('size')}
          />
          <SubtitleStyleNavigationRow
            label={t('player.subtitle_font')}
            value={subtitleFont === 'default' ? t('settings.subtitle_font_default') : subtitleFont}
            onClick={() => setStylePage('font')}
          />
          <SubtitleStyleNavigationRow
            label={t('player.subtitle_character_edge')}
            value={t(`player.subtitle_edge_${subtitleCharacterEdge.replace('-', '_')}`)}
            onClick={() => setStylePage('characterEdge')}
          />
          <SubtitleStyleNavigationRow
            label={t('settings.subtitle.outline_color')}
            value={<ColorOption color={subtitleOutlineColor} />}
            onClick={() => setStylePage('outlineColor')}
          />
          <SubtitleStyleNavigationRow
            label={t('settings.subtitle.outline_opacity')}
            value={`${Math.round(Number(subtitleOutlineOpacity) * 100)}%`}
            onClick={() => setStylePage('outlineOpacity')}
          />
          <SubtitleStyleNavigationRow
            label={t('auto.background_color')}
            value={<ColorOption color={subtitleBackgroundColor} />}
            onClick={() => setStylePage('backgroundColor')}
          />
          <SubtitleStyleNavigationRow
            label={t('auto.background_transparency')}
            value={`${Math.round(Number(subtitleBackgroundOpacity) * 100)}%`}
            onClick={() => setStylePage('backgroundOpacity')}
          />
          <SubtitleStyleNavigationRow
            label={t('settings.subtitle_force_style')}
            value={subtitleForceStyle ? t('common.on') : t('common.off')}
            onClick={() => setStylePage('forceStyle')}
          />
          <SubtitleStyleNavigationRow
            label={t('settings.subtitle_shadow')}
            value={subtitleShadow ? t('common.on') : t('common.off')}
            onClick={() => setStylePage('shadow')}
          />
        </>
      ) : (
        <div style={{ paddingBottom: '0.25rem' }}>
          {stylePage === 'delay' && (
            <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'center', gap: '0.75rem', padding: '1rem 0' }}>
              <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'center', gap: '0.625rem' }}>
                <button className="ui-popover-chip" onClick={() => onAdjustSubtitleDelay?.(-0.5)} style={styleBtn}>
                  −0.5s
                </button>
                <span style={{ color: '#fff', fontSize: '0.8125rem', minWidth: '3.25rem', textAlign: 'center' }}>
                  {subtitleDelay > 0 ? '+' : ''}
                  {subtitleDelay.toFixed(1)}s
                </span>
                <button className="ui-popover-chip" onClick={() => onAdjustSubtitleDelay?.(0.5)} style={styleBtn}>
                  +0.5s
                </button>
                <button className="ui-popover-chip" onClick={() => onResetSubtitleDelay?.()} style={styleBtn}>
                  {t('player.subtitle_reset')}
                </button>
              </div>
              <button
                className="ui-popover-chip"
                disabled={autoSyncing}
                onClick={onAutoSyncSubtitles}
                style={{ ...styleBtn, opacity: autoSyncing ? 0.55 : 1 }}
              >
                {autoSyncing ? t('player.subtitle_capture_loading') : t('player.subtitle_capture')}
              </button>
              {subtitleCaptureCues.map((cue) => (
                <button
                  key={`${cue.start}-${cue.text}`}
                  className="ui-popover-row"
                  onClick={() => onApplySubtitleCapture?.(cue.start)}
                  style={rowBtn}
                >
                  <span style={{ flex: 1, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>{cue.text}</span>
                  <span style={{ color: 'rgba(255,255,255,0.45)', fontVariantNumeric: 'tabular-nums' }}>
                    {new Date(cue.start * 1000).toISOString().slice(14, 19)}
                  </span>
                </button>
              ))}
            </div>
          )}
          {stylePage === 'position' &&
            [
              [100, 'settings.subtitle_position_bottom'],
              [90, 'settings.subtitle_position_low'],
              [80, 'settings.subtitle_position_middle'],
              [70, 'settings.subtitle_position_high'],
            ].map(([position, labelKey]) => (
              <button
                key={position}
                className="ui-popover-row"
                onClick={() => onChooseSubtitlePosition?.(Number(position))}
                style={{ ...rowBtn, color: Number(position) === subtitlePosition ? '#fff' : rowBtn.color }}
              >
                <span style={{ width: '0.875rem', color: 'var(--primary-accent-color)' }}>
                  {Number(position) === subtitlePosition && <Check size={14} />}
                </span>
                {t(String(labelKey))}
              </button>
            ))}
          {stylePage === 'font' &&
            fontOptions.map((font) => (
              <button
                key={font}
                className="ui-popover-row"
                onClick={() => onChooseSubtitleFont?.(font)}
                style={{
                  ...rowBtn,
                  color: font === subtitleFont ? '#fff' : rowBtn.color,
                  fontFamily: font === 'default' ? undefined : font,
                }}
              >
                <span style={{ width: '0.875rem', color: 'var(--primary-accent-color)' }}>
                  {font === subtitleFont && <Check size={14} />}
                </span>
                {font === 'default' ? t('settings.subtitle_font_default') : font}
              </button>
            ))}
          {stylePage === 'size' && (
            <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'center', gap: '0.625rem', padding: '1rem 0' }}>
              <button className="ui-popover-chip" onClick={() => onAdjustSubtitleSize?.(-5)} style={styleBtn}>
                −5%
              </button>
              <span style={{ color: '#fff', fontSize: '0.8125rem', minWidth: '3.25rem', textAlign: 'center' }}>{subtitleSize}%</span>
              <button className="ui-popover-chip" onClick={() => onAdjustSubtitleSize?.(5)} style={styleBtn}>
                +5%
              </button>
              <button className="ui-popover-chip" onClick={() => onChooseSubtitleSize?.(100)} style={styleBtn}>
                {t('player.subtitle_reset')}
              </button>
            </div>
          )}
          {stylePage === 'characterEdge' &&
            CHARACTER_EDGES.map((edge) => (
              <button
                key={edge}
                className="ui-popover-row"
                onClick={() => onChooseSubtitleStyle?.('subtitleCharacterEdge', edge)}
                style={{ ...rowBtn, color: edge === subtitleCharacterEdge ? '#fff' : rowBtn.color }}
              >
                <span style={{ width: '0.875rem', color: 'var(--primary-accent-color)' }}>
                  {edge === subtitleCharacterEdge && <Check size={14} />}
                </span>
                {t(`player.subtitle_edge_${edge.replace('-', '_')}`)}
              </button>
            ))}
          {(['textColor', 'outlineColor', 'backgroundColor'] as const).includes(
            stylePage as 'textColor' | 'outlineColor' | 'backgroundColor',
          ) &&
            SUBTITLE_COLORS.map(({ value }) => (
              <button
                key={value}
                className="ui-popover-row"
                onClick={() => {
                  if (stylePage === 'textColor') onChooseSubtitleColor?.(value);
                  else
                    onChooseSubtitleStyle?.(stylePage === 'outlineColor' ? 'subtitleOutlineColor' : 'subtitleBackgroundColor', value);
                }}
                style={{
                  ...rowBtn,
                  color:
                    (stylePage === 'textColor'
                      ? subtitleColor
                      : stylePage === 'outlineColor'
                        ? subtitleOutlineColor
                        : subtitleBackgroundColor) === value
                      ? '#fff'
                      : rowBtn.color,
                }}
              >
                <span style={{ width: '0.875rem', color: 'var(--primary-accent-color)' }}>
                  {(stylePage === 'textColor'
                    ? subtitleColor
                    : stylePage === 'outlineColor'
                      ? subtitleOutlineColor
                      : subtitleBackgroundColor) === value && <Check size={14} />}
                </span>
                <ColorOption color={value} />
              </button>
            ))}
          {(['textOpacity', 'outlineOpacity', 'backgroundOpacity'] as const).includes(
            stylePage as 'textOpacity' | 'outlineOpacity' | 'backgroundOpacity',
          ) &&
            SUBTITLE_OPACITIES.map((opacity) => (
              <button
                key={opacity}
                className="ui-popover-row"
                onClick={() =>
                  onChooseSubtitleStyle?.(
                    stylePage === 'textOpacity'
                      ? 'subtitleTextOpacity'
                      : stylePage === 'outlineOpacity'
                        ? 'subtitleOutlineOpacity'
                        : 'subtitleBackgroundOpacity',
                    opacity,
                  )
                }
                style={{
                  ...rowBtn,
                  color:
                    (stylePage === 'textOpacity'
                      ? subtitleTextOpacity
                      : stylePage === 'outlineOpacity'
                        ? subtitleOutlineOpacity
                        : subtitleBackgroundOpacity) === opacity
                      ? '#fff'
                      : rowBtn.color,
                }}
              >
                <span style={{ width: '0.875rem', color: 'var(--primary-accent-color)' }}>
                  {(stylePage === 'textOpacity'
                    ? subtitleTextOpacity
                    : stylePage === 'outlineOpacity'
                      ? subtitleOutlineOpacity
                      : subtitleBackgroundOpacity) === opacity && <Check size={14} />}
                </span>
                {Math.round(Number(opacity) * 100)}%
              </button>
            ))}
          {(['forceStyle', 'shadow'] as const).includes(stylePage as 'forceStyle' | 'shadow') &&
            [true, false].map((enabled) => (
              <button
                key={String(enabled)}
                className="ui-popover-row"
                onClick={() => onChooseSubtitleStyle?.(stylePage === 'forceStyle' ? 'subtitleForceStyle' : 'subtitleShadow', enabled)}
                style={{
                  ...rowBtn,
                  color: (stylePage === 'forceStyle' ? subtitleForceStyle : subtitleShadow) === enabled ? '#fff' : rowBtn.color,
                }}
              >
                <span style={{ width: '0.875rem', color: 'var(--primary-accent-color)' }}>
                  {(stylePage === 'forceStyle' ? subtitleForceStyle : subtitleShadow) === enabled && <Check size={14} />}
                </span>
                {enabled ? t('common.on') : t('common.off')}
              </button>
            ))}
        </div>
      )}
    </div>
  );
}
