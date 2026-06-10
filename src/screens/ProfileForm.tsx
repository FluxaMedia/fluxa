import React, { useEffect, useRef, useState } from 'react';
import {
  Camera,
  Check,
  ImageSquare as ImagePlus,
} from '@phosphor-icons/react';
import {
  PROFILE_COLORS,
  createProfileObject,
  profileColor,
  profileInitials,
  saveProfile,
  setActiveProfileId,
} from '../core/profiles';
import { storageRead, storageWrite } from '../core/engine';
import type { AddonDescriptor, UserProfile } from '../core/types';
import { colors } from '../theme';
import { t } from '../i18n';
import { AvatarPickerModal } from './ProfileAvatarPicker';

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

export function AvatarPreview({ profile, size, circular }: { profile: UserProfile; size: number; circular?: boolean }) {
  const color = profileColor(profile);
  return (
    <div
      style={{
        width: size,
        height: size,
        borderRadius: circular ? '50%' : 8,
        background: profile.avatarUrl ? '#11151D' : color,
        border: circular ? 'none' : '1px solid rgba(255,255,255,0.12)',
        boxShadow: circular ? 'none' : `0 16px 40px rgba(0,0,0,0.35), inset 0 0 0 1px ${profile.avatarUrl ? 'rgba(255,255,255,0.05)' : 'rgba(255,255,255,0.18)'}`,
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
        <span style={{ color: '#FFFFFF', fontSize: Math.max(22, Math.round(size * 0.28)), fontWeight: 900, userSelect: 'none' }}>
          {profileInitials(profile)}
        </span>
      )}
    </div>
  );
}

export function ProfileForm({
  existing,
  allProfiles,
  onSaved,
  onCancel,
}: {
  existing: UserProfile | null;
  allProfiles: UserProfile[];
  onSaved: (updated: UserProfile[]) => void;
  onCancel: () => void;
}) {
  const [name, setName] = useState(existing?.name ?? '');
  const [color, setColor] = useState(existing?.color ?? PROFILE_COLORS[0]);
  const [avatarUrl, setAvatarUrl] = useState<string | undefined>(existing?.avatarUrl);
  const [showAvatarPicker, setShowAvatarPicker] = useState(false);
  const [busy, setBusy] = useState(false);
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
      const dataUrl = event.target?.result as string;
      setAvatarUrl(dataUrl);
      setShowAvatarPicker(false);
    };
    reader.readAsDataURL(file);
    e.target.value = '';
  };

  const handleSave = async () => {
    if (!name.trim() || busy) return;
    setBusy(true);
    try {
      const base: UserProfile = existing
        ? { ...existing, name: name.trim(), color, avatarUrl }
        : createProfileObject(name, color);
      const profile: UserProfile = { ...base, name: name.trim(), color, avatarUrl };
      const updated = await saveProfile(profile);
      if (!existing) {
        await setActiveProfileId(profile.id);
        try {
          const stored = (await storageRead<AddonDescriptor[]>('addons')) ?? [];
          await storageWrite('addons', [CINEMETA_DEFAULT, ...stored]);
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
    ...(existing ?? createProfileObject(name || 'Profil', color)),
    name: name.trim() || t('auto.profile'),
    color,
    avatarUrl,
  };
  const canSave = Boolean(name.trim()) && !busy;

  return (
    <section style={S.formShell}>
      <div style={S.previewPanel}>
        <button style={S.avatarEditButton} onClick={() => setShowAvatarPicker(true)} title={t('profiles.choose_image')}>
          <AvatarPreview profile={previewProfile} size={132} />
          <span style={S.cameraBadge}><Camera size={17} /></span>
        </button>
        <div style={S.previewCopy}>
          <p style={S.previewName}>{name.trim() || t('auto.profile')}</p>
          <p style={S.previewHint}>{avatarUrl ? t('profiles.custom_image') : t('profiles.initials_avatar')}</p>
        </div>
      </div>

      <div style={S.formPanel}>
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

        <div style={S.sectionHeader}>
          <span>{t('profiles.color')}</span>
          {avatarUrl && <button style={S.textButton} onClick={() => setAvatarUrl(undefined)}>{t('profiles.use_initials')}</button>}
        </div>
        <div style={S.colorGrid}>
          {PROFILE_COLORS.map((c) => (
            <button
              key={c}
              onClick={() => { setColor(c); if (avatarUrl) setAvatarUrl(undefined); }}
              style={{
                ...S.colorSwatch,
                background: c,
                borderColor: c === color && !avatarUrl ? '#FFFFFF' : 'rgba(255,255,255,0.16)',
                transform: c === color && !avatarUrl ? 'scale(1.08)' : 'scale(1)',
              }}
              aria-label={t('profiles.color_aria', c)}
            >
              {c === color && !avatarUrl && <Check size={15} />}
            </button>
          ))}
        </div>

        <button style={S.imageButton} onClick={() => setShowAvatarPicker(true)}>
          <ImagePlus size={18} />
          {t('profiles.choose_image')}
        </button>

        <div style={S.actions}>
          <button onClick={onCancel} style={S.btnSecondary}>{t('common.cancel')}</button>
          <button
            onClick={() => void handleSave()}
            disabled={!canSave}
            style={{
              ...S.btnPrimary,
              background: canSave ? colors.primary : 'rgba(255,255,255,0.1)',
              color: canSave ? '#FFFFFF' : 'rgba(255,255,255,0.35)',
            }}
          >
            {busy ? t('common.saving') : existing ? t('profiles.save') : t('profiles.create')}
          </button>
        </div>
      </div>

      <input ref={fileInputRef} type="file" accept="image/*" style={{ display: 'none' }} onChange={handleFileUpload} />

      {showAvatarPicker && (
        <AvatarPickerModal
          selected={avatarUrl}
          onSelect={(url) => { setAvatarUrl(url); setShowAvatarPicker(false); }}
          onUpload={() => fileInputRef.current?.click()}
          onClear={() => { setAvatarUrl(undefined); setShowAvatarPicker(false); }}
          onClose={() => setShowAvatarPicker(false)}
        />
      )}
    </section>
  );
}

const S: Record<string, React.CSSProperties> = {
  formShell: { display: 'grid', gridTemplateColumns: 'minmax(260px, 0.82fr) minmax(360px, 1.18fr)', gap: 20, alignItems: 'stretch' },
  previewPanel: { borderRadius: 8, border: '1px solid rgba(255,255,255,0.08)', background: 'rgba(255,255,255,0.045)', padding: 28, display: 'flex', flexDirection: 'column', alignItems: 'center', justifyContent: 'center', textAlign: 'center', minHeight: 388 },
  avatarEditButton: { position: 'relative', border: 'none', background: 'transparent', padding: 0, color: colors.white },
  cameraBadge: { position: 'absolute', right: -8, bottom: -8, width: 38, height: 38, borderRadius: 8, background: colors.primary, border: '3px solid #11151D', display: 'flex', alignItems: 'center', justifyContent: 'center' },
  previewCopy: { marginTop: 26 },
  previewName: { margin: 0, fontSize: 24, fontWeight: 900 },
  previewHint: { margin: '8px 0 0', color: colors.textDim, fontSize: 13, fontWeight: 700 },
  formPanel: { borderRadius: 8, border: '1px solid rgba(255,255,255,0.08)', background: 'rgba(255,255,255,0.045)', padding: 26, display: 'flex', flexDirection: 'column', gap: 16 },
  fieldLabel: { color: colors.textSecondary, fontSize: 12, fontWeight: 800, textTransform: 'uppercase' },
  input: { width: '100%', height: 50, borderRadius: 8, background: 'rgba(0,0,0,0.22)', border: '1px solid rgba(255,255,255,0.12)', color: colors.white, padding: '0 15px', fontSize: 15, fontWeight: 700, outline: 'none' },
  fieldNote: { margin: '-8px 0 0', color: '#FFD280', fontSize: 12, fontWeight: 650 },
  sectionHeader: { marginTop: 8, display: 'flex', alignItems: 'center', justifyContent: 'space-between', color: colors.textSecondary, fontSize: 12, fontWeight: 800, textTransform: 'uppercase' },
  textButton: { border: 'none', background: 'transparent', color: colors.blue, fontSize: 12, fontWeight: 800 },
  colorGrid: { display: 'grid', gridTemplateColumns: 'repeat(8, 34px)', gap: 10 },
  colorSwatch: { width: 34, height: 34, borderRadius: 8, border: '2px solid rgba(255,255,255,0.16)', color: '#FFFFFF', display: 'flex', alignItems: 'center', justifyContent: 'center', transition: 'transform 0.14s ease, border-color 0.14s ease' },
  imageButton: { width: '100%', height: 46, borderRadius: 8, border: '1px solid rgba(255,255,255,0.12)', background: 'rgba(255,255,255,0.06)', color: colors.white, display: 'flex', alignItems: 'center', justifyContent: 'center', gap: 10, fontSize: 14, fontWeight: 800 },
  actions: { marginTop: 'auto', display: 'flex', gap: 10 },
  btnSecondary: { flex: 1, height: 48, borderRadius: 8, background: 'rgba(255,255,255,0.07)', border: '1px solid rgba(255,255,255,0.12)', color: colors.textSecondary, fontSize: 14, fontWeight: 800 },
  btnPrimary: { flex: 1, height: 48, borderRadius: 8, border: 'none', fontSize: 14, fontWeight: 900, transition: 'background 0.2s, color 0.2s' },
};
