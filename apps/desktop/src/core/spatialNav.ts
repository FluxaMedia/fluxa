const NAV_SELECTOR = [
  'button:not([disabled]):not([tabindex="-1"])',
  '[role="button"][tabindex]:not([tabindex="-1"]):not([aria-disabled="true"])',
  'a[href]:not([tabindex="-1"])',
  'input:not([disabled]):not([type="hidden"]):not([tabindex="-1"])',
  'select:not([disabled]):not([tabindex="-1"])',
  'textarea:not([disabled]):not([tabindex="-1"])',
  '[tabindex]:not([tabindex="-1"]):not([aria-hidden="true"])',
].join(',');

type Direction = 'up' | 'down' | 'left' | 'right';

function isVisible(el: HTMLElement): boolean {
  const rect = el.getBoundingClientRect();
  const style = window.getComputedStyle(el);
  return (
    rect.width > 0 &&
    rect.height > 0 &&
    style.display !== 'none' &&
    style.visibility !== 'hidden' &&
    !el.closest('[aria-hidden="true"], [inert]')
  );
}

export function focusNearestCard(current: HTMLElement, direction: Direction): boolean {
  const candidates = Array.from(document.querySelectorAll<HTMLElement>(NAV_SELECTOR)).filter((el) => el !== current && isVisible(el));
  const cr = current.getBoundingClientRect();
  const cx = cr.left + cr.width / 2;
  const cy = cr.top + cr.height / 2;

  let best: HTMLElement | null = null;
  let bestScore = Infinity;

  for (const el of candidates) {
    const r = el.getBoundingClientRect();
    const ex = r.left + r.width / 2;
    const ey = r.top + r.height / 2;
    const dx = ex - cx;
    const dy = ey - cy;
    let primary: number;
    let secondary: number;

    if (direction === 'left') {
      if (dx >= -1) continue;
      primary = -dx;
      secondary = Math.abs(dy);
    } else if (direction === 'right') {
      if (dx <= 1) continue;
      primary = dx;
      secondary = Math.abs(dy);
    } else if (direction === 'up') {
      if (dy >= -1) continue;
      primary = -dy;
      secondary = Math.abs(dx);
    } else {
      if (dy <= 1) continue;
      primary = dy;
      secondary = Math.abs(dx);
    }

    const score = primary + secondary * 2;
    if (score < bestScore) {
      bestScore = score;
      best = el;
    }
  }

  if (!best) return false;
  best.scrollIntoView({ block: 'nearest', inline: 'nearest' });
  best.focus();
  return true;
}

export function isNavCard(el: Element | null): el is HTMLElement {
  return !!el && el instanceof HTMLElement && el.matches(NAV_SELECTOR);
}

export function focusFirstSpatialTarget(): boolean {
  const content = document.querySelector<HTMLElement>('.app-content');
  const candidates = Array.from((content ?? document).querySelectorAll<HTMLElement>(NAV_SELECTOR)).filter(isVisible);
  const target = candidates[0] ?? (content ? undefined : Array.from(document.querySelectorAll<HTMLElement>(NAV_SELECTOR)).find(isVisible));
  if (!target) return false;
  target.focus();
  return true;
}
