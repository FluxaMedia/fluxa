import { RATING_SOURCES, orderedRatingEntries } from './ratingSources';
import { color, fade, fontSize, radius } from '../../design';

interface RatingBadgeProps {
  source: string;
  value: number;
  bare?: boolean;
}

export function RatingBadge({ source, value, bare }: RatingBadgeProps) {
  const info = RATING_SOURCES[source];
  if (!info) return null;
  const icon = info.iconForValue ? info.iconForValue(value) : info.icon;
  const maskColor = info.colorForValue && !info.iconForValue ? info.colorForValue(value) : info.maskColor;
  return (
    <span style={bare ? styles.bareBadge : styles.badge} title={info.label}>
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
  bare?: boolean;
}

export function RatingsRow({ ratings, bare }: RatingsRowProps) {
  const entries = orderedRatingEntries(ratings);
  if (entries.length === 0) return null;
  return (
    <div style={styles.row}>
      {entries.map(({ source, value }) => (
        <RatingBadge key={source} source={source} value={value} bare={bare} />
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
    background: fade.shade(0.55),
    border: `1px solid ${color.line}`,
    borderRadius: radius.md,
    padding: '0.25rem 0.5rem',
  } as const,
  bareBadge: {
    display: 'inline-flex',
    alignItems: 'center',
    gap: '0.375rem',
    flexShrink: 0,
  } as const,
  logo: {
    height: '1.25rem',
    width: 'auto',
    display: 'block',
    borderRadius: radius.xs,
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
    color: color.textPrimary,
    fontSize: fontSize.base,
    fontWeight: 700,
    lineHeight: 1,
  } as const,
};
