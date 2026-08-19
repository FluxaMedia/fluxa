import { describe, expect, it } from 'vitest';
import { assColor, defaultSubtitleStylePrefs, detectSubtitleFormat, toAssDocument, type SubtitleStylePrefs } from './webSubtitles';

const SRT = `1
00:00:01,000 --> 00:00:03,500
Hello <i>there</i>
second line

2
00:00:04,000 --> 00:00:06,000
Bye`;

const VTT = `WEBVTT

intro
00:00:01.000 --> 00:00:03.500 line:90%
Hello there`;

const ASS = `[Script Info]
ScriptType: v4.00+

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Trebuchet,40,&H00FF0000,&H00FF0000,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:01.00,0:00:03.50,Default,,0,0,0,,{\\pos(960,540)}Positioned`;

const prefs = (overrides: Partial<SubtitleStylePrefs> = {}): SubtitleStylePrefs => ({
  ...defaultSubtitleStylePrefs,
  ...overrides,
});

describe('detectSubtitleFormat', () => {
  it('recognises each container the addons hand us', () => {
    expect(detectSubtitleFormat(SRT)).toBe('srt');
    expect(detectSubtitleFormat(VTT)).toBe('vtt');
    expect(detectSubtitleFormat(ASS)).toBe('ass');
  });

  it('treats a bare Dialogue dump as ass', () => {
    expect(detectSubtitleFormat('Dialogue: 0,0:00:01.00,0:00:02.00,Default,,0,0,0,,hi')).toBe('ass');
  });
});

describe('assColor', () => {
  it('emits BGR with inverted alpha', () => {
    expect(assColor('#FFFFFF', 1)).toBe('&H00FFFFFF');
    expect(assColor('#FF0000', 1)).toBe('&H000000FF');
    expect(assColor('#000000', 0.5)).toBe('&H80000000');
  });
});

describe('toAssDocument', () => {
  it('keeps both srt lines as one cue with an ass break', () => {
    const out = toAssDocument(SRT, prefs());
    expect(out).toContain('Dialogue: 0,0:00:01.00,0:00:03.50,Default,,0,0,0,,Hello {\\i1}there{\\i0}\\Nsecond line');
    expect(out).toContain('Dialogue: 0,0:00:04.00,0:00:06.00,Default');
  });

  it('drops vtt cue identifiers and cue settings', () => {
    const out = toAssDocument(VTT, prefs()) ?? '';
    expect(out).toContain('Dialogue: 0,0:00:01.00,0:00:03.50,Default,,0,0,0,,Hello there');
    expect(out).not.toContain('intro');
    expect(out).not.toContain('line:90%');
  });

  it('leaves ass untouched when force style is off', () => {
    expect(toAssDocument(ASS, prefs({ forceStyle: false }))).toContain('Style: Default,Trebuchet,40,&H00FF0000');
  });

  it('rewrites ass styles when force style is on but keeps the events', () => {
    const out = toAssDocument(ASS, prefs({ forceStyle: true, color: '#00FF00' })) ?? '';
    expect(out).not.toContain('Trebuchet');
    expect(out).toContain('Style: Default,Liberation Sans,');
    expect(out).toContain('&H0000FF00');
    expect(out).toContain('{\\pos(960,540)}Positioned');
  });

  it('returns null when nothing timed could be parsed', () => {
    expect(toAssDocument('not a subtitle file at all', prefs())).toBeNull();
  });

  it('carries the size and position prefs into the style line', () => {
    const out = toAssDocument(SRT, prefs({ size: 200, position: 100 })) ?? '';
    const style = out.split('\n').find((line) => line.startsWith('Style: Default')) ?? '';
    const fields = style.split(',');
    expect(fields[2]).toBe('108');
    expect(fields[21]).toBe('40');

    const raised = toAssDocument(SRT, prefs({ position: 50 })) ?? '';
    const raisedStyle = raised.split('\n').find((line) => line.startsWith('Style: Default')) ?? '';
    expect(Number(raisedStyle.split(',')[21])).toBeGreaterThan(40);
  });

  it('switches to an opaque box only when a background is asked for', () => {
    const plain = toAssDocument(SRT, prefs()) ?? '';
    const boxed = toAssDocument(SRT, prefs({ backgroundOpacity: 0.5 })) ?? '';
    const borderStyle = (doc: string) => (doc.split('\n').find((line) => line.startsWith('Style: Default')) ?? '').split(',')[15];
    expect(borderStyle(plain)).toBe('1');
    expect(borderStyle(boxed)).toBe('3');
  });

  it('strips stray braces so file text cannot inject override tags', () => {
    const hostile = '1\n00:00:01,000 --> 00:00:02,000\n{\\an8}not an override';
    const out = toAssDocument(hostile, prefs()) ?? '';
    expect(out).toContain('an8not an override');
  });
});
