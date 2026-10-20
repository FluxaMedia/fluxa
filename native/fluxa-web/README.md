# fluxa-web

The renderer compiled to wasm for the browser and for Samsung Tizen TVs. `www/` is the static shell; `tizen/` holds the widget config and the polyfills for the TV's older web engine.

```bash
scripts/build-web.sh dist-web/site      # static site, ready for GitHub Pages
scripts/package-tizen.sh                # dist-web/fluxa-<version>-tizen.wgt
```

Pushes to master that touch `native`, `core` or `shared` deploy the site through `.github/workflows/pages.yml`.

Query parameters:

- `proxy=<base url>` routes artwork, HLS manifests and subtitles through a server exposing `/proxy?url=`.

Playback uses the page's `<video>` element behind the transparent canvas. HLS goes through the browser or TV pipeline when it can and through hls.js otherwise.

The Tizen package targets Chromium 76 (Tizen 6.0): wasm is built with `target-cpu=mvp` and `-Z build-std`, the JS is bundled into one classic script and the wasm is loaded with XHR because the widget runs from `file://`. The widget is unsigned; install it with Apps2Samsung, which signs it for the TV.
