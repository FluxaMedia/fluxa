import { t } from '../../i18n';

interface Props {
  title: string;
  episodeTitle?: string;
  logoUrl?: string;
  description?: string;
}

export function PlayerPauseMetadataOverlay({ title, episodeTitle, logoUrl, description }: Props) {
  return (
    <div
      style={{
        position: 'absolute',
        inset: 0,
        zIndex: 5,
        display: 'flex',
        flexDirection: 'column',
        justifyContent: 'flex-end',
        padding: '2.5rem 3.5rem 7.5rem',
        pointerEvents: 'none',
        background: 'linear-gradient(90deg, rgba(0,0,0,0.85) 0%, rgba(0,0,0,0.45) 48%, rgba(0,0,0,0) 100%)',
      }}
    >
      <p style={{ color: 'rgba(255,255,255,0.72)', fontSize: '0.9375rem', margin: 0 }}>{t('player.youre_watching')}</p>
      {logoUrl ? (
        <img
          src={logoUrl}
          alt={title}
          style={{ height: '6rem', maxWidth: 'min(22.5rem, 62vw)', objectFit: 'contain', objectPosition: 'left bottom', marginTop: '0.75rem' }}
        />
      ) : (
        <p
          style={{
            color: '#FFFFFF',
            fontSize: '2rem',
            fontWeight: 800,
            margin: '0.75rem 0 0',
            maxWidth: '62%',
            overflow: 'hidden',
            textOverflow: 'ellipsis',
            display: '-webkit-box',
            WebkitLineClamp: 2,
            WebkitBoxOrient: 'vertical',
          }}
        >
          {title}
        </p>
      )}
      {episodeTitle && (
        <p style={{ color: '#FFFFFF', fontSize: '1.375rem', fontWeight: 700, margin: '0.75rem 0 0' }}>{episodeTitle}</p>
      )}
      {description && (
        <p
          style={{
            color: 'rgba(255,255,255,0.84)',
            fontSize: '0.9375rem',
            lineHeight: 1.5,
            margin: '1rem 0 0',
            width: '62%',
            overflow: 'hidden',
            textOverflow: 'ellipsis',
            display: '-webkit-box',
            WebkitLineClamp: 3,
            WebkitBoxOrient: 'vertical',
          }}
        >
          {description}
        </p>
      )}
    </div>
  );
}
