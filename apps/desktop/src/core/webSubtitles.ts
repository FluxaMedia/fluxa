export type SubtitleFormat = 'ass' | 'vtt' | 'srt';

export type SubtitleStylePrefs = {
  font: string;
  size: number;
  color: string;
  textOpacity: number;
  backgroundColor: string;
  backgroundOpacity: number;
  outlineColor: string;
  outlineOpacity: number;
  outlineSize: number;
  bold: boolean;
  shadow: boolean;
  characterEdge: string;
  position: number;
  forceStyle: boolean;
};

export const PLAY_RES_X = 1920;
export const PLAY_RES_Y = 1080;

const BASE_FONT_SIZE = 54;
const BASE_MARGIN_V = 40;

export const defaultSubtitleStylePrefs: SubtitleStylePrefs = {
  font: 'default',
  size: 100,
  color: '#FFFFFF',
  textOpacity: 1,
  backgroundColor: '#000000',
  backgroundOpacity: 0,
  outlineColor: '#000000',
  outlineOpacity: 1,
  outlineSize: 3,
  bold: false,
  shadow: false,
  characterEdge: 'uniform',
  position: 100,
  forceStyle: false,
};

function numberPref(source: Record<string, unknown>, key: string, fallback: number): number {
  const parsed = Number(source[key]);
  return Number.isFinite(parsed) ? parsed : fallback;
}

function stringPref(source: Record<string, unknown>, key: string, fallback: string): string {
  const value = source[key];
  return typeof value === 'string' && value.trim() !== '' ? value : fallback;
}

export function subtitleStylePrefsFrom(source: Record<string, unknown> | null | undefined): SubtitleStylePrefs {
  const prefs = source ?? {};
  return {
    font: stringPref(prefs, 'subtitleFont', defaultSubtitleStylePrefs.font),
    size: numberPref(prefs, 'subtitleSize', defaultSubtitleStylePrefs.size),
    color: stringPref(prefs, 'subtitleColor', defaultSubtitleStylePrefs.color),
    textOpacity: numberPref(prefs, 'subtitleTextOpacity', defaultSubtitleStylePrefs.textOpacity),
    backgroundColor: stringPref(prefs, 'subtitleBackgroundColor', defaultSubtitleStylePrefs.backgroundColor),
    backgroundOpacity: numberPref(prefs, 'subtitleBackgroundOpacity', defaultSubtitleStylePrefs.backgroundOpacity),
    outlineColor: stringPref(prefs, 'subtitleOutlineColor', defaultSubtitleStylePrefs.outlineColor),
    outlineOpacity: numberPref(prefs, 'subtitleOutlineOpacity', defaultSubtitleStylePrefs.outlineOpacity),
    outlineSize: numberPref(prefs, 'subtitleOutlineSize', defaultSubtitleStylePrefs.outlineSize),
    bold: prefs.subtitleBold === true,
    shadow: prefs.subtitleShadow === true,
    characterEdge: stringPref(prefs, 'subtitleCharacterEdge', defaultSubtitleStylePrefs.characterEdge),
    position: numberPref(prefs, 'subtitlePosition', defaultSubtitleStylePrefs.position),
    forceStyle: prefs.subtitleForceStyle === true,
  };
}

function clamp(value: number, min: number, max: number): number {
  return Math.min(max, Math.max(min, value));
}

function channel(hex: string, start: number): number {
  const parsed = Number.parseInt(hex.slice(start, start + 2), 16);
  return Number.isFinite(parsed) ? parsed : 0;
}

export function assColor(hex: string, opacity: number): string {
  const normalized = hex.trim().replace(/^#/, '').padEnd(6, '0');
  const r = channel(normalized, 0);
  const g = channel(normalized, 2);
  const b = channel(normalized, 4);
  const alpha = Math.round((1 - clamp(opacity, 0, 1)) * 255);
  const part = (value: number) => value.toString(16).toUpperCase().padStart(2, '0');
  return `&H${part(alpha)}${part(b)}${part(g)}${part(r)}`;
}

function fontName(font: string): string {
  const trimmed = font.trim();
  return !trimmed || trimmed === 'default' ? 'Liberation Sans' : trimmed;
}

export function buildStyleLine(prefs: SubtitleStylePrefs, name = 'Default'): string {
  const boxed = prefs.backgroundOpacity > 0;
  const edge = prefs.characterEdge;
  const outline = boxed ? clamp(prefs.outlineSize, 0, 20) : edge === 'none' ? 0 : clamp(prefs.outlineSize, 0, 20);
  const shadowDepth = prefs.shadow || edge === 'dropshadow' || edge === 'raised' || edge === 'depressed' ? 2 : 0;
  const primary = assColor(prefs.color, prefs.textOpacity);
  const border = boxed ? assColor(prefs.backgroundColor, prefs.backgroundOpacity) : assColor(prefs.outlineColor, prefs.outlineOpacity);
  const back = assColor(prefs.outlineColor, prefs.outlineOpacity);
  const size = Math.round(BASE_FONT_SIZE * (clamp(prefs.size, 25, 400) / 100));
  const marginV = Math.round(BASE_MARGIN_V + ((100 - clamp(prefs.position, 0, 100)) / 100) * (PLAY_RES_Y * 0.75));
  return [
    `Style: ${name}`,
    fontName(prefs.font),
    String(size),
    primary,
    primary,
    border,
    back,
    prefs.bold ? '-1' : '0',
    '0',
    '0',
    '0',
    '100',
    '100',
    '0',
    '0',
    boxed ? '3' : '1',
    String(outline),
    String(shadowDepth),
    '2',
    '60',
    '60',
    String(marginV),
    '1',
  ].join(',');
}

function assHeader(prefs: SubtitleStylePrefs): string {
  return [
    '[Script Info]',
    'ScriptType: v4.00+',
    'WrapStyle: 0',
    'ScaledBorderAndShadow: yes',
    `PlayResX: ${PLAY_RES_X}`,
    `PlayResY: ${PLAY_RES_Y}`,
    '',
    '[V4+ Styles]',
    'Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding',
    buildStyleLine(prefs),
    '',
    '[Events]',
    'Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text',
  ].join('\n');
}

export function detectSubtitleFormat(text: string): SubtitleFormat {
  if (/^\s*(﻿)?WEBVTT/i.test(text)) return 'vtt';
  if (/^\s*(﻿)?\[Script Info\]/i.test(text) || /^\s*\[V4\+? Styles\]/im.test(text) || /^\s*Dialogue:/im.test(text)) return 'ass';
  return 'srt';
}

function assTime(totalSeconds: number): string {
  const safe = Math.max(0, totalSeconds);
  const hours = Math.floor(safe / 3600);
  const minutes = Math.floor((safe % 3600) / 60);
  const seconds = Math.floor(safe % 60);
  const centis = Math.round((safe - Math.floor(safe)) * 100);
  const pad = (value: number) => String(value).padStart(2, '0');
  return `${hours}:${pad(minutes)}:${pad(centis === 100 ? seconds + 1 : seconds)}.${pad(centis === 100 ? 0 : centis)}`;
}

function parseTimestamp(raw: string): number | null {
  const match = /^(?:(\d+):)?(\d{1,2}):(\d{1,2})[.,](\d{1,3})$/.exec(raw.trim());
  if (!match) return null;
  const [, hours, minutes, seconds, fraction] = match;
  return Number(hours ?? 0) * 3600 + Number(minutes) * 60 + Number(seconds) + Number(fraction.padEnd(3, '0')) / 1000;
}

function inlineToAss(text: string): string {
  return text
    .replace(/\\/g, '')
    .replace(/[{}]/g, '')
    .replace(/<\s*br\s*\/?\s*>/gi, '\\N')
    .replace(/<\s*i\s*>/gi, '{\\i1}')
    .replace(/<\s*\/\s*i\s*>/gi, '{\\i0}')
    .replace(/<\s*b\s*>/gi, '{\\b1}')
    .replace(/<\s*\/\s*b\s*>/gi, '{\\b0}')
    .replace(/<\s*u\s*>/gi, '{\\u1}')
    .replace(/<\s*\/\s*u\s*>/gi, '{\\u0}')
    .replace(/<font[^>]*color\s*=\s*["']?#?([0-9a-f]{6})["']?[^>]*>/gi, (_match, hex: string) => `{\\c${assColor(hex, 1)}&}`)
    .replace(/<\s*\/\s*font\s*>/gi, '{\\c}')
    .replace(/<[^>]+>/g, '')
    .replace(/\r/g, '')
    .split('\n')
    .map((line) => line.trim())
    .filter(Boolean)
    .join('\\N');
}

type Cue = { start: number; end: number; text: string };

function cuesToEvents(cues: Cue[]): string[] {
  return cues.map((cue) => `Dialogue: 0,${assTime(cue.start)},${assTime(cue.end)},Default,,0,0,0,,${cue.text}`);
}

function parseTimedBlocks(text: string): Cue[] {
  const normalized = text.replace(/\r\n?/g, '\n').replace(/^﻿/, '');
  const cues: Cue[] = [];
  for (const block of normalized.split(/\n{2,}/)) {
    const lines = block.split('\n').filter((line) => line.trim() !== '');
    if (lines.length === 0) continue;
    const arrowIndex = lines.findIndex((line) => line.includes('-->'));
    if (arrowIndex < 0) continue;
    const [rawStart, rawRest] = lines[arrowIndex].split('-->');
    if (rawStart === undefined || rawRest === undefined) continue;
    const start = parseTimestamp(rawStart);
    const end = parseTimestamp(rawRest.trim().split(/\s+/)[0] ?? '');
    if (start === null || end === null) continue;
    const body = inlineToAss(lines.slice(arrowIndex + 1).join('\n'));
    if (!body) continue;
    cues.push({ start, end: Math.max(end, start), text: body });
  }
  return cues;
}

function overrideAssStyles(source: string, prefs: SubtitleStylePrefs): string {
  const normalized = source.replace(/\r\n?/g, '\n');
  const lines = normalized.split('\n');
  const styleNames: string[] = [];
  for (const line of lines) {
    const match = /^Style:\s*([^,]+),/.exec(line);
    if (match) styleNames.push(match[1].trim());
  }
  if (styleNames.length === 0) return normalized;
  const replaced = new Set<string>();
  const out = lines.map((line) => {
    const match = /^Style:\s*([^,]+),/.exec(line);
    if (!match) return line;
    const name = match[1].trim();
    if (replaced.has(name)) return line;
    replaced.add(name);
    return buildStyleLine(prefs, name);
  });
  return out.join('\n');
}

export function toAssDocument(text: string, prefs: SubtitleStylePrefs): string | null {
  const format = detectSubtitleFormat(text);
  if (format === 'ass') {
    return prefs.forceStyle ? overrideAssStyles(text, prefs) : text.replace(/\r\n?/g, '\n');
  }
  const cues = parseTimedBlocks(text);
  if (cues.length === 0) return null;
  return `${assHeader(prefs)}\n${cuesToEvents(cues).join('\n')}\n`;
}
