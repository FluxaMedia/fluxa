import React, { useEffect, useRef, useState } from 'react';
import { Camera, ImagePlus } from 'lucide-react';
import {
  PROFILE_COLORS,
  createProfileObject,
  hashPin,
  saveProfile,
  setActiveProfileId,
} from '../core/profiles';
import { saveAddons } from '../core/libraryOps';
import { nuvioPushProfiles } from '../core/nuvioApi';
import { freshNuvioProfile } from '../core/nuvioSync';
import type { AddonDescriptor, UserProfile } from '../core/types';
import type { ProfileAvatarPack } from '../core/profileAvatarPacks';
import { t } from '../i18n';

const FONT = "'Montserrat', -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif";

const CINEMETA_DEFAULT: AddonDescriptor = {
  transportUrl: 'https://v3-cinemeta.strem.io/manifest.json',
  manifest: {
    id: 'com.linvo.cinemeta',
    name: 'Cinemeta',
    description: 'The official add-on for movie and series catalogs',
    version: '3.0.14',
    resources: ['catalog', 'meta'],
    types: ['movie', 'series', 'channel'],
    catalogs: [
      { type: 'movie', id: 'top', name: 'Popular Movies', extra: [{ name: 'genre' }, { name: 'skip' }] },
      { type: 'series', id: 'top', name: 'Popular Series', extra: [{ name: 'genre' }, { name: 'skip' }] },
      { type: 'movie', id: 'imdbRating', name: 'Top Rated Movies', extra: [{ name: 'genre' }, { name: 'skip' }] },
      { type: 'series', id: 'imdbRating', name: 'Top Rated Series', extra: [{ name: 'genre' }, { name: 'skip' }] },
    ],
    logo: 'https://www.strem.io/s/addon-logo/cinemeta.png',
  },
};

const AVATAR_COLORS = ['#E03131', '#1971C2', '#2F9E44', '#E67700', '#7048E8', '#0C8599', '#C2255C', '#5C940D'];

function avatarColor(profile: UserProfile): string {
  let h = 0;
  const seed = profile.id ?? profile.name ?? '';
  for (let i = 0; i < seed.length; i++) h = (h * 31 + seed.charCodeAt(i)) >>> 0;
  return AVATAR_COLORS[h % AVATAR_COLORS.length];
}

function SmileFace({ size }: { size: number }) {
  return (
    <svg
      width={size}
      height={size}
      viewBox="0 0 100 100"
      fill="none"
      xmlns="http://www.w3.org/2000/svg"
      style={{ display: 'block', flexShrink: 0 }}
    >
      <circle cx="30" cy="37" r="8.5" fill="white" />
      <circle cx="67" cy="34" r="7.5" fill="white" />
      <path
        d="M 22 62 C 20 54 26 57 30 65 C 42 75 66 68 74 57"
        stroke="white"
        strokeWidth="7.5"
        strokeLinecap="round"
      />
    </svg>
  );
}

export function AvatarPreview({ profile, size, circular }: { profile: UserProfile; size: number; circular?: boolean }) {
  const color = avatarColor(profile);
  return (
    <div
      style={{
        width: size,
        height: size,
        borderRadius: circular ? '50%' : Math.round(size * 0.13),
        background: profile.avatarUrl ? '#111' : color,
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        overflow: 'hidden',
        flexShrink: 0,
      }}
    >
      {profile.avatarUrl ? (
        <img src={profile.avatarUrl} alt={profile.name} style={{ width: '100%', height: '100%', objectFit: 'cover' }} />
      ) : (
        <SmileFace size={Math.round(size * 0.72)} />
      )}
    </div>
  );
}

export function ProfileForm({
  existing,
  allProfiles,
  avatarPacks,
  onSaved,
  onCancel,
}: {
  existing: UserProfile | null;
  allProfiles: UserProfile[];
  avatarPacks: ProfileAvatarPack[];
  onSaved: (updated: UserProfile[]) => void;
  onCancel: () => void;
}) {
  const [name, setName] = useState(existing?.name ?? '');
  const [avatarUrl, setAvatarUrl] = useState<string | undefined>(existing?.avatarUrl);
  const [busy, setBusy] = useState(false);
  const [pin, setPin] = useState('');
  const [removePin, setRemovePin] = useState(false);
  const nameInputRef = useRef<HTMLInputElement>(null);
  const fileInputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    setTimeout(() => nameInputRef.current?.focus(), 60);
  }, []);

  const handleFileUpload = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file) return;
    const reader = new FileReader();
    reader.onload = (event) => {
      setAvatarUrl(event.target?.result as string);
    };
    reader.readAsDataURL(file);
    e.target.value = '';
  };

  const handleSave = async () => {
    if (!name.trim() || busy) return;
    setBusy(true);
    try {
      const color = existing?.color ?? PROFILE_COLORS[0];
      const remoteAccounts = Array.from(new Map(
        allProfiles
          .filter((candidate) => candidate.nuvioAccessToken && candidate.nuvioUserId)
          .map((candidate) => [candidate.nuvioUserId!, candidate]),
      ).values());
      const inheritedNuvio = !existing && remoteAccounts.length === 1
        ? remoteAccounts[0]
        : undefined;
      const nextRemoteIndex = inheritedNuvio
        ? Math.max(0, ...allProfiles
          .filter((candidate) => candidate.nuvioUserId === inheritedNuvio.nuvioUserId)
          .map((candidate) => candidate.nuvioProfileIndex ?? 0)) + 1
        : undefined;
      const base: UserProfile = existing
        ? { ...existing, name: name.trim(), color, avatarUrl }
        : {
          ...await createProfileObject(name, color),
          ...(inheritedNuvio && nextRemoteIndex != null && nextRemoteIndex <= 6 ? {
            email: inheritedNuvio.email,
            nuvioAccessToken: inheritedNuvio.nuvioAccessToken,
            nuvioRefreshToken: inheritedNuvio.nuvioRefreshToken,
            nuvioTokenExpiresAt: inheritedNuvio.nuvioTokenExpiresAt,
            nuvioUserId: inheritedNuvio.nuvioUserId,
            nuvioEmail: inheritedNuvio.nuvioEmail,
            nuvioProfileIndex: nextRemoteIndex,
          } : {}),
        };
      const pinHash = removePin ? undefined : pin.trim().length === 4 ? await hashPin(pin.trim()) : base.pinHash;
      const profile: UserProfile = { ...base, name: name.trim(), color, avatarUrl, pinHash };
      const updated = await saveProfile(profile);
      if (!existing) {
        await setActiveProfileId(profile.id);
        try { await saveAddons([CINEMETA_DEFAULT]); } catch {}
      }
      if (profile.nuvioAccessToken && profile.nuvioUserId && profile.nuvioProfileIndex != null) {
        try {
          const freshProfile = await freshNuvioProfile(profile);
          const remoteProfiles = [...allProfiles.filter((candidate) =>
            candidate.id !== profile.id && candidate.nuvioUserId === freshProfile.nuvioUserId && candidate.nuvioProfileIndex != null,
          ), freshProfile];
          await nuvioPushProfiles(freshProfile.nuvioAccessToken!, remoteProfiles.map((candidate) => ({
            profile_index: candidate.nuvioProfileIndex!,
            name: candidate.name ?? `Profile ${candidate.nuvioProfileIndex}`,
            avatar_color_hex: candidate.color ?? null,
            avatar_url: candidate.avatarUrl ?? null,
          })));
        } catch {}
      }
      onSaved(updated);
    } finally {
      setBusy(false);
    }
  };

  const duplicateName = Boolean(
    name.trim() &&
    allProfiles.some((p) => p.id !== existing?.id && p.name?.trim().toLowerCase() === name.trim().toLowerCase()),
  );

  const previewProfile: UserProfile = {
    ...(existing ?? { id: 'preview', name: name || 'Profile', color: PROFILE_COLORS[0] }),
    name: name.trim() || t('auto.profile'),
    avatarUrl,
  };

  const pinValid = pin.length === 0 || pin.length === 4;
  const canSave = Boolean(name.trim()) && !duplicateName && !busy && pinValid;

  return (
    <section style={S.formShell}>
      <div style={S.previewPanel}>
        <button style={S.avatarEditButton} onClick={() => fileInputRef.current?.click()} title={t('profiles.choose_image')}>
          <AvatarPreview profile={previewProfile} size={128} />
          <span style={S.cameraBadge}><Camera size={15} /></span>
        </button>
        <p style={S.previewName}>{name.trim() || t('auto.profile')}</p>
        {avatarUrl && (
          <button style={S.clearImageBtn} onClick={() => setAvatarUrl(undefined)}>
            {t('profiles.use_initials')}
          </button>
        )}
      </div>

      <div style={S.formPanel}>
        <div>
          <label style={S.fieldLabel} htmlFor="profile-name">{t('profiles.name')}</label>
          <input
            id="profile-name"
            ref={nameInputRef}
            value={name}
            onChange={(e) => setName(e.target.value)}
            onKeyDown={(e) => { if (e.key === 'Enter') void handleSave(); if (e.key === 'Escape') onCancel(); }}
            placeholder={t('profiles.name_placeholder')}
            style={S.input}
          />
          {duplicateName && <p style={S.fieldNote}>{t('profiles.duplicate_name')}</p>}
        </div>

        <button style={S.imageButton} onClick={() => fileInputRef.current?.click()}>
          <ImagePlus size={16} />
          {t('profiles.choose_image')}
        </button>

        {avatarPacks.length > 0 && (
          <div>
            <label style={S.fieldLabel}>{t('profiles.avatar_packs')}</label>
            <div style={S.avatarPackGroups}>
              {avatarPacks.map((pack) => (
                <section key={pack.id}>
                  <p style={S.packTitle}>{pack.title}</p>
                  <div style={S.avatarPackGrid}>
                    {pack.avatars.map((avatar) => (
                      <button key={avatar.url} onClick={() => setAvatarUrl(avatar.url)} style={{ ...S.packAvatarButton, outline: avatarUrl === avatar.url ? '2px solid #FFFFFF' : 'none' }} title={avatar.name}>
                        <img src={avatar.url} alt={avatar.name} style={S.packAvatarImage} />
                      </button>
                    ))}
                  </div>
                </section>
              ))}
            </div>
          </div>
        )}

        <div>
          <label style={S.fieldLabel} htmlFor="profile-pin">{t('profiles.pin_lock')}</label>
          <input
            id="profile-pin"
            value={pin}
            onChange={(e) => { setPin(e.target.value.replace(/\D/g, '').slice(0, 4)); setRemovePin(false); }}
            placeholder={existing?.pinHash && !removePin ? t('profiles.pin_set_placeholder') : t('profiles.pin_placeholder')}
            inputMode="numeric"
            maxLength={4}
            style={S.input}
          />
          {!pinValid && <p style={S.fieldNote}>{t('profiles.pin_invalid')}</p>}
          {existing?.pinHash && !removePin && (
            <button style={S.clearImageBtn} onClick={() => { setRemovePin(true); setPin(''); }}>
              {t('profiles.remove_pin')}
            </button>
          )}
          {removePin && <p style={S.fieldNote}>{t('profiles.pin_will_be_removed')}</p>}
        </div>

        <div style={S.actions}>
          <button onClick={onCancel} style={S.btnSecondary}>{t('common.cancel')}</button>
          <button
            onClick={() => void handleSave()}
            disabled={!canSave}
            style={{
              ...S.btnPrimary,
              background: canSave ? '#FFFFFF' : 'rgba(255,255,255,0.10)',
              color: canSave ? '#000000' : 'rgba(255,255,255,0.30)',
              cursor: canSave ? 'pointer' : 'default',
            }}
          >
            {busy ? t('common.saving') : existing ? t('profiles.save') : t('profiles.create')}
          </button>
        </div>
      </div>

      <input ref={fileInputRef} type="file" accept="image/*" style={{ display: 'none' }} onChange={handleFileUpload} />
    </section>
  );
}

const S: Record<string, React.CSSProperties> = {
  formShell: { display: 'grid', gridTemplateColumns: 'minmax(13.75rem, 0.75fr) minmax(20rem, 1.25fr)', gap: '0.875rem', alignItems: 'stretch' },
  previewPanel: { position: 'sticky', top: '1rem', alignSelf: 'start', height: 'fit-content', boxSizing: 'border-box', borderRadius: '0.75rem', border: '1px solid rgba(255,255,255,0.08)', background: '#141414', padding: '2rem 1.25rem', display: 'flex', flexDirection: 'column', alignItems: 'center', justifyContent: 'center', textAlign: 'center', gap: 0 },
  avatarEditButton: { position: 'relative', border: 'none', background: 'transparent', padding: 0, color: '#FFFFFF', cursor: 'pointer', outline: 'none' },
  cameraBadge: { position: 'absolute', right: '-0.5rem', bottom: '-0.5rem', width: '2rem', height: '2rem', borderRadius: '0.5rem', background: '#2C2C2C', border: '0.125rem solid #141414', display: 'flex', alignItems: 'center', justifyContent: 'center', color: 'rgba(255,255,255,0.65)' },
  previewName: { margin: '1.25rem 0 0', fontSize: '1.125rem', fontWeight: 600, fontFamily: FONT, letterSpacing: '-0.02em', color: '#FFFFFF' },
  clearImageBtn: { marginTop: '0.625rem', border: 'none', background: 'transparent', color: 'rgba(255,255,255,0.38)', fontSize: '0.75rem', fontWeight: 400, fontFamily: FONT, cursor: 'pointer', outline: 'none' },
  formPanel: { borderRadius: '0.75rem', border: '1px solid rgba(255,255,255,0.08)', background: '#141414', padding: '1.375rem', display: 'flex', flexDirection: 'column', gap: '0.875rem' },
  fieldLabel: { display: 'block', color: 'rgba(255,255,255,0.38)', fontSize: '0.625rem', fontWeight: 500, textTransform: 'uppercase', letterSpacing: '0.08em', fontFamily: FONT, marginBottom: '0.5rem' },
  input: { width: '100%', height: '2.75rem', borderRadius: '0.5rem', background: 'rgba(255,255,255,0.05)', border: '1px solid rgba(255,255,255,0.10)', color: '#FFFFFF', padding: '0 0.8125rem', fontSize: '0.875rem', fontWeight: 500, fontFamily: FONT, outline: 'none', boxSizing: 'border-box' },
  fieldNote: { margin: '0.375rem 0 0', color: '#FFD280', fontSize: '0.75rem', fontWeight: 400, fontFamily: FONT },
  imageButton: { width: '100%', height: '2.625rem', borderRadius: '0.5rem', border: '1px solid rgba(255,255,255,0.10)', background: 'rgba(255,255,255,0.05)', color: 'rgba(255,255,255,0.60)', display: 'flex', alignItems: 'center', justifyContent: 'center', gap: '0.5rem', fontSize: '0.8125rem', fontWeight: 500, fontFamily: FONT, cursor: 'pointer', outline: 'none' },
  avatarPackGrid: { display: 'grid', gridTemplateColumns: 'repeat(auto-fill, minmax(4rem, 1fr))', gap: '0.625rem', marginTop: '0.5rem' },
  avatarPackGroups: { display: 'grid', gap: '1rem' },
  packTitle: { margin: '0.875rem 0 0', color: 'rgba(255,255,255,0.82)', fontSize: '0.8125rem', fontWeight: 600, fontFamily: FONT },
  packAvatarButton: { width: '4rem', height: '4rem', padding: 0, border: 0, borderRadius: '0.625rem', background: '#111', overflow: 'hidden', cursor: 'pointer' },
  packAvatarImage: { width: '100%', height: '100%', objectFit: 'cover', display: 'block' },
  actions: { marginTop: 'auto', display: 'flex', gap: '0.5rem' },
  btnSecondary: { flex: 1, height: '2.75rem', borderRadius: '0.5rem', background: 'rgba(255,255,255,0.06)', border: '1px solid rgba(255,255,255,0.10)', color: 'rgba(255,255,255,0.55)', fontSize: '0.8125rem', fontWeight: 500, fontFamily: FONT, cursor: 'pointer', outline: 'none' },
  btnPrimary: { flex: 1, height: '2.75rem', borderRadius: '0.5rem', border: 'none', fontSize: '0.8125rem', fontWeight: 600, fontFamily: FONT, transition: 'background 0.15s, color 0.15s' },
};
