import type { ButtonHTMLAttributes, CSSProperties, ReactNode } from 'react';
import { Button, IconButton, type ButtonSize, type ButtonVariant } from '../../design';

export type FluxaButtonVariant = ButtonVariant;
export type FluxaButtonSize = ButtonSize;

export function FluxaButton({
  variant = 'primary',
  size = 'md',
  fullWidth = false,
  icon,
  children,
  style,
  ...props
}: ButtonHTMLAttributes<HTMLButtonElement> & {
  variant?: FluxaButtonVariant;
  size?: FluxaButtonSize;
  fullWidth?: boolean;
  icon?: ReactNode;
}) {
  return <Button {...props} variant={variant} size={size} icon={icon} style={{ width: fullWidth ? '100%' : undefined, ...style }}>{children}</Button>;
}

export function FluxaIconButton({
  ariaLabel,
  variant = 'ghost',
  size = 'md',
  children,
  style,
  ...props
}: ButtonHTMLAttributes<HTMLButtonElement> & {
  ariaLabel: string;
  variant?: FluxaButtonVariant;
  size?: FluxaButtonSize;
}) {
  const variantStyle: CSSProperties = variant === 'danger'
    ? { background: 'var(--fluxa-error)', borderColor: 'var(--fluxa-error)' }
    : variant === 'primary'
      ? { background: 'var(--fluxa-accent)', borderColor: 'var(--fluxa-accent)', color: 'var(--fluxa-accent-foreground)' }
      : {};
  return (
    <IconButton
      {...props}
      aria-label={ariaLabel}
      size={size === 'lg' ? '3rem' : size === 'sm' ? '2rem' : '2.5rem'}
      style={{ ...variantStyle, ...style }}
    >
      {children}
    </IconButton>
  );
}
