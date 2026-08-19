import React from 'react';
import { CheckCircle2, Circle } from 'lucide-react';
import { t } from '../../i18n';
import { MS } from './detailStyles';
import { SeasonDropdown, seasonLabel } from './EpisodePanel';

export function ModernDetailSeasonSection({
  seasonNumbers,
  selectedSeason,
  onSeasonChange,
  seasonWatchedMap,
  toggleSeasonWatched,
  prevSeasonDialog,
  onDismissPrevSeasonDialog,
  onConfirmPrevSeasonDialog,
}: {
  seasonNumbers: number[];
  selectedSeason: number;
  onSeasonChange: (season: number) => void;
  seasonWatchedMap: Record<number, boolean>;
  toggleSeasonWatched: () => void;
  prevSeasonDialog: { season: number; unwatchedPrev: number[] } | null;
  onDismissPrevSeasonDialog: () => void;
  onConfirmPrevSeasonDialog: (includePrev: boolean) => void;
}) {
  return (
    <>
      <div style={MS.seasonRowModern}>
        <SeasonDropdown
          seasons={seasonNumbers}
          selected={selectedSeason}
          onChange={onSeasonChange}
          buttonStyle={MS.seasonBtn}
          seasonWatched={seasonWatchedMap}
          hideButtonIndicator
        />
        <button
          onClick={toggleSeasonWatched}
          title={seasonWatchedMap[selectedSeason] ? t('detail.mark_season_unwatched') : t('detail.mark_season_watched')}
          style={MS.seasonWatchedBtn}
        >
          {seasonWatchedMap[selectedSeason] ? (
            <CheckCircle2 size={18} color="rgba(255,255,255,0.75)" />
          ) : (
            <Circle size={18} color="rgba(255,255,255,0.28)" />
          )}
          <span style={MS.seasonWatchedLabel}>
            {t(seasonWatchedMap[selectedSeason] ? 'detail.mark_season_unwatched' : 'detail.mark_season_watched')}
          </span>
        </button>
      </div>

      {prevSeasonDialog && (
        <div style={MS.overlayBackdrop} onClick={onDismissPrevSeasonDialog}>
          <div style={{ ...MS.overlaySheet, maxWidth: '25rem', padding: '1.75rem' }} onClick={(e) => e.stopPropagation()}>
            <p style={{ color: '#fff', fontSize: '0.9375rem', fontWeight: 700, margin: '0 0 0.625rem' }}>
              {t('detail.prev_seasons_dialog_title')}
            </p>
            <p style={{ color: 'rgba(255,255,255,0.55)', fontSize: '0.8125rem', margin: '0 0 1.5rem', lineHeight: '1.25rem' }}>
              {t('detail.prev_seasons_dialog_body', prevSeasonDialog.unwatchedPrev.map((s) => seasonLabel(s)).join(', '))}
            </p>
            <div style={{ display: 'flex', gap: '0.625rem', justifyContent: 'flex-end' }}>
              <button
                style={{
                  background: 'rgba(255,255,255,0.1)',
                  border: 'none',
                  color: '#fff',
                  borderRadius: '0.5rem',
                  padding: '0.5625rem 1.25rem',
                  fontSize: '0.8125rem',
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
                  color: 'var(--primary-accent-foreground-color, #fff)',
                  borderRadius: '0.5rem',
                  padding: '0.5625rem 1.25rem',
                  fontSize: '0.8125rem',
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
