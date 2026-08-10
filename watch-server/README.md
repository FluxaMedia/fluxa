# Fluxa Watch Together Server

Small self-hosted synchronization server for Fluxa Watch Party. **No video/audio traffic is relayed through this server.** It only carries room membership, content IDs, and playback state over WebSocket.

## Docker

```bash
docker compose up -d --build
```

Then use `http://YOUR_SERVER:8787` in the Watch Party panel inside the Fluxa player. For Internet-facing installs, put it behind HTTPS (Caddy/Nginx/Traefik); Fluxa automatically converts `https://` to `wss://`.

Optional environment variables:

- `FLUXA_WATCH_SECRET`: shared server password/token
- `MAX_ROOM_MEMBERS`: default `12`
- `ROOM_TTL_MINUTES`: inactive-room lifetime, default `360`
- `PORT`: default `8787`

Health endpoint: `GET /health`
WebSocket endpoint: `/ws`
