import React, { useState } from 'react';
import { BookMarked, Calendar, ChevronLeft, ChevronRight, Compass, Settings, Home } from 'lucide-react';
import { t } from '../i18n';
import { toggleWindowFullscreen } from '../core/windowGeometry';

function onChromeDoubleClick(e: React.MouseEvent) {
  if (e.target === e.currentTarget) void toggleWindowFullscreen();
}

export type NavRoute = 'home' | 'search' | 'library' | 'discover' | 'calendar' | 'settings';
export type NavBarPosition = 'left' | 'right' | 'top' | 'bottom';
export type NavItemsAlign = 'start' | 'center' | 'end';

const ICONS: Partial<Record<NavRoute, React.ElementType>> = {
  home: Home,
  library: BookMarked,
  discover: Compass,
  calendar: Calendar,
  settings: Settings,
};

function NavIcon({ route, active }: { route: NavRoute; active: boolean }) {
  const Icon = ICONS[route];
  if (!Icon) return null;
  return <Icon size={22} strokeWidth={active ? 2.5 : 1.75} />;
}

const LABEL_KEYS: Record<NavRoute, string> = {
  home: 'nav.home',
  search: 'auto.search',
  library: 'nav.library',
  discover: 'nav.discover',
  calendar: 'nav.calendar',
  settings: 'nav.settings',
};

const ROUTES: NavRoute[] = ['home', 'library', 'discover', 'calendar', 'settings'];

interface SidebarProps {
  activeRoute: NavRoute;
  onNavigate: (route: NavRoute) => void;
  position?: string;
  itemsAlign?: string;
  topOffset?: number;
  alwaysOpen?: boolean;
}

interface TopBarProps {
  activeRoute: NavRoute;
  onNavigate: (route: NavRoute) => void;
  transparent?: boolean;
  position?: string;
  itemsAlign?: string;
  topOffset?: number;
}

const PINNED_KEY = 'fluxa-sidebar-pinned';

function normalizePosition(value?: string, fallback: NavBarPosition = 'left'): NavBarPosition {
  return value === 'right' || value === 'top' || value === 'bottom' || value === 'left' ? value : fallback;
}

function normalizeAlign(value?: string): NavItemsAlign {
  return value === 'start' || value === 'end' || value === 'center' ? value : 'center';
}

function edgeContainerStyle(position: NavBarPosition): React.CSSProperties {
  const offset = 18;
  if (position === 'top') {
    return { top: offset, left: '50%', transform: 'translateX(-50%)', paddingBottom: '1.5rem' };
  }
  if (position === 'bottom') {
    return { bottom: offset, left: '50%', transform: 'translateX(-50%)', paddingTop: '1.5rem' };
  }
  if (position === 'right') {
    return { top: '50%', right: 0, transform: 'translateY(-50%)', paddingLeft: '1.5rem' };
  }
  return { top: '50%', left: 0, transform: 'translateY(-50%)', paddingRight: '1.5rem' };
}

function panelJustify(position: NavBarPosition, align: NavItemsAlign): React.CSSProperties['justifyContent'] {
  if (align === 'center') return 'center';
  if (position === 'right' || position === 'bottom') return align === 'start' ? 'flex-end' : 'flex-start';
  return align === 'start' ? 'flex-start' : 'flex-end';
}

function sidebarPanelRadius(position: NavBarPosition): string {
  if (position === 'right') return '1rem 0 0 1rem';
  if (position === 'top') return '0 0 1rem 1rem';
  if (position === 'bottom') return '1rem 1rem 0 0';
  return '0 1rem 1rem 0';
}

function sidebarBorder(position: NavBarPosition): React.CSSProperties {
  if (position === 'right') return { borderRightWidth: 0 };
  if (position === 'top') return { borderTopWidth: 0 };
  if (position === 'bottom') return { borderBottomWidth: 0 };
  return { borderLeftWidth: 0 };
}

function sidebarClosedTransform(position: NavBarPosition, isOpen: boolean): string {
  if (isOpen) return 'translate(0, 0)';
  if (position === 'right') return 'translateX(4.875rem)';
  if (position === 'top') return 'translateY(-4.875rem)';
  if (position === 'bottom') return 'translateY(4.875rem)';
  return 'translateX(-4.875rem)';
}

export const NavSidebar = React.memo(function NavSidebar({
  activeRoute,
  onNavigate,
  position: positionValue = 'left',
  itemsAlign: alignValue = 'center',
  topOffset = 0,
  alwaysOpen = false,
}: SidebarProps) {
  const [pinned, setPinned] = useState<boolean>(() => {
    try {
      return localStorage.getItem(PINNED_KEY) === 'true';
    } catch {
      return false;
    }
  });
  const [hovered, setHovered] = useState(false);

  const isOpen = alwaysOpen || pinned || hovered;
  const position = normalizePosition(positionValue, 'left');
  const itemsAlign = normalizeAlign(alignValue);
  const isHorizontal = position === 'top' || position === 'bottom';

  const togglePin = (e: React.MouseEvent) => {
    e.stopPropagation();
    const next = !pinned;
    setPinned(next);
    try {
      localStorage.setItem(PINNED_KEY, String(next));
    } catch {}
  };

  return (
    <div
      style={{
        position: 'fixed',
        zIndex: 100,
        ...edgeContainerStyle(position),
        ...(topOffset && position === 'top' ? { top: 18 + topOffset } : {}),
      }}
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
    >
      {/* Sliding panel */}
      <div
        style={{
          transform: sidebarClosedTransform(position, isOpen),
          transition: 'transform 0.25s ease',
          display: 'flex',
          flexDirection: isHorizontal ? 'row' : 'column',
          justifyContent: panelJustify(position, itemsAlign),
          gap: '0.25rem',
          padding: '0.5rem 0.625rem 0.75rem',
          background: '#141414',
          borderRadius: sidebarPanelRadius(position),
          border: '1px solid rgba(255,255,255,0.08)',
          ...sidebarBorder(position),
          boxShadow: '0 0.5rem 2rem rgba(0,0,0,0.35)',
          width: isHorizontal ? 'min(38.75rem, calc(100vw - 3rem))' : 96,
          minHeight: isHorizontal ? 58 : undefined,
          maxHeight: isHorizontal ? undefined : 'calc(100vh - 5rem)',
        }}
        onDoubleClick={onChromeDoubleClick}
      >
        {ROUTES.map((route) => (
          <NavItem key={route} route={route} isActive={route === activeRoute} onNavigate={onNavigate} label={t(LABEL_KEYS[route])} />
        ))}
      </div>

      {!alwaysOpen && (
        <button
          onClick={togglePin}
          title={pinned ? t('auto.hide') : t('auto.show')}
          style={{
            position: 'absolute',
            top: position === 'top' ? (isOpen ? 58 : 0) : position === 'bottom' ? undefined : '50%',
            bottom: position === 'bottom' ? (isOpen ? 58 : 0) : undefined,
            left: position === 'right' ? undefined : position === 'left' ? (isOpen ? 78 : 0) : '50%',
            right: position === 'right' ? (isOpen ? 78 : 0) : undefined,
            transform: position === 'top' || position === 'bottom' ? 'translateX(-50%)' : 'translateY(-50%)',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            width: position === 'top' || position === 'bottom' ? 36 : 20,
            height: position === 'top' || position === 'bottom' ? 20 : 36,
            borderRadius:
              position === 'right'
                ? '0.5rem 0 0 0.5rem'
                : position === 'top'
                  ? '0 0 0.5rem 0.5rem'
                  : position === 'bottom'
                    ? '0.5rem 0.5rem 0 0'
                    : '0 0.5rem 0.5rem 0',
            border: '1px solid rgba(255,255,255,0.08)',
            ...sidebarBorder(position),
            background: '#141414',
            color: 'rgba(255,255,255,0.35)',
            cursor: 'pointer',
            transition: 'left 0.25s ease, right 0.25s ease, top 0.25s ease, bottom 0.25s ease, color 0.15s',
            padding: 0,
            boxShadow: '0.125rem 0 0.5rem rgba(0,0,0,0.3)',
          }}
          onMouseEnter={(e) => (e.currentTarget.style.color = '#FFFFFF')}
          onMouseLeave={(e) => (e.currentTarget.style.color = 'rgba(255,255,255,0.35)')}
        >
          {/* → = pinned/always-open  |  ← = hover-only */}
          {pinned ? (
            position === 'right' ? (
              <ChevronLeft size={13} />
            ) : (
              <ChevronRight size={13} />
            )
          ) : position === 'right' ? (
            <ChevronRight size={13} />
          ) : (
            <ChevronLeft size={13} />
          )}
        </button>
      )}
    </div>
  );
});

export const TopBar = React.memo(function TopBar({
  activeRoute,
  onNavigate,
  transparent = false,
  position: positionValue = 'top',
  itemsAlign: alignValue = 'center',
  topOffset = 0,
}: TopBarProps) {
  const TOP_ROUTES: NavRoute[] = ['home', 'library', 'discover', 'calendar'];
  const position = normalizePosition(positionValue, 'top');
  const itemsAlign = normalizeAlign(alignValue);
  const isVertical = position === 'left' || position === 'right';

  return (
    <div
      style={{
        position: 'fixed',
        top: position === 'bottom' ? undefined : position === 'top' ? 18 + topOffset : '50%',
        bottom: position === 'bottom' ? 18 : undefined,
        left:
          position === 'right'
            ? undefined
            : position === 'left'
              ? 18
              : itemsAlign === 'start'
                ? 24
                : itemsAlign === 'end'
                  ? undefined
                  : '50%',
        right:
          position === 'right' ? 18 : position === 'bottom' || position === 'top' ? (itemsAlign === 'end' ? 24 : undefined) : undefined,
        transform: isVertical ? 'translateY(-50%)' : itemsAlign === 'center' ? 'translateX(-50%)' : undefined,
        zIndex: 100,
        display: 'flex',
        flexDirection: isVertical ? 'column' : 'row',
        alignItems: 'center',
        justifyContent: panelJustify(position, itemsAlign),
        gap: '0.125rem',
        background: transparent ? 'transparent' : 'rgba(10, 12, 20, 0.96)',
        border: transparent ? 'none' : '1px solid rgba(255,255,255,0.10)',
        borderRadius: transparent ? 0 : 999,
        padding: transparent ? '0.875rem 0.375rem 0.5rem' : '0.3125rem 0.375rem',
        boxShadow: transparent ? 'none' : '0 0.5rem 2rem rgba(0,0,0,0.35)',
        maxHeight: isVertical ? 'calc(100vh - 3rem)' : undefined,
      }}
      onDoubleClick={onChromeDoubleClick}
    >
      {TOP_ROUTES.map((route) => (
        <TopBarItem key={route} route={route} isActive={route === activeRoute} onNavigate={onNavigate} label={t(LABEL_KEYS[route])} />
      ))}
    </div>
  );
});

function NavItem({
  route,
  isActive,
  onNavigate,
  label,
}: {
  route: NavRoute;
  isActive: boolean;
  onNavigate: (r: NavRoute) => void;
  label: string;
}) {
  const [hovered, setHovered] = useState(false);

  return (
    <button
      style={{
        position: 'relative',
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'center',
        justifyContent: 'center',
        gap: '0.3125rem',
        width: '4.75rem',
        padding: '0.75rem 0.375rem',
        borderRadius: '0.625rem',
        border: 'none',
        background: isActive ? 'rgba(255,255,255,0.12)' : hovered ? 'rgba(255,255,255,0.07)' : 'transparent',
        color: isActive ? '#FFFFFF' : hovered ? 'rgba(255,255,255,0.85)' : 'rgba(255,255,255,0.40)',
        cursor: 'pointer',
        outline: 'none',
        transition: 'background 0.15s, color 0.15s',
      }}
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
      onClick={() => onNavigate(route)}
      title={label}
    >
      {isActive && (
        <span
          style={{
            position: 'absolute',
            right: 0,
            top: '50%',
            transform: 'translateY(-50%)',
            width: '0.1875rem',
            height: '1.5rem',
            borderRadius: '62.4375rem 0 0 62.4375rem',
            background: '#FFFFFF',
          }}
        />
      )}
      <NavIcon route={route} active={isActive} />
      <span
        style={{
          fontSize: '0.6875rem',
          fontWeight: isActive ? 600 : 400,
          opacity: isActive ? 1 : 0.65,
          lineHeight: 1,
        }}
      >
        {label}
      </span>
    </button>
  );
}

function TopBarItem({
  route,
  isActive,
  onNavigate,
  label,
}: {
  route: NavRoute;
  isActive: boolean;
  onNavigate: (r: NavRoute) => void;
  label: string;
}) {
  const [hovered, setHovered] = useState(false);

  return (
    <button
      onClick={() => onNavigate(route)}
      title={label}
      style={{
        display: 'flex',
        alignItems: 'center',
        gap: '0.375rem',
        height: '2.25rem',
        padding: '0 0.75rem',
        borderRadius: '62.4375rem',
        border: 'none',
        background: hovered ? 'rgba(255,255,255,0.10)' : 'transparent',
        color: isActive ? '#FFFFFF' : hovered ? 'rgba(255,255,255,0.90)' : 'rgba(255,255,255,0.65)',
        textShadow: '0 1px 0.25rem rgba(0,0,0,0.8)',
        cursor: 'pointer',
        outline: 'none',
        transition: 'background 0.15s, color 0.15s',
        fontSize: '0.8125rem',
        fontWeight: isActive ? 700 : 600,
        whiteSpace: 'nowrap',
        flexShrink: 0,
      }}
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
    >
      <NavIcon route={route} active={isActive} />
      {label}
    </button>
  );
}
