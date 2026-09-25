import init, { FluxaWeb } from "./pkg/fluxa_web.js";

const webos = /web0s|webos/i.test(navigator.userAgent);
const formFactor = webos || /smart-tv|smarttv/i.test(navigator.userAgent) ? "tv" : "mobile";

const startWebosProxy = () =>
  new Promise((resolve) => {
    if (!webos || !window.webOS?.service) return resolve(null);
    window.webOS.service.request("luna://com.fluxa.app.proxy", {
      method: "start",
      onSuccess: (response) => resolve(`http://127.0.0.1:${response.port}`),
      onFailure: () => resolve(null),
    });
  });

const imageProxy = new URLSearchParams(location.search).get("imageProxy") ?? (await startWebosProxy());

const webgpu = Boolean(navigator.gpu && (await navigator.gpu.requestAdapter().catch(() => null)));

await init();
const app = new FluxaWeb("fluxa", formFactor, webgpu, imageProxy);
addEventListener("resize", () => app.resize());

const actionHandlers = [];
window.fluxaOnAction = (handler) => actionHandlers.push(handler);

const tick = () => {
  const actions = app.frame();
  if (actions) {
    for (const action of JSON.parse(actions)) {
      for (const handler of actionHandlers) handler(action);
    }
  }
  requestAnimationFrame(tick);
};
requestAnimationFrame(tick);
