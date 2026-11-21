const BASE = "https://api.example.com/v1";

async function call(path, body) {
  const reply = await fetch(BASE + path, {
    method: "POST",
    headers: { "content-type": "application/json", authorization: "Bearer " + storage.get("token", "") },
    body: JSON.stringify(body || {}),
  });
  if (!reply.ok) throw new Error("example.com answered " + reply.status);
  return reply.json();
}

class Provider {
  constructor() {
    this.tracker = {
      status: async () => ({ connected: storage.get("token", "") !== "" }),
      disconnect: async () => storage.remove("token"),
      auth: async (credentials) => {
        const reply = await call("/auth", credentials);
        storage.set("token", reply.token);
        return { connected: true };
      },
      scrobbleStart: (item) => call("/scrobble/start", { title: item.title, position: item.position }),
      scrobblePause: (item) => call("/scrobble/pause", { title: item.title, position: item.position }),
      scrobbleStop: (item) => call("/scrobble/stop", { title: item.title }),
      watched: async () => (await call("/watched")).items,
      library: async () => (await call("/library")).items,
    };
  }
}
