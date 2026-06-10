import React, { useEffect, useState } from 'react';
import { PencilSimple as Pencil, Plus, UserCircle as UserRound, X } from '@phosphor-icons/react';
import { deleteProfile, loadProfiles, profileColor, profileInitials, setActiveProfileId } from '../core/profiles';
import type { UserProfile } from '../core/types';
import { colors } from '../theme';
import { t } from '../i18n';
import { ProfileForm, AvatarPreview } from './ProfileForm';

interface Props {
  onProfileSelected: (profile: UserProfile) => void;
}

export function ProfileSelectionScreen({ onProfileSelected }: Props) {
  const [profiles, setProfiles] = useState<UserProfile[]>([]);
  const [mode, setMode] = useState<'select' | 'create' | 'edit'>('select');
  const [editingProfile, setEditingProfile] = useState<UserProfile | null>(null);

  useEffect(() => {
    loadProfiles().then(setProfiles);
  }, []);

  const handleSelect = async (profile: UserProfile) => {
    await setActiveProfileId(profile.id);
    onProfileSelected(profile);
  };

  const handleDelete = async (id: string) => {
    const updated = await deleteProfile(id);
    setProfiles(updated);
  };

  const handleSaved = (updated: UserProfile[]) => {
    setProfiles(updated);
    setMode('select');
    setEditingProfile(null);
  };

  const handleEdit = (profile: UserProfile) => {
    setEditingProfile(profile);
    setMode('edit');
  };

  const showForm = mode === 'create' || mode === 'edit';

  return (
    <div style={S.root}>
      <div style={S.topBar}>
        <div>
          <p style={S.logo}>fluxa</p>
          <p style={S.kicker}>{t('app.desktop')}</p>
        </div>
        {showForm && (
          <button style={S.closeButton} onClick={() => { setMode('select'); setEditingProfile(null); }} aria-label={t('common.close')}>
            <X size={20} />
          </button>
        )}
      </div>

      <main style={showForm ? S.main : S.mainSelect}>
        {showForm ? (
          <section style={S.hero}>
            <p style={S.eyebrow}>{t('profiles.settings')}</p>
            <h1 style={S.title}>{editingProfile ? t('profiles.edit') : t('profiles.create_new')}</h1>
            <p style={S.subtitle}>{t('profiles.form_subtitle')}</p>
          </section>
        ) : (
          <h1 style={S.selectTitle}>{t('profiles.who_watching')}</h1>
        )}

        {mode === 'select' && (
          <section style={S.profileGrid} aria-label={t('profiles.list')}>
            {profiles.map((profile) => (
              <ProfileCard
                key={profile.id}
                profile={profile}
                onSelect={() => void handleSelect(profile)}
                onDelete={() => void handleDelete(profile.id)}
                onEdit={() => handleEdit(profile)}
              />
            ))}
            <AddProfileCard onClick={() => setMode('create')} />
            {profiles.length === 0 && (
              <div style={S.emptyState}>
                <UserRound size={22} />
                <span>{t('profiles.empty_create_hint')}</span>
              </div>
            )}
          </section>
        )}

        {showForm && (
          <ProfileForm
            existing={editingProfile}
            allProfiles={profiles}
            onSaved={handleSaved}
            onCancel={() => { setMode('select'); setEditingProfile(null); }}
          />
        )}
      </main>
    </div>
  );
}

function ProfileCard({ profile, onSelect, onDelete, onEdit }: {
  profile: UserProfile; onSelect: () => void; onDelete: () => void; onEdit: () => void;
}) {
  const [hovered, setHovered] = useState(false);

  return (
    <article style={S.profileCard} onMouseEnter={() => setHovered(true)} onMouseLeave={() => setHovered(false)}>
      <button onClick={onSelect} style={S.profileSelectButton}>
        <div style={{ ...S.avatarCircleWrap, borderColor: hovered ? '#FFFFFF' : 'rgba(255,255,255,0.18)', transform: hovered ? 'scale(1.04)' : 'scale(1)' }}>
          <AvatarPreview profile={profile} size={150} circular />
        </div>
        <span style={S.profileName}>{profile.name ?? t('auto.profile')}</span>
      </button>
      <button
        style={{ ...S.editPencilBtn, opacity: hovered ? 1 : 0.55 }}
        onClick={(e) => { e.stopPropagation(); onEdit(); }}
        title={t('auto.edit')}
        aria-label={t('profiles.edit')}
      >
        <Pencil size={16} />
      </button>
    </article>
  );
}

function AddProfileCard({ onClick }: { onClick: () => void }) {
  const [hovered, setHovered] = useState(false);

  return (
    <div style={S.profileCard}>
      <button
        onClick={onClick}
        onMouseEnter={() => setHovered(true)}
        onMouseLeave={() => setHovered(false)}
        style={S.profileSelectButton}
      >
        <div style={{ ...S.addCircle, background: hovered ? 'rgba(255,255,255,0.18)' : 'rgba(255,255,255,0.1)', transform: hovered ? 'scale(1.04)' : 'scale(1)' }}>
          <Plus size={44} color="#FFFFFF" />
        </div>
        <span style={S.addLabel}>{t('profiles.add_profile')}</span>
      </button>
    </div>
  );
}

const S: Record<string, React.CSSProperties> = {
  root: { position: 'fixed', inset: 0, zIndex: 9999, background: '#000000', color: colors.white, overflow: 'auto' },
  topBar: { position: 'absolute', top: 0, left: 0, right: 0, height: 76, display: 'flex', alignItems: 'center', justifyContent: 'space-between', padding: '0 44px', zIndex: 2 },
  logo: { margin: 0, fontSize: 24, fontWeight: 900, letterSpacing: 2.5 },
  kicker: { margin: '2px 0 0', color: colors.textDim, fontSize: 11, fontWeight: 700, textTransform: 'uppercase' },
  main: { minHeight: '100%', width: 'min(1120px, calc(100vw - 56px))', margin: '0 auto', padding: '118px 0 56px' },
  mainSelect: { minHeight: '100vh', display: 'flex', flexDirection: 'column', alignItems: 'center', justifyContent: 'center', padding: '80px 40px 60px' },
  selectTitle: { margin: '0 0 56px', fontSize: 48, fontWeight: 900, letterSpacing: 1, textAlign: 'center', textTransform: 'uppercase' },
  hero: { marginBottom: 34, maxWidth: 640 },
  eyebrow: { color: colors.primary, fontSize: 12, fontWeight: 850, textTransform: 'uppercase', margin: '0 0 12px' },
  title: { margin: 0, fontSize: 44, lineHeight: 1.05, fontWeight: 900 },
  subtitle: { margin: '14px 0 0', color: colors.textSecondary, fontSize: 15, lineHeight: 1.65, maxWidth: 560 },
  profileGrid: { display: 'flex', flexDirection: 'row', flexWrap: 'wrap', gap: 48, alignItems: 'flex-start', justifyContent: 'center' },
  profileCard: { display: 'flex', flexDirection: 'column', alignItems: 'center', gap: 0, background: 'transparent', border: 'none' },
  profileSelectButton: { width: 'auto', border: 'none', background: 'transparent', color: colors.white, display: 'flex', flexDirection: 'column', alignItems: 'center', textAlign: 'center', cursor: 'pointer', padding: 0 },
  avatarCircleWrap: { borderRadius: '50%', border: '3px solid rgba(255,255,255,0.18)', overflow: 'hidden', transition: 'border-color 0.18s ease, transform 0.18s ease', width: 150, height: 150, flexShrink: 0 },
  profileName: { marginTop: 18, maxWidth: 160, color: colors.white, fontSize: 17, fontWeight: 900, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap', textTransform: 'uppercase', letterSpacing: 0.5 },
  editPencilBtn: { marginTop: 10, border: 'none', background: 'transparent', color: 'rgba(255,255,255,0.7)', cursor: 'pointer', padding: '4px 8px', display: 'flex', alignItems: 'center', justifyContent: 'center', transition: 'opacity 0.15s ease' },
  addCircle: { width: 150, height: 150, borderRadius: '50%', display: 'flex', alignItems: 'center', justifyContent: 'center', transition: 'background 0.18s ease, transform 0.18s ease', cursor: 'pointer' },
  addLabel: { marginTop: 18, color: 'rgba(255,255,255,0.5)', fontSize: 17, fontWeight: 900, textTransform: 'uppercase', letterSpacing: 0.5 },
  emptyState: { gridColumn: '1 / -1', height: 64, borderRadius: 8, border: '1px solid rgba(255,255,255,0.08)', background: 'rgba(255,255,255,0.035)', color: colors.textSecondary, display: 'flex', alignItems: 'center', justifyContent: 'center', gap: 10, fontSize: 13, fontWeight: 700 },
  closeButton: { width: 38, height: 38, borderRadius: 8, border: '1px solid rgba(255,255,255,0.12)', background: 'rgba(255,255,255,0.06)', color: colors.white, display: 'flex', alignItems: 'center', justifyContent: 'center' },
};
