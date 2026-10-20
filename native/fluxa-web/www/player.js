const HLS_MODULE = "https://cdn.jsdelivr.net/npm/hls.js@1.5.17/dist/hls.mjs";
const STATUS_INTERVAL = 250;
const TRACK_INTERVAL = 1000;

const isHls = (url) => /\.m3u8(\?|#|$)/i.test(url) || /[?&]type=hls\b/i.test(url);

const srtToVtt = (text) =>
  "WEBVTT\n\n" +
  text
    .replace(/\r+/g, "")
    .replace(/^﻿/, "")
    .replace(/(\d{2}:\d{2}:\d{2}),(\d{3})/g, "$1.$2");

export const createPlayer = ({ app, video, proxy, trailer }) => {
  let hls = null;
  let hlsModule = null;
  let url = null;
  let triedProxy = false;
  let fit = "fit";
  let error = null;
  let waiting = false;
  let timer = 0;
  let trackAt = 0;
  let lastTracks = "";
  let externalSeq = 0;
  let externalTracks = new Set();
  let preferred = { audio: "", subtitle: "" };
  let restored = false;

  const wrap = (target) => (proxy ? `${proxy}/proxy?url=${encodeURIComponent(target)}` : target);

  const loadHls = async () => {
    hlsModule ??= (await import(HLS_MODULE)).default;
    return hlsModule;
  };

  const destroy = () => {
    clearInterval(timer);
    timer = 0;
    if (hls) {
      hls.destroy();
      hls = null;
    }
    for (const track of externalTracks) track.src && URL.revokeObjectURL(track.src);
    externalTracks = new Set();
    for (const child of [...video.querySelectorAll("track")]) child.remove();
    video.removeAttribute("src");
    video.load();
  };

  const language = (value) => (value || "").toLowerCase().split(/[-_]/)[0];

  const audioList = () => {
    if (hls) {
      return hls.audioTracks.map((track, index) => ({
        id: `a${index}`,
        subtitle: false,
        language: track.lang,
        title: track.name,
        selected: index === hls.audioTrack,
      }));
    }
    const tracks = video.audioTracks;
    if (!tracks) return [];
    return [...tracks].map((track, index) => ({
      id: `a${index}`,
      subtitle: false,
      language: track.language,
      title: track.label,
      selected: track.enabled,
    }));
  };

  const subtitleList = () =>
    [...video.textTracks]
      .filter((track) => track.kind === "subtitles" || track.kind === "captions")
      .map((track, index) => ({
        id: `s${index}`,
        subtitle: true,
        language: track.language,
        title: track.label,
        selected: track.mode === "showing",
        external: externalTracks.has(track),
      }));

  const subtitleTracks = () =>
    [...video.textTracks].filter((track) => track.kind === "subtitles" || track.kind === "captions");

  const reportTracks = (force) => {
    const list = JSON.stringify([...audioList(), ...subtitleList()]);
    if (!force && list === lastTracks) return;
    lastTracks = list;
    app.video_tracks(list);
    if (!restored && list !== "[]") restoreLanguages();
  };

  const restoreLanguages = () => {
    restored = true;
    const audio = audioList();
    const wantAudio = language(preferred.audio);
    const match = wantAudio && audio.find((track) => language(track.language) === wantAudio);
    if (match && !match.selected) selectAudio(match.id);
    const wantSubtitle = language(preferred.subtitle);
    if (!wantSubtitle) return;
    const pick = subtitleList().find((track) => language(track.language) === wantSubtitle);
    if (pick) selectSubtitle(pick.id);
  };

  const selectAudio = (id) => {
    const index = Number(id.slice(1));
    if (hls) {
      hls.audioTrack = index;
      return;
    }
    const tracks = video.audioTracks;
    if (!tracks) return;
    for (let i = 0; i < tracks.length; i++) tracks[i].enabled = i === index;
  };

  const selectSubtitle = (id) => {
    const index = id === null ? -1 : Number(id.slice(1));
    subtitleTracks().forEach((track, i) => {
      track.mode = i === index ? "showing" : "disabled";
    });
  };

  const report = () => {
    const duration = Number.isFinite(video.duration) ? video.duration : 0;
    const ranges = video.buffered;
    let buffered = 0;
    for (let i = 0; i < ranges.length; i++) {
      if (ranges.start(i) <= video.currentTime && ranges.end(i) >= video.currentTime) buffered = ranges.end(i);
    }
    const volume = video.muted ? 0 : video.volume * 100;
    app.video_status(
      video.currentTime || 0,
      duration,
      video.paused,
      volume,
      video.readyState >= 2 && video.videoWidth > 0,
      waiting || (video.readyState < 3 && !video.paused) ? 0 : -1,
      buffered,
      error,
    );
    const now = performance.now();
    if (now - trackAt > TRACK_INTERVAL) {
      trackAt = now;
      reportTracks(false);
    }
  };

  const applyFit = () => {
    video.style.objectFit = fit === "fill" ? "cover" : fit === "stretch" ? "fill" : "contain";
  };

  const fail = (message) => {
    error = message;
  };

  const start = async (source) => {
    const native = !!video.canPlayType("application/vnd.apple.mpegurl");
    if (isHls(source) && !native) {
      const Hls = await loadHls().catch(() => null);
      if (!Hls || !Hls.isSupported()) return fail("HLS playback is not supported here");
      hls = new Hls({ enableWorker: true, lowLatencyMode: false });
      hls.on(Hls.Events.ERROR, (_event, data) => {
        if (data.fatal) fail(data.details || "playback failed");
      });
      hls.on(Hls.Events.MANIFEST_PARSED, () => reportTracks(true));
      hls.on(Hls.Events.AUDIO_TRACKS_UPDATED, () => reportTracks(true));
      hls.loadSource(proxy ? wrap(source) : source);
      hls.attachMedia(video);
      return;
    }
    video.src = source;
  };

  const load = async (source) => {
    destroy();
    url = source;
    error = null;
    waiting = true;
    restored = false;
    lastTracks = "";
    triedProxy = false;
    applyFit();
    await start(source);
    video.play().catch(() => {});
    timer = setInterval(report, STATUS_INTERVAL);
    report();
  };

  const stop = () => {
    destroy();
    url = null;
    error = null;
    waiting = false;
    app.video_status(0, 0, true, 100, false, -1, 0, null);
    app.video_tracks("[]");
  };

  video.addEventListener("waiting", () => (waiting = true));
  for (const name of ["playing", "canplay", "seeked"]) video.addEventListener(name, () => (waiting = false));
  video.addEventListener("error", () => {
    if (!url) return;
    if (proxy && !triedProxy) {
      triedProxy = true;
      video.src = wrap(url);
      video.play().catch(() => {});
      return;
    }
    const code = video.error?.code;
    fail(video.error?.message || `media error ${code ?? ""}`.trim());
  });

  const addExternalSubtitle = async (source) => {
    const response = await fetch(wrap(source)).catch(() => null);
    if (!response?.ok) return;
    const text = await response.text();
    const body = text.trimStart().startsWith("WEBVTT") ? text : srtToVtt(text);
    const blob = URL.createObjectURL(new Blob([body], { type: "text/vtt" }));
    const element = document.createElement("track");
    element.kind = "subtitles";
    element.label = `External ${++externalSeq}`;
    element.src = blob;
    video.append(element);
    externalTracks.add(element.track);
    reportTracks(true);
  };

  const handleMpv = (args) => {
    const [command, ...rest] = args;
    if (command === "sub-add" && rest[0]) addExternalSubtitle(rest[0]);
  };

  const handlers = {
    configure: (request) => {
      preferred = { audio: request.audioLanguage || "", subtitle: request.subtitleLanguage || "" };
    },
    load: (request) => {
      trailer?.stop();
      load(request.url);
    },
    loadPreview: (request) => trailer?.load(request.url),
    stop: () => {
      trailer?.stop();
      stop();
    },
    togglePause: () => (video.paused ? video.play().catch(() => {}) : video.pause()),
    seek: (request) => {
      video.currentTime = Math.max(0, video.currentTime + request.seconds);
    },
    seekTo: (request) => {
      video.currentTime = Math.max(0, request.seconds);
    },
    toggleMute: () => {
      video.muted = !video.muted;
    },
    setVolume: (request) => {
      video.muted = false;
      video.volume = Math.min(1, Math.max(0, request.value / 100));
    },
    setSpeed: (request) => {
      video.playbackRate = request.value;
    },
    fit: (request) => {
      fit = request.mode || "fit";
      applyFit();
    },
    selectTracks: (request) => {
      if (request.audio) selectAudio(request.audio);
      if (request.subtitle) selectSubtitle(request.subtitle);
      else if (request.subtitlesOff) selectSubtitle(null);
      reportTracks(true);
    },
    mediaSession: (request) => {
      const plan = request.plan || {};
      if (!("mediaSession" in navigator)) return;
      navigator.mediaSession.metadata = new MediaMetadata({
        title: plan.title || "",
        artist: plan.subtitle || "",
        artwork: plan.poster ? [{ src: plan.poster }] : [],
      });
      navigator.mediaSession.playbackState = plan.state === "playing" ? "playing" : "paused";
    },
    mpv: (request) => handleMpv(request.args || []),
  };

  if ("mediaSession" in navigator) {
    const send = (command, value = 0) => app.media_command(command, value);
    const actions = {
      play: () => send("play"),
      pause: () => send("pause"),
      stop: () => send("stop"),
      nexttrack: () => send("next"),
      previoustrack: () => send("previous"),
      seekforward: (details) => send("seekForward", details.seekOffset || 10),
      seekbackward: (details) => send("seekBackward", details.seekOffset || 10),
      seekto: (details) => send("seekTo", details.seekTime || 0),
    };
    for (const [name, handler] of Object.entries(actions)) {
      try {
        navigator.mediaSession.setActionHandler(name, handler);
      } catch {}
    }
  }

  return {
    poll() {
      const requests = JSON.parse(app.poll_video());
      for (const request of requests) handlers[request.type]?.(request);
    },
  };
};
