import { t } from '../../i18n';
import { Toast } from '../Toast';

interface Props {
  bannerOffset: number;
  playbackError?: string | null;
  subtitleWarning?: string[] | null;
  onClosePlayer: () => void;
  onDismissSubtitleWarning?: () => void;
}

export function PlayerStatusToasts({ bannerOffset, playbackError, subtitleWarning, onClosePlayer, onDismissSubtitleWarning }: Props) {
  const wrapperStyle = { position: 'absolute' as const, top: `calc(${bannerOffset}px + 1rem)`, right: '1rem', zIndex: 40 };
  const stopPropagation = (event: React.MouseEvent) => event.stopPropagation();
  return (
    <>
      {playbackError && (
        <div style={wrapperStyle} onMouseDown={stopPropagation} onMouseUp={stopPropagation} onClick={stopPropagation}>
          <Toast
            variant="error"
            title={t('player.playback_error_title')}
            message={t('player.playback_error_detail')}
            details={playbackError}
            detailsLabel={t('player.error_show_details')}
            detailsHideLabel={t('player.error_hide_details')}
            actions={[{ label: t('player.back'), onClick: onClosePlayer, primary: true }]}
            onClose={onClosePlayer}
          />
        </div>
      )}
      {!playbackError && subtitleWarning && subtitleWarning.length > 0 && (
        <div style={wrapperStyle} onMouseDown={stopPropagation} onMouseUp={stopPropagation} onClick={stopPropagation}>
          <Toast
            variant="warning"
            title={t('player.subtitle_addons_failed_title')}
            message={t('player.subtitle_addons_failed', subtitleWarning.join(', '))}
            onClose={onDismissSubtitleWarning}
          />
        </div>
      )}
    </>
  );
}
