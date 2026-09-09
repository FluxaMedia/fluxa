import React from 'react';
import { loadProfilePickerSettings } from '../core/profileAvatarPacks';

export function FluxaSplashScreen() {
  const [backgroundUrl, setBackgroundUrl] = React.useState<string | null>(null);

  React.useEffect(() => {
    let cancelled = false;
    void loadProfilePickerSettings().then((settings) => {
      if (!cancelled) setBackgroundUrl(settings.backgroundUrl ?? null);
    });
    return () => {
      cancelled = true;
    };
  }, []);

  return (
    <div
      className="fluxa-splash"
      role="status"
      aria-live="polite"
      style={backgroundUrl ? { backgroundImage: `linear-gradient(rgba(12,12,12,0.78), rgba(12,12,12,0.88)), url(${backgroundUrl})` } : undefined}
    >
      <div className="fluxa-splash-brand">
        <img className="fluxa-splash-mark" src="/fluxa.png" alt="" />
        <span className="fluxa-splash-wordmark">fluxa</span>
      </div>
      <span className="fluxa-splash-spinner" aria-hidden="true" />
    </div>
  );
}
