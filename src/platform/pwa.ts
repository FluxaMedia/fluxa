const base = import.meta.env.BASE_URL || '/';

function isWebApp(): boolean {
  return import.meta.env.VITE_FLUXA_TARGET === 'web';
}

function linkManifest(): void {
  if (document.querySelector('link[rel="manifest"]')) return;
  const link = document.createElement('link');
  link.rel = 'manifest';
  link.href = `${base}manifest.webmanifest`;
  document.head.appendChild(link);
}

export function startPwa(): void {
  if (!isWebApp()) return;
  linkManifest();
  if (!import.meta.env.PROD || !('serviceWorker' in navigator)) return;
  window.addEventListener('load', () => {
    void navigator.serviceWorker.register(`${base}sw.js`, { scope: base }).catch(() => undefined);
  });
}

export function isStandalone(): boolean {
  return (
    window.matchMedia?.('(display-mode: standalone)').matches || (navigator as Navigator & { standalone?: boolean }).standalone === true
  );
}
