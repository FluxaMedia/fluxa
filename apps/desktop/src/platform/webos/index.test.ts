import { describe, expect, it } from 'vitest';
import { isBitstreamAudioCodec } from './index';

describe('isBitstreamAudioCodec', () => {
  it('recognizes encoded passthrough candidates', () => {
    expect(isBitstreamAudioCodec('audio/eac3')).toBe(true);
    expect(isBitstreamAudioCodec('audio/true-hd')).toBe(true);
    expect(isBitstreamAudioCodec('dts-hd-ma')).toBe(true);
    expect(isBitstreamAudioCodec('audio/vnd.dts')).toBe(true);
    expect(isBitstreamAudioCodec('DTS:X')).toBe(true);
    expect(isBitstreamAudioCodec('dts uhd p2')).toBe(true);
    expect(isBitstreamAudioCodec('dolby-mat')).toBe(true);
  });

  it('does not request bitstream for ordinary PCM-decoded codecs', () => {
    expect(isBitstreamAudioCodec('audio/mp4a-latm')).toBe(false);
    expect(isBitstreamAudioCodec('opus')).toBe(false);
    expect(isBitstreamAudioCodec(null)).toBe(false);
  });
});
