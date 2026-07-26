import React, { useRef, useState } from 'react';
import { ImagePlus, Plus, Trash2 } from 'lucide-react';
import {
  discoverProfileAvatarPacks,
  type ProfilePickerSettings,
  saveProfilePickerSettings,
} from '../core/profileAvatarPacks';
import { t } from '../i18n';

export function ProfilePickerSettings({
  settings,
  onSaved,
  onBack,
}: {
  settings: ProfilePickerSettings;
  onSaved: (settings: ProfilePickerSettings) => void;
  onBack: () => void;
}) {
  const [repositoryUrl, setRepositoryUrl] = useState('');
  const [backgroundUrl, setBackgroundUrl] = useState(settings.backgroundUrl ?? '');
  const [packs, setPacks] = useState(settings.avatarPacks);
  const [busy, setBusy] = useState(false);
  const fileInputRef = useRef<HTMLInputElement>(null);

  const save = async (next: ProfilePickerSettings) => {
    await saveProfilePickerSettings(next);
    onSaved(next);
  };

  const addRepository = async () => {
    if (!repositoryUrl.trim() || busy) return;
    setBusy(true);
    try {
      const discovered = await discoverProfileAvatarPacks(repositoryUrl.trim());
      const existing = new Set(packs.map((pack) => pack.id));
      const nextPacks = [...packs, ...discovered.filter((pack) => !existing.has(pack.id))];
      setPacks(nextPacks);
      setRepositoryUrl('');
      await save({ backgroundUrl: backgroundUrl || undefined, avatarPacks: nextPacks });
    } finally {
      setBusy(false);
    }
  };

  const updateBackground = async (value: string) => {
    setBackgroundUrl(value);
    await save({ backgroundUrl: value || undefined, avatarPacks: packs });
  };

  const uploadBackground = (event: React.ChangeEvent<HTMLInputElement>) => {
    const file = event.target.files?.[0];
    if (!file) return;
    const reader = new FileReader();
    reader.onload = () => void updateBackground(String(reader.result ?? ''));
    reader.readAsDataURL(file);
    event.target.value = '';
  };

  return (
    <section style={S.shell}>
      <div style={S.section}>
        <h2 style={S.heading}>{t('profiles.picker_background')}</h2>
        <p style={S.copy}>{t('profiles.picker_background_desc')}</p>
        <div style={S.row}>
          <input value={backgroundUrl} onChange={(event) => setBackgroundUrl(event.target.value)} onBlur={() => void updateBackground(backgroundUrl)} placeholder={t('profiles.picker_background_placeholder')} style={S.input} />
          <button onClick={() => fileInputRef.current?.click()} style={S.secondaryButton} title={t('profiles.choose_image')}><ImagePlus size={16} /></button>
          {backgroundUrl && <button onClick={() => void updateBackground('')} style={S.secondaryButton} title={t('profiles.clear_background')}><Trash2 size={16} /></button>}
        </div>
      </div>
      <div style={S.section}>
        <h2 style={S.heading}>{t('profiles.avatar_packs')}</h2>
        <p style={S.copy}>{t('profiles.avatar_packs_desc')}</p>
        <div style={S.row}>
          <input value={repositoryUrl} onChange={(event) => setRepositoryUrl(event.target.value)} onKeyDown={(event) => { if (event.key === 'Enter') void addRepository(); }} placeholder={t('profiles.avatar_pack_repository_placeholder')} style={S.input} />
          <button onClick={() => void addRepository()} disabled={!repositoryUrl.trim() || busy} style={S.primaryButton}><Plus size={16} />{busy ? t('common.loading') : t('profiles.add_pack')}</button>
        </div>
        <div style={S.packList}>
          {packs.length === 0 ? <p style={S.empty}>{t('profiles.no_avatar_packs')}</p> : packs.map((pack) => (
            <div key={pack.id} style={S.packRow}>
              <div style={S.packSummary}>
                <div style={S.packPreview}>
                  {pack.avatars.slice(0, 6).map((avatar) => <img key={avatar.url} src={avatar.url} alt="" style={S.packPreviewImage} />)}
                </div>
                <span>{pack.title} · {t('profiles.avatar_pack_count', pack.avatars.length)}</span>
              </div>
              <button onClick={() => { const nextPacks = packs.filter((candidate) => candidate.id !== pack.id); setPacks(nextPacks); void save({ backgroundUrl: backgroundUrl || undefined, avatarPacks: nextPacks }); }} style={S.removeButton} title={t('profiles.remove_pack')}><Trash2 size={16} /></button>
            </div>
          ))}
        </div>
      </div>
      <button onClick={onBack} style={S.backButton}>{t('common.back')}</button>
      <input ref={fileInputRef} type="file" accept="image/*" style={{ display: 'none' }} onChange={uploadBackground} />
    </section>
  );
}

const S: Record<string, React.CSSProperties> = {
  shell: { display: 'grid', gap: '1.5rem', maxWidth: '43rem' },
  section: { padding: '1.25rem', border: '1px solid rgba(255,255,255,0.09)', borderRadius: '0.75rem', background: 'rgba(255,255,255,0.03)' },
  heading: { margin: 0, fontSize: '1rem' },
  copy: { margin: '0.5rem 0 1rem', color: 'rgba(255,255,255,0.48)', fontSize: '0.8125rem' },
  row: { display: 'flex', gap: '0.5rem' },
  input: { minWidth: 0, flex: 1, border: '1px solid rgba(255,255,255,0.12)', borderRadius: '0.5rem', background: '#0C0C0C', color: '#fff', padding: '0.625rem 0.75rem' },
  primaryButton: { border: 0, borderRadius: '0.5rem', background: '#fff', color: '#000', display: 'flex', alignItems: 'center', gap: '0.375rem', padding: '0 0.75rem', cursor: 'pointer', whiteSpace: 'nowrap' },
  secondaryButton: { border: '1px solid rgba(255,255,255,0.12)', borderRadius: '0.5rem', background: 'rgba(255,255,255,0.05)', color: '#fff', padding: '0.625rem', cursor: 'pointer' },
  packList: { display: 'grid', gap: '0.5rem', marginTop: '1rem' },
  packRow: { display: 'flex', justifyContent: 'space-between', alignItems: 'center', color: 'rgba(255,255,255,0.8)', fontSize: '0.8125rem', padding: '0.625rem 0.75rem', background: 'rgba(255,255,255,0.04)', borderRadius: '0.5rem' },
  packSummary: { minWidth: 0, display: 'flex', alignItems: 'center', gap: '0.75rem' },
  packPreview: { display: 'flex', overflow: 'hidden', borderRadius: '0.375rem', flexShrink: 0 },
  packPreviewImage: { width: '2.25rem', height: '2.25rem', objectFit: 'cover', marginLeft: '-0.375rem', border: '1px solid #202020' },
  removeButton: { border: 0, background: 'transparent', color: '#d34a4a', cursor: 'pointer', display: 'flex' },
  empty: { margin: 0, color: 'rgba(255,255,255,0.4)', fontSize: '0.8125rem' },
  backButton: { justifySelf: 'start', border: 0, background: 'transparent', color: '#fff', cursor: 'pointer', padding: '0.5rem 0' },
};
