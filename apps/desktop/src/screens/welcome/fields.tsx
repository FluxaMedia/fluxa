import { ArrowLeft, Eye, EyeOff } from 'lucide-react';
import { t } from '../../i18n';
import { S, FONT } from './styles';

export function TopBar({ onBack }: { onBack: () => void }) {
  return (
    <div style={S.topBar}>
      <div>
        <p style={S.logo}>fluxa</p>
        <p style={S.kicker}>{t('app.desktop')}</p>
      </div>
      <button style={S.backBtn} onClick={onBack} aria-label={t('common.back')}>
        <ArrowLeft size={18} />
        <span style={{ fontSize: '0.8125rem', fontFamily: FONT }}>{t('common.back')}</span>
      </button>
    </div>
  );
}

export function Field({
  label,
  type,
  value,
  onChange,
  placeholder,
  error,
  autoFocus,
}: {
  label: string;
  type: string;
  value: string;
  onChange: (v: string) => void;
  placeholder?: string;
  error?: string;
  autoFocus?: boolean;
}) {
  return (
    <div style={S.fieldWrap}>
      <label style={S.label}>{label}</label>
      <input
        type={type}
        value={value}
        onChange={(e) => onChange(e.target.value)}
        placeholder={placeholder}
        autoFocus={autoFocus}
        style={{ ...S.input, ...(error ? S.inputError : {}) }}
        autoComplete={type === 'email' ? 'email' : 'name'}
      />
      {error && <p style={S.errorText}>{error}</p>}
    </div>
  );
}

export function PasswordField({
  label,
  value,
  onChange,
  placeholder,
  show,
  onToggleShow,
  error,
}: {
  label: string;
  value: string;
  onChange: (v: string) => void;
  placeholder?: string;
  show: boolean;
  onToggleShow: () => void;
  error?: string;
}) {
  return (
    <div style={S.fieldWrap}>
      <label style={S.label}>{label}</label>
      <div style={S.passwordWrap}>
        <input
          type={show ? 'text' : 'password'}
          value={value}
          onChange={(e) => onChange(e.target.value)}
          placeholder={placeholder}
          style={{ ...S.input, ...S.passwordInput, ...(error ? S.inputError : {}) }}
          autoComplete="current-password"
        />
        <button type="button" style={S.eyeBtn} onClick={onToggleShow} tabIndex={-1}>
          {show ? <EyeOff size={16} /> : <Eye size={16} />}
        </button>
      </div>
      {error && <p style={S.errorText}>{error}</p>}
    </div>
  );
}
