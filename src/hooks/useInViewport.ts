import { useEffect, useState } from 'react';
import type React from 'react';

const observerEntries = new Map<string, {
  observer: IntersectionObserver;
  callbacks: WeakMap<Element, (visible: boolean) => void>;
  counts: Map<Element, number>;
}>();

export function useInViewport<T extends Element>(
  ref: React.RefObject<T | null>,
  rootMargin = '200px',
): boolean {
  const [inView, setInView] = useState(false);

  useEffect(() => {
    const node = ref.current;
    if (!node) return;
    let entry = observerEntries.get(rootMargin);
    if (!entry) {
      const callbacks = new WeakMap<Element, (visible: boolean) => void>();
      const counts = new Map<Element, number>();
      const observer = new IntersectionObserver((entries) => {
        for (const intersection of entries) {
          callbacks.get(intersection.target)?.(intersection.isIntersecting);
        }
      }, { rootMargin });
      entry = { observer, callbacks, counts };
      observerEntries.set(rootMargin, entry);
    }
    entry.callbacks.set(node, (visible) => setInView(visible));
    entry.counts.set(node, (entry.counts.get(node) ?? 0) + 1);
    entry.observer.observe(node);
    return () => {
      const count = (entry?.counts.get(node) ?? 1) - 1;
      if (count <= 0) {
        entry?.counts.delete(node);
        entry?.callbacks.delete(node);
        entry?.observer.unobserve(node);
      } else {
        entry?.counts.set(node, count);
      }
    };
  }, [ref, rootMargin]);

  return inView;
}
