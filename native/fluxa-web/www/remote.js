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
