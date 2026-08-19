import React from 'react';
import { BookMarked, Calendar, Compass, Home, Settings } from 'lucide-react';
import { t } from '../i18n';
import { hapticTap } from '../platform/haptics';
import type { NavRoute } from './NavSidebar';

const TABS: { route: NavRoute; icon: React.ElementType; labelKey: string }[] = [
  { route: 'home', icon: Home, labelKey: 'nav.home' },
  { route: 'discover', icon: Compass, labelKey: 'nav.discover' },
  { route: 'library', icon: BookMarked, labelKey: 'nav.library' },
  { route: 'calendar', icon: Calendar, labelKey: 'nav.calendar' },
  { route: 'settings', icon: Settings, labelKey: 'nav.settings' },
];

export const MobileTabBar = React.memo(function MobileTabBar({
  activeRoute,
  onNavigate,
}: {
  activeRoute: NavRoute;
  onNavigate: (route: NavRoute) => void;
}) {
  return (
    <nav className="mobile-tabbar">
      {TABS.map(({ route, icon: Icon, labelKey }) => {
        const active = route === activeRoute;
        return (
          <button
            key={route}
            className="mobile-tabbar-item"
            aria-current={active ? 'page' : undefined}
            onClick={() => {
              hapticTap();
              onNavigate(route);
            }}
            style={{ color: active ? '#FFFFFF' : 'rgba(255,255,255,0.42)' }}
          >
            <Icon size={21} strokeWidth={active ? 2.4 : 1.75} />
            <span style={{ fontWeight: active ? 700 : 500 }}>{t(labelKey)}</span>
          </button>
        );
      })}
    </nav>
  );
});
