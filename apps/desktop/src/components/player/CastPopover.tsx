import { type RefObject } from 'react';
import { Cast, Check, Tv } from 'lucide-react';
import { t } from '../../i18n';
import type { CastDevice } from '../../core/cast';
import { Popover } from '../ui/Popover';

interface CastPopoverProps {
  devices: CastDevice[];
  discovering: boolean;
  activeDeviceId: string | null;
  anchorRef: RefObject<HTMLElement | null>;
  onClose: () => void;
  onSelectDevice: (device: CastDevice) => void;
  onDisconnect: () => void;
}

export function CastPopover({ devices, discovering, activeDeviceId, anchorRef, onClose, onSelectDevice, onDisconnect }: CastPopoverProps) {
  return (
    <Popover open onClose={onClose} anchorRef={anchorRef} placement="top" width="13.75rem" maxHeight="18.75rem">
      <div
        style={{
          color: 'rgba(255,255,255,0.45)',
          fontSize: '0.6875rem',
          fontWeight: 700,
          letterSpacing: '0.05rem',
          padding: '0.25rem 0.875rem 0.5rem',
          textTransform: 'uppercase',
        }}
      >
        {t('player.cast')}
      </div>
      {activeDeviceId && (
        <button
          className="ui-popover-row"
          onClick={onDisconnect}
          style={{
            display: 'flex',
            alignItems: 'center',
            gap: '0.625rem',
            width: '100%',
            background: 'none',
            border: 'none',
            borderBottom: '1px solid rgba(255,255,255,0.07)',
            color: '#fff',
            fontSize: '0.8125rem',
            fontWeight: 600,
            padding: '0.5rem 0.875rem',
            cursor: 'pointer',
            textAlign: 'left',
            marginBottom: '0.25rem',
          }}
        >
          {t('player.stop_casting')}
        </button>
      )}
      {discovering && (
        <div style={{ color: 'rgba(255,255,255,0.45)', fontSize: '0.8125rem', padding: '0.5rem 0.875rem' }}>
          {t('player.cast_searching')}
        </div>
      )}
      {!discovering && devices.length === 0 && (
        <div style={{ color: 'rgba(255,255,255,0.35)', fontSize: '0.8125rem', padding: '0.5rem 0.875rem' }}>
          {t('player.cast_no_devices')}
        </div>
      )}
      {devices.map((device) => (
        <button
          key={device.id}
          className="ui-popover-row"
          onClick={() => onSelectDevice(device)}
          style={{
            display: 'flex',
            alignItems: 'center',
            gap: '0.625rem',
            width: '100%',
            background: 'none',
            border: 'none',
            color: activeDeviceId === device.id ? '#fff' : 'rgba(255,255,255,0.7)',
            fontSize: '0.8125rem',
            fontWeight: activeDeviceId === device.id ? 600 : 400,
            padding: '0.5rem 0.875rem',
            cursor: 'pointer',
            textAlign: 'left',
          }}
        >
          <span style={{ width: '0.875rem', flexShrink: 0, color: 'var(--primary-accent-color)' }}>
            {activeDeviceId === device.id ? (
              <Check size={14} />
            ) : device.kind === 'chromecast' || device.kind === 'fcast' ? (
              <Cast size={14} />
            ) : (
              <Tv size={14} />
            )}
          </span>
          {device.name}
        </button>
      ))}
    </Popover>
  );
}
