const STATUS_INTERVAL = 250;
const SUBTITLE_MIN_MS = 1500;

const language = (value) => (value || "").toLowerCase().split(/[-_]/)[0];

const parseInfo = (raw) => {
  try {
    return JSON.parse(raw || "{}");
  } catch {
    return {};
  }
};

export const hasAvplay = () => !!window.webapis?.avplay;

export const createAvplay = ({ app, fallback, onStop }) => {
  const avplay = window.webapis.avplay;
  const object = document.getElementById("avplayer");
  const subtitle = document.getElementById("avsub");
  let url = null;
  let active = false;
  let timer = 0;
  let current = 0;
  let duration = 0;
  let buffering = true;
  let paused = false;
  let ready = false;
  let error = null;
  let fit = "fit";
  let preferred = { audio: "", subtitle: "" };
  let restored = false;
  let lastTracks = "";
  let subtitleOff = true;
  let subtitleTimer = 0;
  let pendingSubtitles = [];

  const showSubtitle = (text) => {
    clearTimeout(subtitleTimer);
    subtitle.textContent = subtitleOff ? "" : text.replace(/<[^>]+>/g, "");
    subtitle.style.display = subtitle.textContent ? "block" : "none";
  };

  const applyDisplay = () => {
    object.style.display = "block";
    const mode =
      fit === "fill"
        ? "PLAYER_DISPLAY_MODE_FULL_SCREEN"
        : fit === "stretch"
          ? "PLAYER_DISPLAY_MODE_FULL_SCREEN"
          : "PLAYER_DISPLAY_MODE_LETTER_BOX";
    try {
      avplay.setDisplayRect(0, 0, window.innerWidth, window.innerHeight);
      avplay.setDisplayMethod(mode);
    } catch {}
  };

  const streams = () => {
    try {
      return avplay.getTotalTrackInfo() || [];
    } catch {
      return [];
    }
  };

  const selectedIndex = (type) => {
    try {
      return (avplay.getCurrentStreamInfo() || []).find((info) => info.type === type)?.index;
    } catch {
      return undefined;
    }
  };

  const trackList = () => {
    const audioSelected = selectedIndex("AUDIO");
    const textSelected = selectedIndex("TEXT");
    return streams()
      .filter((track) => track.type === "AUDIO" || track.type === "TEXT")
      .map((track) => {
        const info = parseInfo(track.extra_info);
        const text = track.type === "TEXT";
        return {
          id: `${text ? "s" : "a"}${track.index}`,
          subtitle: text,
          language: info.language || info.track_lang || "",
          title: info.track_title || "",
          selected: text ? !subtitleOff && track.index === textSelected : track.index === audioSelected,
        };
      });
  };

  const reportTracks = (force) => {
    const list = JSON.stringify(trackList());
    if (!force && list === lastTracks) return;
    lastTracks = list;
    app.video_tracks(list);
    if (!restored && list !== "[]") restoreLanguages();
  };

  const selectAudio = (index) => {
    try {
      avplay.setSelectTrack("AUDIO", index);
    } catch {}
  };

  const selectSubtitle = (index) => {
    try {
      if (index === null) {
        subtitleOff = true;
        avplay.setSilentSubtitle(true);
        showSubtitle("");
        return;
      }
      subtitleOff = false;
      avplay.setSilentSubtitle(false);
      avplay.setSelectTrack("TEXT", index);
    } catch {}
  };

  const restoreLanguages = () => {
    restored = true;
    const list = trackList();
    const wantAudio = language(preferred.audio);
    const audio = wantAudio && list.find((track) => !track.subtitle && language(track.language) === wantAudio);
    if (audio && !audio.selected) selectAudio(Number(audio.id.slice(1)));
    const wantSubtitle = language(preferred.subtitle);
    if (!wantSubtitle) return;
    const text = list.find((track) => track.subtitle && language(track.language) === wantSubtitle);
    if (text) selectSubtitle(Number(text.id.slice(1)));
  };

  const report = () => {
    app.video_status(current / 1000, duration / 1000, paused, 100, ready, buffering ? 0 : -1, 0, error);
    if (ready) reportTracks(false);
  };

  const teardown = () => {
    clearInterval(timer);
    timer = 0;
    active = false;
    ready = false;
    try {
      avplay.stop();
    } catch {}
    try {
      avplay.close();
    } catch {}
    object.style.display = "none";
    showSubtitle("");
  };

  const begin = (source) => {
    avplay.open(source);
    avplay.setListener({
      onbufferingstart: () => (buffering = true),
      onbufferingprogress: () => (buffering = true),
      onbufferingcomplete: () => (buffering = false),
      oncurrentplaytime: (ms) => {
        current = ms;
        buffering = false;
      },
      onevent: () => {},
      onstreamcompleted: () => {
        paused = true;
        current = duration;
      },
      onerror: (type) => {
        if (active) error = String(type);
      },
      onsubtitlechange: (ms, text) => {
        showSubtitle(text || "");
        subtitleTimer = setTimeout(() => showSubtitle(""), Math.max(Number(ms) || 0, SUBTITLE_MIN_MS));
      },
    });
    applyDisplay();
    avplay.prepareAsync(
      () => {
        if (!active || url !== source) return;
        duration = avplay.getDuration();
        ready = true;
        for (const path of pendingSubtitles) {
          try {
            avplay.setExternalSubtitlePath(path);
          } catch {}
        }
        pendingSubtitles = [];
        selectSubtitle(null);
        avplay.play();
        reportTracks(true);
      },
      () => {
        if (!active || url !== source) return;
        teardown();
        fallback(source);
      },
    );
  };

  const load = (request) => {
    teardown();
    url = request.url;
    active = true;
    current = 0;
    duration = 0;
    buffering = true;
    paused = false;
    error = null;
    restored = false;
    lastTracks = "";
    subtitleOff = true;
    pendingSubtitles = [];
    try {
      begin(url);
    } catch {
      teardown();
      fallback(url);
      return;
    }
    timer = setInterval(report, STATUS_INTERVAL);
    report();
  };

  const stop = () => {
    teardown();
    url = null;
    error = null;
    app.video_status(0, 0, true, 100, false, -1, 0, null);
    app.video_tracks("[]");
  };

  const seekTo = (seconds) => {
    if (!ready) return;
    const target = Math.min(Math.max(0, seconds * 1000), Math.max(0, duration - 500));
    current = target;
    buffering = true;
    avplay.seekTo(
      target,
      () => (buffering = false),
      () => (buffering = false),
    );
  };

  const handlers = {
    configure: (request) => {
      preferred = { audio: request.audioLanguage || "", subtitle: request.subtitleLanguage || "" };
    },
    load,
    stop: () => {
      onStop();
      stop();
    },
    togglePause: () => {
      if (!ready) return;
      paused = !paused;
      if (paused) avplay.pause();
      else avplay.play();
    },
    seek: (request) => seekTo(current / 1000 + request.seconds),
    seekTo: (request) => seekTo(request.seconds),
    setSpeed: (request) => {
      try {
        avplay.setSpeed(request.value);
      } catch {}
    },
    fit: (request) => {
      fit = request.mode || "fit";
      applyDisplay();
    },
    selectTracks: (request) => {
      if (request.audio) selectAudio(Number(request.audio.slice(1)));
      if (request.subtitle) selectSubtitle(Number(request.subtitle.slice(1)));
      else if (request.subtitlesOff) selectSubtitle(null);
      reportTracks(true);
    },
    mpv: (request) => {
      const [command, path] = request.args || [];
      if (command !== "sub-add" || !path) return;
      if (ready) {
        try {
          avplay.setExternalSubtitlePath(path);
        } catch {}
      } else {
        pendingSubtitles.push(path);
      }
    },
  };

  return {
    handlers,
    isActive: () => active,
  };
};
