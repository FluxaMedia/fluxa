import { useState } from 'react';
import { t } from '../../i18n';
import { nuvioAuthErrorKind, nuvioSignIn, type NuvioSession } from '../../core/nuvioApi';
import type { UserProfile } from '../../core/types';
import { S, FONT } from './styles';
import { TopBar, Field, PasswordField } from './fields';
import { assetUrl } from '../../platform/assets';

function buildNuvioProfile(session: NuvioSession, email: string): UserProfile {
  const id = typeof crypto !== 'undefined' && typeof crypto.randomUUID === 'function'
    ? crypto.randomUUID()
    : `${Date.now()}-${Math.floor(Math.random() * 1e9)}`;
  const expiresAt = Math.floor(Date.now() / 1000) + (session.expires_in ?? 3600);
  return {
    id,
    name: '',
    email,
    nuvioAccessToken: session.access_token,
    nuvioRefreshToken: session.refresh_token,
    nuvioTokenExpiresAt: expiresAt,
    nuvioUserId: session.user?.id,
    nuvioEmail: email,
    nuvioProfileIndex: 1,
  };
}

interface NuvioLoginViewProps {
  onBack: () => void;
  onImporting: (profile: UserProfile) => void;
  onContinueLocal: () => Promise<void>;
  localLoading: boolean;
}

export function NuvioLoginView({ onBack, onImporting, onContinueLocal, localLoading }: NuvioLoginViewProps) {
  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');
  const [showPassword, setShowPassword] = useState(false);
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState('');

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!email.trim() || !password) {
      setError(t('auth.error.fill_required'));
      return;
    }
    setError('');
    setSubmitting(true);
    try {
      const session = await nuvioSignIn(email.trim(), password);
      const profile = buildNuvioProfile(session, email.trim());
      onImporting(profile);
    } catch (err) {
      switch (nuvioAuthErrorKind(err)) {
        case 'invalid_credentials':
          setError(t('auth.error.invalid_credentials'));
          break;
        case 'account_exists':
          setError(t('auth.error.account_exists'));
          break;
        case 'email_not_confirmed':
          setError(t('auth.error.email_not_confirmed'));
          break;
        case 'rate_limited':
          setError(t('auth.error.rate_limited'));
          break;
        case 'server':
          setError(t('auth.error.server'));
          break;
        case 'network': {
          const detail = err instanceof Error && err.message ? err.message : '';
          setError(detail ? `${t('auth.error.network')} (${detail})` : t('auth.error.network'));
          break;
        }
        default:
          setError(err instanceof Error ? err.message : t('auth.error.network'));
          break;
      }
      setSubmitting(false);
    }
  };

  return (
    <div style={S.root}>
      <TopBar onBack={onBack} />

      <main style={S.authMain}>
        <div style={S.card}>
          <div style={{ display: 'flex', alignItems: 'center', gap: '0.75rem', marginBottom: '1.75rem' }}>
            <img
              src={assetUrl('nuvio.png')}
              alt="Nuvio"
              style={{ width: '2.25rem', height: '2.25rem', objectFit: 'contain' }}
            />
            <div>
              <p style={{ margin: 0, fontSize: '1rem', fontWeight: 600, fontFamily: FONT }}>{t('auth.nuvio.title')}</p>
              <p style={{ margin: '0.125rem 0 0', fontSize: '0.75rem', color: 'rgba(255,255,255,0.40)', fontFamily: FONT }}>{t('auth.nuvio.subtitle')}</p>
            </div>
          </div>

          {error && <p style={S.globalError}>{error}</p>}

          <form onSubmit={handleSubmit} noValidate style={S.form}>
            <Field
              label={t('auth.field.email')}
              type="email"
              value={email}
              onChange={setEmail}
              placeholder={t('auth.placeholder.email')}
              autoFocus
            />
            <PasswordField
              label={t('auth.field.password')}
              value={password}
              onChange={setPassword}
              placeholder={t('auth.placeholder.password_login')}
              show={showPassword}
              onToggleShow={() => setShowPassword((v) => !v)}
            />
            <button
              type="submit"
              style={{ ...S.submitBtn, marginTop: '0.5rem', opacity: submitting ? 0.6 : 1 }}
              disabled={submitting}
            >
              {submitting ? t('auth.nuvio.signing_in') : t('auth.nuvio.sign_in')}
            </button>
          </form>

          <div style={S.divider}>
            <span style={S.dividerLine} />
            <span style={S.dividerText}>{t('auth.or')}</span>
            <span style={S.dividerLine} />
          </div>

          <button
            style={{ ...S.localBtn, opacity: localLoading ? 0.4 : 1 }}
            onClick={onContinueLocal}
            disabled={localLoading || submitting}
          >
            {localLoading ? t('welcome.loading') : t('welcome.continue_local')}
          </button>
        </div>
      </main>
    </div>
  );
}
