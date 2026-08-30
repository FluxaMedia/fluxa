import { useEffect, useRef, useState } from 'react';
import type { ContentWarning } from '../../core/contentWarnings';

const ROW_HEIGHT = 18;
const ROW_GAP = 2;
const EASE = 'cubic-bezier(0.4, 0, 0.2, 1)';

interface Props {
  warnings: ContentWarning[];
  isVisible: boolean;
  controlsVisible: boolean;
  isPaused: boolean;
  onAnimationComplete: () => void;
}

export function ContentWarningOverlay({ warnings, isVisible, controlsVisible, isPaused, onAnimationComplete }: Props) {
  const count = warnings.length;
  const [containerOpacity, setContainerOpacity] = useState(0);
  const [barGrown, setBarGrown] = useState(false);
  const [rowVisible, setRowVisible] = useState<boolean[]>(() => warnings.map(() => false));
  const animatingRef = useRef(false);
  const isPausedRef = useRef(isPaused);
  const timeoutsRef = useRef<ReturnType<typeof setTimeout>[]>([]);
  isPausedRef.current = isPaused;

  useEffect(() => {
    timeoutsRef.current.forEach(clearTimeout);
    timeoutsRef.current = [];
    if (count === 0) return;

    const schedule = (ms: number, fn: () => void) => {
      const run = () => {
        if (isPausedRef.current) {
          timeoutsRef.current.push(setTimeout(run, 100));
          return;
        }
        fn();
      };
      timeoutsRef.current.push(setTimeout(run, ms));
    };

    if (isVisible && !animatingRef.current) {
      animatingRef.current = true;
      setRowVisible(warnings.map(() => false));
      setBarGrown(false);
      setContainerOpacity(1);

      let elapsed = 300;
      schedule(elapsed, () => setBarGrown(true));
      elapsed += 400;
      for (let i = 0; i < count; i++) {
        elapsed += 80;
        const index = i;
        schedule(elapsed, () => setRowVisible((prev) => prev.map((value, position) => (position === index ? true : value))));
        elapsed += 200;
      }
      elapsed += 5000;
      for (let j = 0; j < count; j++) {
        elapsed += 60;
        const index = count - 1 - j;
        schedule(elapsed, () => setRowVisible((prev) => prev.map((value, position) => (position === index ? false : value))));
        elapsed += 150;
      }
      elapsed += 100;
      schedule(elapsed, () => setBarGrown(false));
      elapsed += 300;
      elapsed += 200;
      schedule(elapsed, () => setContainerOpacity(0));
      elapsed += 200;
      schedule(elapsed, () => {
        animatingRef.current = false;
        onAnimationComplete();
      });
    } else if (!isVisible && animatingRef.current) {
      setRowVisible(warnings.map(() => false));
      setBarGrown(false);
      setContainerOpacity(0);
      animatingRef.current = false;
    }

    return () => {
      timeoutsRef.current.forEach(clearTimeout);
      timeoutsRef.current = [];
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [isVisible]);

  if (count === 0) return null;

  const totalBarHeight = ROW_HEIGHT * count + ROW_GAP * Math.max(count - 1, 0);

  return (
    <div
      style={{
        position: 'fixed',
        top: controlsVisible ? '4.5rem' : '1.5rem',
        left: '2.75rem',
        zIndex: 20,
        display: 'flex',
        alignItems: 'flex-start',
        opacity: containerOpacity,
        transition: `opacity ${containerOpacity ? 300 : 200}ms`,
        pointerEvents: 'none',
        padding: '0.5rem 0.75rem 0.5rem 0.625rem',
        borderRadius: '0.375rem',
        background: 'rgba(0,0,0,0.68)',
        boxShadow: '0 0.25rem 1rem rgba(0,0,0,0.28)',
        backdropFilter: 'blur(0.25rem)',
      }}
    >
      <div
        style={{
          width: '0.1875rem',
          height: `${barGrown ? totalBarHeight : 0}px`,
          borderRadius: '0.0625rem',
          background: 'var(--primary-accent-color)',
          transition: `height ${barGrown ? 400 : 300}ms ${EASE}`,
          flexShrink: 0,
        }}
      />
      <div style={{ display: 'flex', flexDirection: 'column', gap: `${ROW_GAP}px`, marginLeft: '0.625rem' }}>
        {warnings.map((warning, index) => (
          <div
            key={`${warning.label}-${index}`}
            style={{
              height: `${ROW_HEIGHT}px`,
              display: 'flex',
              alignItems: 'center',
              opacity: rowVisible[index] ? 1 : 0,
              transition: `opacity ${rowVisible[index] ? 200 : 150}ms`,
            }}
          >
            <span style={{ color: 'rgba(255,255,255,0.98)', fontSize: '0.75rem', fontWeight: 600 }}>{warning.label}</span>
            <span style={{ color: 'rgba(255,255,255,0.62)', fontSize: '0.75rem', margin: '0 0.25rem' }}>·</span>
            <span style={{ color: 'rgba(255,255,255,0.78)', fontSize: '0.75rem' }}>{warning.severity}</span>
          </div>
        ))}
      </div>
    </div>
  );
}
