import React from 'react';
import * as Sentry from '@sentry/react';
import { HomeScreen as HomeScreenBase } from './screens/HomeScreen';
import { SearchScreen as SearchScreenBase } from './screens/SearchScreen';
import { LibraryScreen as LibraryScreenBase } from './screens/LibraryScreen';

export const HomeScreen = Sentry.withProfiler(HomeScreenBase, { name: 'HomeScreen' });
export const SearchScreen = Sentry.withProfiler(SearchScreenBase, { name: 'SearchScreen' });
export const LibraryScreen = Sentry.withProfiler(LibraryScreenBase, { name: 'LibraryScreen' });

export const DetailScreen = React.lazy(() => import('./screens/DetailScreen').then((m) => ({ default: Sentry.withProfiler(m.DetailScreen, { name: 'DetailScreen' }) })));
export const SettingsScreen = React.lazy(() => import('./screens/SettingsScreen').then((m) => ({ default: Sentry.withProfiler(m.SettingsScreen, { name: 'SettingsScreen' }) })));
export const ReactPlayerOverlay = React.lazy(() => import('./components/ReactPlayerOverlay').then((m) => ({ default: m.ReactPlayerOverlay })));

export const DiscoverScreen = React.lazy(() => import('./screens/DiscoverScreen').then((m) => ({ default: Sentry.withProfiler(m.DiscoverScreen, { name: 'DiscoverScreen' }) })));
export const CalendarScreen = React.lazy(() => import('./screens/CalendarScreen').then((m) => ({ default: Sentry.withProfiler(m.CalendarScreen, { name: 'CalendarScreen' }) })));
export const ProfileSelectionScreen = React.lazy(() => import('./screens/ProfileSelectionScreen').then((m) => ({ default: m.ProfileSelectionScreen })));
export const WelcomeScreen = React.lazy(() => import('./screens/WelcomeScreen').then((m) => ({ default: m.WelcomeScreen })));
