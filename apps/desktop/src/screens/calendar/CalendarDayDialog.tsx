import { Check, Film, X } from 'lucide-react';
import type { CSSProperties } from 'react';
import { useEffect, useState } from 'react';
import { coreReleaseDateReleased } from '../../core/engineCoreContent';
import { t } from '../../i18n';
import { CalendarArtwork } from './CalendarArtwork';
import { calendarPoster, eventEpisodeLabel, formatLongDate, type CalendarItem } from './calendarUtils';

export function CalendarDayPanel({
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
  const [releasedItems, setReleasedItems] = useState<boolean[]>([]);

  useEffect(() => {
    let active = true;
    void Promise.all(items.map((item) => (item.dateIso ? coreReleaseDateReleased(item.dateIso) : Promise.resolve(false))))
      .then((values) => {
        if (active) setReleasedItems(values);
      });
    return () => {
      active = false;
    };
  }, [items]);

  return (
      <aside
        className="calendar-day-panel"
        style={styles.sidePanel}
        role="complementary"
        aria-label={formatLongDate(dateIso)}
      >
        <div style={styles.sidePanelHeader}>
          <div>
            <h2 style={styles.sidePanelTitle}>{formatLongDate(dateIso)}</h2>
            <p style={styles.sidePanelCount}>{t('calendar.scheduled_episodes', items.length)}</p>
          </div>
          <button style={styles.sidePanelClose} onClick={onClose} aria-label={t('common.close')}>
            <X size={21} />
          </button>
        </div>
        {items.length === 0 ? (
          <div style={styles.sidePanelEmpty}>{t('calendar.empty_filtered')}</div>
        ) : (
          <div style={styles.sidePanelList}>
            {items.map((item, index) => (
              <div key={item.id ?? `${item.title}-${index}`} className="calendar-panel-item" style={styles.sidePanelItem}>
                <CalendarArtwork
                  src={calendarPoster(item, resolvedArtwork, seriesArtwork)}
                  fallbackSrc={item.seriesPoster}
                  style={styles.sidePanelPoster}
                  fallback={
                    <div style={styles.sidePanelPosterFallback}>
                      <Film size={19} />
                    </div>
                  }
                />
                <div style={styles.sidePanelText}>
                  <span style={styles.sidePanelItemTitle}>{item.title ?? item.name ?? item.subtitle}</span>
                  <span style={styles.sidePanelItemMeta}>{eventEpisodeLabel(item)}</span>
                </div>
                {releasedItems[index] && <Check size={19} style={styles.sidePanelReleaseCheck} />}
              </div>
            ))}
          </div>
        )}
      </aside>
  );
}
