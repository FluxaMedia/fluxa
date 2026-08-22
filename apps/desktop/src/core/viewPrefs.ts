import { storageRead, storageWrite } from './engine';

export interface ViewPrefs {
  libraryTab?: string;
  librarySort?: string;
  librarySource?: string;
  libraryType?: string;
  homeType?: string;
}

let cache: ViewPrefs = {};

const ready = storageRead<ViewPrefs>('viewPrefs')
  .then((stored) => {
    if (stored) cache = stored;
  })
  .catch(() => {});

export function whenViewPrefsReady(): Promise<void> {
  return ready.then(() => {});
}

export function getViewPrefs(): ViewPrefs {
  return cache;
}

export function setViewPref<K extends keyof ViewPrefs>(key: K, value: ViewPrefs[K]): void {
  cache = { ...cache, [key]: value };
  void storageWrite('viewPrefs', cache);
}
