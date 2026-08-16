export const TV_KEY = {
  BACK: 461,
  EXIT: 1001,
  PLAY: 415,
  PAUSE: 19,
  PLAY_PAUSE: 179,
  STOP: 413,
  FAST_FORWARD: 417,
  REWIND: 412,
  NEXT: 425,
  PREVIOUS: 424,
  RED: 403,
  GREEN: 404,
  YELLOW: 405,
  BLUE: 406,
  CHANNEL_UP: 33,
  CHANNEL_DOWN: 34,
  UP: 38,
  DOWN: 40,
  LEFT: 37,
  RIGHT: 39,
  ENTER: 13,
  INFO: 457,
} as const;

export type TvAction =
  | 'back'
  | 'play'
  | 'pause'
  | 'playPause'
  | 'stop'
  | 'fastForward'
  | 'rewind'
  | 'next'
  | 'previous'
  | 'up'
  | 'down'
  | 'left'
  | 'right'
  | 'enter'
  | 'info'
  | 'red'
  | 'green'
  | 'yellow'
  | 'blue';

const BY_KEY_CODE: Record<number, TvAction> = {
  [TV_KEY.BACK]: 'back',
  [TV_KEY.PLAY]: 'play',
  [TV_KEY.PAUSE]: 'pause',
  [TV_KEY.PLAY_PAUSE]: 'playPause',
  [TV_KEY.STOP]: 'stop',
  [TV_KEY.FAST_FORWARD]: 'fastForward',
  [TV_KEY.REWIND]: 'rewind',
  [TV_KEY.NEXT]: 'next',
  [TV_KEY.PREVIOUS]: 'previous',
  [TV_KEY.UP]: 'up',
  [TV_KEY.DOWN]: 'down',
  [TV_KEY.LEFT]: 'left',
  [TV_KEY.RIGHT]: 'right',
  [TV_KEY.ENTER]: 'enter',
  [TV_KEY.INFO]: 'info',
  [TV_KEY.RED]: 'red',
  [TV_KEY.GREEN]: 'green',
  [TV_KEY.YELLOW]: 'yellow',
  [TV_KEY.BLUE]: 'blue',
};

const BY_KEY_NAME: Record<string, TvAction> = {
  Back: 'back',
  BrowserBack: 'back',
  GoBack: 'back',
  XF86Back: 'back',
  Escape: 'back',
  MediaPlay: 'play',
  MediaPause: 'pause',
  MediaPlayPause: 'playPause',
  MediaStop: 'stop',
  MediaFastForward: 'fastForward',
  MediaRewind: 'rewind',
  MediaTrackNext: 'next',
  MediaTrackPrevious: 'previous',
  ArrowUp: 'up',
  ArrowDown: 'down',
  ArrowLeft: 'left',
  ArrowRight: 'right',
  Enter: 'enter',
  ColorF0Red: 'red',
  ColorF1Green: 'green',
  ColorF2Yellow: 'yellow',
  ColorF3Blue: 'blue',
};

export function tvActionFor(event: KeyboardEvent): TvAction | null {
  return BY_KEY_CODE[event.keyCode] ?? BY_KEY_NAME[event.key] ?? null;
}

export function isTextEntryTarget(target: EventTarget | null): boolean {
  const element = target as HTMLElement | null;
  if (!element) return false;
  return element.tagName === 'INPUT' || element.tagName === 'TEXTAREA' || element.isContentEditable;
}
