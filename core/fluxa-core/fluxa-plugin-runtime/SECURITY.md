# Plugin runtime threat model

Scrapers and provider extensions are untrusted JavaScript from user-added repositories. They run in QuickJS (rquickjs), one runtime and context per plugin, on a small pool of worker threads.

## What a plugin can reach

Only the globals installed by `host_functions.rs` and the polyfills:

- `fetch` / `__native_fetch*`: HTTP(S) GET, HEAD, POST through the embedder's `PluginHttpClient`
- `console`, timers (`setTimeout` family, cancelled when the job ends)
- `crypto`, `CryptoJS`, `Buffer`, `TextEncoder`, base64: native digest, HMAC, PBKDF2, AES, RSA/ECDSA sign and verify, random bytes
- `cheerio` / `LoadDoc`: an HTML parser handle table owned by the current job
- its own `SCRAPER_ID` and `SCRAPER_SETTINGS`

There is no filesystem, process, environment, raw socket, WebSocket, dynamic native module or `eval`-to-host path. Module loading is limited to `require('cheerio')` and `require('crypto-js')`.

## What a plugin must not reach

- Loopback, private, link-local, CGNAT, multicast, reserved and cloud metadata addresses, including IPv4-mapped, NAT64, 6to4 and IPv4-compatible IPv6 forms and every URL spelling of them
- Another plugin's globals, prototypes, cache or settings
- User tokens: Cookie and Authorization headers a plugin sets must not follow a redirect to another origin; the runtime never injects account credentials into plugin requests
- The host process: no panic, abort, OOM or unbounded CPU in native code reachable from JS

## Enforcement

| Control | Where |
| --- | --- |
| Address classification (`is_public_plugin_ip`) used for URLs and resolved addresses | `fluxa-core/src/addons/plugins/network.rs` |
| Policy check on every URL, including each redirect hop | `plugin_http_request_error_trusting`, `guarded_fetch_via` in `native/fluxa-effects/src/executor/plugins.rs` |
| DNS resolved once per hop; the checked addresses are the ones connected to (`resolve_to_addrs`) | same |
| Redirects followed manually (client redirect policy is off), limit 5 | same |
| Cookie, Authorization, Proxy-Authorization dropped on cross-origin redirect | same |
| Response body streamed with a hard cap; oversize is an error, never a silent truncation | same |
| Per-job request count and byte budget | `PluginClient` |
| QuickJS memory limit, 1 MiB stack, interrupt handler deadline, job deadline | `scraper_exec.rs`, `provider_exec.rs`, `lib.rs` |
| Context pool capped (`MAX_QUICKJS_PLUGIN_CONTEXTS`), eviction drops the context and compiled-function cache | `scraper_exec.rs` |
| Caps on random bytes, PBKDF2 iterations and output, PoW search time | `crypto_bridge.rs`, `pow_bridge.rs` |
| Caps on HTML size, total HTML, document count, element handles and nesting depth | `dom_bridge.rs` |

## Known limits

- `trusted_authorities` exempts a `host:port` from the address check so a user can point a plugin at a server on their LAN or loopback. Redirects out of a trusted host are still checked against the normal rules. The list comes from the plugin's saved settings and from loopback defaults in the plugin manifest, so a manifest can request loopback access to one port without a user action. See the audit notes.
- Native calls cannot be interrupted by the QuickJS deadline. Each is bounded by an input cap instead.
- The deadline interrupts JS only; a plugin can still use its full per-job request budget against public hosts.
- A plugin can read anything the user-visible request context contains (title, TMDB id, season, episode, its own settings).
