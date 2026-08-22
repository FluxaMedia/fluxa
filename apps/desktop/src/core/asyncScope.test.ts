import { describe, expect, it } from 'vitest';
import { AsyncScope } from './asyncScope';

describe('AsyncScope', () => {
  it('rejects a completion captured before a profile change', () => {
    const scope = new AsyncScope();
    const profileA = scope.capture();
    scope.invalidate();
    const profileB = scope.capture();

    expect(scope.isCurrent(profileA)).toBe(false);
    expect(scope.isCurrent(profileB)).toBe(true);
  });

  it('rejects playback A after playback B supersedes it', () => {
    const scope = new AsyncScope();
    const playbackA = scope.capture();
    scope.invalidate();
    const playbackB = scope.capture();

    expect(scope.isCurrent(playbackA)).toBe(false);
    expect(scope.isCurrent(playbackB)).toBe(true);
  });

  it('rejects stale torrent cleanup after a replacement playback starts', () => {
    const scope = new AsyncScope();
    const closingPlayback = scope.advance();
    const replacementPlayback = scope.advance();

    expect(scope.isCurrent(closingPlayback)).toBe(false);
    expect(scope.isCurrent(replacementPlayback)).toBe(true);
  });
});
