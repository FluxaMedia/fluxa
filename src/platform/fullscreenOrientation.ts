type LockableOrientation = ScreenOrientation & { lock?: (orientation: string) => Promise<void> };

function orientation(): LockableOrientation | null {
  return (window.screen?.orientation as LockableOrientation | undefined) ?? null;
}

export async function enterFullscreen(element: HTMLElement, landscape: boolean): Promise<void> {
  const request =
    element.requestFullscreen ?? (element as HTMLElement & { webkitRequestFullscreen?: () => Promise<void> }).webkitRequestFullscreen;
  if (!request) return;
  await request.call(element).catch(() => undefined);
  if (!landscape) return;
  await orientation()
    ?.lock?.('landscape')
    .catch(() => undefined);
}

export async function exitFullscreen(): Promise<void> {
  orientation()?.unlock?.();
  if (document.fullscreenElement) await document.exitFullscreen().catch(() => undefined);
}
