let traceSequence = 0;

export interface PerfSpan {
  traceId: string;
  operation: string;
  startedAt: number;
  end(fields?: Record<string, unknown>): void;
}

export function startPerfSpan(operation: string, fields: Record<string, unknown> = {}): PerfSpan {
  const traceId = `${Date.now().toString(36)}-${(++traceSequence).toString(36)}`;
  const startedAt = performance.now();
  return {
    traceId,
    operation,
    startedAt,
    end(extra = {}) {
      console.debug(
        `[fluxa:perf:${operation}]`,
        JSON.stringify({ traceId, operation, ms: Math.round(performance.now() - startedAt), ...fields, ...extra }),
      );
    },
  };
}
