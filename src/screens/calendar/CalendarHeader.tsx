import { ChevronLeft, ChevronRight, Eye, EyeOff } from 'lucide-react';
import type { CSSProperties } from 'react';
import { t } from '../../i18n';
import { monthTitle, shiftMonth } from './calendarUtils';

export function CalendarHeader({ monthStart, isRefreshing, showCompleted, onMonthChange, onToggleCompleted, styles }: { monthStart: Date; isRefreshing: boolean; showCompleted: boolean; onMonthChange: (month: Date) => void; onToggleCompleted: () => void; styles: Record<string, CSSProperties> }) {
  const year = monthStart.getFullYear();
  const month = monthStart.getMonth() + 1;
  return <header style={styles.header}>
    <div />
    <div style={styles.monthControls}>
      <button style={styles.navBtn} onClick={() => onMonthChange(shiftMonth(monthStart, -1))} aria-label={t('calendar.previous_month')}><ChevronLeft size={21} /></button>
      <label style={styles.monthPicker}>
        <h1 style={styles.title}>{monthTitle(monthStart)}</h1>
        <input type="month" value={`${year}-${String(month).padStart(2, '0')}`} aria-label={t('calendar.choose_month')} style={styles.monthInput} onChange={(event) => {
          const [nextYear, nextMonth] = event.currentTarget.value.split('-').map(Number);
          if (nextYear && nextMonth) onMonthChange(new Date(nextYear, nextMonth - 1, 1));
        }} />
      </label>
      <button style={styles.navBtn} onClick={() => onMonthChange(shiftMonth(monthStart, 1))} aria-label={t('calendar.next_month')}><ChevronRight size={21} /></button>
    </div>
    <div style={styles.actions}>
      {isRefreshing && <span style={styles.refreshingLabel}>{t('calendar.checking_new_episodes')}</span>}
      <button style={styles.filterBtn} onClick={onToggleCompleted} title={showCompleted ? t('calendar.hide_completed') : t('calendar.show_completed')} aria-label={showCompleted ? t('calendar.hide_completed') : t('calendar.show_completed')}>
        {showCompleted ? <Eye size={16} /> : <EyeOff size={16} />}
      </button>
    </div>
  </header>;
}
