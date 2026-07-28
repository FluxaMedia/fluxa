import React, { useEffect, useRef, useState } from 'react';
import { verifyPin } from '../core/profiles';
import type { UserProfile } from '../core/types';
import { t } from '../i18n';

export function PinPrompt({ profile, overridePin, onSuccess, onCancel }: {
  profile: UserProfile;
  overridePin?: UserProfile;
  onSuccess: () => void;
  onCancel: () => void;
}) {
  const [pin, setPin] = useState('');
  const [error, setError] = useState(false);
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => { setTimeout(() => inputRef.current?.focus(), 30); }, []);

  const submit = async (value: string) => {
    if (await verifyPin(profile, value) || (overridePin && await verifyPin(overridePin, value))) {
      onSuccess();
    } else {
      setError(true);
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
            setError(false);
            if (next.length === 4) void submit(next);
          }}
          onKeyDown={(e) => { if (e.key === 'Escape') onCancel(); }}
          style={S.input}
        />
        {error && <p style={S.error}>{t('profiles.pin_incorrect')}</p>}
      </div>
    </div>
  );
}

const FONT = "'Montserrat', -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif";

const S: Record<string, React.CSSProperties> = {
  overlay: { position: 'fixed', inset: 0, zIndex: 10000, background: 'rgba(0,0,0,0.7)', display: 'flex', alignItems: 'center', justifyContent: 'center' },
  dialog: { width: '17.5rem', borderRadius: '0.75rem', background: '#141414', border: '1px solid rgba(255,255,255,0.10)', padding: '1.75rem 1.5rem', textAlign: 'center', fontFamily: FONT },
  title: { margin: 0, color: '#FFFFFF', fontSize: '1rem', fontWeight: 700 },
  subtitle: { margin: '0.25rem 0 1.125rem', color: 'rgba(255,255,255,0.45)', fontSize: '0.8125rem' },
  input: { width: '100%', height: '3rem', borderRadius: '0.5rem', background: 'rgba(255,255,255,0.05)', border: '1px solid rgba(255,255,255,0.10)', color: '#FFFFFF', textAlign: 'center', fontSize: '1.5rem', letterSpacing: '0.75rem', outline: 'none', boxSizing: 'border-box' },
  error: { margin: '0.625rem 0 0', color: '#FF8A8A', fontSize: '0.75rem' },
};
