import { useEffect, useState } from 'react';
import { platformConvertFileSrc, platformInvoke as invoke } from '../platform/invoke';

const resolved = new Map<string, Promise<string>>();

function resolveLocalSrc(url: string): Promise<string> {
  if (import.meta.env.VITE_FLUXA_TARGET === 'web' || import.meta.env.VITE_FLUXA_TARGET === 'webos') {
    return Promise.resolve(url);
  }
  let promise = resolved.get(url);
  if (!promise) {
    promise = invoke<string>('cache_poster_image', { url }).then(platformConvertFileSrc);
    promise.catch(() => resolved.delete(url));
    resolved.set(url, promise);
  }
  return promise;
}

export function usePosterSrc(url: string | null | undefined): { src?: string; failed: boolean } {
  const [state, setState] = useState<{ src?: string; failed: boolean }>({ failed: false });

  useEffect(() => {
    if (!url) {
      setState({ failed: false });
      return;
    }
    let cancelled = false;
    setState({ failed: false });
    resolveLocalSrc(url).then(
      (src) => {
        if (!cancelled) setState({ src, failed: false });
      },
      () => {
        if (!cancelled) setState({ failed: true });
      },
    );
    return () => {
      cancelled = true;
    };
  }, [url]);

  return state;
}
