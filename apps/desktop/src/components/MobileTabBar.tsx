import React from 'react';
import { BookMarked, Calendar, Compass, Home } from 'lucide-react';
import { t } from '../i18n';
import { hapticTap } from '../platform/haptics';
import type { NavRoute } from './NavSidebar';

const TABS: { route: NavRoute; icon: React.ElementType; labelKey: string }[] = [
  { route: 'home', icon: Home, labelKey: 'nav.home' },
  { route: 'discover', icon: Compass, labelKey: 'nav.discover' },
  { route: 'calendar', icon: Calendar, labelKey: 'nav.calendar' },
  { route: 'library', icon: BookMarked, labelKey: 'nav.library' },
];

export const MobileTabBar = React.memo(function MobileTabBar({
  activeRoute,
  onNavigate,
  routes,
}: {
  activeRoute: NavRoute;
  onNavigate: (route: NavRoute) => void;
  routes?: NavRoute[];
}) {
  return (
    <nav className="mobile-tabbar">
      {(routes ? TABS.filter(({ route }) => routes.includes(route)) : TABS).map(({ route, icon: Icon, labelKey }) => {
        const active = route === activeRoute;
        return (
          <button
            key={route}
            className={`mobile-tabbar-item${active ? ' is-active' : ''}`}
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
