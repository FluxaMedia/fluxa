import type { CSSProperties } from 'react';
import { FluxaButton } from './ui/FluxaButton';
import { FluxaDialog } from './ui/FluxaDialog';

export function ConfirmDialog({
  title,
  body,
  confirmLabel,
  cancelLabel,
  destructive,
  onConfirm,
  onCancel,
}: {
  title: string;
  body: string;
  confirmLabel: string;
  cancelLabel: string;
  destructive?: boolean;
  onConfirm: () => void;
  onCancel: () => void;
}) {
  return (
    <FluxaDialog title={title} description={body} onDismiss={onCancel} width="20rem">
      <div style={S.actions}>
        <FluxaButton variant="secondary" fullWidth onClick={onCancel}>
          {cancelLabel}
        </FluxaButton>
        <FluxaButton variant={destructive ? 'danger' : 'primary'} fullWidth onClick={onConfirm}>
          {confirmLabel}
        </FluxaButton>
      </div>
    </FluxaDialog>
  );
}

const S: Record<string, CSSProperties> = {
  actions: { display: 'flex', gap: '0.5rem' },
};
