import { mountAccessibility } from "./a11y.js";
import { mountArtwork } from "./artwork.js";
import init, { FluxaWeb } from "./pkg/fluxa_web.js";
import { createPlayer } from "./player.js";
import { exitApp, installRemote, isWebos as webos, mediaAdapter, stageReady, startProxy } from "./webos.js";

const formFactor = webos || /smart-tv|smarttv/i.test(navigator.userAgent) ? "tv" : "mobile";
const query = new URLSearchParams(location.search);

const main = async () => {
  const proxy = query.get("proxy") ?? query.get("imageProxy") ?? (await startProxy());

  const webgpu = Boolean(navigator.gpu && (await navigator.gpu.requestAdapter().catch(() => null)));

  await init();
  const app = new FluxaWeb("fluxa", formFactor, webos ? "webos" : "web", webgpu, proxy);
  const syncAccessibility = mountAccessibility(app, document.getElementById("fluxa"), document.getElementById("a11y"));
  const syncArtwork = mountArtwork(app, document.getElementById("artwork"));
  addEventListener("resize", () => app.resize());
  const player = createPlayer({ app, video: document.getElementById("video"), proxy, adapter: mediaAdapter });
  installRemote(app, () => {
    if (!app.back() && webos) exitApp();
  });
  stageReady();

  const actionHandlers = [];
  window.fluxaOnAction = (handler) => actionHandlers.push(handler);

  let accessAt = 0;
  const tick = (now) => {
    if (now - accessAt > 250) {
      accessAt = now;
      syncAccessibility();
    }
    player.poll();
    const actions = app.frame();
    syncArtwork();
    if (actions) {
      for (const action of JSON.parse(actions)) {
        for (const handler of actionHandlers) handler(action);
      }
    }
    const iconSvg = app.take_app_icon_svg();
    if (iconSvg) {
      document.querySelector("link[rel=icon]").href = `data:image/svg+xml,${encodeURIComponent(iconSvg)}`;
    }
    requestAnimationFrame(tick);
  };
  requestAnimationFrame(tick);
};

main();
