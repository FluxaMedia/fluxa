import React, { useEffect, useMemo, useState } from 'react';
import { CaretLeft as ChevronLeft, CaretRight as ChevronRight } from '@phosphor-icons/react';
import type { AppState } from '../core/types';
import { t } from '../i18n';

const NAV_RAIL_WIDTH = 104;
const CONTENT_PAD = 42;

interface Props {
  state: AppState;
  onDispatch: (actionJson: string) => void;
}

type CalendarItem = {
  id?: string;
  title?: string;
  name?: string;
  subtitle?: string;
  dateIso?: string;
  poster?: string;
};

export function CalendarScreen({ state, onDispatch }: Props) {
  const [monthStart, setMonthStart] = useState(() => firstDayOfMonth(new Date()));
  const year = monthStart.getFullYear();
  const month = monthStart.getMonth() + 1;

  useEffect(() => {
    onDispatch(JSON.stringify({ type: 'calendarMonthRequested', year, month }));
  }, [year, month]);

  const calendarState = (state.calendar ?? {}) as {
    items?: CalendarItem[];
    localItems?: CalendarItem[];
    externalItems?: CalendarItem[];
  };
  const items = [
    ...(calendarState.items ?? []),
    ...(calendarState.localItems ?? []),
    ...(calendarState.externalItems ?? []),
  ];
  const itemsByDate = useMemo(() => groupItemsByDate(items), [items]);
  const cells = useMemo(() => buildMonthCells(monthStart), [monthStart]);

  return (
    <div style={styles.screen}>
      <header style={styles.header}>
        <div>
          <p style={styles.eyebrow}>{t('nav.calendar')}</p>
          <h1 style={styles.title}>{monthTitle(monthStart)}</h1>
        </div>
        <div style={styles.actions}>
          <button style={styles.navBtn} onClick={() => setMonthStart(shiftMonth(monthStart, -1))}>
            <ChevronLeft size={22} />
          </button>
          <button style={styles.navBtn} onClick={() => setMonthStart(shiftMonth(monthStart, 1))}>
            <ChevronRight size={22} />
          </button>
        </div>
      </header>

      <div style={styles.weekRow}>
        {weekdays().map((day) => (
          <div key={day} style={styles.weekday}>{day}</div>
        ))}
      </div>

      <div style={styles.grid}>
        {cells.map((cell) => {
          const dayItems = cell ? (itemsByDate[cell.dateIso] ?? []) : [];
          return (
            <div
              key={cell?.dateIso ?? Math.random()}
              style={{
                ...styles.day,
                opacity: cell?.isCurrentMonth ? 1 : 0.34,
              }}
            >
              {cell && <span style={styles.dayNumber}>{cell.day}</span>}
              <div style={styles.dayItems}>
                {dayItems.slice(0, 3).map((item, index) => (
                  <div key={item.id ?? `${item.title}-${index}`} style={styles.event}>
                    {item.poster && <img src={item.poster} alt="" style={styles.eventPoster} />}
                    <span style={styles.eventText}>{item.title ?? item.name ?? item.subtitle}</span>
                  </div>
                ))}
              </div>
            </div>
          );
        })}
      </div>

      {items.length === 0 && (
        <div style={styles.empty}>{t('calendar.empty')}</div>
      )}
    </div>
  );
}

function firstDayOfMonth(date: Date): Date {
  return new Date(date.getFullYear(), date.getMonth(), 1);
}

function shiftMonth(date: Date, delta: number): Date {
  return new Date(date.getFullYear(), date.getMonth() + delta, 1);
}

function monthTitle(date: Date): string {
  return date.toLocaleDateString(undefined, { month: 'long', year: 'numeric' });
}

function weekdays(): string[] {
  const base = new Date(2024, 0, 1);
  return Array.from({ length: 7 }, (_, index) =>
    new Date(base.getFullYear(), base.getMonth(), base.getDate() + index)
      .toLocaleDateString(undefined, { weekday: 'short' }),
  );
}

function buildMonthCells(monthStart: Date) {
  const first = firstDayOfMonth(monthStart);
  const leading = (first.getDay() + 6) % 7;
  const start = new Date(first);
  start.setDate(first.getDate() - leading);
  return Array.from({ length: 42 }, (_, index) => {
    const date = new Date(start);
    date.setDate(start.getDate() + index);
    return {
      day: date.getDate(),
      dateIso: date.toISOString().slice(0, 10),
      isCurrentMonth: date.getMonth() === monthStart.getMonth(),
    };
  });
}

function groupItemsByDate(items: CalendarItem[]): Record<string, CalendarItem[]> {
  return items.reduce<Record<string, CalendarItem[]>>((acc, item) => {
    const date = item.dateIso?.slice(0, 10);
    if (!date) return acc;
    acc[date] = [...(acc[date] ?? []), item];
    return acc;
  }, {});
}

const styles: Record<string, React.CSSProperties> = {
  screen: {
    minHeight: '100%',
    background: '#040508',
    color: '#fff',
    padding: `34px 42px 80px ${NAV_RAIL_WIDTH + CONTENT_PAD}px`,
    overflowY: 'auto',
    fontFamily: 'sans-serif',
  },
  header: {
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'space-between',
    marginBottom: 24,
  },
  eyebrow: {
    margin: '0 0 6px',
    color: 'rgba(255,255,255,0.5)',
    fontSize: 13,
    fontWeight: 800,
    textTransform: 'uppercase',
    letterSpacing: 0,
  },
  title: {
    margin: 0,
    fontSize: 36,
    fontWeight: 900,
    letterSpacing: 0,
  },
  actions: {
    display: 'flex',
    gap: 10,
  },
  navBtn: {
    width: 42,
    height: 42,
    borderRadius: 8,
    border: '1px solid rgba(255,255,255,0.12)',
    background: 'rgba(255,255,255,0.06)',
    color: '#fff',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    cursor: 'pointer',
  },
  weekRow: {
    display: 'grid',
    gridTemplateColumns: 'repeat(7, minmax(0, 1fr))',
    gap: 8,
    marginBottom: 8,
  },
  weekday: {
    color: 'rgba(255,255,255,0.48)',
    fontSize: 12,
    fontWeight: 800,
    paddingLeft: 8,
  },
  grid: {
    display: 'grid',
    gridTemplateColumns: 'repeat(7, minmax(0, 1fr))',
    gap: 8,
  },
  day: {
    minHeight: 104,
    borderRadius: 8,
    border: '1px solid rgba(255,255,255,0.08)',
    background: 'rgba(255,255,255,0.035)',
    padding: 8,
    overflow: 'hidden',
  },
  dayNumber: {
    color: 'rgba(255,255,255,0.72)',
    fontSize: 13,
    fontWeight: 800,
  },
  dayItems: {
    display: 'flex',
    flexDirection: 'column',
    gap: 5,
    marginTop: 8,
  },
  event: {
    display: 'flex',
    alignItems: 'center',
    gap: 6,
    minHeight: 22,
    overflow: 'hidden',
  },
  eventPoster: {
    width: 18,
    height: 22,
    objectFit: 'cover',
    borderRadius: 3,
    flexShrink: 0,
  },
  eventText: {
    color: '#fff',
    fontSize: 11,
    fontWeight: 700,
    whiteSpace: 'nowrap',
    overflow: 'hidden',
    textOverflow: 'ellipsis',
  },
  empty: {
    marginTop: 24,
    color: 'rgba(255,255,255,0.54)',
    fontSize: 15,
  },
};
