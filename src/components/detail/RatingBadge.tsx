import { RATING_SOURCES, orderedRatingEntries } from './ratingSources';

interface RatingBadgeProps {
  source: string;
  value: number;
}

export function RatingBadge({ source, value }: RatingBadgeProps) {
  const info = RATING_SOURCES[source];
  if (!info) return null;
  const icon = info.iconForValue ? info.iconForValue(value) : info.icon;
  const maskColor = info.colorForValue && !info.iconForValue ? info.colorForValue(value) : info.maskColor;
  return (
    <span style={styles.badge} title={info.label}>
      {maskColor ? (
        <span
          role="img"
          aria-label={info.label}
          style={{
            ...styles.logo,
            ...styles.maskedLogo,
            backgroundColor: maskColor,
            maskImage: `url(${icon})`,
            WebkitMaskImage: `url(${icon})`,
          }}
        />
      ) : (
        <img src={icon} alt={info.label} style={styles.logo} />
      )}
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
    gap: '0.5rem',
    flexWrap: 'wrap',
  } as const,
  badge: {
    display: 'inline-flex',
    alignItems: 'center',
    gap: '0.4375rem',
    flexShrink: 0,
    background: 'rgba(0,0,0,0.55)',
    border: '1px solid rgba(255,255,255,0.1)',
    borderRadius: '0.4375rem',
    padding: '0.25rem 0.5rem',
  } as const,
  logo: {
    height: '1.25rem',
    width: 'auto',
    display: 'block',
    borderRadius: '0.1875rem',
    userSelect: 'none',
  } as const,
  maskedLogo: {
    width: '1.25rem',
    maskSize: 'contain',
    WebkitMaskSize: 'contain',
    maskRepeat: 'no-repeat',
    WebkitMaskRepeat: 'no-repeat',
    maskPosition: 'center',
    WebkitMaskPosition: 'center',
  } as const,
  score: {
    color: 'rgba(255,255,255,0.92)',
    fontSize: '0.9rem',
    fontWeight: 700,
    lineHeight: 1,
  } as const,
};
