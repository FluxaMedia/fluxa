interface Props {
  background?: string | null;
  logo?: string | null;
  title?: string;
  episodeLine?: string;
}

export function PlayerLoadingOverlay({ background, logo, title, episodeLine }: Props) {
  return (
    <div
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
      <div style={{ position: 'relative', zIndex: 1, textAlign: 'center', padding: '0 40px' }}>
        {logo ? (
          <img
            src={logo}
            alt={title ?? ''}
            style={{
              maxWidth: 480,
              maxHeight: 160,
              objectFit: 'contain',
              filter: 'drop-shadow(0 4px 24px rgba(0,0,0,0.8))',
            }}
          />
        ) : (
          <span
            style={{
              color: '#fff',
              fontSize: 36,
              fontWeight: 900,
              fontFamily: "'Montserrat', sans-serif",
              letterSpacing: '-0.5px',
              textShadow: '0 2px 16px rgba(0,0,0,0.8)',
            }}
          >
            {title ?? 'Fluxa'}
          </span>
        )}
        {episodeLine && (
          <p
            style={{
              color: 'rgba(255,255,255,0.65)',
              fontSize: 15,
              marginTop: 12,
              fontFamily: 'sans-serif',
              textShadow: '0 1px 8px rgba(0,0,0,0.8)',
            }}
          >
            {episodeLine}
          </p>
        )}
      </div>
    </div>
  );
}
