import { invoke } from "@tauri-apps/api/core";
import { withSentrySpan } from "./sentryRuntime";
import type { CoreMethod } from "./coreMethods";

export async function coreInvoke<T>(
  method: CoreMethod,
  argsJson: string,
): Promise<T | null> {
  return withSentrySpan(`coreInvoke:${method}`, "fluxa.core", async () => {
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
