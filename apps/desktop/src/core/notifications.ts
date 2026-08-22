import { isBrowserTarget } from '../platform/browser';

let permissionGranted: boolean | null = null;

async function ensurePermission(): Promise<boolean> {
  if (isBrowserTarget()) {
    if (!('Notification' in window)) return false;
    if (Notification.permission === 'granted') return true;
    if (Notification.permission === 'denied') return false;
    return (await Notification.requestPermission()) === 'granted';
  }
  if (permissionGranted !== null) return permissionGranted;
  const { isPermissionGranted, requestPermission } = await import('@tauri-apps/plugin-notification');
  permissionGranted = await isPermissionGranted();
  if (!permissionGranted) permissionGranted = (await requestPermission()) === 'granted';
  return permissionGranted;
}

export async function notify(title: string, body?: string): Promise<void> {
  if (!(await ensurePermission().catch(() => false))) return;
  if (isBrowserTarget()) new Notification(title, body ? { body } : undefined);
  else (await import('@tauri-apps/plugin-notification')).sendNotification({ title, body });
}
