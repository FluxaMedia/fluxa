import React from 'react';
import { t } from '../i18n';
import { FluxaButton } from './ui/FluxaButton';
import { FluxaDialog } from './ui/FluxaDialog';

interface Props {
  addonName: string;
  onConfirm: () => void;
}

export function AddonAddedDialog({ addonName, onConfirm }: Props) {
  return (
    <FluxaDialog
      title={t('addons.installed_dialog_title')}
      description={t('addons.installed_dialog_body', addonName)}
      onDismiss={onConfirm}
      width="min(25rem, calc(100vw - 2rem))"
    >
      <div style={S.actions}>
        <FluxaButton onClick={onConfirm}>{t('common.ok')}</FluxaButton>
      </div>
    </FluxaDialog>
  );
}

const S: Record<string, React.CSSProperties> = {
  actions: {
    display: 'flex',
    gap: '0.625rem',
    justifyContent: 'flex-end',
  },
};
