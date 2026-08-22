import { useCallback, useEffect, useRef, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { open } from '@tauri-apps/plugin-dialog';

interface DiscoveredTv {
  host: string;
  name: string;
}

interface Progress {
  step: string;
  message: string;
}

type Phase = 'idle' | 'working' | 'done' | 'error';

const STEPS = ['key', 'connect', 'upload', 'install', 'launch'] as const;

export default function App() {
  const [tvs, setTvs] = useState<DiscoveredTv[]>([]);
  const [scanning, setScanning] = useState(false);
  const [host, setHost] = useState('');
  const [passphrase, setPassphrase] = useState('');
  const [ipkPath, setIpkPath] = useState('');
  const [launch, setLaunch] = useState(true);
  const [phase, setPhase] = useState<Phase>('idle');
  const [progress, setProgress] = useState<Progress | null>(null);
  const [error, setError] = useState<string | null>(null);
  const unlistenRef = useRef<(() => void) | null>(null);

  useEffect(() => {
    void listen<Progress>('install-progress', (event) => setProgress(event.payload))
      .then((stop) => { unlistenRef.current = stop; });
    return () => unlistenRef.current?.();
  }, []);

  const scan = useCallback(async () => {
    setScanning(true);
    try {
      const found = await invoke<DiscoveredTv[]>('discover_tvs');
      setTvs(found);
      if (found.length > 0 && !host) setHost(found[0].host);
    } finally {
      setScanning(false);
    }
  }, [host]);

  useEffect(() => { void scan(); }, []);

  const pickIpk = async () => {
    const selected = await open({
      multiple: false,
      filters: [{ name: 'webOS package', extensions: ['ipk'] }],
    });
    if (typeof selected === 'string') setIpkPath(selected);
  };

  const run = async () => {
    setPhase('working');
    setError(null);
    setProgress(null);
    try {
      await invoke('install_ipk', {
        host: host.trim(),
        passphrase: passphrase.trim(),
        ipkPath,
        appId: 'com.fluxa.app',
        launch,
      });
      setPhase('done');
    } catch (err) {
      setError(String(err));
      setPhase('error');
    }
  };

  const ready = host.trim() !== '' && passphrase.trim() !== '' && ipkPath !== '' && phase !== 'working';
  const activeStep = progress ? STEPS.indexOf(progress.step as (typeof STEPS)[number]) : -1;

  return (
    <main style={S.page}>
      <h1 style={S.title}>Fluxa for LG webOS</h1>
      <p style={S.lede}>
        Turn on Developer Mode on the TV first, then keep this window open while it installs.
      </p>

      <section style={S.section}>
        <div style={S.rowBetween}>
          <label style={S.label}>Television</label>
          <button type="button" style={S.ghostButton} onClick={() => void scan()} disabled={scanning}>
            {scanning ? 'Scanning…' : 'Scan again'}
          </button>
        </div>
        {tvs.length > 0 && (
          <div style={S.list}>
            {tvs.map((tv) => (
              <button
                key={tv.host}
                type="button"
                onClick={() => setHost(tv.host)}
                style={{ ...S.listItem, borderColor: tv.host === host ? 'rgba(255,255,255,0.45)' : 'rgba(255,255,255,0.1)' }}
              >
                <span>{tv.name}</span>
                <span style={S.muted}>{tv.host}</span>
              </button>
            ))}
          </div>
        )}
        <input
          style={S.input}
          value={host}
          onChange={(event) => setHost(event.target.value)}
          placeholder="192.168.1.50"
          spellCheck={false}
        />
        {tvs.length === 0 && !scanning && (
          <p style={S.hint}>No TV answered. Type its IP address, shown in the Developer Mode app.</p>
        )}
      </section>

      <section style={S.section}>
        <label style={S.label}>Passphrase</label>
        <input
          style={S.input}
          value={passphrase}
          onChange={(event) => setPassphrase(event.target.value.toUpperCase())}
          placeholder="Shown in the Developer Mode app"
          spellCheck={false}
        />
      </section>

      <section style={S.section}>
        <label style={S.label}>Package</label>
        <div style={S.row}>
          <input style={{ ...S.input, flex: 1 }} value={ipkPath} readOnly placeholder="Choose the .ipk file" />
          <button type="button" style={S.ghostButton} onClick={() => void pickIpk()}>Browse</button>
        </div>
      </section>

      <label style={S.checkboxRow}>
        <input type="checkbox" checked={launch} onChange={(event) => setLaunch(event.target.checked)} />
        <span>Open Fluxa on the TV when it finishes</span>
      </label>

      <button type="button" style={{ ...S.primaryButton, opacity: ready ? 1 : 0.4 }} onClick={() => void run()} disabled={!ready}>
        {phase === 'working' ? 'Installing…' : 'Install'}
      </button>

      {phase !== 'idle' && (
        <section style={S.status}>
          {STEPS.map((step, index) => (
            <div key={step} style={S.stepRow}>
              <span style={{ ...S.stepDot, background: index <= activeStep && phase !== 'error' ? '#fff' : 'rgba(255,255,255,0.18)' }} />
              <span style={{ color: index <= activeStep ? '#fff' : 'rgba(255,255,255,0.4)' }}>{STEP_LABELS[step]}</span>
            </div>
          ))}
          {progress && phase === 'working' && <p style={S.hint}>{progress.message}</p>}
          {phase === 'done' && <p style={S.success}>Installed. Fluxa is on the TV's app list.</p>}
          {error && <p style={S.error}>{error}</p>}
        </section>
      )}
    </main>
  );
}

const STEP_LABELS: Record<string, string> = {
  key: 'Fetch developer key',
  connect: 'Connect to the TV',
  upload: 'Upload package',
  install: 'Install',
  launch: 'Launch',
};

const S = {
  page: { minHeight: '100vh', padding: '1.75rem', background: '#0b0d12', color: '#fff', fontFamily: 'system-ui, sans-serif', display: 'flex', flexDirection: 'column', gap: '1.1rem' },
  title: { margin: 0, fontSize: '1.25rem', fontWeight: 700 },
  lede: { margin: 0, fontSize: '0.85rem', color: 'rgba(255,255,255,0.6)', lineHeight: 1.5 },
  section: { display: 'flex', flexDirection: 'column', gap: '0.5rem' },
  row: { display: 'flex', gap: '0.5rem', alignItems: 'center' },
  rowBetween: { display: 'flex', justifyContent: 'space-between', alignItems: 'center' },
  label: { fontSize: '0.78rem', fontWeight: 600, color: 'rgba(255,255,255,0.75)' },
  input: { padding: '0.6rem 0.7rem', borderRadius: '0.45rem', border: '1px solid rgba(255,255,255,0.12)', background: '#141922', color: '#fff', fontSize: '0.88rem', outline: 'none' },
  list: { display: 'flex', flexDirection: 'column', gap: '0.35rem' },
  listItem: { display: 'flex', justifyContent: 'space-between', padding: '0.55rem 0.7rem', borderRadius: '0.45rem', border: '1px solid rgba(255,255,255,0.1)', background: '#141922', color: '#fff', cursor: 'pointer', fontSize: '0.85rem' },
  muted: { color: 'rgba(255,255,255,0.45)' },
  hint: { margin: 0, fontSize: '0.78rem', color: 'rgba(255,255,255,0.5)', lineHeight: 1.5 },
  checkboxRow: { display: 'flex', alignItems: 'center', gap: '0.5rem', fontSize: '0.85rem', color: 'rgba(255,255,255,0.8)' },
  primaryButton: { padding: '0.7rem', borderRadius: '0.45rem', border: 0, background: '#fff', color: '#000', fontSize: '0.9rem', fontWeight: 700, cursor: 'pointer' },
  ghostButton: { padding: '0.5rem 0.7rem', borderRadius: '0.45rem', border: '1px solid rgba(255,255,255,0.14)', background: 'transparent', color: '#fff', fontSize: '0.8rem', cursor: 'pointer' },
  status: { display: 'flex', flexDirection: 'column', gap: '0.4rem', padding: '0.85rem', borderRadius: '0.5rem', border: '1px solid rgba(255,255,255,0.1)', background: '#141922' },
  stepRow: { display: 'flex', alignItems: 'center', gap: '0.55rem', fontSize: '0.82rem' },
  stepDot: { width: '0.5rem', height: '0.5rem', borderRadius: '50%' },
  success: { margin: '0.3rem 0 0', fontSize: '0.82rem', color: '#fff' },
  error: { margin: '0.3rem 0 0', fontSize: '0.82rem', color: 'rgba(255,255,255,0.9)', lineHeight: 1.5 },
} as const;
