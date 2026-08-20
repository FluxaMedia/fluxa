import React from 'react';
import { color, fade, fontSize, radius } from '../../design';
import { CheckCircle2, Circle } from 'lucide-react';
import { t } from '../../i18n';
import { MS } from './detailStyles';
import { SeasonDropdown, seasonLabel } from './EpisodePanel';

export function SeasonControls({
  seasonNumbers,
  selectedSeason,
  onSeasonChange,
  seasonWatchedMap,
  toggleSeasonWatched,
}: {
  seasonNumbers: number[];
  selectedSeason: number;
  onSeasonChange: (season: number) => void;
  seasonWatchedMap: Record<number, boolean>;
  toggleSeasonWatched: () => void;
}) {
  const watched = seasonWatchedMap[selectedSeason] === true;
  return (
    <>
      <button
        onClick={toggleSeasonWatched}
        title={watched ? t('detail.mark_season_unwatched') : t('detail.mark_season_watched')}
        aria-label={watched ? t('detail.mark_season_unwatched') : t('detail.mark_season_watched')}
        style={MS.seasonWatchedBtn}
      >
        {watched ? <CheckCircle2 size={16} color={color.textBody} /> : <Circle size={16} color={color.textFaint} />}
      </button>
      <SeasonDropdown
        seasons={seasonNumbers}
        selected={selectedSeason}
        onChange={onSeasonChange}
        buttonStyle={MS.seasonBtn}
        seasonWatched={seasonWatchedMap}
        hideButtonIndicator
      />
    </>
  );
}

export function PrevSeasonDialog({
  prevSeasonDialog,
  onDismissPrevSeasonDialog,
  onConfirmPrevSeasonDialog,
}: {
  prevSeasonDialog: { season: number; unwatchedPrev: number[] } | null;
  onDismissPrevSeasonDialog: () => void;
  onConfirmPrevSeasonDialog: (includePrev: boolean) => void;
}) {
  return (
    <>
      {prevSeasonDialog && (
        <div style={MS.overlayBackdrop} onClick={onDismissPrevSeasonDialog}>
          <div style={{ ...MS.overlaySheet, maxWidth: '25rem', padding: '1.75rem' }} onClick={(e) => e.stopPropagation()}>
            <p style={{ color: color.textPrimary, fontSize: fontSize.md, fontWeight: 700, margin: '0 0 0.625rem' }}>
              {t('detail.prev_seasons_dialog_title')}
            </p>
            <p style={{ color: color.textMuted, fontSize: fontSize.base, margin: '0 0 1.5rem', lineHeight: '1.25rem' }}>
              {t('detail.prev_seasons_dialog_body', prevSeasonDialog.unwatchedPrev.map((s) => seasonLabel(s)).join(', '))}
            </p>
            <div style={{ display: 'flex', gap: '0.625rem', justifyContent: 'flex-end' }}>
              <button
                style={{
                  background: color.fillHover,
                  border: 'none',
                  color: color.textPrimary,
                  borderRadius: radius.md,
                  padding: '0.5625rem 1.25rem',
                  fontSize: fontSize.base,
                  fontWeight: 600,
                  cursor: 'pointer',
                }}
                onClick={() => onConfirmPrevSeasonDialog(false)}
              >
                {t('detail.prev_seasons_dialog_no')}
              </button>
              <button
                style={{
                  background: 'var(--primary-accent-color)',
                  border: 'none',
                  color: `var(--primary-accent-foreground-color, ${color.textPrimary})`,
                  borderRadius: radius.md,
                  padding: '0.5625rem 1.25rem',
                  fontSize: fontSize.base,
                  fontWeight: 600,
                  cursor: 'pointer',
                }}
                onClick={() => onConfirmPrevSeasonDialog(true)}
              >
                {t('detail.prev_seasons_dialog_yes')}
              </button>
            </div>
          </div>
        </div>
      )}
    </>
  );
}
