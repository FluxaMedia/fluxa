type PerformanceMemory = {
  usedJSHeapSize: number;
  totalJSHeapSize: number;
  jsHeapSizeLimit: number;
};

type BenchmarkMark = { name: string; atMs: number };
type BenchmarkMeasure = { name: string; durationMs: number; startMs: number };
type ReactCommit = {
  id: string;
  phase: string;
  actualDurationMs: number;
  baseDurationMs: number;
  startMs: number;
  commitMs: number;
};

export type PerformanceBenchmarkReport = {
  target: string;
  url: string;
  durationMs: number;
  marks: BenchmarkMark[];
  measures: BenchmarkMeasure[];
  reactCommits: ReactCommit[];
  longTasks: Array<{ durationMs: number; startMs: number }>;
  frame: { sampleDurationMs: number; frames: number; fps: number; droppedFrames: number; worstFrameMs: number };
  memory?: PerformanceMemory;
  resources: Array<{ name: string; initiatorType: string; durationMs: number; transferSize: number; decodedBodySize: number }>;
};

type BenchmarkApi = {
  report: () => PerformanceBenchmarkReport;
  mark: (name: string) => void;
  measure: (name: string, startMark: string, endMark: string) => void;
  stop: () => PerformanceBenchmarkReport;
};

declare global {
  interface Window {
    __FLUXA_PERFORMANCE__?: BenchmarkApi;
  }
}

type BenchmarkState = {
  startedAt: number;
  marks: BenchmarkMark[];
  measures: BenchmarkMeasure[];
  reactCommits: ReactCommit[];
  longTasks: Array<{ durationMs: number; startMs: number }>;
  frameTimes: number[];
  frameHandle: number | null;
  observer: PerformanceObserver | null;
};

let state: BenchmarkState | null = null;

function reportEndpoint(): string | null {
  if (typeof location === 'undefined') return null;
  try {
    return new URLSearchParams(location.search).get('benchmarkReportUrl') || import.meta.env.VITE_FLUXA_BENCHMARK_REPORT_URL || null;
  } catch {
    return null;
  }
}

function now(): number {
  return typeof performance === 'undefined' ? 0 : performance.now();
}

export function performanceBenchmarkEnabled(): boolean {
  if (typeof window === 'undefined') return false;
  try {
    return import.meta.env.VITE_FLUXA_BENCHMARK === '1' || new URLSearchParams(window.location.search).get('benchmark') === '1';
  } catch {
    return false;
  }
}

function relativeTime(value: number): number {
  return Math.round((value - (state?.startedAt ?? value)) * 100) / 100;
}

export function benchmarkMark(name: string): void {
  if (!state) return;
  const at = now();
  state.marks.push({ name, atMs: relativeTime(at) });
  try {
    performance.mark(`fluxa:${name}`);
  } catch {
    // Older webOS engines may expose performance without user timing marks.
  }
}

export function benchmarkMeasure(name: string, startMark: string, endMark: string): void {
  if (!state) return;
  const start = state.marks.find((item) => item.name === startMark);
  let end: BenchmarkMark | undefined;
  for (let index = state.marks.length - 1; index >= 0; index -= 1) {
    if (state.marks[index].name === endMark) {
      end = state.marks[index];
      break;
    }
  }
  if (!start || !end) return;
  state.measures.push({ name, startMs: start.atMs, durationMs: Math.max(0, Math.round((end.atMs - start.atMs) * 100) / 100) });
  try {
    performance.measure(`fluxa:${name}`, `fluxa:${startMark}`, `fluxa:${endMark}`);
  } catch {
    // Keep the in-memory measurement when the browser has partial User Timing support.
  }
}

export function benchmarkReactCommit(
  id: string,
  phase: string,
  actualDuration: number,
  baseDuration: number,
  startTime: number,
  commitTime: number,
): void {
  if (!state) return;
  state.reactCommits.push({
    id,
    phase,
    actualDurationMs: Math.round(actualDuration * 100) / 100,
    baseDurationMs: Math.round(baseDuration * 100) / 100,
    startMs: relativeTime(startTime),
    commitMs: relativeTime(commitTime),
  });
}

function resources() {
  if (typeof performance === 'undefined' || typeof performance.getEntriesByType !== 'function') return [];
  return performance.getEntriesByType('resource').map((entry) => {
    const resource = entry as PerformanceResourceTiming;
    let name = resource.name;
    try {
      const url = new URL(resource.name, location.href);
      name = `${url.origin}${url.pathname}`;
    } catch {
      name = resource.name.split('?')[0];
    }
    return {
      name,
      initiatorType: resource.initiatorType,
      durationMs: Math.round(resource.duration * 100) / 100,
      transferSize: resource.transferSize || 0,
      decodedBodySize: resource.decodedBodySize || 0,
    };
  });
}

function memory(): PerformanceMemory | undefined {
  const value = (performance as Performance & { memory?: PerformanceMemory }).memory;
  return value
    ? {
        usedJSHeapSize: value.usedJSHeapSize,
        totalJSHeapSize: value.totalJSHeapSize,
        jsHeapSizeLimit: value.jsHeapSizeLimit,
      }
    : undefined;
}

function report(): PerformanceBenchmarkReport {
  const current = now();
  const frameTimes = state?.frameTimes ?? [];
  const sampleDurationMs = Math.max(1, current - (state?.startedAt ?? current));
  const frames = frameTimes.length;
  const droppedFrames = frameTimes.filter((duration) => duration > 50).length;
  return {
    target: import.meta.env.VITE_FLUXA_TARGET || 'unknown',
    url: typeof location === 'undefined' ? '' : location.href,
    durationMs: Math.round(sampleDurationMs * 100) / 100,
    marks: state?.marks ?? [],
    measures: state?.measures ?? [],
    reactCommits: state?.reactCommits ?? [],
    longTasks: state?.longTasks ?? [],
    frame: {
      sampleDurationMs: Math.round(sampleDurationMs * 100) / 100,
      frames,
      fps: Math.round((frames / (sampleDurationMs / 1000)) * 100) / 100,
      droppedFrames,
      worstFrameMs: frameTimes.length > 0 ? Math.round(Math.max(...frameTimes) * 100) / 100 : 0,
    },
    memory: memory(),
    resources: resources(),
  };
}

function sampleFrame(previous: number): void {
  if (!state) return;
  const current = now();
  state.frameTimes.push(current - previous);
  state.frameHandle = window.requestAnimationFrame(() => sampleFrame(current));
}

export function initPerformanceBenchmark(): void {
  if (!performanceBenchmarkEnabled() || state) return;
  state = {
    startedAt: now(),
    marks: [],
    measures: [],
    reactCommits: [],
    longTasks: [],
    frameTimes: [],
    frameHandle: null,
    observer: null,
  };

  const PerformanceObserverCtor = typeof PerformanceObserver === 'undefined' ? null : PerformanceObserver;
  if (PerformanceObserverCtor) {
    try {
      state.observer = new PerformanceObserver((list) => {
        for (const entry of list.getEntries()) {
          state?.longTasks.push({ durationMs: Math.round(entry.duration * 100) / 100, startMs: relativeTime(entry.startTime) });
        }
      });
      state.observer.observe({ type: 'longtask', buffered: true });
    } catch {
      state.observer = null;
    }
  }

  state.frameHandle = window.requestAnimationFrame((timestamp) => sampleFrame(timestamp));
  window.__FLUXA_PERFORMANCE__ = {
    report,
    mark: benchmarkMark,
    measure: benchmarkMeasure,
    stop: () => {
      if (state?.frameHandle != null) window.cancelAnimationFrame(state.frameHandle);
      state?.observer?.disconnect();
      const result = report();
      state = null;
      return result;
    },
  };
  benchmarkMark('benchmark:start');
  window.setTimeout(() => {
    if (!state) return;
    const result = report();
    console.info('[fluxa:benchmark]', JSON.stringify(result));
    const endpoint = reportEndpoint();
    if (endpoint) {
      void fetch(endpoint, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(result),
      }).catch(() => undefined);
    }
  }, 15000);
}
