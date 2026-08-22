import type React from 'react';
import { t } from '../i18n';
import type { ExternalHandoffPrompt as PromptState } from '../hooks/useExternalHandoff';

function clock(seconds: number): string {
  const total = Math.max(0, Math.floor(seconds));
  const hours = Math.floor(total / 3600);
  const minutes = Math.floor((total % 3600) / 60);
  const rest = total % 60;
  return hours > 0
    ? `${hours}:${String(minutes).padStart(2, '0')}:${String(rest).padStart(2, '0')}`
    : `${minutes}:${String(rest).padStart(2, '0')}`;
}

export function ExternalHandoffPrompt({
  prompt,
  onCommit,
  onDismiss,
}: {
  prompt: PromptState;
  onCommit: (timePos: number, duration: number) => void;
  onDismiss: () => void;
}) {
  const { session, estimate } = prompt;
  const label = session.episode
    ? `${session.meta.name} · ${t('format.season_episode_short', session.episode.season ?? 1, session.episode.episode ?? session.episode.number ?? 1)}`
    : session.meta.name;

  return (
    <div style={styles.backdrop} onClick={onDismiss}>
      <div style={styles.sheet} onClick={(event) => event.stopPropagation()}>
        <p style={styles.title}>{t('external.prompt_title')}</p>
        <p style={styles.subtitle}>{label}</p>
        <button style={styles.primary} onClick={() => onCommit(estimate.duration, estimate.duration)}>
          {t('external.mark_watched')}
        </button>
        {!estimate.finished && estimate.duration > 0 && (
          <button style={styles.secondary} onClick={() => onCommit(estimate.timePos, estimate.duration)}>
            {t('external.save_position', clock(estimate.timePos))}
          </button>
        )}
        <button style={styles.ghost} onClick={onDismiss}>
          {t('external.save_nothing')}
        </button>
      </div>
    </div>
  );
}

const styles: Record<string, React.CSSProperties> = {
  backdrop: {
    position: 'fixed',
    inset: 0,
    zIndex: 9997,
    background: 'var(--fluxa-scrim)',
    display: 'flex',
    alignItems: 'flex-end',
    justifyContent: 'center',
  },
  sheet: {
    width: 'min(28rem, 100%)',
    display: 'flex',
    flexDirection: 'column',
    gap: '0.5rem',
    padding: '1.25rem 1rem calc(1.25rem + env(safe-area-inset-bottom, 0px))',
    background: 'var(--fluxa-surface-raised)',
    border: '1px solid var(--fluxa-border)',
    borderRadius: '1rem 1rem 0 0',
  },
  title: { color: 'var(--fluxa-text-primary)', fontSize: '1rem', fontWeight: 800, margin: 0 },
  subtitle: { color: 'var(--fluxa-text-secondary)', fontSize: '0.8125rem', margin: '0 0 0.5rem' },
  primary: {
    minHeight: '2.75rem',
    borderRadius: '0.625rem',
    border: 'none',
    background: 'var(--fluxa-accent)',
    color: 'var(--fluxa-accent-foreground)',
    fontSize: '0.875rem',
    fontWeight: 800,
  },
  secondary: {
    minHeight: '2.75rem',
    borderRadius: '0.625rem',
    border: '1px solid var(--fluxa-border-strong)',
    background: 'transparent',
    color: 'var(--fluxa-text-primary)',
    fontSize: '0.875rem',
    fontWeight: 700,
  },
  ghost: {
    minHeight: '2.5rem',
    border: 'none',
    background: 'transparent',
    color: 'var(--fluxa-text-muted)',
    fontSize: '0.8125rem',
    fontWeight: 700,
  },
};
