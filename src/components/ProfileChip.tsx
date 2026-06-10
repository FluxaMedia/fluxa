import React, { useEffect, useRef, useState } from 'react';
import { profileColor, profileInitials } from '../core/profiles';
import type { UserProfile } from '../core/types';
import { t } from '../i18n';

interface Props {
  profile: UserProfile;
  allProfiles: UserProfile[];
  onSwitchProfile: () => void;
  onSwitchToProfile: (p: UserProfile) => void;
  onOpenSettings: () => void;
  onEditProfile: () => void;
}

export function ProfileChip({ profile, allProfiles, onSwitchProfile, onSwitchToProfile, onOpenSettings, onEditProfile }: Props) {
  const [open, setOpen] = useState(false);
  const containerRef = useRef<HTMLDivElement>(null);
  const color = profileColor(profile);
  const initials = profileInitials(profile);

  useEffect(() => {
    if (!open) return;
    const handler = (e: MouseEvent) => {
      if (containerRef.current && !containerRef.current.contains(e.target as Node)) {
        setOpen(false);
      }
    };
    document.addEventListener('mousedown', handler);
    return () => document.removeEventListener('mousedown', handler);
  }, [open]);

  const close = () => setOpen(false);

  return (
    <div ref={containerRef} style={{ position: 'relative', zIndex: 50, flexShrink: 0 }}>
      {/* Avatar button */}
      <button
        onClick={() => setOpen((v) => !v)}
        style={{
          width: 42,
          height: 42,
          borderRadius: '50%',
          background: profile.avatarUrl ? 'transparent' : color,
          border: open ? '2px solid rgba(255,255,255,0.75)' : '2px solid rgba(255,255,255,0.15)',
          padding: 0,
          cursor: 'pointer',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          overflow: 'hidden',
          boxShadow: open
            ? `0 0 0 3px ${color}55, 0 8px 24px rgba(0,0,0,0.5)`
            : '0 4px 14px rgba(0,0,0,0.4)',
          transition: 'border-color 0.18s, box-shadow 0.18s',
          flexShrink: 0,
        }}
      >
        {profile.avatarUrl ? (
          <img src={profile.avatarUrl} alt={profile.name} style={{ width: '100%', height: '100%', objectFit: 'cover' }} />
        ) : (
          <span style={{ color: '#FFF', fontSize: 14, fontWeight: 900, fontFamily: 'sans-serif', userSelect: 'none' }}>
            {initials}
          </span>
        )}
      </button>

      {/* Dropdown panel */}
      {open && (
        <div
          style={{
            position: 'absolute',
            top: 'calc(100% + 10px)',
            right: 0,
            width: 220,
            background: '#141923',
            border: '1px solid rgba(255,255,255,0.09)',
            borderRadius: 12,
            boxShadow: '0 24px 64px rgba(0,0,0,0.75)',
            overflow: 'hidden',
            zIndex: 200,
          }}
        >
          {/* Profile avatars row */}
          <div style={{ padding: '16px 16px 12px', borderBottom: '1px solid rgba(255,255,255,0.07)' }}>
            <div style={{ display: 'flex', gap: 12, justifyContent: allProfiles.length <= 3 ? 'center' : 'flex-start', flexWrap: 'wrap' }}>
              {allProfiles.map((p) => (
                <ProfileAvatar
                  key={p.id}
                  profile={p}
                  active={p.id === profile.id}
                  onClick={() => {
                    if (p.id !== profile.id) {
                      onSwitchToProfile(p);
                    }
                    close();
                  }}
                />
              ))}
            </div>
          </div>

          {/* Menu items */}
          <div style={{ padding: '6px 0' }}>
            <DropdownItem
              icon={<ManageIcon />}
              label={t('profiles.manage')}
              onClick={() => { close(); onEditProfile(); }}
            />
            <DropdownItem
              icon={<SwitchIcon />}
              label={t('settings.switch_profiles')}
              onClick={() => { close(); onSwitchProfile(); }}
            />
            <div style={{ height: 1, background: 'rgba(255,255,255,0.07)', margin: '4px 0' }} />
            <DropdownItem
              icon={<AccountIcon />}
              label={t('nav.settings')}
              onClick={() => { close(); onOpenSettings(); }}
            />
          </div>
        </div>
      )}
    </div>
  );
}

function ProfileAvatar({ profile, active, onClick }: { profile: UserProfile; active: boolean; onClick: () => void }) {
  const [hovered, setHovered] = useState(false);
  const color = profileColor(profile);
  const initials = profileInitials(profile);

  return (
    <button
      onClick={onClick}
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
      style={{
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'center',
        gap: 6,
        background: 'none',
        border: 'none',
        cursor: active ? 'default' : 'pointer',
        padding: 0,
        minWidth: 52,
      }}
    >
      <div
        style={{
          width: 52,
          height: 52,
          borderRadius: 8,
          background: profile.avatarUrl ? 'transparent' : color,
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          overflow: 'hidden',
          flexShrink: 0,
          outline: active
            ? `2px solid #FFFFFF`
            : hovered
            ? `2px solid rgba(255,255,255,0.5)`
            : '2px solid transparent',
          outlineOffset: 2,
          transition: 'outline-color 0.15s',
          opacity: hovered && !active ? 0.85 : 1,
        }}
      >
        {profile.avatarUrl ? (
          <img src={profile.avatarUrl} alt="" style={{ width: '100%', height: '100%', objectFit: 'cover' }} />
        ) : (
          <span style={{ color: '#FFF', fontSize: 17, fontWeight: 900, fontFamily: 'sans-serif', userSelect: 'none' }}>
            {initials}
          </span>
        )}
      </div>
      <span
        style={{
          color: active ? '#FFFFFF' : 'rgba(255,255,255,0.6)',
          fontSize: 11,
          fontWeight: active ? 700 : 500,
          fontFamily: 'sans-serif',
          whiteSpace: 'nowrap',
          overflow: 'hidden',
          textOverflow: 'ellipsis',
          maxWidth: 52,
          transition: 'color 0.15s',
        }}
      >
        {profile.name ?? t('auto.profile')}
      </span>
    </button>
  );
}

function DropdownItem({
  icon,
  label,
  onClick,
  danger,
}: {
  icon: React.ReactNode;
  label: string;
  onClick: () => void;
  danger?: boolean;
}) {
  const [hovered, setHovered] = useState(false);
  return (
    <button
      onClick={onClick}
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
      style={{
        width: '100%',
        height: 40,
        background: hovered ? 'rgba(255,255,255,0.07)' : 'transparent',
        border: 'none',
        display: 'flex',
        alignItems: 'center',
        gap: 11,
        padding: '0 16px',
        cursor: 'pointer',
        color: danger ? '#FF6B6B' : hovered ? '#FFFFFF' : 'rgba(255,255,255,0.75)',
        fontSize: 13.5,
        fontWeight: 550,
        fontFamily: 'sans-serif',
        transition: 'background 0.12s, color 0.12s',
        textAlign: 'left',
      }}
    >
      <span style={{ display: 'flex', flexShrink: 0, opacity: 0.8 }}>{icon}</span>
      {label}
    </button>
  );
}

function ManageIcon() {
  return <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor"><path d="M3 17.25V21h3.75L17.81 9.94l-3.75-3.75L3 17.25zm17.71-10.46a1 1 0 0 0 0-1.41l-2.34-2.34a1 1 0 0 0-1.41 0l-1.83 1.83 3.75 3.75 1.83-1.83z"/></svg>;
}
function SwitchIcon() {
  return <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor"><path d="M6.99 11L3 15l3.99 4v-3H14v-2H6.99v-3zM21 9l-3.99-4v3H10v2h7.01v3L21 9z"/></svg>;
}
function AccountIcon() {
  return <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor"><path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm0 3c1.66 0 3 1.34 3 3s-1.34 3-3 3-3-1.34-3-3 1.34-3 3-3zm0 14.2a7.2 7.2 0 0 1-6-3.22c.03-1.99 4-3.08 6-3.08 1.99 0 5.97 1.09 6 3.08a7.2 7.2 0 0 1-6 3.22z"/></svg>;
}
