# fluxa-web

The renderer compiled to wasm for the browser and for webOS TVs. `www/` is the static shell; `webos/` holds the webOS app metadata, icons and the Luna proxy service.

```bash
scripts/build-web.sh dist-web/site      # static site, ready for GitHub Pages
scripts/package-webos.sh                # dist-web/fluxa-<version>-webos.ipk
```

Pushes to master that touch `native`, `core` or `shared` deploy the site through `.github/workflows/pages.yml`. Release tags also attach the ipk.

Query parameters:

- `proxy=<base url>` routes artwork, HLS manifests and subtitles through a server exposing `/proxy?url=`. On webOS the bundled Luna service provides this.

Playback uses the page's `<video>` element behind the transparent canvas. HLS goes through the browser or TV pipeline when it can and through hls.js otherwise. On webOS the app declares container, HDR and audio passthrough to the media pipeline with `setMediaOption` before loading, so decoding stays on the TV's hardware decoders.

webOS needs Chromium 87 or newer (webOS 22+).

Install on a TV in developer mode:

```bash
ares-install --device <name> dist-web/fluxa-<version>-webos.ipk
ares-launch --device <name> com.fluxa.app
```
