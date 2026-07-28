import { RATING_SOURCES, orderedRatingEntries } from './ratingSources';

interface RatingBadgeProps {
  source: string;
  value: number;
}

export function RatingBadge({ source, value }: RatingBadgeProps) {
  const info = RATING_SOURCES[source];
  if (!info) return null;
  return (
    <span style={styles.badge} title={info.label}>
      <img src={info.icon} alt={info.label} style={styles.logo} />
      <span style={styles.score}>{info.format(value)}</span>
    </span>
  );
}

interface RatingsRowProps {
  ratings: Record<string, number> | null | undefined;
}

export function RatingsRow({ ratings }: RatingsRowProps) {
  const entries = orderedRatingEntries(ratings);
  if (entries.length === 0) return null;
  return (
    <div style={styles.row}>
      {entries.map(({ source, value }) => (
        <RatingBadge key={source} source={source} value={value} />
      ))}
    </div>
  );
}

const styles = {
  row: {
    display: 'flex',
    alignItems: 'center',
    gap: '0.875rem',
    flexWrap: 'wrap',
  } as const,
  badge: {
    display: 'inline-flex',
    alignItems: 'center',
    gap: '0.375rem',
    flexShrink: 0,
  } as const,
  logo: {
    height: '1rem',
    width: 'auto',
    display: 'block',
    borderRadius: '0.1875rem',
    userSelect: 'none',
  } as const,
  score: {
    color: 'rgba(255,255,255,0.92)',
    fontSize: '0.9rem',
    fontWeight: 700,
    lineHeight: 1,
    textShadow: '0 1px 0.1875rem rgba(0,0,0,0.8)',
  } as const,
};
