import type { CSSProperties } from 'react';
import { CalendarArtwork } from './CalendarArtwork';
import { calendarPoster, eventEpisodeCode, todayIso, type CalendarCell, type CalendarItem } from './calendarUtils';
import { t } from '../../i18n';

export function CalendarGrid({
  cells,
  itemsByDate,
  selectedDateIso,
  onSelectDate,
  resolvedArtwork,
  seriesArtwork,
  styles,
}: {
  cells: CalendarCell[];
  itemsByDate: Record<string, CalendarItem[]>;
  selectedDateIso: string | null;
  onSelectDate: (dateIso: string) => void;
  resolvedArtwork: Record<string, string>;
  seriesArtwork: Record<string, string>;
  styles: Record<string, CSSProperties>;
}) {
  return (
    <div className="calendar-grid" style={styles.grid}>
      {cells.map((cell, index) => {
        const dayItems = itemsByDate[cell.dateIso] ?? [];
        const hasItems = dayItems.length > 0;
        const today = cell.dateIso === todayIso();
        const selected = cell.dateIso === selectedDateIso;
        return (
          <div
            key={cell.dateIso ?? `blank-${index}`}
            className="calendar-day"
            role="button"
            tabIndex={0}
            onClick={() => onSelectDate(cell.dateIso)}
            onKeyDown={(event) => {
              if (event.key === 'Enter' || event.key === ' ') {
                event.preventDefault();
                onSelectDate(cell.dateIso);
              }
            }}
            style={{
              ...styles.day,
              opacity: cell.isCurrentMonth ? 1 : 0.48,
              borderColor: selected
                ? 'rgba(255,255,255,0.52)'
                : today
                  ? 'rgba(255,255,255,0.34)'
                  : hasItems
                    ? 'rgba(255,255,255,0.12)'
                    : 'rgba(255,255,255,0.09)',
              background: selected ? 'rgba(255,255,255,0.085)' : hasItems ? '#111214' : '#0c0e10',
              cursor: 'pointer',
            }}
          >
            <div style={styles.dayHeader}>
              <span style={{ ...styles.dayNumber, ...(today ? styles.todayNumber : {}) }}>{cell.day}</span>
              {hasItems && <span style={styles.eventCount}>{dayItems.length}</span>}
            </div>
            <div className="calendar-day-items" style={styles.dayItems}>
              {dayItems.slice(0, 3).map((item, itemIndex) => (
                <div key={item.id ?? `${item.title}-${itemIndex}`} style={styles.event}>
                  <CalendarArtwork
                    src={calendarPoster(item, resolvedArtwork, seriesArtwork)}
                    fallbackSrc={item.seriesPoster}
                    style={styles.eventArtwork}
                    fallback={<span style={styles.eventArtworkFallback}>•</span>}
                  />
                  <span style={styles.eventTextWrap}>
                    <span style={styles.eventText}>{item.title ?? item.name ?? item.subtitle}</span>
                    <span style={styles.eventEpisode}>{eventEpisodeCode(item)}</span>
                  </span>
                </div>
              ))}
              {dayItems.length > 3 && <span style={styles.moreEvents}>{t('calendar.more_events', dayItems.length - 3)}</span>}
            </div>
          </div>
        );
      })}
    </div>
  );
}
