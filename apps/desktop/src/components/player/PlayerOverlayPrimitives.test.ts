import { describe, expect, it } from 'vitest';
import { parseChapters, skipToastText } from './PlayerOverlayPrimitives';

describe('parseChapters', () => {
  it('reads the MKV chapter scan response shape', () => {
    expect(
      parseChapters(
        JSON.stringify({
          chapters: [
            { title: 'Prologue', startMs: 0 },
            { title: 'Opening', startMs: 6050 },
          ],
          seekOffset: null,
        }),
      ),
    ).toEqual([
      { title: 'Prologue', startMs: 0 },
      { title: 'Opening', startMs: 6050 },
    ]);
  });

  it('keeps accepting the legacy array shape', () => {
    expect(parseChapters('[{"title":"Episode","startTime":97883}]')).toEqual([{ title: 'Episode', startMs: 97883 }]);
  });

  it('includes the skip provider in the player feedback text', () => {
    expect(skipToastText('intro', 110000, 'Chapters')).toBe('Intro skipped to 1:50 via Chapters');
  });
});
