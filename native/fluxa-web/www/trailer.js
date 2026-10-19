const STATUS_INTERVAL = 250;

export const createTrailer = ({ app, layer, place }) => {
  const video = document.createElement("video");
  video.muted = true;
  video.defaultMuted = true;
  video.playsInline = true;
  video.preload = "auto";
  video.disablePictureInPicture = true;
  layer.append(video);

  let active = false;
  let error = null;
  let timer = 0;

  const report = () => {
    const duration = Number.isFinite(video.duration) ? video.duration : 0;
    app.video_status(
      video.currentTime || 0,
      duration,
      video.paused,
      0,
      video.readyState >= 2 && video.videoWidth > 0,
      -1,
      0,
      error,
    );
  };

  const stop = () => {
    clearInterval(timer);
    timer = 0;
    active = false;
    error = null;
    video.style.display = "none";
    video.removeAttribute("src");
    video.load();
  };

  video.addEventListener("error", () => {
    if (active) error = video.error?.message || `media error ${video.error?.code ?? ""}`.trim();
  });

  return {
    load(url) {
      stop();
      active = true;
      video.src = url;
      video.play().catch(() => {});
      timer = setInterval(report, STATUS_INTERVAL);
    },
    stop,
    place(item) {
      if (!item || !active) {
        video.style.display = "none";
        return;
      }
      video.style.display = "block";
      place(video, item);
    },
  };
};
