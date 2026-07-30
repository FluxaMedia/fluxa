import React from 'react';
import * as Sentry from '@sentry/react';
import { HomeScreen as HomeScreenBase } from './screens/HomeScreen';
import { SearchScreen as SearchScreenBase } from './screens/SearchScreen';
import { DetailScreen as DetailScreenBase } from './screens/DetailScreen';
import { LibraryScreen as LibraryScreenBase } from './screens/LibraryScreen';
import { SettingsScreen as SettingsScreenBase } from './screens/SettingsScreen';

export const HomeScreen = Sentry.withProfiler(HomeScreenBase, { name: 'HomeScreen' });
export const SearchScreen = Sentry.withProfiler(SearchScreenBase, { name: 'SearchScreen' });
export const DetailScreen = Sentry.withProfiler(DetailScreenBase, { name: 'DetailScreen' });
export const LibraryScreen = Sentry.withProfiler(LibraryScreenBase, { name: 'LibraryScreen' });
export const SettingsScreen = Sentry.withProfiler(SettingsScreenBase, { name: 'SettingsScreen' });

export const DiscoverScreen = React.lazy(() => import('./screens/DiscoverScreen').then((m) => ({ default: Sentry.withProfiler(m.DiscoverScreen, { name: 'DiscoverScreen' }) })));
export const CalendarScreen = React.lazy(() => import('./screens/CalendarScreen').then((m) => ({ default: Sentry.withProfiler(m.CalendarScreen, { name: 'CalendarScreen' }) })));
export const ProfileSelectionScreen = React.lazy(() => import('./screens/ProfileSelectionScreen').then((m) => ({ default: m.ProfileSelectionScreen })));
export const WelcomeScreen = React.lazy(() => import('./screens/WelcomeScreen').then((m) => ({ default: m.WelcomeScreen })));
