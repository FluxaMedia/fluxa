import { useEffect, type Dispatch, type MutableRefObject, type SetStateAction } from 'react';
import { listen } from '@tauri-apps/api/event';

export function usePlayerTitleReset({ setTitle, setEpisodeTitle, setAbLoopStage, setNextEpisodeDismissed, autoSkippedKeysRef, stallCountRef, prevPausedForCacheRef, bufferHistoryRef, networkHistoryRef, resetTorrentSpeedHistory }: { setTitle: Dispatch<SetStateAction<string>>; setEpisodeTitle: Dispatch<SetStateAction<string>>; setAbLoopStage: Dispatch<SetStateAction<'none' | 'a' | 'ab'>>; setNextEpisodeDismissed: Dispatch<SetStateAction<boolean>>; autoSkippedKeysRef: MutableRefObject<Set<string>>; stallCountRef: MutableRefObject<number>; prevPausedForCacheRef: MutableRefObject<boolean>; bufferHistoryRef: MutableRefObject<number[]>; networkHistoryRef: MutableRefObject<number[]>; resetTorrentSpeedHistory: () => void }) {
  useEffect(() => {
    let cancelled = false;
    void listen<{ title?: string; episodeTitle?: string }>('native-player-title', (event) => {
      if (cancelled) return;
      setTitle(event.payload.title ?? '');
      setEpisodeTitle(event.payload.episodeTitle ?? '');
      setAbLoopStage('none');
      setNextEpisodeDismissed(false);
      autoSkippedKeysRef.current.clear();
      stallCountRef.current = 0;
      prevPausedForCacheRef.current = false;
      bufferHistoryRef.current = [];
      networkHistoryRef.current = [];
      resetTorrentSpeedHistory();
    }).catch(() => undefined);
    return () => { cancelled = true; };
  }, [autoSkippedKeysRef, bufferHistoryRef, networkHistoryRef, prevPausedForCacheRef, resetTorrentSpeedHistory, setAbLoopStage, setEpisodeTitle, setNextEpisodeDismissed, setTitle, stallCountRef]);
}
