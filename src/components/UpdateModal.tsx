import React from 'react';
import type { Update } from '@tauri-apps/plugin-updater';
import { t } from '../i18n';
import { useEscapeKey } from '../hooks/useEscapeKey';

export type UpdateState =
  | { phase: 'idle' }
  | { phase: 'checking' }
  | { phase: 'up-to-date' }
  | { phase: 'available'; update: Update }
  | { phase: 'downloading'; progress: number }
  | { phase: 'installing' }
  | { phase: 'error'; message: string };

interface Props {
  state: UpdateState;
  onClose: () => void;
}

export async function startUpdateCheck(setState: (s: UpdateState) => void): Promise<void> {
  setState({ phase: 'checking' });
  try {
    const { check: checkUpdate } = await import('@tauri-apps/plugin-updater');
    const update = await checkUpdate();
    if (update?.available) {
      setState({ phase: 'available', update });
    } else {
      setState({ phase: 'up-to-date' });
    }
  } catch {
    setState({ phase: 'error', message: t('update.check_failed') });
  }
}

export async function installUpdate(update: Update, setState: (s: UpdateState) => void): Promise<void> {
  let downloaded = 0;
  let total = 0;
  setState({ phase: 'downloading', progress: 0 });
  try {
    await update.downloadAndInstall((event) => {
      if (event.event === 'Started') {
        total = event.data.contentLength ?? 0;
      } else if (event.event === 'Progress') {
        downloaded += event.data.chunkLength ?? 0;
        const pct = total > 0 ? Math.round((downloaded / total) * 100) : 0;
        setState({ phase: 'downloading', progress: pct });
      } else if (event.event === 'Finished') {
        setState({ phase: 'installing' });
      }
    });
    const { relaunch } = await import('@tauri-apps/plugin-process');
    await relaunch();
  } catch {
    setState({ phase: 'error', message: t('update.install_failed') });
  }
}

export function UpdateModal({ state, onClose }: Props) {
  const canClose = state.phase !== 'idle' && state.phase !== 'downloading' && state.phase !== 'installing';
  useEscapeKey(() => {
    if (canClose) onClose();
  });
  if (state.phase === 'idle') return null;

  return (
    <div style={overlay} onClick={canClose ? onClose : undefined}>
      <div style={card} onClick={(e) => e.stopPropagation()}>
        {state.phase === 'checking' && <CheckingView />}
        {state.phase === 'up-to-date' && <UpToDateView onClose={onClose} />}
        {state.phase === 'available' && <AvailableView update={state.update} onClose={onClose} />}
        {state.phase === 'downloading' && <DownloadingView progress={state.progress} />}
        {state.phase === 'installing' && <InstallingView />}
        {state.phase === 'error' && <ErrorView message={state.message} onClose={onClose} />}
      </div>
    </div>
  );
}

function CheckingView() {
  return (
    <>
      <Title>{t('update.checking')}</Title>
      <Spinner />
    </>
  );
}

function UpToDateView({ onClose }: { onClose: () => void }) {
  return (
    <>
      <Title>{t('update.up_to_date')}</Title>
      <Subtitle>{t('update.up_to_date_subtitle')}</Subtitle>
      <ButtonRow>
        <Btn onClick={onClose} primary>
          {t('common.close')}
        </Btn>
      </ButtonRow>
    </>
  );
}

function AvailableView({ update, onClose }: { update: Update; onClose: () => void }) {
  const [installing, setInstalling] = React.useState(false);
  const [installState, setInstallState] = React.useState<UpdateState>({ phase: 'idle' });

  const handleInstall = async () => {
    setInstalling(true);
    await installUpdate(update, setInstallState);
    setInstalling(false);
  };

  if (installState.phase === 'downloading') {
    return <DownloadingView progress={(installState as { phase: 'downloading'; progress: number }).progress} />;
  }
  if (installState.phase === 'installing') {
    return <InstallingView />;
  }
  if (installState.phase === 'error') {
    return <ErrorView message={(installState as { phase: 'error'; message: string }).message} onClose={onClose} />;
  }

  return (
    <>
      <Tag>{t('update.available')}</Tag>
      <Title>{t('update.version', update.version)}</Title>
      {update.body && (
        <div style={changelogBox}>
          <pre style={changelogText}>{update.body.trim()}</pre>
        </div>
      )}
      <ButtonRow>
        <Btn onClick={onClose}>{t('update.later')}</Btn>
        <Btn onClick={() => void handleInstall()} primary disabled={installing}>
          {installing ? t('update.installing') : t('update.now')}
        </Btn>
      </ButtonRow>
    </>
  );
}

function DownloadingView({ progress }: { progress: number }) {
  return (
    <>
      <Title>{t('update.downloading')}</Title>
      <div style={progressTrack}>
        <div style={{ ...progressBar, width: `${progress}%` }} />
      </div>
      <Subtitle style={{ marginTop: '0.5rem' }}>{progress}%</Subtitle>
    </>
  );
}

function InstallingView() {
  return (
    <>
      <Title>{t('update.installing')}</Title>
      <Subtitle>{t('update.installing_subtitle')}</Subtitle>
      <Spinner />
    </>
  );
}

function ErrorView({ message, onClose }: { message: string; onClose: () => void }) {
  return (
    <>
      <Title>{t('update.failed')}</Title>
      <Subtitle>{message}</Subtitle>
      <ButtonRow>
        <Btn onClick={onClose} primary>
          {t('common.close')}
        </Btn>
      </ButtonRow>
    </>
  );
}

function Title({ children }: { children: React.ReactNode }) {
  return <p style={titleStyle}>{children}</p>;
}

function Subtitle({ children, style }: { children: React.ReactNode; style?: React.CSSProperties }) {
  return <p style={{ ...subtitleStyle, ...style }}>{children}</p>;
}

function Tag({ children }: { children: React.ReactNode }) {
  return <span style={tagStyle}>{children}</span>;
}

function Spinner() {
  return (
    <div style={spinnerWrap}>
      <svg width="32" height="32" viewBox="0 0 32 32" style={{ animation: 'spin 1s linear infinite' }}>
        <style>{`@keyframes spin { to { transform: rotate(360deg); } }`}</style>
        <circle cx="16" cy="16" r="12" fill="none" stroke="rgba(255,255,255,0.15)" strokeWidth="3" />
        <path d="M16 4a12 12 0 0 1 12 12" fill="none" stroke="rgba(255,255,255,0.8)" strokeWidth="3" strokeLinecap="round" />
      </svg>
    </div>
  );
}

function ButtonRow({ children }: { children: React.ReactNode }) {
  return <div style={btnRow}>{children}</div>;
}

function Btn({
  children,
  onClick,
  primary,
  disabled,
}: {
  children: React.ReactNode;
  onClick?: () => void;
  primary?: boolean;
  disabled?: boolean;
}) {
  return (
    <button
      style={{ ...btnBase, ...(primary ? btnPrimary : btnSecondary), ...(disabled ? btnDisabled : {}) }}
      onClick={onClick}
      disabled={disabled}
    >
      {children}
    </button>
  );
}

const overlay: React.CSSProperties = {
  position: 'fixed',
  inset: 0,
  background: 'rgba(0,0,0,0.72)',
  backdropFilter: 'blur(0.5rem)',
  display: 'flex',
  alignItems: 'center',
  justifyContent: 'center',
  zIndex: 9999,
};

const card: React.CSSProperties = {
  background: '#13141a',
  border: '1px solid rgba(255,255,255,0.08)',
  borderRadius: '1rem',
  padding: '2rem 1.75rem',
  width: '26.25rem',
  maxWidth: 'calc(100vw - 3rem)',
  boxShadow: '0 1.5rem 4rem rgba(0,0,0,0.6)',
};

const titleStyle: React.CSSProperties = {
  color: '#fff',
  fontSize: '1.125rem',
  fontWeight: 700,
  margin: '0 0 0.375rem',
};

const subtitleStyle: React.CSSProperties = {
  color: 'rgba(255,255,255,0.5)',
  fontSize: '0.8125rem',
  margin: '0 0 1.25rem',
  lineHeight: 1.5,
};

const tagStyle: React.CSSProperties = {
  display: 'inline-block',
  background: 'rgba(99,102,241,0.18)',
  color: '#a5b4fc',
  fontSize: '0.6875rem',
  fontWeight: 700,
  letterSpacing: '0.06em',
  textTransform: 'uppercase',
  padding: '0.1875rem 0.5rem',
  borderRadius: '0.375rem',
  marginBottom: '0.625rem',
};

const changelogBox: React.CSSProperties = {
  background: 'rgba(255,255,255,0.04)',
  border: '1px solid rgba(255,255,255,0.07)',
  borderRadius: '0.5rem',
  padding: '0.75rem 0.875rem',
  maxHeight: '13.75rem',
  overflowY: 'auto',
  marginBottom: '1.5rem',
};

const changelogText: React.CSSProperties = {
  color: 'rgba(255,255,255,0.65)',
  fontSize: '0.75rem',
  margin: 0,
  fontFamily: 'monospace',
  lineHeight: 1.7,
  whiteSpace: 'pre-wrap',
  wordBreak: 'break-word',
};

const progressTrack: React.CSSProperties = {
  background: 'rgba(255,255,255,0.08)',
  borderRadius: '6.1875rem',
  height: '0.375rem',
  overflow: 'hidden',
  margin: '1.25rem 0 0.25rem',
};

const progressBar: React.CSSProperties = {
  background: '#6366f1',
  height: '100%',
  borderRadius: '6.1875rem',
  transition: 'width 0.2s',
};

const spinnerWrap: React.CSSProperties = {
  display: 'flex',
  justifyContent: 'center',
  padding: '1.5rem 0 1rem',
};

const btnRow: React.CSSProperties = {
  display: 'flex',
  gap: '0.625rem',
  justifyContent: 'flex-end',
  marginTop: '0.25rem',
};

const btnBase: React.CSSProperties = {
  padding: '0.5625rem 1.25rem',
  borderRadius: '0.5rem',
  fontSize: '0.8125rem',
  fontWeight: 600,
  cursor: 'pointer',
  border: 'none',
  transition: 'opacity 0.15s',
};

const btnPrimary: React.CSSProperties = {
  background: '#fff',
  color: '#000',
};

const btnSecondary: React.CSSProperties = {
  background: 'rgba(255,255,255,0.07)',
  color: 'rgba(255,255,255,0.75)',
};

const btnDisabled: React.CSSProperties = {
  opacity: 0.4,
  cursor: 'not-allowed',
};
