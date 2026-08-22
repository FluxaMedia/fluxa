import { useCallback, useRef, useSyncExternalStore } from 'react';
import { DEFAULT_STATE } from '../appConstants';
import { mergeAppState } from './mergeState';
import type { AppState } from './types';

type Listener = () => void;
type Equality<T> = (left: T, right: T) => boolean;

export class AppStateStore {
  private state: AppState;
  private listeners = new Set<Listener>();

  constructor(initialState: AppState = DEFAULT_STATE) {
    this.state = initialState;
  }

  getState = (): AppState => this.state;

  replace = (state: AppState) => {
    if (state === this.state) return;
    this.state = state;
    this.listeners.forEach((listener) => listener());
  };

  update = (patch: Partial<AppState>) => {
    this.replace(mergeAppState(this.state, patch));
  };

  subscribe = (listener: Listener) => {
    this.listeners.add(listener);
    return () => this.listeners.delete(listener);
  };
}

export function useAppStateSelector<T>(store: AppStateStore, selector: (state: AppState) => T, isEqual: Equality<T> = Object.is): T {
  const selectorRef = useRef(selector);
  const equalityRef = useRef(isEqual);
  const selectedRef = useRef(selector(store.getState()));
  selectorRef.current = selector;
  equalityRef.current = isEqual;

  const subscribe = useCallback(
    (notify: Listener) =>
      store.subscribe(() => {
        const next = selectorRef.current(store.getState());
        if (equalityRef.current(selectedRef.current, next)) return;
        selectedRef.current = next;
        notify();
      }),
    [store],
  );

  const getSnapshot = useCallback(() => selectedRef.current, []);
  return useSyncExternalStore(subscribe, getSnapshot, getSnapshot);
}

export function appStateSliceEqual(...keys: Array<keyof AppState>): Equality<AppState> {
  return (left, right) => keys.every((key) => left[key] === right[key]);
}
