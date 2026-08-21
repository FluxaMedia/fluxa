import React, { useEffect, useRef, useState } from 'react';
import { verifyPin } from '../core/profiles';
import { nuvioVerifyProfilePin } from '../core/nuvioPin';
import type { UserProfile } from '../core/types';
import { t } from '../i18n';

export function PinPrompt({
  profile,
  overridePin,
  onSuccess,
  onCancel,
}: {
  profile: UserProfile;
  overridePin?: UserProfile;
  onSuccess: () => void;
  onCancel: () => void;
}) {
  const [pin, setPin] = useState('');
  const [error, setError] = useState<string | null>(null);
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    setTimeout(() => inputRef.current?.focus(), 30);
  }, []);

  const submit = async (value: string) => {
    const remoteResult = profile.nuvioPinEnabled ? await nuvioVerifyProfilePin(profile, value) : null;
    const local = !profile.nuvioPinEnabled && profile.pinHash ? await verifyPin(profile, value) : false;
    const remote = remoteResult?.unlocked ?? false;
    const overrideResult = overridePin?.nuvioPinEnabled ? await nuvioVerifyProfilePin(overridePin, value) : null;
    const override =
      overridePin &&
      (overridePin.nuvioPinEnabled
        ? (overrideResult?.unlocked ?? false)
        : overridePin.pinHash
          ? await verifyPin(overridePin, value)
          : false);
    if (local || remote || override) {
      onSuccess();
    } else {
      const retry = remoteResult?.retry_after_seconds ?? overrideResult?.retry_after_seconds ?? 0;
      setError(retry > 0 ? `Locked. Try again in ${retry}s.` : 'Incorrect PIN');
      setPin('');
    }
  };

  return (
    <div style={S.overlay} onClick={onCancel}>
      <div style={S.dialog} onClick={(e) => e.stopPropagation()}>
        <p style={S.title}>{t('profiles.enter_pin')}</p>
        <p style={S.subtitle}>{profile.name}</p>
        <input
          ref={inputRef}
          type="password"
          inputMode="numeric"
          maxLength={4}
          value={pin}
          onChange={(e) => {
            const next = e.target.value.replace(/\D/g, '').slice(0, 4);
            setPin(next);
            setError(null);
            if (next.length === 4) void submit(next);
          }}
          onKeyDown={(e) => {
            if (e.key === 'Escape') onCancel();
          }}
          style={S.input}
        />
        {error && <p style={S.error}>{error}</p>}
      </div>
    </div>
  );
}

const FONT = "'Montserrat', -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif";

const S: Record<string, React.CSSProperties> = {
  overlay: {
    position: 'fixed',
    inset: 0,
    zIndex: 10000,
    background: 'var(--fluxa-scrim)',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
  },
  dialog: {
    width: '17.5rem',
    borderRadius: '0.75rem',
    background: 'var(--fluxa-surface-raised)',
    border: '1px solid var(--fluxa-border)',
    padding: '1.75rem 1.5rem',
    textAlign: 'center',
    fontFamily: FONT,
  },
  title: { margin: 0, color: 'var(--fluxa-text-primary)', fontSize: '1rem', fontWeight: 700 },
  subtitle: { margin: '0.25rem 0 1.125rem', color: 'var(--fluxa-text-muted)', fontSize: '0.8125rem' },
  input: {
    width: '100%',
    height: '3rem',
    borderRadius: '0.5rem',
    background: 'var(--fluxa-border)',
    border: '1px solid var(--fluxa-border)',
    color: 'var(--fluxa-text-primary)',
    textAlign: 'center',
    fontSize: '1.5rem',
    letterSpacing: '0.75rem',
    outline: 'none',
    boxSizing: 'border-box',
  },
  error: { margin: '0.625rem 0 0', color: '#FF8A8A', fontSize: '0.75rem' },
};
