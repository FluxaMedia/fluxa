import type { CSSProperties, RefObject } from 'react';
import { Cast, ChevronLeft, Link2, ListPlus, Pause, PictureInPicture2, Play, Settings } from 'lucide-react';
import { t } from '../../i18n';

interface PlayerHeaderProps {
  style: CSSProperties;
  bannerOffset: number;
  title: string;
  episodeTitle: string;
  hdrLabel: string | null;
  activeCastDeviceId: string | null;
  activeCastDeviceName: string;
  castPaused: boolean;
  castButtonRef: RefObject<HTMLButtonElement | null>;
  streamLinksButtonRef: RefObject<HTMLButtonElement | null>;
  settingsButtonRef: RefObject<HTMLButtonElement | null>;
  showSegmentMarker: boolean;
  canMarkSegments: boolean;
  onClose: () => void;
  onResetActivity: () => void;
  onToggleCastPause: () => void;
  onOpenCast: () => void;
  onToggleMiniPlayer: () => void;
  onOpenStreamLinks: () => void;
  onToggleSettings: () => void;
  onToggleSegmentMarker: () => void;
}

const iconButtonStyle: CSSProperties = { width: '2.25rem', height: '2.25rem', borderRadius: '50%', border: 'none', background: 'rgba(255,255,255,0.1)', color: '#fff', display: 'inline-flex', alignItems: 'center', justifyContent: 'center', cursor: 'pointer', flexShrink: 0 };

const hdrBadgeStyle: CSSProperties = { display: 'inline-flex', alignItems: 'center', border: '1px solid rgba(255,255,255,0.3)', borderRadius: '0.1875rem', padding: '0.0625rem 0.25rem', color: 'rgba(255,255,255,0.75)', fontSize: '0.625rem', fontWeight: 700, letterSpacing: '0.03em', flexShrink: 0 };

export function PlayerHeader({ style, bannerOffset, title, episodeTitle, hdrLabel, activeCastDeviceId, activeCastDeviceName, castPaused, castButtonRef, streamLinksButtonRef, settingsButtonRef, showSegmentMarker, canMarkSegments, onClose, onResetActivity, onToggleCastPause, onOpenCast, onToggleMiniPlayer, onOpenStreamLinks, onToggleSettings, onToggleSegmentMarker }: PlayerHeaderProps) {
  const stopAndRun = (event: React.MouseEvent, action: () => void) => {
    event.stopPropagation();
    onResetActivity();
    action();
  };

  return (
    <div style={{ ...style, position: 'absolute', top: bannerOffset, left: 0, right: 0, zIndex: 3, display: 'flex', alignItems: 'center', padding: '0.875rem 0.75rem', gap: '0.375rem' }}>
      <button onClick={(event) => stopAndRun(event, onClose)} className="fluxa-ibtn" style={iconButtonStyle} title={t('player.back')}><ChevronLeft size={22} /></button>
      <div style={{ flex: 1, minWidth: 0, padding: '0 0.375rem', overflow: 'hidden' }}>
        {(title || episodeTitle) && <div style={{ display: 'flex', alignItems: 'baseline', gap: '0.375rem', overflow: 'hidden' }}>
          {title && <span style={{ color: '#fff', fontSize: '0.9375rem', fontWeight: 700, whiteSpace: 'nowrap', overflow: 'hidden', textOverflow: 'ellipsis', flex: '0 1 auto', minWidth: 0 }}>{title}</span>}
          {title && episodeTitle && <span style={{ color: 'rgba(255,255,255,0.25)', fontSize: '0.875rem', flexShrink: 0 }}>·</span>}
          {episodeTitle && <span style={{ color: 'rgba(255,255,255,0.55)', fontSize: '0.8125rem', whiteSpace: 'nowrap', overflow: 'hidden', textOverflow: 'ellipsis', minWidth: 0 }}>{episodeTitle}</span>}
          {hdrLabel && <span style={hdrBadgeStyle}>{hdrLabel}</span>}
        </div>}
        {activeCastDeviceId && <div style={{ display: 'flex', alignItems: 'center', gap: '0.375rem', marginTop: '0.125rem' }}><Cast size={11} style={{ color: 'var(--primary-accent-color)' }} /><span style={{ color: 'rgba(255,255,255,0.55)', fontSize: '0.6875rem' }}>{t('player.casting_to', activeCastDeviceName)}</span></div>}
      </div>
      {activeCastDeviceId && <button onClick={(event) => stopAndRun(event, onToggleCastPause)} className="fluxa-ibtn" style={iconButtonStyle} title={castPaused ? t('player.play') : t('player.pause')}>{castPaused ? <Play size={18} /> : <Pause size={18} />}</button>}
      <button ref={castButtonRef} onClick={(event) => { event.stopPropagation(); onOpenCast(); }} className="fluxa-ibtn" style={iconButtonStyle} title={t('player.cast')}><Cast size={20} /></button>
      <button onClick={(event) => stopAndRun(event, onToggleMiniPlayer)} className="fluxa-ibtn" style={iconButtonStyle} title={t('player.picture_in_picture')}><PictureInPicture2 size={20} /></button>
      <button ref={streamLinksButtonRef} onClick={(event) => stopAndRun(event, onOpenStreamLinks)} className="fluxa-ibtn" style={iconButtonStyle} title={t('player.stream_links')}><Link2 size={20} /></button>
      <button ref={settingsButtonRef} onClick={(event) => stopAndRun(event, onToggleSettings)} className="fluxa-ibtn" style={iconButtonStyle} title={t('player.settings')}><Settings size={20} /></button>
      {canMarkSegments && <button onClick={(event) => stopAndRun(event, onToggleSegmentMarker)} className="fluxa-ibtn" style={{ ...iconButtonStyle, color: showSegmentMarker ? 'var(--primary-accent-color)' : '#fff' }} title={t('player.mark_segment_title')}><ListPlus size={20} /></button>}
    </div>
  );
}
