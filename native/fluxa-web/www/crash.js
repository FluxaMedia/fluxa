const dsn = "__FLUXA_SENTRY_DSN__";
const release = "__FLUXA_VERSION__";
const limit = 5;

const parse = (value) => {
  try {
    const url = new URL(value);
    const project = url.pathname.slice(1);
    return url.username && project ? { key: url.username, origin: url.origin, project } : null;
  } catch {
    return null;
  }
};

const scrub = (text) => String(text ?? "").replace(/(https?:\/\/[^\s?#]+)[?#]\S*/g, "$1");

export const createCrashReporter = () => {
  const target = parse(dsn);
  const sent = new Set();
  let enabled = true;

  const send = (type, message, stack) => {
    if (!target || !enabled || sent.size >= limit) return;
    const text = scrub(message);
    if (sent.has(text)) return;
    sent.add(text);
    const id = crypto.randomUUID().replaceAll("-", "");
    const event = {
      event_id: id,
      timestamp: Date.now() / 1000,
      platform: "javascript",
      level: "error",
      release,
      tags: { os: "web" },
      exception: { values: [{ type, value: text, stacktrace: stack ? { frames: [{ filename: scrub(stack) }] } : undefined }] },
    };
    const body = [{ event_id: id, sent_at: new Date().toISOString() }, { type: "event" }, event]
      .map((part) => JSON.stringify(part))
      .join("\n");
    fetch(`${target.origin}/api/${target.project}/envelope/?sentry_key=${target.key}&sentry_version=7`, {
      method: "POST",
      body,
      keepalive: true,
      mode: "no-cors",
    }).catch(() => {});
  };

  addEventListener("error", (event) => send(event.error?.name ?? "Error", event.message, event.error?.stack));
  addEventListener("unhandledrejection", (event) =>
    send(event.reason?.name ?? "UnhandledRejection", event.reason?.message ?? event.reason, event.reason?.stack),
  );

  return {
    setEnabled(value) {
      enabled = value;
    },
  };
};
