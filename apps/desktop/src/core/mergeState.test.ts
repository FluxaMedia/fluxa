import { describe, expect, it } from 'vitest';
import { mergeAppState } from './mergeState';
import type { AppState, LibraryItem } from './types';

function appState(continueWatching: LibraryItem[]): AppState {
  return {
    home: { continueWatching },
  } as AppState;
}

describe('mergeAppState', () => {
  it('does not keep an old episode label when the resume video changes', () => {
    const previous = appState([
      {
        id: 'series-1',
        name: 'Example Series',
        type: 'series',
        lastVideoId: 'series-1:1:2',
        lastEpisodeName: 'Episode Two',
      },
    ]);
    const updated = {
      home: {
        continueWatching: [
          {
            id: 'series-1',
            name: 'Example Series',
            type: 'series',
            lastVideoId: 'series-1:1:4',
          },
        ],
      },
    };

    const result = mergeAppState(previous, updated);

    expect(result.home.continueWatching?.[0]).toMatchObject({
      lastVideoId: 'series-1:1:4',
    });
    expect(result.home.continueWatching?.[0]?.lastEpisodeName).toBeUndefined();
  });
});
