import { useState } from 'react';
import { t } from '../../i18n';
import { createProfileObject, saveProfile, setActiveProfileId } from '../../core/profiles';
import type { UserProfile } from '../../core/types';
import { S } from './styles';
import { TopBar, Field } from './fields';

interface ProfileSetupViewProps {
  onBack: () => void;
  onDone: (profile: UserProfile) => Promise<void>;
}

export function ProfileSetupView({ onBack, onDone }: ProfileSetupViewProps) {
  const [name, setName] = useState('');
  const [submitting, setSubmitting] = useState(false);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setSubmitting(true);
    const profile = await createProfileObject(name.trim() || t('auth.profile.default_name'), '#FFFFFF');
    await saveProfile(profile);
    await setActiveProfileId(profile.id);
    await onDone(profile);
  };

  return (
    <div style={S.root}>
      <TopBar onBack={onBack} />

      <main style={S.authMain}>
        <div style={S.card}>
          <p style={S.eyebrow}>{t('auth.profile.eyebrow')}</p>
          <h2 style={S.cardTitle}>{t('auth.profile.title')}</h2>
          <p style={S.cardSubtitle}>{t('auth.profile.subtitle')}</p>

          <form onSubmit={handleSubmit} noValidate style={{ ...S.form, marginTop: '1.75rem' }}>
            <Field
              label={t('auth.profile.name_label')}
              type="text"
              value={name}
              onChange={setName}
              placeholder={t('auth.profile.name_placeholder')}
              autoFocus
            />
            <button type="submit" style={{ ...S.submitBtn, marginTop: '0.5rem', opacity: submitting ? 0.6 : 1 }} disabled={submitting}>
              {submitting ? t('welcome.loading') : t('auth.profile.continue')}
            </button>
          </form>
        </div>
      </main>
    </div>
  );
}
