import React from 'react';
import { color, fontSize, font, lineHeight, radius, space, tracking, weight } from './tokens';

type ButtonVariant = 'primary' | 'secondary' | 'ghost';
type ButtonSize = 'sm' | 'md';

const buttonSizing: Record<ButtonSize, React.CSSProperties> = {
  sm: { height: '2.25rem', padding: `0 ${space[3]}`, fontSize: fontSize.base },
  md: { height: '2.75rem', padding: `0 ${space[6]}`, fontSize: fontSize.md },
};

const buttonSkin: Record<ButtonVariant, React.CSSProperties> = {
  primary: { background: color.light, color: color.onLight, border: 'none' },
  secondary: { background: color.fill, color: color.textPrimary, border: `1px solid ${color.line}` },
  ghost: { background: 'transparent', color: color.textMuted, border: 'none' },
};

export function Button({
  variant = 'secondary',
  size = 'md',
  icon,
  children,
  style,
  className,
  ...rest
}: {
  variant?: ButtonVariant;
  size?: ButtonSize;
  icon?: React.ReactNode;
} & React.ButtonHTMLAttributes<HTMLButtonElement>) {
  return (
    <button
      {...rest}
      className={['fx-btn', `fx-btn-${variant}`, className].filter(Boolean).join(' ')}
      style={{
        display: 'inline-flex',
        alignItems: 'center',
        justifyContent: 'center',
        gap: space[2],
        borderRadius: radius.md,
        fontFamily: font.body,
        fontWeight: weight.bold,
        whiteSpace: 'nowrap',
        flexShrink: 0,
        ...buttonSizing[size],
        ...buttonSkin[variant],
        ...style,
      }}
    >
      {icon}
      {children != null && <span style={{ overflow: 'hidden', textOverflow: 'ellipsis' }}>{children}</span>}
    </button>
  );
}

export function IconButton({
  active,
  size = '2.25rem',
  shape = 'circle',
  children,
  style,
  className,
  ...rest
}: {
  active?: boolean;
  size?: string;
  shape?: 'circle' | 'square';
} & React.ButtonHTMLAttributes<HTMLButtonElement>) {
  return (
    <button
      {...rest}
      className={['fx-iconbtn', active ? 'is-active' : '', className].filter(Boolean).join(' ')}
      style={{
        width: size,
        height: size,
        borderRadius: shape === 'circle' ? radius.circle : radius.md,
        border: `1.5px solid ${active ? color.textPrimary : color.outline}`,
        background: active ? color.fillActive : 'transparent',
        color: color.textPrimary,
        display: 'grid',
        placeItems: 'center',
        padding: 0,
        flexShrink: 0,
        ...style,
      }}
    >
      {children}
    </button>
  );
}

export function Chip({
  selected,
  children,
  style,
  className,
  ...rest
}: { selected?: boolean } & React.ButtonHTMLAttributes<HTMLButtonElement>) {
  return (
    <button
      {...rest}
      className={['fx-chip', selected ? 'is-selected' : '', className].filter(Boolean).join(' ')}
      style={{
        display: 'inline-flex',
        alignItems: 'center',
        gap: space[2],
        fontSize: fontSize.sm,
        fontWeight: weight.semibold,
        color: selected ? color.onLight : color.textBody,
        background: selected ? color.light : color.fill,
        border: `1px solid ${selected ? 'transparent' : color.line}`,
        borderRadius: radius.md,
        padding: `${space[2]} ${space[3]}`,
        whiteSpace: 'nowrap',
        ...style,
      }}
    >
      {children}
    </button>
  );
}

export function SectionLabel({ children, style }: { children: React.ReactNode; style?: React.CSSProperties }) {
  return (
    <span
      style={{
        fontSize: fontSize.xs,
        fontWeight: weight.bold,
        letterSpacing: tracking.wide,
        textTransform: 'uppercase',
        color: color.textDim,
        ...style,
      }}
    >
      {children}
    </span>
  );
}

export function Divider({ style }: { style?: React.CSSProperties }) {
  return <div style={{ height: '1px', background: color.line, ...style }} />;
}

export function MetaText({
  tone = 'muted',
  children,
  style,
}: {
  tone?: 'body' | 'muted' | 'dim';
  children: React.ReactNode;
  style?: React.CSSProperties;
}) {
  const tones = { body: color.textBody, muted: color.textMuted, dim: color.textDim };
  return (
    <span style={{ fontSize: fontSize.base, fontWeight: weight.medium, lineHeight: lineHeight.normal, color: tones[tone], ...style }}>
      {children}
    </span>
  );
}
