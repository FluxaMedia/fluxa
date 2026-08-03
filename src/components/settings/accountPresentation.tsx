import React, { useEffect, useState } from 'react';
import { t } from '../../i18n';
import { nuvioAuthErrorKind } from '../../core/nuvioApi';

const PROVIDER_ICON: Record<string, { src: string; alt: string; background: string }> = {
  trakt: { src: '/trakt.svg', alt: 'Trakt', background: 'rgba(237,28,36,0.12)' },
  simkl: { src: '/simkl.svg', alt: 'Simkl', background: 'rgba(28,177,74,0.12)' },
  anilist: { src: '/anilist.svg', alt: 'AniList', background: 'rgba(2,169,255,0.12)' },
  stremio: { src: '/stremio.svg', alt: 'Stremio', background: 'rgba(123,91,245,0.12)' },
  nuvio: { src: '/nuvio.png', alt: 'Nuvio', background: 'rgba(255,255,255,0.06)' },
};

export function providerIcon(key: string): React.ReactNode {
  const icon = PROVIDER_ICON[key];
  if (!icon) return null;
  return (
    <div style={{ width: '1.5rem', height: '1.5rem', borderRadius: '0.375rem', background: icon.background, display: 'flex', alignItems: 'center', justifyContent: 'center', overflow: 'hidden', flexShrink: 0 }}>
      <img src={icon.src} alt={icon.alt} style={{ width: '1.0625rem', height: '1.0625rem', objectFit: 'contain' }} />
    </div>
  );
}

export function generateCodeVerifier(): string {
  const array = new Uint8Array(48);
  crypto.getRandomValues(array);
  return btoa(String.fromCharCode(...array))
    .replace(/\+/g, '-')
    .replace(/\//g, '_')
    .replace(/=/g, '')
    .slice(0, 64);
}

export async function codeChallenge(verifier: string): Promise<string> {
  const digest = await crypto.subtle.digest('SHA-256', new TextEncoder().encode(verifier));
  return btoa(String.fromCharCode(...new Uint8Array(digest)))
    .replace(/\+/g, '-')
    .replace(/\//g, '_')
    .replace(/=/g, '');
}

export interface OAuthCodePayload {
  code: string;
  state: string | null;
}

export type OAuthService = 'trakt' | 'anilist' | 'simkl';
export type IntegrationService = OAuthService | 'nuvio' | 'stremio';

function tokenRefreshCountdown(seconds: number): string {
  const total = Math.max(0, Math.floor(seconds));
  const hours = Math.floor(total / 3600);
  const minutes = Math.floor((total % 3600) / 60);
  const remainingSeconds = total % 60;
  return hours > 0
    ? t('format.duration_hours_minutes_seconds', hours, minutes, remainingSeconds)
    : t('format.duration_minutes_seconds', minutes, remainingSeconds);
}

export function ProviderTokenStatus({ expiresAt, verified, refreshScheduled }: { expiresAt?: number; verified: boolean; refreshScheduled: boolean }) {
  const [now, setNow] = useState(() => Math.floor(Date.now() / 1000));

  useEffect(() => {
    const timer = window.setInterval(() => setNow(Math.floor(Date.now() / 1000)), 1000);
    return () => window.clearInterval(timer);
  }, []);

  const refreshAt = expiresAt ? expiresAt - 60 : undefined;
  const expired = expiresAt != null && expiresAt <= now;
  const status = expired
    ? t('settings.token_status_expired')
    : verified
    ? t('settings.token_status_verified')
    : t('sync.device.connected');
  const refresh = !refreshScheduled || refreshAt == null
    ? t('settings.token_refresh_not_scheduled')
    : refreshAt <= now
    ? t('settings.token_refresh_due')
    : t('settings.token_refresh_in', tokenRefreshCountdown(refreshAt - now));

  return (
    <div style={{ display: 'grid', gap: '0.25rem', padding: '0 1.125rem 0.75rem', color: 'rgba(255,255,255,0.66)', fontSize: '0.75rem' }}>
      <span>{t('settings.token_status', status)}</span>
      <span>{refresh}</span>
    </div>
  );
}

export function credentialAuthErrorMessage(err: unknown): string {
  switch (nuvioAuthErrorKind(err)) {
    case 'invalid_credentials':
      return t('auth.error.invalid_credentials');
    case 'email_not_confirmed':
      return t('auth.error.email_not_confirmed');
    case 'rate_limited':
      return t('auth.error.rate_limited');
    case 'server':
      return t('auth.error.server');
    case 'network': {
      const detail = err instanceof Error && err.message ? err.message : '';
      return detail ? `${t('auth.error.network')} (${detail})` : t('auth.error.network');
    }
    default:
      return err instanceof Error && err.message ? err.message : t('auth.error.network');
  }
}

export function CredentialLoginForm({
  busy,
  onSubmit,
  onCancel,
}: {
  busy: boolean;
  onSubmit: (email: string, password: string) => void;
  onCancel: () => void;
}) {
  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');
  const canSubmit = !busy && email.trim().length > 0 && password.length > 0;
  const input: React.CSSProperties = {
    height: '1.875rem',
    borderRadius: '0.4375rem',
    border: '1px solid rgba(255,255,255,0.12)',
    background: 'rgba(255,255,255,0.05)',
    color: '#fff',
    fontSize: '0.7813rem',
    padding: '0 0.625rem',
    outline: 'none',
    flex: 1,
    minWidth: 0,
  };
  const btn: React.CSSProperties = {
    height: '1.875rem',
    borderRadius: '0.4375rem',
    border: '1px solid rgba(255,255,255,0.12)',
    background: 'rgba(255,255,255,0.06)',
    color: '#fff',
    fontSize: '0.75rem',
    fontWeight: 500,
    cursor: 'pointer',
    padding: '0 0.75rem',
    whiteSpace: 'nowrap',
  };
  return (
    <div style={{ padding: '0 1.125rem 0.75rem', borderBottom: '1px solid rgba(255,255,255,0.055)', display: 'flex', gap: '0.5rem', alignItems: 'center' }}>
      <input
        type="email"
        placeholder={t('auth.placeholder.email')}
        value={email}
        onChange={(e) => setEmail(e.target.value)}
        style={input}
        autoFocus
      />
      <input
        type="password"
        placeholder={t('auth.placeholder.password_login')}
        value={password}
        onChange={(e) => setPassword(e.target.value)}
        onKeyDown={(e) => { if (e.key === 'Enter' && canSubmit) onSubmit(email.trim(), password); }}
        style={input}
      />
      <button disabled={!canSubmit} onClick={() => onSubmit(email.trim(), password)} style={{ ...btn, opacity: canSubmit ? 1 : 0.5 }}>
        {busy ? t('auth.signing_in') : t('auth.sign_in')}
      </button>
      <button onClick={onCancel} disabled={busy} style={btn}>{t('common.cancel')}</button>
    </div>
  );
}

export function AuthKeyLoginForm({
  busy,
  onSubmit,
  onCancel,
}: {
  busy: boolean;
  onSubmit: (authKey: string) => void;
  onCancel: () => void;
}) {
  const [authKey, setAuthKey] = useState('');
  const canSubmit = !busy && authKey.trim().length > 0;
  const input: React.CSSProperties = {
    height: '1.875rem',
    borderRadius: '0.4375rem',
    border: '1px solid rgba(255,255,255,0.12)',
    background: 'rgba(255,255,255,0.05)',
    color: '#fff',
    fontSize: '0.7813rem',
    padding: '0 0.625rem',
    outline: 'none',
    flex: 1,
    minWidth: 0,
  };
  const btn: React.CSSProperties = {
    height: '1.875rem',
    borderRadius: '0.4375rem',
    border: '1px solid rgba(255,255,255,0.12)',
    background: 'rgba(255,255,255,0.06)',
    color: '#fff',
    fontSize: '0.75rem',
    fontWeight: 500,
    cursor: 'pointer',
    padding: '0 0.75rem',
    whiteSpace: 'nowrap',
  };
  return (
    <div style={{ padding: '0 1.125rem 0.75rem', borderBottom: '1px solid rgba(255,255,255,0.055)', display: 'flex', flexDirection: 'column', gap: '0.5rem' }}>
      <p style={{ margin: 0, fontSize: '0.6875rem', color: 'rgba(255,255,255,0.55)' }}>{t('auth.stremio.authkey_hint')}</p>
      <div style={{ display: 'flex', gap: '0.5rem', alignItems: 'center' }}>
        <input
          type="text"
          placeholder={t('auth.placeholder.stremio_authkey')}
          value={authKey}
          onChange={(e) => setAuthKey(e.target.value)}
          onKeyDown={(e) => { if (e.key === 'Enter' && canSubmit) onSubmit(authKey.trim()); }}
          style={input}
          autoFocus
        />
        <button disabled={!canSubmit} onClick={() => onSubmit(authKey.trim())} style={{ ...btn, opacity: canSubmit ? 1 : 0.5 }}>
          {busy ? t('auth.signing_in') : t('auth.sign_in')}
        </button>
        <button onClick={onCancel} disabled={busy} style={btn}>{t('common.cancel')}</button>
      </div>
    </div>
  );
}
