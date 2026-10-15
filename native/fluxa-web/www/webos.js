export const isWebos = /web0s|webos/i.test(navigator.userAgent);

const PROXY_SERVICE = "luna://com.fluxa.app.proxy";

export const luna = (uri, method, parameters = {}) =>
  new Promise((resolve, reject) => {
    if (typeof PalmServiceBridge === "undefined") return reject(new Error("service bus unavailable"));
    const bridge = new PalmServiceBridge();
    bridge.onservicecallback = (message) => {
      bridge.cancel?.();
      const response = JSON.parse(message);
      if (response.returnValue === false) reject(new Error(response.errorText || `${uri}/${method} failed`));
      else resolve(response);
    };
    bridge.call(`${uri}/${method}`, JSON.stringify(parameters));
  });

export const lunaSubscribe = (uri, method, parameters, onEvent) => {
  if (typeof PalmServiceBridge === "undefined") return () => {};
  const bridge = new PalmServiceBridge();
  bridge.onservicecallback = (message) => onEvent(JSON.parse(message));
  bridge.call(`${uri}/${method}`, JSON.stringify({ ...parameters, subscribe: true }));
  return () => bridge.cancel?.();
};

export const startProxy = async () => {
  if (!isWebos) return null;
  const response = await luna(PROXY_SERVICE, "start").catch(() => null);
  return response?.port ? `http://127.0.0.1:${response.port}` : null;
};

export const stageReady = () => {
  try {
    window.PalmSystem?.stageReady();
  } catch {}
};

export const exitApp = () => {
  try {
    window.PalmSystem?.deactivate();
  } catch {}
  window.close();
};

let capabilityPromise = null;

export const capabilities = () => {
  capabilityPromise ??= (async () => {
    if (!isWebos) return null;
    const info = (() => {
      try {
        return JSON.parse(window.PalmSystem?.deviceInfo || "{}");
      } catch {
        return {};
      }
    })();
    const configs = await luna("luna://com.webos.service.config", "getConfigs", {
      configNames: ["tv.model.supportHDR", "tv.model.supportDolbyHDRContents", "tv.model.supportDolbyAtmos"],
    }).catch(() => null);
    const values = configs?.configs ?? {};
    return {
      hdr10: values["tv.model.supportHDR"] ?? true,
      dolbyVision: values["tv.model.supportDolbyHDRContents"] ?? false,
      dolbyAtmos: values["tv.model.supportDolbyAtmos"] ?? false,
      uhd: (info.screenWidth || 0) >= 3840,
      model: info.modelName || "",
    };
  })();
  return capabilityPromise;
};

const CONTAINERS = {
  mkv: "MKV",
  webm: "WEBM",
  mp4: "MP4",
  m4v: "MP4",
  mov: "MP4",
  avi: "AVI",
  ts: "MPEG2-TS",
  m2ts: "MPEG2-TS",
  mts: "MPEG2-TS",
  m3u8: "HLS",
  mpd: "DASH",
};

const containerFor = (url) => {
  const path = url.split("?")[0].split("#")[0].toLowerCase();
  return CONTAINERS[path.slice(path.lastIndexOf(".") + 1)] ?? "MKV";
};

const hdrKindFor = (url) => {
  const name = decodeURIComponent(url).toLowerCase();
  if (/\b(dv|dovi|dolby[ ._-]?vision)\b|dvhe|dvh1/.test(name)) return "dolbyVision";
  if (/\bhlg\b/.test(name)) return "hlg";
  if (/\b(hdr10\+?|hdr|2160p.*pq|smpte2084)\b/.test(name)) return "hdr10";
  return "none";
};

const bitstreamCodec = (url) => /truehd|atmos|e-?ac-?3|\bddp\b|\bdts\b|\bac3\b/i.test(decodeURIComponent(url));

export const mediaAdapter = {
  async beforeLoad(video, url, config) {
    if (!isWebos) return;
    const device = await capabilities();
    const container = containerFor(url);
    const wanted = hdrKindFor(url);
    const hdr =
      wanted === "dolbyVision" && (device?.dolbyVision === false || config.dolbyVisionMode === "off")
        ? "hdr10"
        : wanted !== "none" && device?.hdr10 === false
          ? "none"
          : wanted;
    const option = {
      mediaFormat: { type: container },
      transmission: { contentsType: "VOD" },
      hdrInfo: hdr === "none" ? { isHDRContent: false } : { isHDRContent: true, hdrType: hdr },
      audioInfo: {
        isMultiChannel: true,
        isBitstreamRequired: config.passthrough === true && bitstreamCodec(url),
      },
    };
    if (container === "HLS" || container === "DASH") option.adaptiveStreaming = { audioOnly: false, seamlessPlay: true };
    const payload = JSON.stringify({ mediaTransportType: "URI", option });
    try {
      if (typeof video.setMediaOption === "function") video.setMediaOption(payload);
      else video.mediaOption = payload;
    } catch {}
  },
};

const MEDIA_BY_CODE = {
  415: ["play"],
  19: ["pause"],
  179: ["toggle"],
  413: ["stop"],
  417: ["fastForward", 0],
  412: ["rewind", 0],
  425: ["next"],
  424: ["previous"],
};

const MEDIA_BY_NAME = {
  MediaPlay: ["play"],
  MediaPause: ["pause"],
  MediaPlayPause: ["toggle"],
  MediaStop: ["stop"],
  MediaFastForward: ["fastForward", 0],
  MediaRewind: ["rewind", 0],
  MediaTrackNext: ["next"],
  MediaTrackPrevious: ["previous"],
};

const BACK_CODES = new Set([461, 10009]);
const BACK_NAMES = new Set(["GoBack", "BrowserBack", "XF86Back"]);

export const installRemote = (app, onBack) => {
  addEventListener(
    "keydown",
    (event) => {
      if (event.target instanceof HTMLInputElement) return;
      if (BACK_CODES.has(event.keyCode) || BACK_NAMES.has(event.key)) {
        event.preventDefault();
        event.stopImmediatePropagation();
        onBack();
        return;
      }
      const media = MEDIA_BY_CODE[event.keyCode] ?? MEDIA_BY_NAME[event.key];
      if (media) {
        event.preventDefault();
        event.stopImmediatePropagation();
        app.media_command(media[0], media[1] ?? 0);
      }
    },
    true,
  );
};
