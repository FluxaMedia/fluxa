import React, { useEffect, useState } from 'react';
import { Lock, Pencil, Plus, Settings, Trash2, UserRound, X } from 'lucide-react';
import { deleteProfile, loadProfiles, profileColor, profileInitials, setActiveProfileId } from '../core/profiles';
import { coreInvoke } from '../core/engine';
import { nuvioDeleteProfileData, nuvioPushProfiles } from '../core/nuvioApi';
import { freshNuvioProfile } from '../core/nuvioSync';
import type { UserProfile } from '../core/types';
import { colors } from '../theme';
import { t } from '../i18n';
import { ProfileForm, AvatarPreview } from './ProfileForm';
import { PinPrompt } from '../components/PinPrompt';
import { ConfirmDialog } from '../components/ConfirmDialog';
import { loadProfilePickerSettings, refreshAllAvatarPacks, type ProfilePickerSettings } from '../core/profileAvatarPacks';
import { ProfilePickerSettings as ProfilePickerSettingsView } from './ProfilePickerSettings';

interface Props {
  onProfileSelected: (profile: UserProfile) => void;
  onProfilesChanged?: (profiles: UserProfile[]) => void;
}

export function ProfileSelectionScreen({ onProfileSelected, onProfilesChanged }: Props) {
  const [profiles, setProfiles] = useState<UserProfile[]>([]);
  const [primaryProfileId, setPrimaryProfileId] = useState<string | null>(null);
  const [mode, setMode] = useState<'select' | 'create' | 'edit' | 'settings'>('select');
  const [editingProfile, setEditingProfile] = useState<UserProfile | null>(null);
  const [pinProfile, setPinProfile] = useState<UserProfile | null>(null);
  const [pinGate, setPinGate] = useState<{ profile: UserProfile; action: 'edit' | 'delete' } | null>(null);
  const [confirmDeleteProfile, setConfirmDeleteProfile] = useState<UserProfile | null>(null);
  const [pickerSettings, setPickerSettings] = useState<ProfilePickerSettings>({ avatarPacks: [] });
  const primaryProfile = profiles.find((p) => p.id === primaryProfileId) ?? null;

  useEffect(() => {
    loadProfiles().then(setProfiles);
    loadProfilePickerSettings().then(setPickerSettings);
  }, []);

  useEffect(() => {
    coreInvoke<string>('primaryProfileId', JSON.stringify(profiles)).then(setPrimaryProfileId);
  }, [profiles]);

  const handleSelect = async (profile: UserProfile) => {
    if (profile.pinHash) { setPinProfile(profile); return; }
    await setActiveProfileId(profile.id);
    onProfileSelected(profile);
  };

  const handleDelete = async (id: string) => {
    const deleted = profiles.find((profile) => profile.id === id);
    const updated = await deleteProfile(id);
    setProfiles(updated);
    onProfilesChanged?.(updated);
    if (deleted?.nuvioAccessToken && deleted.nuvioUserId && deleted.nuvioProfileIndex != null) {
      void (async () => {
        try {
          const freshProfile = await freshNuvioProfile(deleted);
          const remoteProfiles = updated.filter((profile) =>
            profile.nuvioUserId === freshProfile.nuvioUserId && profile.nuvioProfileIndex != null,
          );
          await nuvioDeleteProfileData(freshProfile.nuvioAccessToken!, freshProfile.nuvioProfileIndex!);
          await nuvioPushProfiles(freshProfile.nuvioAccessToken!, remoteProfiles.map((profile) => ({
            profile_index: profile.nuvioProfileIndex!,
            name: profile.name ?? `Profile ${profile.nuvioProfileIndex}`,
            avatar_color_hex: profile.color ?? null,
            avatar_url: profile.avatarUrl ?? null,
            uses_primary_addons: profile.usesPrimaryAddons ?? false,
            uses_primary_plugins: profile.usesPrimaryPlugins ?? false,
          })));
        } catch {}
      })();
    }
  };

  const handleSaved = (updated: UserProfile[]) => {
    setProfiles(updated);
    onProfilesChanged?.(updated);
    setMode('select');
    setEditingProfile(null);
  };

  const handleEdit = (profile: UserProfile) => {
    setEditingProfile(profile);
    setMode('edit');
  };

  const requestEdit = (profile: UserProfile) => {
    if (profile.pinHash) { setPinGate({ profile, action: 'edit' }); return; }
    handleEdit(profile);
  };

  const requestDelete = (profile: UserProfile) => {
    if (profile.pinHash) { setPinGate({ profile, action: 'delete' }); return; }
    setConfirmDeleteProfile(profile);
  };

  const showForm = mode === 'create' || mode === 'edit';
  const isSettings = mode === 'settings';

  return (
    <div style={{ ...S.root, ...(pickerSettings.backgroundUrl ? { backgroundImage: `linear-gradient(rgba(12,12,12,0.78), rgba(12,12,12,0.88)), url(${pickerSettings.backgroundUrl})`, backgroundSize: 'cover', backgroundPosition: 'center' } : {}) }}>
      <div style={S.topBar}>
        <div>
          <p style={S.logo}>fluxa</p>
          <p style={S.kicker}>{t('app.desktop')}</p>
        </div>
        {(showForm || isSettings) ? (
          <button style={S.closeButton} onClick={() => { setMode('select'); setEditingProfile(null); }} aria-label={t('common.close')}>
            <X size={20} />
          </button>
        ) : (
          <button style={S.closeButton} onClick={() => setMode('settings')} aria-label={t('profiles.picker_settings')} title={t('profiles.picker_settings')}>
            <Settings size={19} />
          </button>
        )}
      </div>

      <main style={showForm || isSettings ? S.main : S.mainSelect}>
        {showForm ? (
          <section style={S.hero}>
            <p style={S.eyebrow}>{t('profiles.settings')}</p>
            <h1 style={S.title}>{editingProfile ? t('profiles.edit') : t('profiles.create_new')}</h1>
            <p style={S.subtitle}>{t('profiles.form_subtitle')}</p>
          </section>
        ) : isSettings ? (
          <section style={S.hero}>
            <p style={S.eyebrow}>{t('profiles.settings')}</p>
            <h1 style={S.title}>{t('profiles.picker_settings')}</h1>
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
                isPrimary={primaryProfileId === profile.id}
                onSelect={() => void handleSelect(profile)}
                onDelete={() => requestDelete(profile)}
                onEdit={() => requestEdit(profile)}
              />
            ))}
            <AddProfileCard onClick={() => setMode('create')} />
          </section>
        )}

        {showForm && (
          <ProfileForm
            existing={editingProfile}
            allProfiles={profiles}
            avatarPacks={pickerSettings.avatarPacks}
            onSaved={handleSaved}
            onCancel={() => { setMode('select'); setEditingProfile(null); }}
            onRefreshAvatarPacks={async () => {
              await refreshAllAvatarPacks();
              setPickerSettings(await loadProfilePickerSettings());
            }}
          />
        )}

        {isSettings && (
          <ProfilePickerSettingsView settings={pickerSettings} onSaved={setPickerSettings} onBack={() => setMode('select')} />
        )}
      </main>

      {pinProfile && (
        <PinPrompt
          profile={pinProfile}
          onCancel={() => setPinProfile(null)}
          onSuccess={() => {
            const profile = pinProfile;
            setPinProfile(null);
            void (async () => { await setActiveProfileId(profile.id); onProfileSelected(profile); })();
          }}
        />
      )}

      {pinGate && (
        <PinPrompt
          profile={pinGate.profile}
          overridePin={primaryProfile && primaryProfile.id !== pinGate.profile.id ? primaryProfile : undefined}
          onCancel={() => setPinGate(null)}
          onSuccess={() => {
            const { profile, action } = pinGate;
            setPinGate(null);
            if (action === 'edit') handleEdit(profile);
            else setConfirmDeleteProfile(profile);
          }}
        />
      )}

      {confirmDeleteProfile && (
        <ConfirmDialog
          title={t('profiles.delete_confirm_title')}
          body={t('profiles.delete_confirm_body', confirmDeleteProfile.name ?? t('auto.profile'))}
          confirmLabel={t('profiles.delete')}
          cancelLabel={t('common.cancel')}
          destructive
          onCancel={() => setConfirmDeleteProfile(null)}
          onConfirm={() => {
            const id = confirmDeleteProfile.id;
            setConfirmDeleteProfile(null);
            void handleDelete(id);
          }}
        />
      )}
    </div>
  );
}

function ProfileCard({ profile, isPrimary, onSelect, onDelete, onEdit }: {
  profile: UserProfile; isPrimary: boolean; onSelect: () => void; onDelete: () => void; onEdit: () => void;
}) {
  const [hovered, setHovered] = useState(false);

  return (
    <article style={S.profileCard} onMouseEnter={() => setHovered(true)} onMouseLeave={() => setHovered(false)}>
      <button onClick={onSelect} style={S.profileSelectButton}>
        <div style={S.avatarOuterWrap}>
          <div style={{ ...S.avatarCircleWrap, opacity: hovered ? 1 : 0.85, transform: hovered ? 'scale(1.04)' : 'scale(1)' }}>
            <AvatarPreview profile={profile} size={130} circular />
          </div>
          {profile.pinHash && (
            <span style={S.lockBadge} title={t('profiles.pin_lock')}>
              <Lock size={18} />
            </span>
          )}
        </div>
        <span style={S.profileName}>{profile.name ?? t('auto.profile')}</span>
        {isPrimary && <span style={S.primaryBadge}>{t('profiles.primary_badge')}</span>}
      </button>
      <div style={{ display: 'flex', gap: '0.25rem' }}>
        <button
          style={{ ...S.editPencilBtn, opacity: hovered ? 1 : 0.55 }}
          onClick={(e) => { e.stopPropagation(); onEdit(); }}
          title={t('auto.edit')}
          aria-label={t('profiles.edit')}
        >
          <Pencil size={16} />
        </button>
        <button
          style={{ ...S.editPencilBtn, opacity: hovered ? 1 : 0.55, color: 'rgba(255,80,80,0.8)' }}
          onClick={(e) => { e.stopPropagation(); onDelete(); }}
          title={t('profiles.delete')}
          aria-label={t('profiles.delete')}
        >
          <Trash2 size={16} />
        </button>
      </div>
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
        <div style={{ ...S.addCircle, background: hovered ? 'rgba(255,255,255,0.10)' : 'rgba(255,255,255,0.04)', transform: hovered ? 'scale(1.04)' : 'scale(1)' }}>
          <Plus size={36} color="rgba(255,255,255,0.55)" />
        </div>
        <span style={S.addLabel}>{t('profiles.add_profile')}</span>
      </button>
    </div>
  );
}

const FONT = "'Montserrat', -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif";

const S: Record<string, React.CSSProperties> = {
  root: { position: 'fixed', inset: 0, zIndex: 9999, background: '#0C0C0C', color: colors.white, overflow: 'auto', fontFamily: FONT },
  topBar: { position: 'absolute', top: 0, left: 0, right: 0, height: '4.5rem', display: 'flex', alignItems: 'center', justifyContent: 'space-between', padding: '0 2.75rem', zIndex: 2 },
  logo: { margin: 0, fontSize: '1.375rem', fontWeight: 700, letterSpacing: '0.125rem', fontFamily: FONT },
  kicker: { margin: '0.125rem 0 0', color: 'rgba(255,255,255,0.28)', fontSize: '0.625rem', fontWeight: 500, textTransform: 'uppercase', letterSpacing: '0.09em', fontFamily: FONT },
  main: { minHeight: '100%', width: 'min(68.75rem, calc(100vw - 3.5rem))', margin: '0 auto', padding: '6.875rem 0 3.5rem' },
  mainSelect: { minHeight: '100vh', display: 'flex', flexDirection: 'column', alignItems: 'center', justifyContent: 'center', padding: '5rem 2.5rem 3.75rem' },
  selectTitle: { margin: '0 0 3.25rem', fontSize: '2.375rem', fontWeight: 700, letterSpacing: '-0.03em', textAlign: 'center', fontFamily: FONT },
  hero: { marginBottom: '2rem', maxWidth: '37.5rem' },
  eyebrow: { color: 'rgba(255,255,255,0.40)', fontSize: '0.6875rem', fontWeight: 500, textTransform: 'uppercase', letterSpacing: '0.09em', margin: '0 0 0.875rem', fontFamily: FONT },
  title: { margin: 0, fontSize: '2.5rem', lineHeight: 1.05, fontWeight: 700, letterSpacing: '-0.03em', fontFamily: FONT },
  subtitle: { margin: '0.75rem 0 0', color: 'rgba(255,255,255,0.45)', fontSize: '0.875rem', lineHeight: 1.65, maxWidth: '32.5rem', fontFamily: FONT },
  profileGrid: { display: 'flex', flexDirection: 'row', flexWrap: 'wrap', gap: '2.5rem', alignItems: 'flex-start', justifyContent: 'center' },
  profileCard: { display: 'flex', flexDirection: 'column', alignItems: 'center', gap: 0, background: 'transparent', border: 'none' },
  profileSelectButton: { width: 'auto', border: 'none', background: 'transparent', color: colors.white, display: 'flex', flexDirection: 'column', alignItems: 'center', textAlign: 'center', cursor: 'pointer', padding: 0, outline: 'none' },
  avatarOuterWrap: { position: 'relative', width: '8.125rem', height: '8.125rem', flexShrink: 0 },
  avatarCircleWrap: { borderRadius: '50%', overflow: 'hidden', transition: 'opacity 0.15s, transform 0.15s', width: '8.125rem', height: '8.125rem', flexShrink: 0 },
  lockBadge: { position: 'absolute', right: '0.125rem', bottom: '0.125rem', width: '2.25rem', height: '2.25rem', borderRadius: '50%', background: 'rgba(12,12,12,0.9)', border: '1px solid rgba(255,255,255,0.15)', display: 'flex', alignItems: 'center', justifyContent: 'center', color: 'rgba(255,255,255,0.85)' },
  profileName: { marginTop: '0.875rem', maxWidth: '8.75rem', color: 'rgba(255,255,255,0.85)', fontSize: '0.875rem', fontWeight: 500, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap', fontFamily: FONT },
  primaryBadge: { marginTop: '0.375rem', padding: '0.125rem 0.5rem', borderRadius: '0.25rem', background: 'rgba(255,255,255,0.10)', color: 'rgba(255,255,255,0.55)', fontSize: '0.625rem', fontWeight: 600, textTransform: 'uppercase', letterSpacing: '0.06em' },
  editPencilBtn: { marginTop: '0.5rem', border: 'none', background: 'transparent', color: 'rgba(255,255,255,0.45)', cursor: 'pointer', padding: '0.25rem 0.5rem', display: 'flex', alignItems: 'center', justifyContent: 'center', transition: 'opacity 0.15s', outline: 'none' },
  addCircle: { width: '8.125rem', height: '8.125rem', borderRadius: '50%', display: 'flex', alignItems: 'center', justifyContent: 'center', transition: 'background 0.15s, transform 0.15s', cursor: 'pointer', border: '1px solid rgba(255,255,255,0.10)' },
  addLabel: { marginTop: '0.875rem', color: 'rgba(255,255,255,0.35)', fontSize: '0.875rem', fontWeight: 500, fontFamily: FONT },
  emptyState: { gridColumn: '1 / -1', height: '3.75rem', borderRadius: '0.5rem', border: '1px solid rgba(255,255,255,0.07)', background: 'rgba(255,255,255,0.03)', color: 'rgba(255,255,255,0.4)', display: 'flex', alignItems: 'center', justifyContent: 'center', gap: '0.625rem', fontSize: '0.8125rem', fontFamily: FONT },
  closeButton: { width: '2.25rem', height: '2.25rem', borderRadius: '0.5rem', border: '1px solid rgba(255,255,255,0.10)', background: 'rgba(255,255,255,0.05)', color: 'rgba(255,255,255,0.7)', display: 'flex', alignItems: 'center', justifyContent: 'center', cursor: 'pointer', outline: 'none' },
};
