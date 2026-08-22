import React, { useEffect, useMemo, useState } from 'react';
import { FolderOpen, Loader2, RefreshCw } from 'lucide-react';
import { platformOpenDialog } from '../platform/browser';
import { LocalMediaItem, localStream, resolveLocalMedia } from '../core/localMedia';
import type { Meta, Stream, Video } from '../core/types';
import { VirtualizedPosterGrid } from './VirtualizedPosterGrid';
import type { PosterPrefs } from '../core/posterPrefs';

export function LocalMediaPanel({
  posterPrefs,
  onPlay,
}: {
  posterPrefs: PosterPrefs;
  onPlay: (stream: Stream, meta: Meta, episode?: Video) => void;
}) {
  const [root, setRoot] = useState(() => localStorage.getItem('fluxa.localMedia.root') ?? '');
  const [items, setItems] = useState<LocalMediaItem[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const scan = async (directory = root) => {
    if (!directory) return;
    setLoading(true);
    setError(null);
    try {
      const result = await resolveLocalMedia(directory);
      setItems(result);
      setRoot(directory);
      localStorage.setItem('fluxa.localMedia.root', directory);
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : 'Local media scan failed');
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    if (root) void scan();
  }, []);

  const chooseFolder = async () => {
    const selected = await platformOpenDialog({ directory: true, multiple: false, title: 'Select local media folder' });
    if (typeof selected === 'string') await scan(selected);
  };

  const metas = useMemo(() => items as unknown as Meta[], [items]);
  return (
    <div style={{ flex: 1, minHeight: 0, display: 'flex', flexDirection: 'column' }}>
      <div style={{ display: 'flex', alignItems: 'center', gap: 10, padding: '8px 24px 14px' }}>
        <button style={buttonStyle} onClick={() => void chooseFolder()}>
          <FolderOpen size={16} /> Select folder
        </button>
        {root && (
          <span
            style={{ color: 'rgba(255,255,255,0.55)', fontSize: 12, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}
          >
            {root}
          </span>
        )}
        {root && (
          <button style={iconButtonStyle} onClick={() => void scan()} title="Refresh">
            <RefreshCw size={15} />
          </button>
        )}
        {loading && <Loader2 size={16} color="#aaa" className="spin" />}
      </div>
      {error && <p style={{ color: '#ff8e8e', padding: '0 24px' }}>{error}</p>}
      {!loading && !error && !items.length && (
        <p style={{ color: 'rgba(255,255,255,0.55)', padding: '24px' }}>Choose a folder to scan local movies and shows.</p>
      )}
      {!!metas.length && (
        <VirtualizedPosterGrid
          items={metas}
          selectedId={null}
          posterPrefs={posterPrefs}
          onHover={() => false}
          onClick={(meta) => {
            const item = items.find((candidate) => candidate.id === meta.id);
            const file = item?.localFiles[0];
            if (item && file) {
              const playback = localStream(item, file);
              onPlay(playback.stream, item, playback.episode);
            }
          }}
          onScrollActivity={() => {}}
        />
      )}
    </div>
  );
}

const buttonStyle: React.CSSProperties = {
  display: 'inline-flex',
  alignItems: 'center',
  gap: 7,
  border: '1px solid rgba(255,255,255,0.15)',
  borderRadius: 7,
  background: 'rgba(255,255,255,0.08)',
  color: '#fff',
  padding: '8px 12px',
  cursor: 'pointer',
};
const iconButtonStyle: React.CSSProperties = { ...buttonStyle, padding: 8 };
