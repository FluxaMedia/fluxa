const NAV_SELECTOR = '[role="button"][tabindex]';

type Direction = 'up' | 'down' | 'left' | 'right';

function isVisible(el: HTMLElement): boolean {
  const rect = el.getBoundingClientRect();
  return rect.width > 0 && rect.height > 0;
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
