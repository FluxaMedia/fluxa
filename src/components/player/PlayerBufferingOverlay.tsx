interface Props {
  logoUrl?: string;
  progress: number;
}

export function PlayerBufferingOverlay({ logoUrl, progress }: Props) {
  return (
    <div
      style={{
        position: 'absolute',
        inset: 0,
        zIndex: 4,
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'center',
        justifyContent: 'center',
        gap: '1rem',
        background: 'rgba(0,0,0,0.48)',
        pointerEvents: 'none',
      }}
    >
      {logoUrl && (
        <div style={{ position: 'relative', width: 'min(30rem, 72vw)', height: '10rem' }}>
          <img
            src={logoUrl}
            alt=""
            className="fluxa-loading-logo-dim fluxa-loading-motion"
            style={{
              position: 'absolute',
              inset: 0,
              width: '100%',
              height: '100%',
              objectFit: 'contain',
              opacity: 0.35,
              filter: 'drop-shadow(0 0.25rem 1.5rem rgba(0,0,0,0.8)) brightness(0.72)',
            }}
          />
          <div
            className="fluxa-loading-logo-reveal"
            style={{ position: 'absolute', inset: 0, clipPath: `inset(0 ${(100 - progress).toFixed(2)}% 0 0)` }}
          >
            <img
              src={logoUrl}
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
        </div>
      )}
    </div>
  );
}
