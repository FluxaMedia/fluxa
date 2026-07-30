import React, { useEffect, useMemo, useRef, useState } from "react";
import {
  Bell,
} from "lucide-react";
import type { AppState, LibraryItem } from "../core/types";
import {
  refreshExternalCalendarItems,
  refreshCalendarMonth,
  refreshWatchlistAirDates,
} from "../core/libraryEffects";
import { coreInvoke } from "../core/engine";
import { t } from "../i18n";
import { Toast } from "../components/Toast";
import { CalendarDayDialog } from './calendar/CalendarDayDialog';
import { CalendarGrid } from './calendar/CalendarGrid';
import { CalendarHeader } from './calendar/CalendarHeader';
import { useCalendarArtwork } from './calendar/useCalendarArtwork';
import { calendarStyles as styles } from './calendar/calendarStyles';
import {
  buildMonthCells,
  calendarArtworkKey,
  calendarPoster,
  calendarSeriesArtworkKey,
  eventEpisodeCode,
  firstDayOfMonth,
  groupItemsByDate,
  localDateKeyFromIso,
  todayIso,
  weekdays,
  type CalendarItem,
} from './calendar/calendarUtils';

const NAV_RAIL_WIDTH = 6.5;
const CONTENT_PAD = 2.625;
const AIR_DATES_REFRESH_THROTTLE_MS = 60_000;
let lastAirDatesRefreshAt = 0;

interface Props {
  state: AppState;
  onDispatch: (actionJson: string) => void;
}

export const CalendarScreen = React.memo(
  function CalendarScreen({ state, onDispatch }: Props) {
    const [monthStart, setMonthStart] = useState(() =>
      firstDayOfMonth(new Date())
    );
    const [showCompleted, setShowCompleted] = useState(false);
    const [selectedDateIso, setSelectedDateIso] = useState<string | null>(null);
    const [isRefreshingAirDates, setIsRefreshingAirDates] = useState(false);
    const [traktCalendarError, setTraktCalendarError] = useState<string | null>(null);
    const year = monthStart.getFullYear();
    const month = monthStart.getMonth() + 1;

    useEffect(() => {
      const calendar = state.calendar as {
        year?: number;
        month?: number;
        items?: unknown[];
      } | undefined;
      if (
        calendar?.year === year && calendar?.month === month &&
        Array.isArray(calendar.items)
      ) return;
      onDispatch(
        JSON.stringify({ type: "calendarMonthRequested", year, month }),
      );
    }, [year, month]);

    useEffect(() => {
      let cancelled = false;
      refreshCalendarMonth(year, month)
        .then(() => {
          if (!cancelled) {
            onDispatch(
              JSON.stringify({ type: "calendarMonthRequested", year, month }),
            );
          }
        })
        .catch(() => {});
      return () => {
        cancelled = true;
      };
    }, [year, month]);

    useEffect(() => {
      if (
        Date.now() - lastAirDatesRefreshAt < AIR_DATES_REFRESH_THROTTLE_MS
      ) return;
      lastAirDatesRefreshAt = Date.now();
      let cancelled = false;
      setIsRefreshingAirDates(true);
      refreshWatchlistAirDates()
        .then(() => refreshExternalCalendarItems())
        .then((result) => {
          if (!cancelled) {
            setTraktCalendarError(result.traktError ?? null);
            onDispatch(
              JSON.stringify({ type: "calendarMonthRequested", year, month }),
            );
          }
        })
        .catch(() => {})
        .finally(() => {
          if (!cancelled) setIsRefreshingAirDates(false);
        });
      return () => {
        cancelled = true;
      };
    }, []);

    useEffect(() => {
      const onKeyDown = (event: KeyboardEvent) => {
        if (event.key === "Escape") setSelectedDateIso(null);
      };
      window.addEventListener("keydown", onKeyDown);
      return () => window.removeEventListener("keydown", onKeyDown);
    }, []);

    const calendar = (state.calendar ?? {}) as {
      items?: CalendarItem[];
      localItems?: CalendarItem[];
      externalItems?: CalendarItem[];
    };
    const items = useMemo(
      () => [
        ...(calendar.items ?? []),
        ...(calendar.localItems ?? []),
        ...(calendar.externalItems ?? []),
      ].map((item) => ({
        ...item,
        contentId: item.contentId ?? item.seriesId ?? item.metaId,
        seriesId: item.seriesId ?? item.contentId ?? item.metaId,
      })),
      [calendar.items, calendar.localItems, calendar.externalItems],
    );
    const completedItems =
      (state.library.lastWrite?.completed ?? state.library.completed ??
        []) as LibraryItem[];
    const [visibleItems, setVisibleItems] = useState<CalendarItem[]>([]);
    useEffect(() => {
      let active = true;
      void coreInvoke<CalendarItem[]>(
        "calendarVisibilityPlan",
        JSON.stringify({
          items,
          completedItems,
          showCompleted,
          todayIso: localDateKeyFromIso(new Date().toISOString()),
        }),
      )
        .then((plan) => {
          if (active) setVisibleItems(plan ?? []);
        })
        .catch(() => {});
      return () => {
        active = false;
      };
    }, [items, completedItems, showCompleted]);

    const cells = useMemo(() => buildMonthCells(monthStart), [monthStart]);
    const { resolvedArtwork, resolvedEpisodes } = useCalendarArtwork(visibleItems);

    const displayItems = useMemo(
      () =>
        visibleItems.map((item) => ({
          ...item,
          ...resolvedEpisodes[calendarArtworkKey(item)],
        })),
      [visibleItems, resolvedEpisodes],
    );
    const seriesArtwork = useMemo(() => {
      const artwork: Record<string, string> = {};
      for (const item of displayItems) {
        const image = calendarPoster(item, resolvedArtwork);
        const key = calendarSeriesArtworkKey(item);
        if (image && !artwork[key]) artwork[key] = image;
      }
      return artwork;
    }, [displayItems, resolvedArtwork]);
    const itemsByDate = useMemo(() => groupItemsByDate(displayItems), [
      displayItems,
    ]);
    const selectedItems = selectedDateIso
      ? itemsByDate[selectedDateIso] ?? []
      : [];

    return (
      <div style={styles.screen}>
        {traktCalendarError && (
          <div style={{ position: "fixed", top: "1rem", right: "1rem", zIndex: 100 }}>
            <Toast
              variant="error"
              title={t("calendar.trakt_error_title")}
              message={t("calendar.trakt_error_message")}
              details={traktCalendarError}
              detailsLabel={t("player.error_show_details")}
              detailsHideLabel={t("player.error_hide_details")}
              onClose={() => setTraktCalendarError(null)}
            />
          </div>
        )}
        <CalendarHeader monthStart={monthStart} isRefreshing={isRefreshingAirDates} showCompleted={showCompleted} onMonthChange={(nextMonth) => { setMonthStart(nextMonth); setSelectedDateIso(null); }} onToggleCompleted={() => { const next = !showCompleted; setShowCompleted(next); if (next) setVisibleItems(items); }} styles={styles} />

        <div style={styles.weekRow}>
          {weekdays().map((day) => (
            <div key={day} style={styles.weekday}>{day}</div>
          ))}
        </div>
        <CalendarGrid cells={cells} itemsByDate={itemsByDate} selectedDateIso={selectedDateIso} onSelectDate={setSelectedDateIso} resolvedArtwork={resolvedArtwork} seriesArtwork={seriesArtwork} styles={styles} />

        {visibleItems.length === 0 && (
          <div style={styles.empty}>
            <Bell size={18} />
            <span>
              {items.length === 0
                ? t("calendar.empty")
                : t("calendar.empty_filtered")}
            </span>
          </div>
        )}

        {selectedDateIso && <CalendarDayDialog dateIso={selectedDateIso} items={selectedItems} onClose={() => setSelectedDateIso(null)} resolvedArtwork={resolvedArtwork} seriesArtwork={seriesArtwork} styles={styles} />}
      </div>
    );
  },
  (prev, next) =>
    prev.state.calendar === next.state.calendar &&
    prev.state.library === next.state.library &&
    prev.onDispatch === next.onDispatch,
);
