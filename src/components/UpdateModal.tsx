import React from 'react';
import { check as checkUpdate, type Update } from '@tauri-apps/plugin-updater';
import { relaunch } from '@tauri-apps/plugin-process';

export type UpdateState =
  | { phase: 'idle' }
  | { phase: 'checking' }
  | { phase: 'up-to-date' }
  | { phase: 'available'; update: Update }
  | { phase: 'downloading'; progress: number }
  | { phase: 'error'; message: string };

interface Props {
  state: UpdateState;
  onClose: () => void;
}

export async function startUpdateCheck(
  setState: (s: UpdateState) => void,
): Promise<void> {
  setState({ phase: 'checking' });
  try {
    const update = await checkUpdate();
    if (update?.available) {
      setState({ phase: 'available', update });
    } else {
      setState({ phase: 'up-to-date' });
    }
  } catch {
    setState({ phase: 'error', message: 'Could not reach update server.' });
  }
}

export async function installUpdate(
  update: Update,
  setState: (s: UpdateState) => void,
): Promise<void> {
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
      }
    });
    await relaunch();
  } catch {
    setState({ phase: 'error', message: 'Update installation failed.' });
  }
}

export function UpdateModal({ state, onClose }: Props) {
  if (state.phase === 'idle') return null;

  const canClose = state.phase !== 'downloading';

  return (
    <div style={overlay} onClick={canClose ? onClose : undefined}>
      <div style={card} onClick={(e) => e.stopPropagation()}>
        {state.phase === 'checking' && <CheckingView />}
        {state.phase === 'up-to-date' && <UpToDateView onClose={onClose} />}
        {state.phase === 'available' && <AvailableView update={state.update} onClose={onClose} />}
        {state.phase === 'downloading' && <DownloadingView progress={state.progress} />}
        {state.phase === 'error' && <ErrorView message={state.message} onClose={onClose} />}
      </div>
    </div>
  );
}

function CheckingView() {
  return (
    <>
      <Title>Checking for updates…</Title>
      <Spinner />
    </>
  );
}

function UpToDateView({ onClose }: { onClose: () => void }) {
  return (
    <>
      <Title>You're up to date</Title>
      <Subtitle>Fluxa Desktop is running the latest version.</Subtitle>
      <ButtonRow>
        <Btn onClick={onClose} primary>Close</Btn>
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
  if (installState.phase === 'error') {
    return <ErrorView message={(installState as { phase: 'error'; message: string }).message} onClose={onClose} />;
  }

  return (
    <>
      <Tag>Update available</Tag>
      <Title>Version {update.version}</Title>
      {update.body && (
        <div style={changelogBox}>
          <pre style={changelogText}>{update.body.trim()}</pre>
        </div>
      )}
      <ButtonRow>
        <Btn onClick={onClose}>Later</Btn>
        <Btn onClick={() => void handleInstall()} primary disabled={installing}>
          {installing ? 'Installing…' : 'Update now'}
        </Btn>
      </ButtonRow>
    </>
  );
}

function DownloadingView({ progress }: { progress: number }) {
  return (
    <>
      <Title>Downloading update…</Title>
      <div style={progressTrack}>
        <div style={{ ...progressBar, width: `${progress}%` }} />
      </div>
      <Subtitle style={{ marginTop: 8 }}>{progress}%</Subtitle>
    </>
  );
}

function ErrorView({ message, onClose }: { message: string; onClose: () => void }) {
  return (
    <>
      <Title>Update failed</Title>
      <Subtitle>{message}</Subtitle>
      <ButtonRow>
        <Btn onClick={onClose} primary>Close</Btn>
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

function Btn({ children, onClick, primary, disabled }: {
  children: React.ReactNode;
  onClick?: () => void;
  primary?: boolean;
  disabled?: boolean;
}) {
  return (
    <button style={{ ...btnBase, ...(primary ? btnPrimary : btnSecondary), ...(disabled ? btnDisabled : {}) }} onClick={onClick} disabled={disabled}>
      {children}
    </button>
  );
}



const overlay: React.CSSProperties = {
  position: 'fixed', inset: 0,
  background: 'rgba(0,0,0,0.72)',
  backdropFilter: 'blur(8px)',
  display: 'flex', alignItems: 'center', justifyContent: 'center',
  zIndex: 9999,
};

const card: React.CSSProperties = {
  background: '#13141a',
  border: '1px solid rgba(255,255,255,0.08)',
  borderRadius: 16,
  padding: '32px 28px',
  width: 420,
  maxWidth: 'calc(100vw - 48px)',
  boxShadow: '0 24px 64px rgba(0,0,0,0.6)',
};

const titleStyle: React.CSSProperties = {
  color: '#fff',
  fontSize: 18,
  fontWeight: 700,
  margin: '0 0 6px',
  fontFamily: 'sans-serif',
};

const subtitleStyle: React.CSSProperties = {
  color: 'rgba(255,255,255,0.5)',
  fontSize: 13,
  margin: '0 0 20px',
  fontFamily: 'sans-serif',
  lineHeight: 1.5,
};

const tagStyle: React.CSSProperties = {
  display: 'inline-block',
  background: 'rgba(99,102,241,0.18)',
  color: '#a5b4fc',
  fontSize: 11,
  fontWeight: 700,
  letterSpacing: '0.06em',
  textTransform: 'uppercase',
  padding: '3px 8px',
  borderRadius: 6,
  marginBottom: 10,
  fontFamily: 'sans-serif',
};

const changelogBox: React.CSSProperties = {
  background: 'rgba(255,255,255,0.04)',
  border: '1px solid rgba(255,255,255,0.07)',
  borderRadius: 8,
  padding: '12px 14px',
  maxHeight: 220,
  overflowY: 'auto',
  marginBottom: 24,
};

const changelogText: React.CSSProperties = {
  color: 'rgba(255,255,255,0.65)',
  fontSize: 12,
  margin: 0,
  fontFamily: 'monospace',
  lineHeight: 1.7,
  whiteSpace: 'pre-wrap',
  wordBreak: 'break-word',
};

const progressTrack: React.CSSProperties = {
  background: 'rgba(255,255,255,0.08)',
  borderRadius: 99,
  height: 6,
  overflow: 'hidden',
  margin: '20px 0 4px',
};

const progressBar: React.CSSProperties = {
  background: '#6366f1',
  height: '100%',
  borderRadius: 99,
  transition: 'width 0.2s',
};

const spinnerWrap: React.CSSProperties = {
  display: 'flex',
  justifyContent: 'center',
  padding: '24px 0 16px',
};

const btnRow: React.CSSProperties = {
  display: 'flex',
  gap: 10,
  justifyContent: 'flex-end',
  marginTop: 4,
};

const btnBase: React.CSSProperties = {
  padding: '9px 20px',
  borderRadius: 8,
  fontSize: 13,
  fontWeight: 600,
  fontFamily: 'sans-serif',
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
