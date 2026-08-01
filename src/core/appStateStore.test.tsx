import { act, render } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { DEFAULT_STATE } from '../appConstants';
import { AppStateStore, useAppStateSelector } from './appStateStore';

describe('AppStateStore', () => {
  it('notifies a selector only when its selected slice changes', () => {
    const store = new AppStateStore(DEFAULT_STATE);
    let renders = 0;

    function Probe() {
      useAppStateSelector(store, (state) => state.home);
      renders += 1;
      return null;
    }

    render(<Probe />);
    expect(renders).toBe(1);

    act(() => store.update({ player: { ...DEFAULT_STATE.player } }));
    expect(renders).toBe(1);

    act(() => store.update({ home: { ...DEFAULT_STATE.home, isLoading: true } }));
    expect(renders).toBe(2);
  });
});
