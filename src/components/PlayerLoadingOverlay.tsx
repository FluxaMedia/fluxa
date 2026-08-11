import { ChevronLeft, TriangleAlert } from 'lucide-react';
import { useEffect, useState } from 'react';
import { subscribePlayerStatus } from '../core/playerStatusStore';
import { playerTorrentStats } from '../core/mpvPlayer';
import { t } from '../i18n';

interface Props {
  background?: string | null;
  logo?: string | null;
  title?: string;
  episodeLine?: string;
  status?: string;
  error?: string | null;
  isTorrentStream?: boolean;
  source?: {
    title?: string;
    addon?: string;
    filename?: string;
    fileIdx?: number;
    infoHash?: string;
    sources?: string[];
  };
  onBack?: () => void;
}

const BUFFER_TARGET_SECS = 5;
const MIN_VISIBLE_PROGRESS = 0.045;

export function PlayerLoadingOverlay({ background, logo, title, episodeLine, status, error, isTorrentStream, source, onBack }: Props) {
  const [hasMeasuredProgress, setHasMeasuredProgress] = useState(false);
  const [progress, setProgress] = useState(0);
  const [detailsOpen, setDetailsOpen] = useState(false);
  const [copied, setCopied] = useState(false);
  const [logoLoadFailed, setLogoLoadFailed] = useState(false);
  const failed = !!error;

  const errorLines = (error ?? '').split('\n').map((l) => l.trim()).filter(Boolean);
  const errorSummary = errorLines[0] ? errorLines[0].charAt(0).toUpperCase() + errorLines[0].slice(1) : '';
  const errorDetails = errorLines.slice(1).join('\n');

  const copyErrorDetails = () => {
    const sourceDetails = source ? JSON.stringify(source, null, 2) : '';
    void navigator.clipboard.writeText([error, sourceDetails].filter(Boolean).join('\n\n')).then(() => {
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    }).catch(() => undefined);
  };

  useEffect(() => {
    setLogoLoadFailed(false);
  }, [logo]);

  useEffect(() => {
    if (failed) return;
    return subscribePlayerStatus((status) => {
      const bufferingPct = parseFloat(status.cacheBufferingState ?? '');
      const cached = parseFloat(status.demuxerCacheDuration ?? '0') || 0;
      let next = 0;
      if (status.pausedForCache === 'yes' && Number.isFinite(bufferingPct) && bufferingPct > 0) {
        next = bufferingPct / 100;
      } else if (cached > 0.05) {
        next = cached / BUFFER_TARGET_SECS;
      }
      if (next > 0) {
        setHasMeasuredProgress(true);
        setProgress((prev) => Math.max(prev, MIN_VISIBLE_PROGRESS, Math.min(1, next)));
      }
    });
  }, [failed]);

  useEffect(() => {
    if (failed || !isTorrentStream) return;
    let cancelled = false;
    const poll = async () => {
      const ts = await playerTorrentStats().catch(() => null);
      if (cancelled || !ts) return;
      const next = (ts.preload || 0) / 100;
      if (next > 0) {
        setHasMeasuredProgress(true);
        setProgress((prev) => Math.max(prev, MIN_VISIBLE_PROGRESS, Math.min(1, next)));
      }
    };
    void poll();
    const id = setInterval(() => { void poll(); }, 500);
    return () => { cancelled = true; clearInterval(id); };
  }, [failed, isTorrentStream]);

  return (
    <div
      className="fluxa-player-loading-overlay"
      style={{
        position: 'fixed',
        inset: 0,
        zIndex: 9999,
        background: '#000',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
      }}
    >
      {onBack && (
        <button
          onClick={onBack}
          style={{
            position: 'absolute',
            top: '1rem',
            left: '1rem',
            zIndex: 1,
            background: 'rgba(0,0,0,0.45)',
            backdropFilter: 'blur(0.375rem)',
            border: 'none',
            borderRadius: '0.625rem',
            width: '2.625rem',
            height: '2.625rem',
            color: '#fff',
            cursor: 'pointer',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
          }}
        >
          <ChevronLeft size={22} />
        </button>
      )}
      {background && (
        <img
          src={background}
          alt=""
          style={{
            position: 'absolute',
            inset: 0,
            width: '100%',
            height: '100%',
            objectFit: 'cover',
            opacity: 0.35,
            pointerEvents: 'none',
          }}
        />
      )}
      <div
        style={{
          position: 'absolute',
          inset: 0,
          background: 'linear-gradient(to bottom, rgba(0,0,0,0.1) 0%, rgba(0,0,0,0.65) 100%)',
          pointerEvents: 'none',
        }}
      />
      <div style={{ position: 'relative', zIndex: 1, textAlign: 'center', padding: '0 2.5rem' }}>
        {logo && !logoLoadFailed ? (
          <div style={{ position: 'relative', width: '30rem', maxWidth: '100%', height: '10rem', margin: '0 auto' }}>
            <img
              src={logo}
              alt={title ?? ''}
              onError={() => setLogoLoadFailed(true)}
              className={failed || hasMeasuredProgress ? 'fluxa-loading-logo-dim fluxa-loading-motion' : 'fluxa-loading-logo-breathe fluxa-loading-motion'}
              style={{
                position: 'absolute',
                inset: 0,
                width: '100%',
                height: '100%',
                objectFit: 'contain',
                opacity: failed ? 0.55 : hasMeasuredProgress ? 0.35 : undefined,
                filter: failed || hasMeasuredProgress
                  ? 'drop-shadow(0 0.25rem 1.5rem rgba(0,0,0,0.8)) brightness(0.72)'
                  : 'drop-shadow(0 0.25rem 1.5rem rgba(0,0,0,0.8))',
              }}
            />
            {!failed && hasMeasuredProgress && (
              <div
                className="fluxa-loading-logo-reveal"
                style={{
                  position: 'absolute',
                  inset: 0,
                  clipPath: `inset(0 ${(100 - progress * 100).toFixed(2)}% 0 0)`,
                }}
              >
                <img
                  src={logo}
                  alt=""
                  style={{
                    position: 'absolute',
                    inset: 0,
                    width: '100%',
                    height: '100%',
                    objectFit: 'contain',
                    filter: 'drop-shadow(0 0.25rem 1.5rem rgba(0,0,0,0.8))',
                  }}
                />
              </div>
            )}
          </div>
        ) : (
          <span
            className={failed ? undefined : 'fluxa-loading-logo-breathe fluxa-loading-motion'}
            style={{
              color: '#fff',
              fontSize: '2.25rem',
              fontWeight: 900,
              fontFamily: "'Montserrat', sans-serif",
              letterSpacing: '-0.0313rem',
              textShadow: '0 0.125rem 1rem rgba(0,0,0,0.8)',
              opacity: failed ? 0.75 : undefined,
              display: 'inline-block',
            }}
          >
            {title ?? 'Fluxa'}
          </span>
        )}
        {episodeLine && (
          <p
            style={{
              color: 'rgba(255,255,255,0.65)',
              fontSize: '0.9375rem',
              marginTop: '0.75rem',
              textShadow: '0 1px 0.5rem rgba(0,0,0,0.8)',
            }}
          >
            {episodeLine}
          </p>
        )}
        {!failed && status && (
          <p
            style={{
              color: 'rgba(255,255,255,0.4)',
              fontSize: '0.7813rem',
              fontWeight: 600,
              letterSpacing: '0.0125rem',
              marginTop: '0.625rem',
              textShadow: '0 1px 0.5rem rgba(0,0,0,0.8)',
            }}
          >
            {status}
          </p>
        )}
        {failed && (
          <div
            style={{
              marginTop: '1.625rem',
              display: 'flex',
              flexDirection: 'column',
              alignItems: 'center',
              gap: '0.875rem',
              width: 'min(32.5rem, 100%)',
              marginLeft: 'auto',
              marginRight: 'auto',
            }}
          >
            <div
              style={{
                width: '100%',
                boxSizing: 'border-box',
                background: 'rgba(18,18,22,0.72)',
                border: '1px solid rgba(255,255,255,0.08)',
                borderRadius: '0.875rem',
                backdropFilter: 'blur(1rem)',
                boxShadow: '0 1rem 3rem rgba(0,0,0,0.5)',
                padding: '1rem 1.125rem',
                textAlign: 'left',
              }}
            >
              <div style={{ display: 'flex', alignItems: 'center', gap: '0.75rem' }}>
                <span
                  style={{
                    width: '2.25rem',
                    height: '2.25rem',
                    borderRadius: '0.625rem',
                    background: 'rgba(255,90,90,0.13)',
                    display: 'flex',
                    alignItems: 'center',
                    justifyContent: 'center',
                    flexShrink: 0,
                  }}
                >
                  <TriangleAlert size={18} color="#ff7b7b" />
                </span>
                <div style={{ minWidth: 0 }}>
                  <p style={{ color: '#fff', fontSize: '0.9375rem', fontWeight: 700, margin: 0, letterSpacing: '0.0063rem' }}>
                    {t('player.playback_error') || 'Playback failed'}
                  </p>
                  <p
                    style={{
                      color: 'rgba(255,255,255,0.6)',
                      fontSize: '0.8125rem',
                      margin: '0.1875rem 0 0',
                      lineHeight: 1.4,
                      overflow: 'hidden',
                      textOverflow: 'ellipsis',
                      whiteSpace: 'nowrap',
                    }}
                  >
                    {errorSummary}
                  </p>
                </div>
              </div>
              {errorDetails && (
                <>
                  <button
                    onClick={() => setDetailsOpen((v) => !v)}
                    style={{
                      background: 'none',
                      border: 'none',
                      padding: 0,
                      marginTop: '0.75rem',
                      color: 'rgba(255,255,255,0.45)',
                      fontSize: '0.75rem',
                      fontWeight: 600,
                      cursor: 'pointer',
                      letterSpacing: '0.0125rem',
                    }}
                  >
                    {detailsOpen ? t('player.error_hide_details') : t('player.error_show_details')}
                  </button>
                  {detailsOpen && (
                    <pre
                      style={{
                        margin: '0.625rem 0 0',
                        padding: '0.625rem 0.75rem',
                        background: 'rgba(0,0,0,0.4)',
                        border: '1px solid rgba(255,255,255,0.06)',
                        borderRadius: '0.5rem',
                        color: 'rgba(255,255,255,0.55)',
                        fontSize: '0.7188rem',
                        fontFamily: "'Cascadia Mono', 'Consolas', monospace",
                        lineHeight: 1.55,
                        whiteSpace: 'pre-wrap',
                        wordBreak: 'break-word',
                        maxHeight: '8.75rem',
                        overflowY: 'auto',
                      }}
                    >
                      {errorDetails}
                    </pre>
                  )}
                </>
              )}
            </div>
            <div style={{ display: 'flex', gap: '0.625rem' }}>
              {onBack && (
                <button
                  onClick={onBack}
                  style={{
                    background: '#fff',
                    border: 'none',
                    borderRadius: '0.625rem',
                    padding: '0.625rem 1.75rem',
                    color: '#000',
                    fontSize: '0.8438rem',
                    fontWeight: 700,
                    cursor: 'pointer',
                  }}
                >
                  {t('common.back') || 'Go Back'}
                </button>
              )}
              <button
                onClick={copyErrorDetails}
                style={{
                  background: 'rgba(255,255,255,0.08)',
                  border: '1px solid rgba(255,255,255,0.14)',
                  borderRadius: '0.625rem',
                  padding: '0.625rem 1.25rem',
                  color: 'rgba(255,255,255,0.85)',
                  fontSize: '0.8438rem',
                  fontWeight: 600,
                  cursor: 'pointer',
                  backdropFilter: 'blur(0.375rem)',
                }}
              >
                {copied ? t('player.error_copied') : t('player.error_copy')}
              </button>
            </div>
          </div>
        )}
      </div>
      <style>{`
        @keyframes fluxa-loading-logo-breathe {
          from { opacity: 0.38; transform: scale(0.992); }
          to { opacity: 0.86; transform: scale(1.012); }
        }
        .fluxa-player-loading-overlay .fluxa-loading-logo-breathe {
          animation: fluxa-loading-logo-breathe 1.08s cubic-bezier(0.4,0,0.2,1) infinite alternate !important;
          transform-origin: center;
          will-change: opacity, transform;
        }
        .fluxa-player-loading-overlay .fluxa-loading-logo-dim {
          opacity: 0.35;
          filter: drop-shadow(0 0.25rem 1.5rem rgba(0,0,0,0.8)) brightness(0.72) !important;
          transition: opacity 0.2s ease, filter 0.2s ease !important;
        }
        .fluxa-player-loading-overlay .fluxa-loading-logo-reveal {
          transition: clip-path 0.42s cubic-bezier(0.4,0,0.2,1) !important;
          will-change: clip-path;
        }
      `}</style>
    </div>
  );
}
