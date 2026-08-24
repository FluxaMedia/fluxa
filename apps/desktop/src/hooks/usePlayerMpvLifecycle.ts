import { useCallback, type Dispatch, type MutableRefObject, type SetStateAction } from 'react';
import { appPrefs } from '../core/appPrefs';
import {
  embeddedMpvAddSubtitle,
  embeddedMpvApplyPreferences,
  embeddedMpvLoad,
  embeddedMpvSetHttpHeaders,
  embeddedMpvSetLoadingArtwork,
  embeddedMpvSetTitle,
  embeddedMpvShowLoading,
  initEmbeddedMpv,
  playerTorrentSiblingSubtitles,
} from '../core/mpvPlayer';
import { runWithConcurrency } from '../core/fetchPlanning';
import type { PlayerArtwork, PlayerDisplayTitle, PlayerSubtitleSource } from '../core/playerUtils';
import type { ResolvedSubtitles } from '../core/subtitles';
import type { AppState } from '../core/types';

// mpv downloads each track as it is added, and subtitle hosts answer a rapid
// run of requests with 429s that fail every track. Only the preferred track --
// resolveSubtitles has already moved it to the front -- and local files are
// loaded up front; the rest are added when the viewer picks one.
const SUBTITLE_ADD_CONCURRENCY = 4;

function isLocalSubtitle(url: string): boolean {
  return !/^https?:\/\//i.test(url) || /^https?:\/\/(127\.0\.0\.1|localhost)[:/]/i.test(url);
}

function eagerSubtitles<T extends { url: string }>(subtitles: T[]): T[] {
  let remoteAllowance = 1;
  return subtitles.filter((subtitle) => isLocalSubtitle(subtitle.url) || remoteAllowance-- > 0);
}

type Options = {
  stateRef: MutableRefObject<AppState>;
  mpvInitializedRef: MutableRefObject<boolean>;
  playerUsesTorrentRef: MutableRefObject<boolean>;
  inNativePlayerRef: MutableRefObject<boolean>;
  artworkPrefetchRef: MutableRefObject<Promise<unknown> | null>;
  pendingArtworkRef: MutableRefObject<PlayerArtwork | null>;
  isCancelled: (generation: number) => boolean;
  stopTorrent: () => Promise<unknown>;
  debugLog: (message: string) => void;
  setPlayerTitle: Dispatch<SetStateAction<string | undefined>>;
  setPlayerUrl: Dispatch<SetStateAction<string | null>>;
  setPlayerUsesTorrent: Dispatch<SetStateAction<boolean>>;
  setPlayerSubtitleUrl: Dispatch<SetStateAction<string | undefined>>;
  setPlayerSubtitles: Dispatch<SetStateAction<PlayerSubtitleSource[]>>;
  setPlayerSubtitleWarning: Dispatch<SetStateAction<string[] | null>>;
};

export function usePlayerMpvLifecycle(options: Options) {
  const {
    stateRef,
    mpvInitializedRef,
    playerUsesTorrentRef,
    inNativePlayerRef,
    artworkPrefetchRef,
    pendingArtworkRef,
    isCancelled,
    stopTorrent,
    debugLog,
    setPlayerTitle,
    setPlayerUrl,
    setPlayerUsesTorrent,
    setPlayerSubtitleUrl,
    setPlayerSubtitles,
    setPlayerSubtitleWarning,
  } = options;

  return useCallback(
    async (
      generation: number,
      url: string,
      title: PlayerDisplayTitle | undefined,
      usesTorrent: boolean,
      subtitlesPromise: Promise<ResolvedSubtitles>,
      artworkPromise: Promise<unknown> | undefined,
      resumeAtSeconds?: number,
      totalDurationSeconds?: number,
      httpHeaders?: Record<string, string>,
      isAnimePlayback?: boolean,
    ) => {
      if (isCancelled(generation)) return;
      if (!usesTorrent && playerUsesTorrentRef.current) await stopTorrent();
      setPlayerTitle(title?.contentTitle);
      setPlayerUrl(url);
      setPlayerUsesTorrent(usesTorrent);
      if (!mpvInitializedRef.current) {
        await initEmbeddedMpv();
        mpvInitializedRef.current = true;
      }
      if (isCancelled(generation)) return;
      await embeddedMpvApplyPreferences({
        ...appPrefs(stateRef.current),
        isTorrentPlayback: usesTorrent,
        isAnimePlayback: !!isAnimePlayback,
      }).catch(() => undefined);
      await embeddedMpvSetTitle(title?.contentTitle, title?.episodeLine).catch(() => undefined);
      if (isCancelled(generation)) return;
      if (!inNativePlayerRef.current) {
        if (artworkPrefetchRef.current)
          await Promise.race([artworkPrefetchRef.current, new Promise<void>((resolve) => setTimeout(resolve, 2000))]);
        if (isCancelled(generation)) return;
        inNativePlayerRef.current = true;
        await embeddedMpvShowLoading(title?.contentTitle ?? 'Fluxa', title?.episodeLine);
        void embeddedMpvSetLoadingArtwork(
          title?.contentTitle ?? 'Fluxa',
          title?.episodeLine,
          pendingArtworkRef.current?.background,
          pendingArtworkRef.current?.logo,
        ).catch(() => undefined);
        if (isCancelled(generation)) return;
      } else if (artworkPromise) {
        await Promise.race([artworkPromise, new Promise<void>((resolve) => setTimeout(resolve, 2000))]);
        if (isCancelled(generation)) return;
      }
      await embeddedMpvSetHttpHeaders(httpHeaders).catch(() => undefined);
      await embeddedMpvLoad(url, resumeAtSeconds, totalDurationSeconds);
      const { subtitles, failedAddons } = await subtitlesPromise.catch(() => ({ subtitles: [], failedAddons: [] }) as ResolvedSubtitles);
      if (isCancelled(generation)) return;
      if (usesTorrent) {
        const seenUrls = new Set(subtitles.map((subtitle) => subtitle.url));
        const siblingSubtitles = await playerTorrentSiblingSubtitles().catch(() => []);
        for (const sibling of siblingSubtitles) {
          if (!sibling.url || seenUrls.has(sibling.url)) continue;
          seenUrls.add(sibling.url);
          subtitles.push({
            url: sibling.url,
            label: sibling.title ?? sibling.language ?? '',
            lang: sibling.language,
            addonName: 'Torrent',
          });
        }
      }
      if (isCancelled(generation)) return;
      const addedSubtitles = eagerSubtitles(subtitles);
      setPlayerSubtitles(subtitles);
      debugLog(
        `subtitles: resolved=${subtitles.length}, added=${addedSubtitles.length}, failedAddons=${failedAddons.join(',') || 'none'}`,
      );
      setPlayerSubtitleUrl(subtitles.find((subtitle) => /^https?:\/\//i.test(subtitle.url))?.url);
      const failedTrackAddons: string[] = [];
      await runWithConcurrency(addedSubtitles, SUBTITLE_ADD_CONCURRENCY, (subtitle) =>
        embeddedMpvAddSubtitle(subtitle.url, subtitle.addonName ?? subtitle.label, subtitle.lang).catch(() => {
          if (subtitle.addonName) failedTrackAddons.push(subtitle.addonName);
        }),
      );
      if (isCancelled(generation)) return;
      if (!isCancelled(generation))
        setPlayerSubtitleWarning(
          Array.from(new Set([...failedAddons, ...failedTrackAddons])).length
            ? Array.from(new Set([...failedAddons, ...failedTrackAddons]))
            : null,
        );
    },
    [
      artworkPrefetchRef,
      debugLog,
      inNativePlayerRef,
      isCancelled,
      mpvInitializedRef,
      pendingArtworkRef,
      playerUsesTorrentRef,
      setPlayerSubtitleUrl,
      setPlayerSubtitles,
      setPlayerSubtitleWarning,
      setPlayerTitle,
      setPlayerUrl,
      setPlayerUsesTorrent,
      stateRef,
      stopTorrent,
    ],
  );
}
