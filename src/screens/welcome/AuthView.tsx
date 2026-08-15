import { useState } from 'react';
import { t } from '../../i18n';
import { assetUrl } from '../../platform/assets';
import { S } from './styles';
import { TopBar, Field, PasswordField } from './fields';

type AuthTab = 'login' | 'signup';

interface AuthViewProps {
  tab: AuthTab;
  onTabChange: (t: AuthTab) => void;
  onBack: () => void;
  onSubmit: () => void;
  onNuvioClick: () => void;
  onContinueLocal: () => Promise<void>;
  localLoading: boolean;
}

export function AuthView({ tab, onTabChange, onBack, onSubmit, onNuvioClick, onContinueLocal, localLoading }: AuthViewProps) {
  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');
  const [confirmPassword, setConfirmPassword] = useState('');
  const [showPassword, setShowPassword] = useState(false);
  const [showConfirm, setShowConfirm] = useState(false);
  const [submitting, setSubmitting] = useState(false);
  const [errors, setErrors] = useState<Record<string, string>>({});

  const validate = (): boolean => {
    const next: Record<string, string> = {};
    if (!email.trim()) next.email = t('auth.error.email_required');
    else if (!/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(email)) next.email = t('auth.error.email_invalid');
    if (!password) next.password = t('auth.error.password_required');
    else if (password.length < 8) next.password = t('auth.error.password_too_short');
    if (tab === 'signup' && password !== confirmPassword) next.confirmPassword = t('auth.error.passwords_mismatch');
    setErrors(next);
    return Object.keys(next).length === 0;
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!validate()) return;
    setSubmitting(true);
    await new Promise((r) => setTimeout(r, 600));
    onSubmit();
  };

  const handleTabChange = (next: AuthTab) => {
    onTabChange(next);
    setErrors({});
    setPassword('');
    setConfirmPassword('');
    setShowPassword(false);
    setShowConfirm(false);
  };

  return (
    <div style={S.root}>
      <TopBar onBack={onBack} />

      <main style={S.authMain}>
        <div style={S.card}>
          <button style={S.nuvioBtn} onClick={onNuvioClick}>
            <img
              src={assetUrl('nuvio.png')}
              alt="Nuvio"
              style={{ width: '1.375rem', height: '1.375rem', objectFit: 'contain', flexShrink: 0 }}
            />
            <span>{t('auth.continue_with_nuvio')}</span>
          </button>

          <div style={S.divider}>
            <span style={S.dividerLine} />
            <span style={S.dividerText}>{t('auth.or')}</span>
            <span style={S.dividerLine} />
          </div>

          <div style={S.tabs}>
            <button
              style={{ ...S.tabBtn, ...(tab === 'login' ? S.tabBtnActive : {}) }}
              onClick={() => handleTabChange('login')}
            >
              {t('auth.log_in')}
            </button>
            <button
              style={{ ...S.tabBtn, ...(tab === 'signup' ? S.tabBtnActive : {}) }}
              onClick={() => handleTabChange('signup')}
            >
              {t('auth.sign_up')}
            </button>
          </div>

          <form onSubmit={handleSubmit} noValidate style={S.form}>
            <Field
              label={t('auth.field.email')}
              type="email"
              value={email}
              onChange={setEmail}
              placeholder={t('auth.placeholder.email')}
              error={errors.email}
              autoFocus
            />
            <PasswordField
              label={t('auth.field.password')}
              value={password}
              onChange={setPassword}
              placeholder={tab === 'login' ? t('auth.placeholder.password_login') : t('auth.placeholder.password_signup')}
              show={showPassword}
              onToggleShow={() => setShowPassword((v) => !v)}
              error={errors.password}
            />
            {tab === 'signup' && (
              <PasswordField
                label={t('auth.field.confirm_password')}
                value={confirmPassword}
                onChange={setConfirmPassword}
                placeholder={t('auth.placeholder.confirm_password')}
                show={showConfirm}
                onToggleShow={() => setShowConfirm((v) => !v)}
                error={errors.confirmPassword}
              />
            )}

            {tab === 'login' && (
              <div style={{ textAlign: 'right', marginTop: '-0.25rem' }}>
                <button type="button" style={S.forgotBtn}>
                  {t('auth.forgot_password')}
                </button>
              </div>
            )}

            <button
              type="submit"
              style={{ ...S.submitBtn, opacity: submitting ? 0.6 : 1 }}
              disabled={submitting}
            >
              {submitting
                ? t('welcome.loading')
                : tab === 'login'
                  ? t('auth.log_in')
                  : t('auth.create_account')}
            </button>
          </form>

          <button
            style={{ ...S.localBtn, marginTop: '1.25rem', opacity: localLoading ? 0.4 : 1 }}
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
