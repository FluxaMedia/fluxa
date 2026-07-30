import { invoke } from "@tauri-apps/api/core";
import * as Sentry from "@sentry/react";
import type { CoreMethod } from "./coreMethods";

export async function coreInvoke<T>(
  method: CoreMethod,
  argsJson: string,
): Promise<T | null> {
  return Sentry.startSpan(
    { name: `coreInvoke:${method}`, op: "fluxa.core" },
    async () => {
      const raw = await invoke<string>("core_invoke", { method, argsJson });
      const envelope = JSON.parse(raw) as {
        ok: boolean;
        value?: T;
        error?: { kind: string; message: string };
      };
      if (!envelope.ok) {
        throw new Error(
          `[core] ${method}: ${envelope.error?.message ?? "unknown error"}`,
        );
      }
      return envelope.value ?? null;
    },
  );
}

