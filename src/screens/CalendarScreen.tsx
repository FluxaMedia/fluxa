import React, { useEffect, useMemo, useState } from "react";
import {
  Bell,
  Check,
  ChevronLeft,
  ChevronRight,
  Eye,
  EyeOff,
  Film,
  X,
} from "lucide-react";
import type { AppState, LibraryItem, Video } from "../core/types";
import {
  refreshExternalCalendarItems,
  refreshWatchlistAirDates,
} from "../core/libraryEffects";
import { fetchMetaDetail } from "../core/detailEffects";
import { coreInvoke } from "../core/engine";
import { t } from "../i18n";
import { Toast } from "../components/Toast";

const NAV_RAIL_WIDTH = 6.5;
const CONTENT_PAD = 2.625;
const AIR_DATES_REFRESH_THROTTLE_MS = 60_000;
let lastAirDatesRefreshAt = 0;

interface Props {
  state: AppState;
  onDispatch: (actionJson: string) => void;
}

type CalendarItem = {
  id?: string;
  title?: string;
  name?: string;
  subtitle?: string;
  episodeTitle?: string;
  seasonNumber?: number;
  episodeNumber?: number;
  season?: number;
  episode?: number;
  number?: number;
  time?: string;
  airTime?: string;
  releaseTime?: string;
  dateIso?: string;
  poster?: string;
  seriesPoster?: string;
  episodePoster?: string;
  contentId?: string;
  seriesId?: string;
  metaType?: string;
};

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
      ],
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
        });
      return () => {
        active = false;
      };
    }, [items, completedItems, showCompleted]);

    const cells = useMemo(() => buildMonthCells(monthStart), [monthStart]);
    const [resolvedArtwork, setResolvedArtwork] = useState<
      Record<string, string>
    >({});
    const [resolvedEpisodes, setResolvedEpisodes] = useState<
      Record<string, Partial<CalendarItem>>
    >({});

    useEffect(() => {
      let active = true;
      const unresolved = visibleItems.filter((item) => {
        const id = item.contentId ?? item.seriesId;
        return id && !resolvedArtwork[calendarArtworkKey(item)];
      });
      if (unresolved.length === 0) return;
      const itemsBySeries = new Map<string, CalendarItem[]>();
      for (const item of unresolved) {
        const id = item.contentId ?? item.seriesId ?? "";
        itemsBySeries.set(id, [...(itemsBySeries.get(id) ?? []), item]);
      }
      void Promise.all(
        [...itemsBySeries].map(async ([id, seriesItems]) => {
          const meta = await fetchMetaDetail({
            id,
            contentType: seriesItems[0]?.metaType ?? "series",
          }) as { poster?: string; videos?: Video[] } | null;
          return seriesItems.map((item) => {
            const season = item.seasonNumber ?? item.season;
            const episode = item.episodeNumber ?? item.episode ?? item.number;
            const matchedEpisode = meta?.videos?.find((video) =>
              season != null && episode != null
                ? video.season === season &&
                  (video.episode ?? video.number) === episode
                : !!item.dateIso && !!video.released &&
                  localDateKeyFromIso(video.released) ===
                    localDateKeyFromIso(item.dateIso)
            );
            const episodePoster = matchedEpisode?.thumbnail;
            const artwork = episodePoster ?? meta?.poster;
            return {
              key: calendarArtworkKey(item),
              artwork,
              episode: matchedEpisode
                ? {
                  seasonNumber: matchedEpisode.season,
                  episodeNumber: matchedEpisode.episode ??
                    matchedEpisode.number,
                  episodeTitle: matchedEpisode.title ?? matchedEpisode.name,
                  episodePoster,
                }
                : null,
            };
          });
        }),
      ).then((entries) => {
        if (!active) return;
        const resolutions = entries.flat();
        const posters = resolutions.flatMap((entry) =>
          entry.artwork ? [[entry.key, entry.artwork] as const] : []
        );
        if (posters.length > 0) {
          setResolvedArtwork((current) => ({
            ...current,
            ...Object.fromEntries(posters),
          }));
        }
        const episodes = resolutions.flatMap((entry) =>
          entry.episode ? [[entry.key, entry.episode] as const] : []
        );
        if (episodes.length > 0) {
          setResolvedEpisodes((current) => ({
            ...current,
            ...Object.fromEntries(episodes),
          }));
        }
      });
      return () => {
        active = false;
      };
    }, [visibleItems, resolvedArtwork]);

    const displayItems = useMemo(
      () =>
        visibleItems.map((item) => ({
          ...item,
          ...resolvedEpisodes[calendarArtworkKey(item)],
        })),
      [visibleItems, resolvedEpisodes],
    );
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
        <header style={styles.header}>
          <div />
          <div style={styles.monthControls}>
            <button
              style={styles.navBtn}
              onClick={() => {
                setMonthStart(shiftMonth(monthStart, -1));
                setSelectedDateIso(null);
              }}
              aria-label={t("calendar.previous_month")}
            >
              <ChevronLeft size={21} />
            </button>
            <label style={styles.monthPicker}>
              <h1 style={styles.title}>{monthTitle(monthStart)}</h1>
              <input
                type="month"
                value={`${year}-${String(month).padStart(2, "0")}`}
                aria-label={t("calendar.choose_month")}
                style={styles.monthInput}
                onChange={(event) => {
                  const [nextYear, nextMonth] = event.currentTarget.value
                    .split("-")
                    .map(Number);
                  if (nextYear && nextMonth) {
                    setMonthStart(new Date(nextYear, nextMonth - 1, 1));
                    setSelectedDateIso(null);
                  }
                }}
              />
            </label>
            <button
              style={styles.navBtn}
              onClick={() => {
                setMonthStart(shiftMonth(monthStart, 1));
                setSelectedDateIso(null);
              }}
              aria-label={t("calendar.next_month")}
            >
              <ChevronRight size={21} />
            </button>
          </div>
          <div style={styles.actions}>
            {isRefreshingAirDates && (
              <span style={styles.refreshingLabel}>
                {t("calendar.checking_new_episodes")}
              </span>
            )}
            <button
              style={styles.filterBtn}
              onClick={() => {
                const next = !showCompleted;
                setShowCompleted(next);
                if (next) setVisibleItems(items);
              }}
              title={showCompleted
                ? t("calendar.hide_completed")
                : t("calendar.show_completed")}
              aria-label={showCompleted
                ? t("calendar.hide_completed")
                : t("calendar.show_completed")}
            >
              {showCompleted ? <Eye size={16} /> : <EyeOff size={16} />}
            </button>
          </div>
        </header>

        <div style={styles.weekRow}>
          {weekdays().map((day) => (
            <div key={day} style={styles.weekday}>{day}</div>
          ))}
        </div>
        <div style={styles.grid}>
          {cells.map((cell, index) => {
            const dayItems = cell ? itemsByDate[cell.dateIso] ?? [] : [];
            const hasItems = dayItems.length > 0;
            const today = cell?.dateIso === todayIso();
            const selected = cell?.dateIso === selectedDateIso;
            return (
              <div
                key={cell?.dateIso ?? `blank-${index}`}
                role={cell ? "button" : undefined}
                tabIndex={cell ? 0 : undefined}
                onClick={cell
                  ? () => setSelectedDateIso(cell.dateIso)
                  : undefined}
                onKeyDown={cell
                  ? (event) => {
                    if (event.key === "Enter" || event.key === " ") {
                      event.preventDefault();
                      setSelectedDateIso(cell.dateIso);
                    }
                  }
                  : undefined}
                style={{
                  ...styles.day,
                  opacity: cell?.isCurrentMonth ? 1 : 0.22,
                  borderColor: selected
                    ? "rgba(255,255,255,0.52)"
                    : today
                    ? "rgba(255,255,255,0.3)"
                    : hasItems
                    ? "rgba(255,255,255,0.055)"
                    : "transparent",
                  background: hasItems ? "#111214" : "transparent",
                  cursor: cell ? "pointer" : "default",
                }}
              >
                <CalendarArtwork
                  src={calendarPoster(dayItems[0], resolvedArtwork)}
                  fallbackSrc={dayItems[0]?.seriesPoster}
                  style={styles.dayBackdrop}
                />
                {hasItems && <div style={styles.dayShade} />}
                {cell && (
                  <div style={styles.dayHeader}>
                    <span
                      style={{
                        ...styles.dayNumber,
                        ...(today ? styles.todayNumber : {}),
                      }}
                    >
                      {cell.day}
                    </span>
                  </div>
                )}
                <div style={styles.dayItems}>
                  {dayItems.slice(0, 3).map((item, itemIndex) => (
                    <div
                      key={item.id ?? `${item.title}-${itemIndex}`}
                      style={styles.event}
                    >
                      <span style={styles.eventText}>
                        {item.title ?? item.name ?? item.subtitle}
                      </span>
                      <span style={styles.eventEpisode}>
                        {eventEpisodeCode(item)}
                      </span>
                    </div>
                  ))}
                  {dayItems.length > 3 && (
                    <span style={styles.moreEvents}>
                      {t("calendar.more_events", dayItems.length - 3)}
                    </span>
                  )}
                </div>
              </div>
            );
          })}
        </div>

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

        {selectedDateIso && (
          <div
            style={styles.modalOverlay}
            onMouseDown={() => setSelectedDateIso(null)}
          >
            <section
              style={styles.modal}
              role="dialog"
              aria-modal="true"
              aria-label={formatLongDate(selectedDateIso)}
              onMouseDown={(event) => event.stopPropagation()}
            >
              <div style={styles.modalHeader}>
                <div>
                  <h2 style={styles.modalTitle}>
                    {formatLongDate(selectedDateIso)}
                  </h2>
                  <p style={styles.modalCount}>
                    {t("calendar.scheduled_episodes", selectedItems.length)}
                  </p>
                </div>
                <button
                  style={styles.closeBtn}
                  onClick={() => setSelectedDateIso(null)}
                  aria-label={t("common.close")}
                >
                  <X size={21} />
                </button>
              </div>
              {selectedItems.length === 0
                ? (
                  <div style={styles.modalEmpty}>
                    {t("calendar.empty_filtered")}
                  </div>
                )
                : (
                  <div style={styles.modalList}>
                    {selectedItems.map((item, index) => (
                      <div
                        key={item.id ?? `${item.title}-${index}`}
                        style={styles.modalItem}
                      >
                        <CalendarArtwork
                          src={calendarPoster(item, resolvedArtwork)}
                          fallbackSrc={item.seriesPoster}
                          style={styles.modalPoster}
                          fallback={
                            <div style={styles.modalPosterFallback}>
                              <Film size={19} />
                            </div>
                          }
                        />
                        <div style={styles.modalText}>
                          <span style={styles.modalItemTitle}>
                            {item.title ?? item.name ?? item.subtitle}
                          </span>
                          <span style={styles.modalItemMeta}>
                            {eventEpisodeLabel(item)}
                          </span>
                        </div>
                        {isReleased(item) && (
                          <Check size={19} style={styles.releaseCheck} />
                        )}
                      </div>
                    ))}
                  </div>
                )}
            </section>
          </div>
        )}
      </div>
    );
  },
  (prev, next) =>
    prev.state.calendar === next.state.calendar &&
    prev.state.library === next.state.library &&
    prev.onDispatch === next.onDispatch,
);

function firstDayOfMonth(date: Date): Date {
  return new Date(date.getFullYear(), date.getMonth(), 1);
}
function shiftMonth(date: Date, delta: number): Date {
  return new Date(date.getFullYear(), date.getMonth() + delta, 1);
}
function monthTitle(date: Date): string {
  return date.toLocaleDateString(undefined, { month: "long", year: "numeric" });
}
function weekdays(): string[] {
  return Array.from(
    { length: 7 },
    (_, index) =>
      new Date(2024, 0, 1 + index).toLocaleDateString(undefined, {
        weekday: "short",
      }),
  );
}
function localDateKey(date: Date): string {
  return `${date.getFullYear()}-${
    String(date.getMonth() + 1).padStart(2, "0")
  }-${String(date.getDate()).padStart(2, "0")}`;
}
function todayIso(): string {
  return localDateKey(new Date());
}
function localDateKeyFromIso(dateIso: string): string {
  if (/^\d{4}-\d{2}-\d{2}$/.test(dateIso)) return dateIso;
  const date = new Date(dateIso);
  return Number.isNaN(date.getTime())
    ? dateIso.slice(0, 10)
    : localDateKey(date);
}
function formatLongDate(dateIso: string): string {
  const [year, month, day] = dateIso.split("-").map(Number);
  return new Date(year, month - 1, day).toLocaleDateString(undefined, {
    weekday: "long",
    month: "long",
    day: "numeric",
    year: "numeric",
  });
}
function eventEpisodeLabel(item: CalendarItem): string {
  const episodeCode = eventEpisodeCode(item);
  const episodeTitle = item.episodeTitle?.trim();
  const subtitle = item.subtitle?.trim();
  const detail = episodeTitle ||
    (subtitle && subtitle.toLowerCase() !== "episode" ? subtitle : "");
  const time = item.airTime ?? item.releaseTime ?? item.time ?? "";
  return [episodeCode, detail, time].filter(Boolean).join(" • ") ||
    t("calendar.episode");
}
function eventEpisodeCode(item: CalendarItem): string {
  const season = item.seasonNumber ?? item.season;
  const episode = item.episodeNumber ?? item.episode ?? item.number;
  if (season != null && episode != null) return `S${season} • E${episode}`;
  if (season != null) return `S${season}`;
  if (episode != null) return `E${episode}`;
  return t("calendar.episode");
}
function isReleased(item: CalendarItem): boolean {
  return !!item.dateIso && localDateKeyFromIso(item.dateIso) <= todayIso();
}

function buildMonthCells(monthStart: Date) {
  const first = firstDayOfMonth(monthStart);
  const leading = (first.getDay() + 6) % 7;
  const last = new Date(first.getFullYear(), first.getMonth() + 1, 0);
  const trailing = (7 - ((leading + last.getDate()) % 7)) % 7;
  const start = new Date(first);
  start.setDate(first.getDate() - leading);
  return Array.from(
    { length: leading + last.getDate() + trailing },
    (_, index) => {
      const date = new Date(start);
      date.setDate(start.getDate() + index);
      return {
        day: date.getDate(),
        dateIso: localDateKey(date),
        isCurrentMonth: date.getMonth() === monthStart.getMonth(),
      };
    },
  );
}

function groupItemsByDate(
  items: CalendarItem[],
): Record<string, CalendarItem[]> {
  return items.reduce<Record<string, CalendarItem[]>>((grouped, item) => {
    if (!item.dateIso) return grouped;
    const date = localDateKeyFromIso(item.dateIso);
    grouped[date] = [...(grouped[date] ?? []), item];
    return grouped;
  }, {});
}

function CalendarArtwork({
  src,
  fallbackSrc,
  style,
  fallback = null,
}: {
  src?: string;
  fallbackSrc?: string;
  style: React.CSSProperties;
  fallback?: React.ReactNode;
}) {
  const [currentSrc, setCurrentSrc] = useState(src ?? fallbackSrc);

  useEffect(() => {
    setCurrentSrc(src ?? fallbackSrc);
  }, [src, fallbackSrc]);

  if (!currentSrc) return <>{fallback}</>;
  return (
    <img
      key={currentSrc}
      src={currentSrc}
      alt=""
      style={style}
      onError={() =>
        setCurrentSrc((previous) =>
          previous === fallbackSrc ? undefined : fallbackSrc
        )}
    />
  );
}

function calendarPoster(
  item: CalendarItem | undefined,
  resolved: Record<string, string>,
): string | undefined {
  if (!item) return undefined;
  return resolved[calendarArtworkKey(item)] ?? item.episodePoster ??
    item.poster ?? item.seriesPoster;
}

function calendarArtworkKey(item: CalendarItem): string {
  const id = item.contentId ?? item.seriesId ?? item.id ?? "";
  return `${id}:${item.dateIso ?? ""}`;
}

const styles: Record<string, React.CSSProperties> = {
  screen: {
    height: "100%",
    background: "#050608",
    color: "#fff",
    padding: `0 2.625rem 3rem ${NAV_RAIL_WIDTH + CONTENT_PAD}rem`,
    overflowY: "auto",
    boxSizing: "border-box",
  },
  header: {
    height: "3.5rem",
    display: "grid",
    gridTemplateColumns: "1fr auto 1fr",
    alignItems: "center",
    borderBottom: "1px solid rgba(255,255,255,0.06)",
    marginBottom: "0.75rem",
  },
  title: {
    margin: 0,
    fontSize: "1.4rem",
    fontWeight: 800,
    color: "rgba(255,255,255,0.82)",
    whiteSpace: "nowrap",
  },
  monthControls: {
    display: "flex",
    alignItems: "center",
    justifyContent: "center",
    gap: "0.75rem",
  },
  monthPicker: {
    minWidth: "11rem",
    padding: "0.4rem 0.8rem",
    position: "relative",
    display: "grid",
    placeItems: "center",
    borderRadius: "0.75rem",
    cursor: "pointer",
  },
  monthInput: {
    position: "absolute",
    inset: 0,
    width: "100%",
    height: "100%",
    opacity: 0,
    cursor: "pointer",
  },
  actions: {
    justifySelf: "end",
    display: "flex",
    alignItems: "center",
    gap: "0.375rem",
  },
  refreshingLabel: {
    fontSize: "0.6875rem",
    color: "rgba(255,255,255,0.42)",
    marginRight: "0.375rem",
  },
  navBtn: {
    width: "2.25rem",
    height: "2.25rem",
    borderRadius: "50%",
    border: "1px solid rgba(255,255,255,0.11)",
    background: "rgba(255,255,255,0.025)",
    color: "rgba(255,255,255,0.65)",
    display: "grid",
    placeItems: "center",
    cursor: "pointer",
  },
  filterBtn: {
    width: "2.25rem",
    height: "2.25rem",
    display: "grid",
    placeItems: "center",
    borderRadius: "50%",
    border: "1px solid rgba(255,255,255,0.11)",
    background: "rgba(255,255,255,0.025)",
    color: "rgba(255,255,255,0.62)",
    cursor: "pointer",
  },
  weekRow: {
    display: "grid",
    gridTemplateColumns: "repeat(7, minmax(0, 1fr))",
    gap: "0.5rem",
    marginBottom: "0.5rem",
  },
  weekday: {
    color: "rgba(255,255,255,0.67)",
    fontSize: "0.875rem",
    fontWeight: 750,
    padding: "0.25rem 0.75rem",
  },
  grid: {
    display: "grid",
    gridTemplateColumns: "repeat(7, minmax(0, 1fr))",
    gap: "0.5rem",
  },
  day: {
    minHeight: "10.6rem",
    borderRadius: "0.45rem",
    border: "1px solid rgba(255,255,255,0.055)",
    background: "#111214",
    overflow: "hidden",
    position: "relative",
    isolation: "isolate",
    padding: "0.65rem 0.75rem",
    boxSizing: "border-box",
  },
  dayBackdrop: {
    position: "absolute",
    inset: 0,
    width: "100%",
    height: "100%",
    objectFit: "cover",
    zIndex: -2,
  },
  dayShade: {
    position: "absolute",
    inset: 0,
    background:
      "linear-gradient(180deg, rgba(4,5,7,0.1), rgba(4,5,7,0.68) 47%, rgba(4,5,7,0.95))",
    zIndex: -1,
  },
  dayHeader: {
    display: "flex",
    alignItems: "center",
    justifyContent: "space-between",
    minHeight: "1.5rem",
  },
  dayNumber: {
    color: "rgba(255,255,255,0.52)",
    fontSize: "0.875rem",
    fontWeight: 700,
  },
  todayNumber: {
    width: "2rem",
    height: "2rem",
    display: "grid",
    placeItems: "center",
    borderRadius: "50%",
    background: "#fff",
    color: "#111",
  },
  dayItems: {
    display: "flex",
    flexDirection: "column",
    gap: "0.36rem",
    marginTop: "0.6rem",
    position: "relative",
  },
  event: {
    display: "flex",
    alignItems: "baseline",
    gap: "0.45rem",
    minWidth: 0,
  },
  eventText: {
    minWidth: 0,
    flex: 1,
    color: "rgba(255,255,255,0.88)",
    fontSize: "0.79rem",
    fontWeight: 700,
    whiteSpace: "nowrap",
    overflow: "hidden",
    textOverflow: "ellipsis",
  },
  eventEpisode: {
    color: "rgba(255,255,255,0.56)",
    fontSize: "0.8rem",
    fontWeight: 700,
    whiteSpace: "nowrap",
  },
  moreEvents: {
    color: "rgba(255,255,255,0.48)",
    fontSize: "0.72rem",
    fontWeight: 700,
    marginTop: "0.05rem",
  },
  empty: {
    marginTop: "1.5rem",
    color: "rgba(255,255,255,0.54)",
    fontSize: "0.9375rem",
    display: "flex",
    alignItems: "center",
    gap: "0.5rem",
  },
  modalOverlay: {
    position: "fixed",
    inset: 0,
    zIndex: 50,
    display: "grid",
    placeItems: "center",
    padding: "1.5rem",
    background: "rgba(0,0,0,0.73)",
    backdropFilter: "blur(4px)",
  },
  modal: {
    width: "min(56rem, calc(100vw - 3rem))",
    maxHeight: "min(42rem, calc(100vh - 3rem))",
    overflowY: "auto",
    borderRadius: "1.25rem",
    border: "1px solid rgba(255,255,255,0.08)",
    background: "#111216",
    boxShadow: "0 1.5rem 5rem rgba(0,0,0,0.55)",
    padding: "1.65rem 1.75rem",
  },
  modalHeader: {
    display: "flex",
    justifyContent: "space-between",
    alignItems: "flex-start",
    gap: "1rem",
    marginBottom: "1.25rem",
  },
  modalTitle: {
    margin: 0,
    fontSize: "1.65rem",
    lineHeight: 1.15,
    color: "rgba(255,255,255,0.85)",
  },
  modalCount: {
    margin: "0.35rem 0 0",
    color: "rgba(255,255,255,0.5)",
    fontSize: "0.95rem",
    fontWeight: 600,
  },
  closeBtn: {
    width: "2rem",
    height: "2rem",
    border: "none",
    background: "transparent",
    color: "rgba(255,255,255,0.62)",
    display: "grid",
    placeItems: "center",
    cursor: "pointer",
  },
  modalEmpty: { color: "rgba(255,255,255,0.48)", padding: "1rem 0" },
  modalList: { display: "flex", flexDirection: "column", gap: "1.15rem" },
  modalItem: {
    display: "flex",
    alignItems: "center",
    gap: "1.35rem",
    minHeight: "6.75rem",
    padding: "0.75rem",
    borderRadius: "0.85rem",
    border: "1px solid rgba(255,255,255,0.055)",
    background: "rgba(255,255,255,0.025)",
  },
  modalPoster: {
    width: "12rem",
    height: "6.75rem",
    borderRadius: "0.65rem",
    objectFit: "cover",
    flexShrink: 0,
  },
  modalPosterFallback: {
    width: "12rem",
    height: "6.75rem",
    borderRadius: "0.65rem",
    background: "rgba(255,255,255,0.06)",
    color: "rgba(255,255,255,0.42)",
    display: "grid",
    placeItems: "center",
    flexShrink: 0,
  },
  modalText: {
    minWidth: 0,
    flex: 1,
    display: "flex",
    flexDirection: "column",
    gap: "0.45rem",
  },
  modalItemTitle: {
    color: "rgba(255,255,255,0.82)",
    fontSize: "1.05rem",
    fontWeight: 720,
    lineHeight: 1.35,
  },
  modalItemMeta: {
    color: "rgba(255,255,255,0.5)",
    fontSize: "0.93rem",
    fontWeight: 600,
  },
  releaseCheck: { color: "rgba(255,255,255,0.54)", flexShrink: 0 },
};
