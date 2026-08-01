import React, { type ComponentProps } from 'react';
import { HomeScreen, DetailScreen, CalendarScreen, DiscoverScreen, LibraryScreen, SearchScreen, SettingsScreen } from '../appScreens';
import { GlobalSearchBar } from './GlobalSearchBar';
import { useAppStateSelector, type AppStateStore } from '../core/appStateStore';

function shallowEqual<T extends Record<string, unknown>>(left: T, right: T): boolean {
  const keys = Object.keys(left);
  return keys.length === Object.keys(right).length && keys.every((key) => left[key] === right[key]);
}

type StoreProp = { store: AppStateStore };

export function HomeRoute({ store, ...props }: StoreProp & Omit<ComponentProps<typeof HomeScreen>, 'state'>) {
  const state = useAppStateSelector(store, (value) => ({ home: value.home, settings: value.settings, addons: value.addons }), shallowEqual);
  return <HomeScreen state={state} {...props} />;
}

export function DetailRoute({ store, ...props }: StoreProp & Omit<ComponentProps<typeof DetailScreen>, 'state'>) {
  const state = useAppStateSelector(store, (value) => ({ detail: value.detail, home: value.home, library: value.library, settings: value.settings }), shallowEqual);
  return <DetailScreen state={state} {...props} />;
}

export function CalendarRoute({ store, ...props }: StoreProp & Omit<ComponentProps<typeof CalendarScreen>, 'state'>) {
  const state = useAppStateSelector(store, (value) => ({ calendar: value.calendar, library: value.library }), shallowEqual);
  return <CalendarScreen state={state} {...props} />;
}

export function DiscoverRoute({ store, ...props }: StoreProp & Omit<ComponentProps<typeof DiscoverScreen>, 'state'>) {
  const state = useAppStateSelector(store, (value) => ({ discover: value.discover, settings: value.settings, addons: value.addons }), shallowEqual);
  return <DiscoverScreen state={state} {...props} />;
}

export function LibraryRoute({ store, ...props }: StoreProp & Omit<ComponentProps<typeof LibraryScreen>, 'state'>) {
  const state = useAppStateSelector(store, (value) => ({ library: value.library, settings: value.settings, home: value.home }), shallowEqual);
  return <LibraryScreen state={state} {...props} />;
}

export function SearchRoute({ store, ...props }: StoreProp & Omit<ComponentProps<typeof SearchScreen>, 'state'>) {
  const state = useAppStateSelector(store, (value) => ({ search: value.search, settings: value.settings }), shallowEqual);
  return <SearchScreen state={state} {...props} />;
}

export function SettingsRoute({ store, ...props }: StoreProp & Omit<ComponentProps<typeof SettingsScreen>, 'state'>) {
  const state = useAppStateSelector(store, (value) => ({ addons: value.addons, plugins: value.plugins }), shallowEqual);
  return <SettingsScreen state={state} {...props} />;
}

export function GlobalSearchRoute({ store, ...props }: StoreProp & Omit<ComponentProps<typeof GlobalSearchBar>, 'state'>) {
  const state = useAppStateSelector(store, (value) => ({ home: value.home, search: value.search, settings: value.settings }), shallowEqual);
  return <GlobalSearchBar state={state} {...props} />;
}
