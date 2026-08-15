import { platformEmit as emit } from '../../platform/browser';
import type { RefObject } from 'react';
import type { Video } from '../../core/types';
import { NextEpCard } from './NextEpCard';
import { EpisodePanel, type EpisodeInfo } from './EpisodePanel';
import { PlayerFeedback } from './PlayerFeedback';
import { PlayerSkipPrompt } from './PlayerSkipPrompt';
import type { ActiveSkip, FeedbackFlash } from './PlayerOverlayPrimitives';

export function PlayerOverlayDecorations({ controlsVisible, feedback, muted, volumeLevel, showSeekOverlay, activeSkip, showNextEpCard, nextEpSubtitle, nextEpThumbnail, countdown, autoPlayCountdownSecs, showEpisodePanel, episodes, currentEpisode, skipFillRef, onActivity, onDismissSkip, onDismissNextEpisode, onCloseEpisodePanel }: { controlsVisible: boolean; feedback: FeedbackFlash | null; muted: boolean; volumeLevel: number; showSeekOverlay: boolean; activeSkip: ActiveSkip | null; showNextEpCard: boolean; nextEpSubtitle: string | null; nextEpThumbnail: string | null; countdown: number | null; autoPlayCountdownSecs: number; showEpisodePanel: boolean; episodes: EpisodeInfo[]; currentEpisode?: Video | null; skipFillRef: RefObject<HTMLDivElement | null>; onActivity: () => void; onDismissSkip: () => void; onDismissNextEpisode: () => void; onCloseEpisodePanel: () => void }) {
  return <>
    <div style={{ position: 'absolute', left: 0, right: 0, top: 0, height: '8.75rem', background: 'linear-gradient(to bottom, rgba(0,0,0,0.7) 0%, transparent 100%)', zIndex: 1, opacity: controlsVisible ? 1 : 0, transition: 'opacity 0.4s ease', pointerEvents: 'none' }} />
    <div style={{ position: 'absolute', left: 0, right: 0, bottom: 0, height: '14.375rem', background: 'linear-gradient(to top, rgba(0,0,0,0.88) 0%, rgba(0,0,0,0.5) 45%, transparent 100%)', zIndex: 1, opacity: controlsVisible ? 1 : 0, transition: 'opacity 0.4s ease', pointerEvents: 'none' }} />
    <PlayerFeedback feedback={feedback} muted={muted} volumeLevel={volumeLevel} />
    {showSeekOverlay && <div style={{ position: 'absolute', inset: 0, zIndex: 4, pointerEvents: 'none', display: 'flex', alignItems: 'center', justifyContent: 'center' }}><div style={{ width: '2.25rem', height: '2.25rem', borderRadius: '50%', border: '0.1875rem solid rgba(255,255,255,0.15)', borderTopColor: 'rgba(255,255,255,0.75)', animation: 'fluxa-seek-spin 0.75s linear infinite' }} /></div>}
    {!showNextEpCard && <PlayerSkipPrompt activeSkip={activeSkip} skipFillRef={skipFillRef} onDismiss={onDismissSkip} onActivity={onActivity} />}
    {showNextEpCard && nextEpSubtitle && !showEpisodePanel && <NextEpCard subtitle={nextEpSubtitle} thumbnail={nextEpThumbnail} countdown={countdown ?? 0} countdownTotal={autoPlayCountdownSecs} bottom={106} onPlay={() => { onActivity(); void emit('native-player-next-episode', null); }} onDismiss={onDismissNextEpisode} />}
    {showEpisodePanel && <EpisodePanel episodes={episodes} currentEpisode={currentEpisode ?? null} onClose={onCloseEpisodePanel} />}
  </>;
}
