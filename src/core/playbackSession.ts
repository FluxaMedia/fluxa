import { coreInvoke, dispatchAction } from './engine';
import { pumpEffects } from './effectRunner';
import { scrobblePlaybackAction } from './scrobble';
import { saveProfile } from './profiles';
import type { AppState, Meta, Stream, UserProfile, Video } from './types';

export type PlaybackSnapshot = {
  timePos: number;
  duration: number;
};

export type ScrobbleEvent = 'start' | 'pause' | 'stop';

export type ScrobbleFlags = {
  hasStarted: boolean;
  hasPaused: boolean;
  hasStopped: boolean;
};

export function snapshotIsUsable(snapshot: PlaybackSnapshot | null): snapshot is PlaybackSnapshot {
  return Boolean(
    snapshot
    && Number.isFinite(snapshot.timePos)
    && Number.isFinite(snapshot.duration)
    && snapshot.duration > 0,
  );
}

export async function persistPlaybackProgress(options: {
  meta: Meta | null;
  episode: Video | null;
  stream: Stream | null;
  nextEpisode: Video | null;
  snapshot: PlaybackSnapshot;
  streamIndex: number | null;
  prefs: Record<string, unknown>;
  scrobbleTraktPause?: boolean;
  updateState: (state: Partial<AppState>) => void;
}): Promise<void> {
  const { meta, episode, stream, nextEpisode, snapshot, streamIndex, prefs, scrobbleTraktPause, updateState } = options;
  if (!meta) return;
  const plan = await coreInvoke<{ shouldScrobble: boolean; progressAction: Record<string, unknown> }>('playbackClosePlan', JSON.stringify({
    meta,
    episode,
    stream,
    nextEpisode,
    timePos: snapshot.timePos,
    duration: Math.floor(snapshot.duration),
    streamIndex,
    prefs,
    scrobbleTraktPause: scrobbleTraktPause ?? false,
  }));
  if (!plan?.shouldScrobble || !plan.progressAction) return;
  const result = await dispatchAction(JSON.stringify(plan.progressAction));
  if (!result) return;
  updateState(result.state);
  if (result.effects.length > 0) await pumpEffects(result.effects, updateState);
}

export async function runScrobbleLifecycle(options: {
  event: ScrobbleEvent;
  profile: UserProfile | null;
  meta: Meta | null;
  episode: Video | null;
  snapshot: PlaybackSnapshot;
  flags: ScrobbleFlags;
  onProfileUpdated?: (profile: UserProfile) => void;
}): Promise<ScrobbleEvent | null> {
  const { event, profile, meta, episode, snapshot, flags, onProfileUpdated } = options;
  if (!profile || !meta || !snapshotIsUsable(snapshot)) return null;
  const action = await coreInvoke<{ action: ScrobbleEvent }>('playerScrobbleLifecycleAction', JSON.stringify({
    event,
    token: profile.traktAccessToken ?? profile.simklAccessToken,
    hasStarted: flags.hasStarted,
    hasPaused: flags.hasPaused,
    hasStopped: flags.hasStopped,
    progress: (snapshot.timePos / snapshot.duration) * 100,
  }));
  if (!action) return null;
  scrobblePlaybackAction(
    profile,
    meta,
    episode,
    snapshot.timePos,
    snapshot.duration,
    action.action,
    (revoked) => { void saveProfile(revoked); onProfileUpdated?.(revoked); },
  );
  return action.action;
}
