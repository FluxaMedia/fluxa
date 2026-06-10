import React, { useState } from 'react';
import { Books, CalendarBlank, CaretLeft, CaretRight, Compass, GearSix, House } from '@phosphor-icons/react';
import { t } from '../i18n';

export type NavRoute = 'home' | 'search' | 'library' | 'discover' | 'calendar' | 'settings';
export type NavBarPosition = 'left' | 'right' | 'top' | 'bottom';
export type NavItemsAlign = 'start' | 'center' | 'end';

const ICONS: Partial<Record<NavRoute, React.ReactNode>> = {
  home: <House size={20} weight="fill" />,
  library: <Books size={20} weight="fill" />,
  discover: <Compass size={20} />,
  calendar: <CalendarBlank size={20} weight="fill" />,
  settings: <GearSix size={20} weight="fill" />,
};

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
}

interface TopBarProps {
  activeRoute: NavRoute;
  onNavigate: (route: NavRoute) => void;
  transparent?: boolean;
  position?: string;
  itemsAlign?: string;
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
    return { top: offset, left: '50%', transform: 'translateX(-50%)', paddingBottom: 24 };
  }
  if (position === 'bottom') {
    return { bottom: offset, left: '50%', transform: 'translateX(-50%)', paddingTop: 24 };
  }
  if (position === 'right') {
    return { top: '50%', right: 0, transform: 'translateY(-50%)', paddingLeft: 24 };
  }
  return { top: '50%', left: 0, transform: 'translateY(-50%)', paddingRight: 24 };
}

function panelJustify(position: NavBarPosition, align: NavItemsAlign): React.CSSProperties['justifyContent'] {
  if (align === 'center') return 'center';
  if (position === 'right' || position === 'bottom') return align === 'start' ? 'flex-end' : 'flex-start';
  return align === 'start' ? 'flex-start' : 'flex-end';
}

function sidebarPanelRadius(position: NavBarPosition): string {
  if (position === 'right') return '16px 0 0 16px';
  if (position === 'top') return '0 0 16px 16px';
  if (position === 'bottom') return '16px 16px 0 0';
  return '0 16px 16px 0';
}

function sidebarBorder(position: NavBarPosition): React.CSSProperties {
  if (position === 'right') return { borderRightWidth: 0 };
  if (position === 'top') return { borderTopWidth: 0 };
  if (position === 'bottom') return { borderBottomWidth: 0 };
  return { borderLeftWidth: 0 };
}

function sidebarClosedTransform(position: NavBarPosition, isOpen: boolean): string {
  if (isOpen) return 'translate(0, 0)';
  if (position === 'right') return 'translateX(78px)';
  if (position === 'top') return 'translateY(-78px)';
  if (position === 'bottom') return 'translateY(78px)';
  return 'translateX(-78px)';
}

export function NavSidebar({ activeRoute, onNavigate, position: positionValue = 'left', itemsAlign: alignValue = 'center' }: SidebarProps) {
  const [pinned, setPinned] = useState<boolean>(() => {
    try { return localStorage.getItem(PINNED_KEY) !== 'false'; } catch { return true; }
  });
  const [hovered, setHovered] = useState(false);

  const isOpen = pinned || hovered;
  const position = normalizePosition(positionValue, 'left');
  const itemsAlign = normalizeAlign(alignValue);
  const isHorizontal = position === 'top' || position === 'bottom';

  const togglePin = (e: React.MouseEvent) => {
    e.stopPropagation();
    const next = !pinned;
    setPinned(next);
    try { localStorage.setItem(PINNED_KEY, String(next)); } catch {}
  };

  return (
    <div
      style={{ position: 'fixed', zIndex: 100, ...edgeContainerStyle(position) }}
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
          gap: 4,
          padding: '8px 10px 12px',
          background: 'rgba(10, 12, 20, 0.96)',
          borderRadius: sidebarPanelRadius(position),
          border: '1px solid rgba(232, 93, 63, 0.30)',
          ...sidebarBorder(position),
          boxShadow: '0 8px 32px rgba(0,0,0,0.35), 0 0 16px rgba(232, 93, 63, 0.12)',
          width: isHorizontal ? 'min(620px, calc(100vw - 48px))' : 78,
          minHeight: isHorizontal ? 58 : undefined,
          maxHeight: isHorizontal ? undefined : 'calc(100vh - 80px)',
        }}
      >
        {ROUTES.map((route) => (
          <NavItem
            key={route}
            route={route}
            isActive={route === activeRoute}
            onNavigate={onNavigate}
            label={t(LABEL_KEYS[route])}
          />
        ))}
      </div>

      {/* Toggle button — always visible regardless of sidebar state */}
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
          borderRadius: position === 'right' ? '8px 0 0 8px' : position === 'top' ? '0 0 8px 8px' : position === 'bottom' ? '8px 8px 0 0' : '0 8px 8px 0',
          border: '1px solid rgba(232, 93, 63, 0.30)',
          ...sidebarBorder(position),
          background: 'rgba(10, 12, 20, 0.96)',
          color: 'rgba(255,255,255,0.35)',
          cursor: 'pointer',
          transition: 'left 0.25s ease, right 0.25s ease, top 0.25s ease, bottom 0.25s ease, color 0.15s',
          padding: 0,
          boxShadow: '2px 0 8px rgba(0,0,0,0.3)',
        }}
        onMouseEnter={(e) => (e.currentTarget.style.color = '#E85D3F')}
        onMouseLeave={(e) => (e.currentTarget.style.color = 'rgba(255,255,255,0.35)')}
      >
        {/* → = pinned/always-open  |  ← = hover-only */}
        {pinned
          ? (position === 'right' ? <CaretLeft size={13} weight="bold" /> : <CaretRight size={13} weight="bold" />)
          : (position === 'right' ? <CaretRight size={13} weight="bold" /> : <CaretLeft size={13} weight="bold" />)}
      </button>
    </div>
  );
}

export function TopBar({ activeRoute, onNavigate, transparent = false, position: positionValue = 'top', itemsAlign: alignValue = 'center' }: TopBarProps) {
  const TOP_ROUTES: NavRoute[] = ['home', 'library', 'discover', 'calendar'];
  const position = normalizePosition(positionValue, 'top');
  const itemsAlign = normalizeAlign(alignValue);
  const isVertical = position === 'left' || position === 'right';

  return (
    <div
      style={{
        position: 'fixed',
        top: position === 'bottom' ? undefined : position === 'top' ? 18 : '50%',
        bottom: position === 'bottom' ? 18 : undefined,
        left: position === 'right' ? undefined : position === 'left' ? 18 : itemsAlign === 'start' ? 24 : itemsAlign === 'end' ? undefined : '50%',
        right: position === 'right' ? 18 : position === 'bottom' || position === 'top' ? (itemsAlign === 'end' ? 24 : undefined) : undefined,
        transform: isVertical ? 'translateY(-50%)' : itemsAlign === 'center' ? 'translateX(-50%)' : undefined,
        zIndex: 100,
        display: 'flex',
        flexDirection: isVertical ? 'column' : 'row',
        alignItems: 'center',
        justifyContent: panelJustify(position, itemsAlign),
        gap: 2,
        background: transparent ? 'transparent' : 'rgba(10, 12, 20, 0.96)',
        border: transparent ? 'none' : '1px solid rgba(255,255,255,0.10)',
        borderRadius: transparent ? 0 : 999,
        padding: transparent ? '14px 6px 8px' : '5px 6px',
        boxShadow: transparent ? 'none' : '0 8px 32px rgba(0,0,0,0.35)',
        maxHeight: isVertical ? 'calc(100vh - 48px)' : undefined,
      }}
    >
      {TOP_ROUTES.map((route) => (
        <TopBarItem
          key={route}
          route={route}
          isActive={route === activeRoute}
          onNavigate={onNavigate}
          label={t(LABEL_KEYS[route])}
        />
      ))}
    </div>
  );
}



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
        gap: 5,
        width: 58,
        padding: '10px 6px',
        borderRadius: 10,
        border: 'none',
        background: isActive
          ? 'rgba(255,255,255,0.12)'
          : hovered
          ? 'rgba(255,255,255,0.07)'
          : 'transparent',
        color: isActive ? '#FFFFFF' : hovered ? 'rgba(255,255,255,0.85)' : 'rgba(255,255,255,0.50)',
        cursor: 'pointer',
        outline: 'none',
        transition: 'all 0.2s ease',
        transform: hovered ? 'translateY(-3px)' : 'translateY(0)',
        boxShadow: isActive || hovered ? '0 8px 32px rgba(0,0,0,0.2)' : 'none',
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
            width: 3,
            height: 24,
            borderRadius: '999px 0 0 999px',
            background: '#E85D3F',
            boxShadow: '0 0 8px rgba(232, 93, 63, 0.6)',
          }}
        />
      )}
      {ICONS[route]}
      <span
        style={{
          fontSize: 9,
          fontWeight: 700,
          textTransform: 'uppercase',
          letterSpacing: '0.04em',
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
        gap: 6,
        height: 36,
        padding: '0 12px',
        borderRadius: 999,
        border: 'none',
        background: hovered ? 'rgba(255,255,255,0.10)' : 'transparent',
        color: isActive ? '#FFFFFF' : hovered ? 'rgba(255,255,255,0.90)' : 'rgba(255,255,255,0.65)',
        textShadow: '0 1px 4px rgba(0,0,0,0.8)',
        cursor: 'pointer',
        outline: 'none',
        transition: 'background 0.15s, color 0.15s',
        fontFamily: 'sans-serif',
        fontSize: 13,
        fontWeight: isActive ? 700 : 600,
        whiteSpace: 'nowrap',
        flexShrink: 0,
      }}
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
    >
      {ICONS[route]}
      {label}
    </button>
  );
}
