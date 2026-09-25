import React, { useRef, useState } from 'react';
import { AvatarPreview } from '../screens/ProfileForm';
import { PinPrompt } from './PinPrompt';
import type { UserProfile } from '../core/types';
import { t } from '../i18n';
import { Popover } from './ui/Popover';

interface Props {
  profile: UserProfile;
  allProfiles: UserProfile[];
  onSwitchProfile: () => void;
  onSwitchToProfile: (p: UserProfile) => void | Promise<void>;
  onOpenSettings: () => void;
  onEditProfile: () => void;
  showName?: boolean;
  active?: boolean;
}

export function ProfileChip({ profile, allProfiles, onSwitchProfile, onSwitchToProfile, onOpenSettings, onEditProfile, showName = false, active = false }: Props) {
  const [open, setOpen] = useState(false);
  const [pinProfile, setPinProfile] = useState<UserProfile | null>(null);
  const avatarBtnRef = useRef<HTMLButtonElement>(null);
  const avatarSize = showName ? 28 : 42;
  const avatarButtonSize = showName ? 32 : 42;

  const close = () => setOpen(false);

  return (
    <div
      style={{
        position: 'relative',
        display: 'flex',
        alignItems: 'center',
        gap: '0.375rem',
        flexShrink: 0,
        height: '2.625rem',
        boxSizing: 'border-box',
        padding: showName ? '0 0.875rem' : 0,
        borderRadius: '999px',
        background: showName && active ? 'var(--fluxa-fill-active)' : 'transparent',
        cursor: showName ? 'pointer' : undefined,
        transition: 'background 0.15s',
      }}
      onClick={showName ? onOpenSettings : undefined}
    >
      <button
        ref={avatarBtnRef}
        onClick={(event) => {
          event.stopPropagation();
          setOpen((v) => !v);
        }}
        style={{
          width: `${avatarButtonSize}px`,
          height: `${avatarButtonSize}px`,
          borderRadius: '50%',
          background: 'transparent',
          border: open ? `${showName ? '0.125rem' : '0.125rem'} solid var(--fluxa-focus)` : `${showName ? '0.0625rem' : '0.125rem'} solid var(--fluxa-border-strong)`,
          padding: 0,
          cursor: 'pointer',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          overflow: 'hidden',
          boxShadow: '0 0.25rem 0.875rem var(--fluxa-scrim, rgba(0,0,0,0.4))',
          transition: 'border-color 0.18s, box-shadow 0.18s',
          flexShrink: 0,
        }}
      >
        <AvatarPreview profile={profile} size={avatarSize} circular />
      </button>
      {showName && (
        <button
          onClick={(event) => {
            event.stopPropagation();
            onOpenSettings();
          }}
          title={t('nav.settings')}
          style={{
            maxWidth: '10rem',
            overflow: 'hidden',
            textOverflow: 'ellipsis',
            whiteSpace: 'nowrap',
            border: 'none',
            background: 'transparent',
            color: 'var(--fluxa-text-secondary)',
            fontFamily: 'inherit',
            fontSize: '0.875rem',
            fontWeight: 600,
            textShadow: '0 1px 0.25rem var(--fluxa-scrim, rgba(0,0,0,0.8))',
            cursor: 'pointer',
            padding: 0,
            height: '100%',
            display: 'flex',
            alignItems: 'center',
            borderRadius: '999px',
            outline: 'none',
            transition: 'background 0.15s, color 0.15s',
          }}
          onMouseEnter={(event) => {
            event.currentTarget.style.background = 'transparent';
            event.currentTarget.style.color = 'var(--fluxa-text-primary)';
          }}
          onMouseLeave={(event) => {
            event.currentTarget.style.background = 'transparent';
            event.currentTarget.style.color = 'var(--fluxa-text-secondary)';
          }}
        >
          {profile.name ?? t('auto.profile')}
        </button>
      )}
      <Popover open={open} onClose={close} anchorRef={avatarBtnRef} placement="bottom-end" gap={10} width="13.75rem" surface="navigation">
        <div style={{ padding: '0.375rem 0' }}>
          {allProfiles.map((p) => (
            <ProfileRow
              key={p.id}
              profile={p}
              active={p.id === profile.id}
              onClick={() => {
                if (p.id !== profile.id) {
                  if (p.pinHash) {
                    setPinProfile(p);
                  } else {
                    onSwitchToProfile(p);
                  }
                }
                close();
              }}
            />
          ))}
        </div>

        <div style={{ height: 1, background: 'var(--fluxa-border)' }} />

        <div style={{ padding: '0.375rem 0' }}>
          <DropdownItem
            icon={<ManageIcon />}
            label={t('profiles.manage')}
            onClick={() => {
              close();
              onEditProfile();
            }}
          />
          <DropdownItem
            icon={<GearIcon />}
            label={t('nav.settings')}
            onClick={() => {
              close();
              onOpenSettings();
            }}
          />
        </div>
      </Popover>

      {pinProfile && (
        <PinPrompt
          profile={pinProfile}
          onCancel={() => setPinProfile(null)}
          onSuccess={() => {
            const target = pinProfile;
            setPinProfile(null);
            onSwitchToProfile(target);
          }}
        />
      )}
    </div>
  );
}

function ProfileRow({ profile, active, onClick }: { profile: UserProfile; active: boolean; onClick: () => void }) {
  const [hovered, setHovered] = useState(false);

  return (
    <button
      onClick={onClick}
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
      style={{
        width: '100%',
        height: '2.75rem',
        background: active ? 'var(--fluxa-fill-active)' : hovered ? 'var(--fluxa-fill-hover)' : 'transparent',
        border: 'none',
        display: 'flex',
        alignItems: 'center',
        gap: '0.625rem',
        padding: '0 0.875rem',
        cursor: active ? 'default' : 'pointer',
        transition: 'background 0.12s',
      }}
    >
      <div style={{ borderRadius: '50%', overflow: 'hidden', flexShrink: 0 }}>
        <AvatarPreview profile={profile} size={28} circular />
      </div>
      <span
        style={{
          flex: 1,
          color: active ? 'var(--fluxa-text-primary)' : 'var(--fluxa-text-secondary)',
          fontFamily: 'inherit',
          fontSize: '0.875rem',
          fontWeight: 600,
          whiteSpace: 'nowrap',
          overflow: 'hidden',
          textOverflow: 'ellipsis',
          textAlign: 'left',
        }}
      >
        {profile.name ?? t('auto.profile')}
      </span>
      {active && (
        <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor" style={{ color: 'var(--fluxa-text-secondary)', flexShrink: 0 }}>
          <path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41L9 16.17z" />
        </svg>
      )}
    </button>
  );
}

function DropdownItem({ icon, label, onClick, danger }: { icon: React.ReactNode; label: string; onClick: () => void; danger?: boolean }) {
  const [hovered, setHovered] = useState(false);
  return (
    <button
      onClick={onClick}
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
      style={{
        width: '100%',
        height: '2.5rem',
        background: hovered ? 'var(--fluxa-fill-hover)' : 'transparent',
        border: 'none',
        display: 'flex',
        alignItems: 'center',
        gap: '0.6875rem',
        padding: '0 1rem',
        cursor: 'pointer',
        color: danger ? 'var(--fluxa-error)' : hovered ? 'var(--fluxa-text-primary)' : 'var(--fluxa-text-secondary)',
        fontFamily: 'inherit',
        fontSize: '0.875rem',
        fontWeight: 600,
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
  return (
    <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
      <path d="M3 17.25V21h3.75L17.81 9.94l-3.75-3.75L3 17.25zm17.71-10.46a1 1 0 0 0 0-1.41l-2.34-2.34a1 1 0 0 0-1.41 0l-1.83 1.83 3.75 3.75 1.83-1.83z" />
    </svg>
  );
}

function GearIcon() {
  return (
    <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
      <path d="M19.14 12.94c.04-.3.06-.61.06-.94s-.02-.64-.07-.94l2.03-1.58a.49.49 0 0 0 .12-.61l-1.92-3.32a.49.49 0 0 0-.59-.22l-2.39.96a7.02 7.02 0 0 0-1.62-.94l-.36-2.54a.484.484 0 0 0-.48-.41h-3.84c-.24 0-.43.17-.47.41l-.36 2.54c-.59.24-1.13.57-1.62.94l-2.39-.96a.47.47 0 0 0-.59.22L2.74 8.87a.47.47 0 0 0 .12.61l2.03 1.58c-.05.3-.07.62-.07.94s.02.64.07.94l-2.03 1.58a.49.49 0 0 0-.12.61l1.92 3.32c.12.22.37.29.59.22l2.39-.96c.5.38 1.03.7 1.62.94l.36 2.54c.05.24.24.41.48.41h3.84c.24 0 .44-.17.47-.41l.36-2.54c.59-.24 1.13-.56 1.62-.94l2.39.96c.22.08.47 0 .59-.22l1.92-3.32a.47.47 0 0 0-.12-.61l-2.01-1.58zM12 15.6c-1.98 0-3.6-1.62-3.6-3.6s1.62-3.6 3.6-3.6 3.6 1.62 3.6 3.6-1.62 3.6-3.6 3.6z" />
    </svg>
  );
}
