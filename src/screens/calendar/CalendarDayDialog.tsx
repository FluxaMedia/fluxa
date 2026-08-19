import { Check, Film, X } from 'lucide-react';
import type { CSSProperties } from 'react';
import { t } from '../../i18n';
import { CalendarArtwork } from './CalendarArtwork';
import { calendarPoster, eventEpisodeLabel, formatLongDate, isReleased, type CalendarItem } from './calendarUtils';

export function CalendarDayDialog({
  dateIso,
  items,
  onClose,
  resolvedArtwork,
  seriesArtwork,
  styles,
}: {
  dateIso: string;
  items: CalendarItem[];
  onClose: () => void;
  resolvedArtwork: Record<string, string>;
  seriesArtwork: Record<string, string>;
  styles: Record<string, CSSProperties>;
}) {
  return (
    <div style={styles.modalOverlay} onMouseDown={onClose}>
      <section
        className="calendar-modal"
        style={styles.modal}
        role="dialog"
        aria-modal="true"
        aria-label={formatLongDate(dateIso)}
        onMouseDown={(event) => event.stopPropagation()}
      >
        <div style={styles.modalHeader}>
          <div>
            <h2 style={styles.modalTitle}>{formatLongDate(dateIso)}</h2>
            <p style={styles.modalCount}>{t('calendar.scheduled_episodes', items.length)}</p>
          </div>
          <button style={styles.closeBtn} onClick={onClose} aria-label={t('common.close')}>
            <X size={21} />
          </button>
        </div>
        {items.length === 0 ? (
          <div style={styles.modalEmpty}>{t('calendar.empty_filtered')}</div>
        ) : (
          <div style={styles.modalList}>
            {items.map((item, index) => (
              <div key={item.id ?? `${item.title}-${index}`} className="calendar-modal-item" style={styles.modalItem}>
                <CalendarArtwork
                  src={calendarPoster(item, resolvedArtwork, seriesArtwork)}
                  fallbackSrc={item.seriesPoster}
                  style={styles.modalPoster}
                  fallback={
                    <div style={styles.modalPosterFallback}>
                      <Film size={19} />
                    </div>
                  }
                />
                <div style={styles.modalText}>
                  <span style={styles.modalItemTitle}>{item.title ?? item.name ?? item.subtitle}</span>
                  <span style={styles.modalItemMeta}>{eventEpisodeLabel(item)}</span>
                </div>
                {isReleased(item) && <Check size={19} style={styles.releaseCheck} />}
              </div>
            ))}
          </div>
        )}
      </section>
    </div>
  );
}
