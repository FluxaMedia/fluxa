import React from 'react';
import { getSentryModule } from './core/sentryRuntime';
import { HomeScreen as HomeScreenBase } from './screens/HomeScreen';

function withOptionalProfiler<P extends object>(Base: React.ComponentType<P>, name: string): React.ComponentType<P> {
  return function ProfiledScreen(props: P) {
    const profiledRef = React.useRef<React.ComponentType<P> | null>(null);
    const sentry = getSentryModule();
    if (sentry && !profiledRef.current) {
      profiledRef.current = sentry.withProfiler(Base, { name });
    }
    const Comp = profiledRef.current ?? Base;
    return <Comp {...props} />;
  };
}

export const HomeScreen = withOptionalProfiler(HomeScreenBase, 'HomeScreen');

export const SearchScreen = React.lazy(() => import('./screens/SearchScreen').then((m) => ({ default: withOptionalProfiler(m.SearchScreen, 'SearchScreen') })));
export const LibraryScreen = React.lazy(() => import('./screens/LibraryScreen').then((m) => ({ default: withOptionalProfiler(m.LibraryScreen, 'LibraryScreen') })));

export const DetailScreen = React.lazy(() => import('./screens/DetailScreen').then((m) => ({ default: withOptionalProfiler(m.DetailScreen, 'DetailScreen') })));
export const SettingsScreen = React.lazy(() => import('./screens/SettingsScreen').then((m) => ({ default: withOptionalProfiler(m.SettingsScreen, 'SettingsScreen') })));
export const ReactPlayerOverlay = React.lazy(() => import('./components/ReactPlayerOverlay').then((m) => ({ default: m.ReactPlayerOverlay })));

export const DiscoverScreen = React.lazy(() => import('./screens/DiscoverScreen').then((m) => ({ default: withOptionalProfiler(m.DiscoverScreen, 'DiscoverScreen') })));
export const CalendarScreen = React.lazy(() => import('./screens/CalendarScreen').then((m) => ({ default: withOptionalProfiler(m.CalendarScreen, 'CalendarScreen') })));
export const ProfileSelectionScreen = React.lazy(() => import('./screens/ProfileSelectionScreen').then((m) => ({ default: m.ProfileSelectionScreen })));
export const WelcomeScreen = React.lazy(() => import('./screens/WelcomeScreen').then((m) => ({ default: m.WelcomeScreen })));
