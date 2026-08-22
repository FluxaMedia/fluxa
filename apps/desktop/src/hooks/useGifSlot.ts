import { useEffect, useState } from 'react';

const MAX_CONCURRENT_GIFS = 4;
const active = new Set<string>();
const waiters = new Set<() => void>();

function tryAcquire(id: string): boolean {
  if (active.has(id)) return true;
  if (active.size >= MAX_CONCURRENT_GIFS) return false;
  active.add(id);
  return true;
}

function release(id: string) {
  if (active.delete(id)) waiters.forEach((attempt) => attempt());
}

export function useGifSlot(id: string, want: boolean): boolean {
  const [granted, setGranted] = useState(false);

  useEffect(() => {
    if (!want) return;
    const attempt = () => {
      if (tryAcquire(id)) setGranted(true);
    };
    attempt();
    waiters.add(attempt);
    return () => {
      waiters.delete(attempt);
      release(id);
      setGranted(false);
    };
  }, [id, want]);

  return granted;
}
