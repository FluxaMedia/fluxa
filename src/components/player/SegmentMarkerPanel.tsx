import { useEffect, useMemo, useState, type CSSProperties } from 'react';
import { X, Play, Square, Send, Trash2 } from 'lucide-react';
import { t } from '../../i18n';
import { POPOVER_SURFACE } from '../ui/Popover';
import { submitIntroDbSegments, submitSkipDbSegments, submitTheIntroDbSegments } from '../../core/introEffects';

type SegmentType = 'intro' | 'outro' | 'recap' | 'preview';
type ProviderId = 'introDb' | 'skipDb' | 'theIntroDb';

type PendingSegment = {
  id: string;
  type: SegmentType;
  startMs: number;
  endMs: number;
};

function fmt(ms: number): string {
  const s = Math.floor(ms / 1000);
  const m = Math.floor(s / 60);
  const rem = s % 60;
  return `${m}:${rem.toString().padStart(2, '0')}`;
}

const TYPE_OPTIONS: { value: SegmentType; labelKey: string }[] = [
  { value: 'intro', labelKey: 'player.mark_segment_type_intro' },
  { value: 'outro', labelKey: 'player.mark_segment_type_outro' },
  { value: 'recap', labelKey: 'player.mark_segment_type_recap' },
  { value: 'preview', labelKey: 'player.mark_segment_type_preview' },
];

interface SegmentMarkerPanelProps {
  onClose: () => void;
  getPosMs: () => number;
  imdbId: string | null;
  tmdbId?: number | null;
  mediaType?: 'movie' | 'tv';
  season: number | null;
  episode: number | null;
  introDbApiKey: string;
  skipDbApiKey: string;
  theIntroDbApiKey: string;
  coverage: Record<string, string[]>;
}

export function SegmentMarkerPanel({
  onClose,
  getPosMs,
  imdbId,
  tmdbId,
  mediaType,
  season,
  episode,
  introDbApiKey,
  skipDbApiKey,
  theIntroDbApiKey,
  coverage,
}: SegmentMarkerPanelProps) {
  const [segType, setSegType] = useState<SegmentType>('intro');
  const [draftStartMs, setDraftStartMs] = useState<number | null>(null);
  const [pending, setPending] = useState<PendingSegment[]>([]);
  const [status, setStatus] = useState<'idle' | 'submitting' | 'success' | 'error'>('idle');
  const [error, setError] = useState<string | null>(null);
  const [selectedProvider, setSelectedProvider] = useState<ProviderId | ''>('');

  const hasContext = !!season && !!episode && (!!imdbId || !!tmdbId);

  const availableProviders = useMemo(
    () =>
      [
        { id: 'introDb' as const, label: 'IntroDB', enabled: !!introDbApiKey && !!imdbId },
        { id: 'skipDb' as const, label: 'SkipDB', enabled: !!skipDbApiKey && !!imdbId },
        { id: 'theIntroDb' as const, label: 'TheIntroDB', enabled: !!theIntroDbApiKey && !!tmdbId },
      ].filter((p) => p.enabled),
    [introDbApiKey, skipDbApiKey, theIntroDbApiKey, imdbId, tmdbId],
  );

  useEffect(() => {
    if (availableProviders.some((p) => p.id === selectedProvider)) return;
    setSelectedProvider(availableProviders[0]?.id ?? '');
  }, [availableProviders, selectedProvider]);

  const startDraft = () => {
    setStatus('idle');
    setError(null);
    setDraftStartMs(getPosMs());
  };
  const cancelDraft = () => setDraftStartMs(null);
  const stopDraft = () => {
    if (draftStartMs == null) return;
    const endMs = getPosMs();
    if (endMs <= draftStartMs) {
      setDraftStartMs(null);
      return;
    }
    const maxDurationMs = segType === 'outro' || segType === 'preview' ? 900_000 : 300_000;
    if (endMs - draftStartMs < 5_000 || endMs - draftStartMs > maxDurationMs) {
      setError(t('player.mark_segment_duration_error'));
      setDraftStartMs(null);
      return;
    }
    setStatus('idle');
    setError(null);
    setPending((prev) => [...prev, { id: `${Date.now()}-${prev.length}`, type: segType, startMs: draftStartMs, endMs }]);
    setDraftStartMs(null);
  };
  const removePending = (id: string) => {
    setStatus('idle');
    setError(null);
    setPending((prev) => prev.filter((s) => s.id !== id));
  };

  const submit = async () => {
    if (!hasContext || pending.length === 0 || !selectedProvider) return;
    setStatus('submitting');
    try {
      const segments = pending.map((s) => ({ startTime: s.startMs, endTime: s.endMs, type: s.type }));
      if (selectedProvider === 'introDb') {
        const introDbSegments = segments.filter((s) => s.type !== 'preview');
        if (introDbSegments.length === 0) throw new Error(t('player.mark_segment_error_unknown'));
        await submitIntroDbSegments({
          apiKey: introDbApiKey,
          imdbId: imdbId!,
          season: season!,
          episode: episode!,
          segments: introDbSegments,
        });
      } else if (selectedProvider === 'skipDb') {
        await submitSkipDbSegments({ apiKey: skipDbApiKey, imdbId: imdbId!, season: season!, episode: episode!, segments });
      } else {
        await submitTheIntroDbSegments({
          apiKey: theIntroDbApiKey,
          tmdbId: tmdbId!,
          mediaType: mediaType ?? 'movie',
          imdbId: imdbId ?? undefined,
          season: season!,
          episode: episode!,
          segments,
        });
      }
      setStatus('success');
      setPending([]);
    } catch (reason) {
      setStatus('error');
      setError(reason instanceof Error ? reason.message : t('player.mark_segment_error_unknown'));
    }
  };

  return (
    <div
      style={{
        ...POPOVER_SURFACE,
        position: 'absolute',
        top: '4.5rem',
        right: '1.375rem',
        zIndex: 6,
        width: '20rem',
        padding: '0.875rem',
        display: 'flex',
        flexDirection: 'column',
        gap: '0.75rem',
      }}
      onClick={(e) => e.stopPropagation()}
      onMouseDown={(e) => e.stopPropagation()}
    >
      <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
        <span style={{ color: '#fff', fontSize: '0.8125rem', fontWeight: 700 }}>{t('player.mark_segment_title')}</span>
        <button
          onClick={onClose}
          style={{
            background: 'transparent',
            border: 'none',
            color: 'rgba(255,255,255,0.6)',
            cursor: 'pointer',
            padding: '0.125rem',
            display: 'flex',
          }}
        >
          <X size={16} />
        </button>
      </div>

      {!hasContext && <div style={{ color: 'rgba(255,255,255,0.45)', fontSize: '0.75rem' }}>{t('player.mark_segment_no_metadata')}</div>}

      {hasContext && (
        <>
          <div style={{ color: 'rgba(255,255,255,0.52)', fontSize: '0.75rem', lineHeight: 1.45 }}>{t('player.mark_segment_help')}</div>
          <div style={{ display: 'flex', gap: '0.375rem' }}>
            {TYPE_OPTIONS.map((opt) => (
              <button
                key={opt.value}
                onClick={() => setSegType(opt.value)}
                style={{
                  flex: 1,
                  padding: '0.375rem 0',
                  borderRadius: '0.375rem',
                  border: '1px solid rgba(255,255,255,0.12)',
                  background: segType === opt.value ? 'rgba(255,255,255,0.16)' : 'transparent',
                  color: segType === opt.value ? '#fff' : 'rgba(255,255,255,0.55)',
                  fontSize: '0.75rem',
                  fontWeight: 600,
                  cursor: 'pointer',
                }}
              >
                {t(opt.labelKey)}
              </button>
            ))}
          </div>

          <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
            {draftStartMs == null ? (
              <button onClick={startDraft} style={markBtnStyle}>
                <Play size={14} />
                {t('player.mark_segment_start')}
              </button>
            ) : (
              <>
                <button onClick={stopDraft} style={markBtnStyle}>
                  <Square size={14} />
                  {t('player.mark_segment_stop')}
                </button>
                <span style={{ color: 'rgba(255,255,255,0.55)', fontSize: '0.75rem' }}>
                  {t('player.mark_segment_started_at', fmt(draftStartMs))}
                </span>
                <button
                  onClick={cancelDraft}
                  style={{
                    marginLeft: 'auto',
                    background: 'transparent',
                    border: 'none',
                    color: 'rgba(255,255,255,0.45)',
                    fontSize: '0.75rem',
                    cursor: 'pointer',
                  }}
                >
                  {t('player.mark_segment_cancel')}
                </button>
              </>
            )}
          </div>

          {pending.length === 0 ? (
            <div style={{ color: 'rgba(255,255,255,0.35)', fontSize: '0.75rem', padding: '0.25rem 0' }}>
              {t('player.mark_segment_empty')}
            </div>
          ) : (
            <div style={{ display: 'flex', flexDirection: 'column', gap: '0.25rem', maxHeight: '9rem', overflowY: 'auto' }}>
              {pending.map((seg) => (
                <div
                  key={seg.id}
                  style={{
                    display: 'flex',
                    alignItems: 'center',
                    justifyContent: 'space-between',
                    padding: '0.375rem 0.5rem',
                    borderRadius: '0.375rem',
                    background: 'rgba(255,255,255,0.05)',
                  }}
                >
                  <span style={{ color: '#fff', fontSize: '0.75rem' }}>
                    {t(TYPE_OPTIONS.find((o) => o.value === seg.type)!.labelKey)} · {fmt(seg.startMs)}–{fmt(seg.endMs)}
                  </span>
                  <button
                    onClick={() => removePending(seg.id)}
                    style={{
                      background: 'transparent',
                      border: 'none',
                      color: 'rgba(255,255,255,0.45)',
                      cursor: 'pointer',
                      display: 'flex',
                    }}
                  >
                    <Trash2 size={14} />
                  </button>
                </div>
              ))}
            </div>
          )}

          {availableProviders.length > 0 && (
            <select
              value={selectedProvider}
              onChange={(e) => setSelectedProvider(e.target.value as ProviderId)}
              style={{
                padding: '0.5rem',
                borderRadius: '0.375rem',
                border: '1px solid rgba(255,255,255,0.14)',
                background: 'rgba(255,255,255,0.08)',
                color: '#fff',
                fontSize: '0.75rem',
                fontWeight: 600,
              }}
            >
              {availableProviders.map((p) => (
                <option key={p.id} value={p.id}>
                  {p.label}
                  {coverage[p.id]?.includes(segType) ? ` ✓ ${t('player.mark_segment_already_covered')}` : ''}
                </option>
              ))}
            </select>
          )}

          <button
            onClick={() => void submit()}
            disabled={pending.length === 0 || status === 'submitting' || !selectedProvider}
            style={{
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              gap: '0.375rem',
              padding: '0.5rem 0',
              borderRadius: '0.375rem',
              border: 'none',
              background: pending.length === 0 || !selectedProvider ? 'rgba(255,255,255,0.08)' : '#fff',
              color: pending.length === 0 || !selectedProvider ? 'rgba(255,255,255,0.35)' : '#000',
              fontSize: '0.8125rem',
              fontWeight: 700,
              cursor: pending.length === 0 || !selectedProvider ? 'default' : 'pointer',
            }}
          >
            <Send size={14} />
            {status === 'submitting' ? t('player.mark_segment_submitting') : t('player.mark_segment_submit', String(pending.length))}
          </button>

          {status === 'success' && <div style={{ color: '#54D17A', fontSize: '0.75rem' }}>{t('player.mark_segment_success')}</div>}
          {error && (
            <div style={{ color: '#FF8A3D', fontSize: '0.75rem', lineHeight: 1.4 }}>
              {status === 'error' ? t('player.mark_segment_error', error) : error}
            </div>
          )}
        </>
      )}
    </div>
  );
}

const markBtnStyle: CSSProperties = {
  display: 'flex',
  alignItems: 'center',
  gap: '0.375rem',
  padding: '0.375rem 0.625rem',
  borderRadius: '0.375rem',
  border: '1px solid rgba(255,255,255,0.14)',
  background: 'rgba(255,255,255,0.08)',
  color: '#fff',
  fontSize: '0.75rem',
  fontWeight: 600,
  cursor: 'pointer',
};
