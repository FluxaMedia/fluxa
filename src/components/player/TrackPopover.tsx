import { useEffect, useRef, useState, type CSSProperties, type KeyboardEvent, type ReactNode, type RefObject } from 'react';
import { Check, ChevronLeft, ChevronRight, Settings } from 'lucide-react';
import { t } from '../../i18n';
import type { PlayerTrackOption } from '../../core/mpvPlayer';
import { Popover } from '../ui/Popover';
import { BUILTIN_SUBTITLE_FONTS } from '../../core/subtitleFonts';
import { listCustomFonts } from '../../core/customFonts';

const SUBTITLE_OPACITIES = ['1.0', '0.75', '0.5', '0.25', '0.0'];
const SUBTITLE_COLORS: { value: string; labelKey: string }[] = [
  { value: '#FFFFFF', labelKey: 'auto.white' },
  { value: '#000000', labelKey: 'auto.black' },
  { value: '#FFE45C', labelKey: 'auto.yellow' },
  { value: '#FF5D5D', labelKey: 'auto.red' },
  { value: '#3F7CFF', labelKey: 'auto.blue' },
  { value: '#54D17A', labelKey: 'auto.green' },
  { value: '#FF8A3D', labelKey: 'auto.orange' },
];

const LANG_NAMES: Record<string, string> = {
  en: 'English', eng: 'English',
  tr: 'Turkish', tur: 'Turkish',
  ja: 'Japanese', jpn: 'Japanese',
  ko: 'Korean', kor: 'Korean',
  zh: 'Chinese', chi: 'Chinese', zho: 'Chinese',
  de: 'German', ger: 'German', deu: 'German',
  fr: 'French', fre: 'French', fra: 'French',
  es: 'Spanish', spa: 'Spanish',
  it: 'Italian', ita: 'Italian',
  pt: 'Portuguese', por: 'Portuguese',
  ru: 'Russian', rus: 'Russian',
  ar: 'Arabic', ara: 'Arabic',
  hi: 'Hindi', hin: 'Hindi',
  cs: 'Czech', cze: 'Czech', ces: 'Czech',
  da: 'Danish', dan: 'Danish',
  el: 'Greek', gre: 'Greek', ell: 'Greek',
  et: 'Estonian', est: 'Estonian',
  fi: 'Finnish', fin: 'Finnish',
  nl: 'Dutch', dut: 'Dutch', nld: 'Dutch',
  sv: 'Swedish', swe: 'Swedish',
  no: 'Norwegian', nor: 'Norwegian', nob: 'Norwegian', nno: 'Norwegian',
  pl: 'Polish', pol: 'Polish',
  ro: 'Romanian', rum: 'Romanian', ron: 'Romanian',
  sk: 'Slovak', slo: 'Slovak', slk: 'Slovak',
  sl: 'Slovenian', slv: 'Slovenian',
  hu: 'Hungarian', hun: 'Hungarian',
  uk: 'Ukrainian', ukr: 'Ukrainian',
  vi: 'Vietnamese', vie: 'Vietnamese',
  th: 'Thai', tha: 'Thai',
  id: 'Indonesian', ind: 'Indonesian',
  he: 'Hebrew', heb: 'Hebrew',
  ms: 'Malay', may: 'Malay', msa: 'Malay',
  bg: 'Bulgarian', bul: 'Bulgarian',
  ca: 'Catalan', cat: 'Catalan',
  lv: 'Latvian', lav: 'Latvian',
  lt: 'Lithuanian', lit: 'Lithuanian',
  is: 'Icelandic', ice: 'Icelandic', isl: 'Icelandic',
  fa: 'Persian', per: 'Persian', fas: 'Persian',
  ur: 'Urdu', urd: 'Urdu',
  bn: 'Bengali', ben: 'Bengali',
  ta: 'Tamil', tam: 'Tamil',
  te: 'Telugu', tel: 'Telugu',
  ml: 'Malayalam', mal: 'Malayalam',
  mr: 'Marathi', mar: 'Marathi',
  gu: 'Gujarati', guj: 'Gujarati',
  pa: 'Punjabi', pan: 'Punjabi',
  fil: 'Filipino', tgl: 'Filipino',
  sr: 'Serbian', srp: 'Serbian',
  hr: 'Croatian', hrv: 'Croatian',
  bs: 'Bosnian', bos: 'Bosnian',
  mk: 'Macedonian', mac: 'Macedonian', mkd: 'Macedonian',
  sq: 'Albanian', alb: 'Albanian', sqi: 'Albanian',
  ka: 'Georgian', geo: 'Georgian', kat: 'Georgian',
  hy: 'Armenian', arm: 'Armenian', hye: 'Armenian',
  az: 'Azerbaijani', aze: 'Azerbaijani',
  kk: 'Kazakh', kaz: 'Kazakh',
  uz: 'Uzbek', uzb: 'Uzbek',
  mn: 'Mongolian', mon: 'Mongolian',
  km: 'Khmer', khm: 'Khmer',
  lo: 'Lao', lao: 'Lao',
  my: 'Burmese', bur: 'Burmese', mya: 'Burmese',
  am: 'Amharic', amh: 'Amharic',
  sw: 'Swahili', swa: 'Swahili',
  af: 'Afrikaans', afr: 'Afrikaans',
  gl: 'Galician', glg: 'Galician',
  cy: 'Welsh', wel: 'Welsh', cym: 'Welsh',
};

function ColorOption({ color }: { color: string }) {
  const label = SUBTITLE_COLORS.find((c) => c.value === color)?.labelKey;
  return (
    <>
      <span style={{ width: '0.75rem', height: '0.75rem', borderRadius: '50%', background: color, border: '1px solid rgba(255,255,255,0.25)', flexShrink: 0 }} />
      {label ? t(label) : color}
    </>
  );
}

function langDisplayName(code: string | null): string {
  if (!code) return t('player.unknown_language');
  return LANG_NAMES[code.toLowerCase()] ?? code.toUpperCase();
}

function trackSourceLabel(track: PlayerTrackOption): string {
  if (track.source) return track.source;
  return track.external ? t('player.external_source') : t('player.embedded_source');
}

type TrackGroup = { key: string; label: string; tracks: PlayerTrackOption[] };
export type SubtitleCaptureCue = { start: number; end: number; text: string };
type SubtitleStylePage = 'delay' | 'position' | 'textColor' | 'textOpacity' | 'size' | 'font' | 'characterEdge' | 'outlineColor' | 'outlineOpacity' | 'backgroundColor' | 'backgroundOpacity' | 'forceStyle' | 'shadow';
const CHARACTER_EDGES = ['none', 'raised', 'depressed', 'uniform', 'drop-shadow'] as const;

function groupTracks(tracks: PlayerTrackOption[]): TrackGroup[] {
  const groups = new Map<string, TrackGroup>();
  for (const track of tracks) {
    const label = langDisplayName(track.lang);
    const key = track.lang ? label : 'und';
    let group = groups.get(key);
    if (!group) {
      group = { key, label, tracks: [] };
      groups.set(key, group);
    }
    group.tracks.push(track);
  }
  return Array.from(groups.values());
}

const styleBtn: CSSProperties = {
  background: 'rgba(255,255,255,0.07)',
  border: '1px solid rgba(255,255,255,0.10)',
  borderRadius: '0.375rem',
  color: 'rgba(255,255,255,0.85)',
  fontSize: '0.75rem',
  padding: '0.3125rem 0.5625rem',
  cursor: 'pointer',
};

const rowBtn: CSSProperties = {
  display: 'flex', alignItems: 'center', gap: '0.625rem', width: '100%', background: 'none', border: 'none',
  color: 'rgba(255,255,255,0.7)', fontSize: '0.8125rem', padding: '0.5rem 0.875rem', cursor: 'pointer', textAlign: 'left',
};

function SubtitleStyleNavigationRow({ label, value, onClick }: { label: string; value: ReactNode; onClick: () => void }) {
  return (
    <button type="button" onClick={onClick} style={{ width: '100%', minHeight: '2.25rem', display: 'flex', alignItems: 'center', justifyContent: 'space-between', gap: '0.75rem', background: 'none', border: 'none', borderBottom: '1px solid rgba(255,255,255,0.06)', color: 'rgba(255,255,255,0.78)', fontSize: '0.75rem', padding: 0, cursor: 'pointer', textAlign: 'left' }}>
      <span>{label}</span>
      <span style={{ display: 'flex', alignItems: 'center', gap: '0.25rem', maxWidth: '55%', color: 'rgba(255,255,255,0.72)', overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>{value}<ChevronRight size={14} style={{ flexShrink: 0, color: 'rgba(255,255,255,0.4)' }} /></span>
    </button>
  );
}

interface TrackPopoverProps {
  type: 'audio' | 'sub' | 'speed';
  audioTracks: PlayerTrackOption[];
  subTracks: PlayerTrackOption[];
  playbackSpeed: number;
  anchorRef: RefObject<HTMLElement | null>;
  onClose: () => void;
  onSetSpeed: (speed: number) => void;
  onSelectTrack: (type: 'audio' | 'sub', id: string) => void;
  onDisableSubs: () => void;
  subtitleDelay?: number;
  subtitlePosition?: number;
  subtitleFont?: string;
  subtitleSize?: number;
  subtitleColor?: string;
  subtitleTextOpacity?: string;
  subtitleBackgroundColor?: string;
  subtitleBackgroundOpacity?: string;
  subtitleOutlineColor?: string;
  subtitleOutlineOpacity?: string;
  subtitleForceStyle?: boolean;
  subtitleCharacterEdge?: string;
  subtitleShadow?: boolean;
  onAdjustSubtitleDelay?: (delta: number) => void;
  onChooseSubtitlePosition?: (position: number) => void;
  onResetSubtitleDelay?: () => void;
  autoSyncing?: boolean;
  onAutoSyncSubtitles?: () => void;
  subtitleCaptureCues?: SubtitleCaptureCue[];
  onApplySubtitleCapture?: (cueStart: number) => void;
  onChooseSubtitleFont?: (font: string) => void;
  onChooseSubtitleSize?: (size: number) => void;
  onAdjustSubtitleSize?: (delta: number) => void;
  onChooseSubtitleColor?: (color: string) => void;
  onChooseSubtitleStyle?: (key: 'subtitleTextOpacity' | 'subtitleBackgroundColor' | 'subtitleBackgroundOpacity' | 'subtitleOutlineColor' | 'subtitleOutlineOpacity' | 'subtitleForceStyle' | 'subtitleCharacterEdge' | 'subtitleShadow', value: string | boolean) => void;
}

export function TrackPopover({
  type, audioTracks, subTracks, playbackSpeed, anchorRef, onClose,
  onSetSpeed, onSelectTrack, onDisableSubs,
  subtitleDelay = 0, subtitlePosition = 100, subtitleFont = 'default', subtitleSize = 100, subtitleColor = '#FFFFFF', subtitleTextOpacity = '1.0', subtitleBackgroundColor = '#000000', subtitleBackgroundOpacity = '0.5', subtitleOutlineColor = '#000000', subtitleOutlineOpacity = '1.0', subtitleForceStyle = false, subtitleCharacterEdge = 'uniform', subtitleShadow = false,
  onAdjustSubtitleDelay, onChooseSubtitlePosition, onResetSubtitleDelay, autoSyncing = false, onAutoSyncSubtitles, subtitleCaptureCues = [], onApplySubtitleCapture, onChooseSubtitleFont, onChooseSubtitleSize, onAdjustSubtitleSize, onChooseSubtitleColor, onChooseSubtitleStyle,
}: TrackPopoverProps) {
  const [showStyle, setShowStyle] = useState(false);
  const [stylePage, setStylePage] = useState<SubtitleStylePage | null>(null);
  const [openGroup, setOpenGroup] = useState<string | null>(null);
  const [customFontFamilies, setCustomFontFamilies] = useState<string[]>([]);
  const contentRef = useRef<HTMLDivElement>(null);
  useEffect(() => { setShowStyle(false); setStylePage(null); setOpenGroup(null); }, [type]);
  useEffect(() => { void listCustomFonts().then((fonts) => setCustomFontFamilies(fonts.map((f) => f.family))); }, []);
  useEffect(() => {
    contentRef.current?.querySelector<HTMLButtonElement>('button:not(:disabled)')?.focus();
  }, [type, showStyle, stylePage, openGroup]);

  const onListKeyDown = (e: KeyboardEvent) => {
    if (e.key !== 'ArrowDown' && e.key !== 'ArrowUp') return;
    const buttons = Array.from(contentRef.current?.querySelectorAll<HTMLButtonElement>('button:not(:disabled)') ?? []);
    if (buttons.length === 0) return;
    const currentIndex = buttons.indexOf(document.activeElement as HTMLButtonElement);
    const nextIndex = ((currentIndex < 0 ? 0 : currentIndex + (e.key === 'ArrowDown' ? 1 : -1)) + buttons.length) % buttons.length;
    e.preventDefault();
    buttons[nextIndex]?.focus();
  };
  const fontOptions = [...BUILTIN_SUBTITLE_FONTS, ...customFontFamilies];

  const tracks = type === 'audio' ? audioTracks : subTracks;
  const noSubSelected = !subTracks.some((tr) => tr.selected);
  const groups = groupTracks(tracks);
  const activeGroup = groups.find((g) => g.key === openGroup) ?? null;

  const selectFromGroup = (group: TrackGroup, track: PlayerTrackOption) => {
    onSelectTrack(type as 'audio' | 'sub', track.id);
  };

  const openOrSelectGroup = (group: TrackGroup) => {
    if (group.tracks.length === 1) {
      selectFromGroup(group, group.tracks[0]);
    } else {
      setOpenGroup(group.key);
    }
  };

  return (
    <Popover
      open
      onClose={onClose}
      anchorRef={anchorRef}
      placement="top"
      width={type === 'speed' ? 150 : 260}
      maxHeight="34rem"
    >
      <div ref={contentRef} onKeyDown={onListKeyDown}>
      <div style={{ display: 'flex', alignItems: 'center', gap: '0.375rem', justifyContent: 'space-between', padding: '0.25rem 0.875rem 0.5rem' }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: '0.375rem', minWidth: 0 }}>
          {(activeGroup && !showStyle) || stylePage ? (
            <button
              className="ui-popover-icon"
              onClick={() => stylePage ? setStylePage(null) : setOpenGroup(null)}
              style={{ background: 'none', border: 'none', color: 'rgba(255,255,255,0.6)', cursor: 'pointer', padding: '0.1875rem', display: 'flex' }}
              title={t('player.back')}
            >
              <ChevronLeft size={14} />
            </button>
          ) : null}
          <span style={{ color: 'rgba(255,255,255,0.45)', fontSize: '0.6875rem', fontWeight: 700, letterSpacing: '0.05rem', textTransform: 'uppercase', overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>
            {showStyle
              ? stylePage === 'delay' ? t('player.subtitle_delay')
                : stylePage === 'position' ? t('settings.subtitle_position')
                : stylePage === 'textColor' ? t('player.subtitle_color')
                  : stylePage === 'textOpacity' ? t('auto.text_transparency')
                    : stylePage === 'size' ? t('player.subtitle_size')
                      : stylePage === 'font' ? t('player.subtitle_font')
                        : stylePage === 'outlineColor' ? t('settings.subtitle.outline_color')
                          : stylePage === 'outlineOpacity' ? t('settings.subtitle.outline_opacity')
                            : stylePage === 'backgroundColor' ? t('auto.background_color')
                              : stylePage === 'backgroundOpacity' ? t('auto.background_transparency')
                                : stylePage === 'characterEdge' ? t('player.subtitle_character_edge')
                                  : stylePage === 'forceStyle' ? t('settings.subtitle_force_style')
                                  : stylePage === 'shadow' ? t('settings.subtitle_shadow')
                                    : t('player.subtitle_settings')
              : activeGroup
                ? activeGroup.label
                : type === 'audio' ? t('player.audio_title') : type === 'sub' ? t('player.subtitles_title') : t('player.speed_title')}
          </span>
        </div>
        {type === 'sub' && !activeGroup && (
          <button
            className="ui-popover-icon"
            onClick={() => setShowStyle((v) => { if (v) setStylePage(null); return !v; })}
            style={{ background: 'none', border: 'none', color: showStyle ? 'var(--primary-accent-color)' : 'rgba(255,255,255,0.5)', cursor: 'pointer', padding: '0.1875rem', display: 'flex', flexShrink: 0 }}
            title={t('player.subtitle_settings')}
          >
            <Settings size={14} />
          </button>
        )}
      </div>
      {type === 'sub' && showStyle ? (
        <div style={{ padding: '0 0.875rem 0.625rem' }}>
          {!stylePage ? <>
            <SubtitleStyleNavigationRow label={t('player.subtitle_delay')} value={`${subtitleDelay > 0 ? '+' : ''}${subtitleDelay.toFixed(1)}s`} onClick={() => setStylePage('delay')} />
            <SubtitleStyleNavigationRow label={t('settings.subtitle_position')} value={subtitlePosition === 100 ? t('settings.subtitle_position_bottom') : subtitlePosition === 90 ? t('settings.subtitle_position_low') : subtitlePosition === 80 ? t('settings.subtitle_position_middle') : t('settings.subtitle_position_high')} onClick={() => setStylePage('position')} />
            <SubtitleStyleNavigationRow label={t('player.subtitle_color')} value={<ColorOption color={subtitleColor} />} onClick={() => setStylePage('textColor')} />
            <SubtitleStyleNavigationRow label={t('auto.text_transparency')} value={`${Math.round(Number(subtitleTextOpacity) * 100)}%`} onClick={() => setStylePage('textOpacity')} />
            <SubtitleStyleNavigationRow label={t('player.subtitle_size')} value={`${subtitleSize}%`} onClick={() => setStylePage('size')} />
            <SubtitleStyleNavigationRow label={t('player.subtitle_font')} value={subtitleFont === 'default' ? t('settings.subtitle_font_default') : subtitleFont} onClick={() => setStylePage('font')} />
            <SubtitleStyleNavigationRow label={t('player.subtitle_character_edge')} value={t(`player.subtitle_edge_${subtitleCharacterEdge.replace('-', '_')}`)} onClick={() => setStylePage('characterEdge')} />
            <SubtitleStyleNavigationRow label={t('settings.subtitle.outline_color')} value={<ColorOption color={subtitleOutlineColor} />} onClick={() => setStylePage('outlineColor')} />
            <SubtitleStyleNavigationRow label={t('settings.subtitle.outline_opacity')} value={`${Math.round(Number(subtitleOutlineOpacity) * 100)}%`} onClick={() => setStylePage('outlineOpacity')} />
            <SubtitleStyleNavigationRow label={t('auto.background_color')} value={<ColorOption color={subtitleBackgroundColor} />} onClick={() => setStylePage('backgroundColor')} />
            <SubtitleStyleNavigationRow label={t('auto.background_transparency')} value={`${Math.round(Number(subtitleBackgroundOpacity) * 100)}%`} onClick={() => setStylePage('backgroundOpacity')} />
            <SubtitleStyleNavigationRow label={t('settings.subtitle_force_style')} value={subtitleForceStyle ? t('common.on') : t('common.off')} onClick={() => setStylePage('forceStyle')} />
            <SubtitleStyleNavigationRow label={t('settings.subtitle_shadow')} value={subtitleShadow ? t('common.on') : t('common.off')} onClick={() => setStylePage('shadow')} />
          </> : <div style={{ paddingBottom: '0.25rem' }}>
            {stylePage === 'delay' && <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'center', gap: '0.75rem', padding: '1rem 0' }}><div style={{ display: 'flex', alignItems: 'center', justifyContent: 'center', gap: '0.625rem' }}><button className="ui-popover-chip" onClick={() => onAdjustSubtitleDelay?.(-0.5)} style={styleBtn}>−0.5s</button><span style={{ color: '#fff', fontSize: '0.8125rem', minWidth: '3.25rem', textAlign: 'center' }}>{subtitleDelay > 0 ? '+' : ''}{subtitleDelay.toFixed(1)}s</span><button className="ui-popover-chip" onClick={() => onAdjustSubtitleDelay?.(0.5)} style={styleBtn}>+0.5s</button><button className="ui-popover-chip" onClick={() => onResetSubtitleDelay?.()} style={styleBtn}>{t('player.subtitle_reset')}</button></div><button className="ui-popover-chip" disabled={autoSyncing} onClick={onAutoSyncSubtitles} style={{ ...styleBtn, opacity: autoSyncing ? 0.55 : 1 }}>{autoSyncing ? t('player.subtitle_capture_loading') : t('player.subtitle_capture')}</button>{subtitleCaptureCues.map((cue) => <button key={`${cue.start}-${cue.text}`} className="ui-popover-row" onClick={() => onApplySubtitleCapture?.(cue.start)} style={rowBtn}><span style={{ flex: 1, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>{cue.text}</span><span style={{ color: 'rgba(255,255,255,0.45)', fontVariantNumeric: 'tabular-nums' }}>{new Date(cue.start * 1000).toISOString().slice(14, 19)}</span></button>)}</div>}
            {stylePage === 'position' && [[100, 'settings.subtitle_position_bottom'], [90, 'settings.subtitle_position_low'], [80, 'settings.subtitle_position_middle'], [70, 'settings.subtitle_position_high']].map(([position, labelKey]) => <button key={position} className="ui-popover-row" onClick={() => onChooseSubtitlePosition?.(Number(position))} style={{ ...rowBtn, color: Number(position) === subtitlePosition ? '#fff' : rowBtn.color }}><span style={{ width: '0.875rem', color: 'var(--primary-accent-color)' }}>{Number(position) === subtitlePosition && <Check size={14} />}</span>{t(String(labelKey))}</button>)}
            {stylePage === 'font' && fontOptions.map((font) => <button key={font} className="ui-popover-row" onClick={() => onChooseSubtitleFont?.(font)} style={{ ...rowBtn, color: font === subtitleFont ? '#fff' : rowBtn.color, fontFamily: font === 'default' ? undefined : font }}><span style={{ width: '0.875rem', color: 'var(--primary-accent-color)' }}>{font === subtitleFont && <Check size={14} />}</span>{font === 'default' ? t('settings.subtitle_font_default') : font}</button>)}
            {stylePage === 'size' && (
              <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'center', gap: '0.625rem', padding: '1rem 0' }}>
                <button className="ui-popover-chip" onClick={() => onAdjustSubtitleSize?.(-5)} style={styleBtn}>−5%</button>
                <span style={{ color: '#fff', fontSize: '0.8125rem', minWidth: '3.25rem', textAlign: 'center' }}>{subtitleSize}%</span>
                <button className="ui-popover-chip" onClick={() => onAdjustSubtitleSize?.(5)} style={styleBtn}>+5%</button>
                <button className="ui-popover-chip" onClick={() => onChooseSubtitleSize?.(100)} style={styleBtn}>{t('player.subtitle_reset')}</button>
              </div>
            )}
            {stylePage === 'characterEdge' && CHARACTER_EDGES.map((edge) => <button key={edge} className="ui-popover-row" onClick={() => onChooseSubtitleStyle?.('subtitleCharacterEdge', edge)} style={{ ...rowBtn, color: edge === subtitleCharacterEdge ? '#fff' : rowBtn.color }}><span style={{ width: '0.875rem', color: 'var(--primary-accent-color)' }}>{edge === subtitleCharacterEdge && <Check size={14} />}</span>{t(`player.subtitle_edge_${edge.replace('-', '_')}`)}</button>)}
            {(['textColor', 'outlineColor', 'backgroundColor'] as const).includes(stylePage as 'textColor' | 'outlineColor' | 'backgroundColor') && SUBTITLE_COLORS.map(({ value }) => <button key={value} className="ui-popover-row" onClick={() => { if (stylePage === 'textColor') onChooseSubtitleColor?.(value); else onChooseSubtitleStyle?.(stylePage === 'outlineColor' ? 'subtitleOutlineColor' : 'subtitleBackgroundColor', value); }} style={{ ...rowBtn, color: (stylePage === 'textColor' ? subtitleColor : stylePage === 'outlineColor' ? subtitleOutlineColor : subtitleBackgroundColor) === value ? '#fff' : rowBtn.color }}><span style={{ width: '0.875rem', color: 'var(--primary-accent-color)' }}>{(stylePage === 'textColor' ? subtitleColor : stylePage === 'outlineColor' ? subtitleOutlineColor : subtitleBackgroundColor) === value && <Check size={14} />}</span><ColorOption color={value} /></button>)}
            {(['textOpacity', 'outlineOpacity', 'backgroundOpacity'] as const).includes(stylePage as 'textOpacity' | 'outlineOpacity' | 'backgroundOpacity') && SUBTITLE_OPACITIES.map((opacity) => <button key={opacity} className="ui-popover-row" onClick={() => onChooseSubtitleStyle?.(stylePage === 'textOpacity' ? 'subtitleTextOpacity' : stylePage === 'outlineOpacity' ? 'subtitleOutlineOpacity' : 'subtitleBackgroundOpacity', opacity)} style={{ ...rowBtn, color: (stylePage === 'textOpacity' ? subtitleTextOpacity : stylePage === 'outlineOpacity' ? subtitleOutlineOpacity : subtitleBackgroundOpacity) === opacity ? '#fff' : rowBtn.color }}><span style={{ width: '0.875rem', color: 'var(--primary-accent-color)' }}>{(stylePage === 'textOpacity' ? subtitleTextOpacity : stylePage === 'outlineOpacity' ? subtitleOutlineOpacity : subtitleBackgroundOpacity) === opacity && <Check size={14} />}</span>{Math.round(Number(opacity) * 100)}%</button>)}
            {(['forceStyle', 'shadow'] as const).includes(stylePage as 'forceStyle' | 'shadow') && [true, false].map((enabled) => <button key={String(enabled)} className="ui-popover-row" onClick={() => onChooseSubtitleStyle?.(stylePage === 'forceStyle' ? 'subtitleForceStyle' : 'subtitleShadow', enabled)} style={{ ...rowBtn, color: (stylePage === 'forceStyle' ? subtitleForceStyle : subtitleShadow) === enabled ? '#fff' : rowBtn.color }}><span style={{ width: '0.875rem', color: 'var(--primary-accent-color)' }}>{(stylePage === 'forceStyle' ? subtitleForceStyle : subtitleShadow) === enabled && <Check size={14} />}</span>{enabled ? t('common.on') : t('common.off')}</button>)}
          </div>}
        </div>
      ) : type === 'speed' ? (
        [0.25, 0.5, 0.75, 1.0, 1.25, 1.5, 1.75, 2.0].map((s) => (
          <button
            key={s}
            className="ui-popover-row"
            onClick={() => onSetSpeed(s)}
            style={{ ...rowBtn, color: playbackSpeed === s ? '#fff' : rowBtn.color, fontWeight: playbackSpeed === s ? 700 : 400 }}
          >
            <span style={{ width: '0.875rem', color: 'var(--primary-accent-color)' }}>{playbackSpeed === s && <Check size={14} />}</span>
            {s === 1.0 ? t('player.normal') : `${s}×`}
          </button>
        ))
      ) : activeGroup ? (
        activeGroup.tracks.map((track) => (
          <button
            key={track.id}
            className="ui-popover-row"
            onClick={() => selectFromGroup(activeGroup, track)}
            style={{ ...rowBtn, color: track.selected ? '#fff' : rowBtn.color, fontWeight: track.selected ? 600 : 400, justifyContent: 'space-between' }}
          >
            <span style={{ display: 'flex', alignItems: 'center', gap: '0.625rem', minWidth: 0 }}>
              <span style={{ width: '0.875rem', flexShrink: 0, color: 'var(--primary-accent-color)' }}>{track.selected && <Check size={14} />}</span>
              <span style={{ overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>{trackSourceLabel(track)}</span>
            </span>
            {track.format && (
              <span style={{ flexShrink: 0, fontSize: '0.6875rem', color: 'rgba(255,255,255,0.4)', border: '1px solid rgba(255,255,255,0.15)', borderRadius: '0.25rem', padding: '1px 0.375rem' }}>
                {track.format}
              </span>
            )}
          </button>
        ))
      ) : (
        <>
          {type === 'sub' && (
            <button
              className="ui-popover-row"
              onClick={onDisableSubs}
              style={{ ...rowBtn, borderBottom: '1px solid rgba(255,255,255,0.07)', color: noSubSelected ? '#fff' : rowBtn.color, fontWeight: noSubSelected ? 600 : 400, marginBottom: '0.25rem' }}
            >
              <span style={{ width: '0.875rem', color: 'var(--primary-accent-color)' }}>
                {noSubSelected && <Check size={14} />}
              </span>
              {t('player.subtitles_off')}
            </button>
          )}
          {groups.map((group) => {
            const groupSelected = group.tracks.some((tr) => tr.selected);
            const soleTrack = group.tracks.length === 1 ? group.tracks[0] : null;
            return (
              <button
                key={group.key}
                className="ui-popover-row"
                onClick={() => openOrSelectGroup(group)}
                style={{ ...rowBtn, color: groupSelected ? '#fff' : rowBtn.color, fontWeight: groupSelected ? 600 : 400, justifyContent: 'space-between' }}
              >
                <span style={{ display: 'flex', alignItems: 'center', gap: '0.625rem', minWidth: 0 }}>
                  <span style={{ width: '0.875rem', flexShrink: 0, color: 'var(--primary-accent-color)' }}>{groupSelected && <Check size={14} />}</span>
                  <span style={{ display: 'flex', flexDirection: 'column', gap: '0.125rem', minWidth: 0 }}>
                    <span style={{ overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>{group.label}</span>
                    {soleTrack && (
                      <span style={{ overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap', fontSize: '0.6875rem', fontWeight: 400, color: 'rgba(255,255,255,0.4)' }}>
                        {trackSourceLabel(soleTrack)}
                      </span>
                    )}
                  </span>
                </span>
                {soleTrack?.format ? (
                  <span style={{ flexShrink: 0, fontSize: '0.6875rem', color: 'rgba(255,255,255,0.4)', border: '1px solid rgba(255,255,255,0.15)', borderRadius: '0.25rem', padding: '1px 0.375rem' }}>
                    {soleTrack.format}
                  </span>
                ) : group.tracks.length > 1 ? (
                  <span style={{ fontSize: '0.6875rem', color: 'rgba(255,255,255,0.35)' }}>{group.tracks.length}</span>
                ) : null}
              </button>
            );
          })}
          {groups.length === 0 && (
            <div style={{ color: 'rgba(255,255,255,0.35)', fontSize: '0.8125rem', padding: '0.5rem 0.875rem' }}>{t('player.no_tracks_available')}</div>
          )}
        </>
      )}
      </div>
    </Popover>
  );
}
