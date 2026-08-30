import { useEffect, useState } from 'react';
import { t } from '../../i18n';
import { coreShortenSynopsis } from '../../core/engine';

interface Props {
  title: string;
  episodeTitle?: string;
  logoUrl?: string;
  description?: string;
  chapterTitle?: string;
}

export function PlayerPauseMetadataOverlay({ title, episodeTitle, logoUrl, description, chapterTitle }: Props) {
  const [shortDescription, setShortDescription] = useState(description);

  useEffect(() => {
    let active = true;
    if (!description?.trim()) {
      setShortDescription(undefined);
      return () => {
        active = false;
      };
    }
    setShortDescription(undefined);
    void coreShortenSynopsis(description).then((value) => {
      if (active) setShortDescription(value);
    });
    return () => {
      active = false;
    };
  }, [description]);

  return (
    <div
      style={{
        position: 'absolute',
        inset: 0,
        zIndex: 5,
        display: 'flex',
        flexDirection: 'column',
        justifyContent: 'flex-end',
        padding: '2.5rem 2.5rem 7.5rem 4rem',
        pointerEvents: 'none',
        background: 'linear-gradient(90deg, rgba(0,0,0,0.85) 0%, rgba(0,0,0,0.45) 48%, rgba(0,0,0,0) 100%)',
      }}
    >
      <p style={{ color: 'rgba(255,255,255,0.72)', fontSize: '0.9375rem', margin: 0 }}>{t('player.youre_watching')}</p>
      {logoUrl ? (
        <img
          src={logoUrl}
          alt={title}
          style={{ height: '6rem', maxWidth: 'min(20rem, 43vw)', objectFit: 'contain', objectPosition: 'left bottom', marginTop: '0.75rem' }}
        />
      ) : (
        <p
          style={{
            color: '#FFFFFF',
            fontSize: '2rem',
            fontWeight: 800,
            margin: '0.75rem 0 0',
            maxWidth: 'min(38rem, 43vw)',
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
        <p style={{ color: '#FFFFFF', fontSize: '1.375rem', fontWeight: 700, margin: '0.75rem 0 0', maxWidth: '43vw' }}>{episodeTitle}</p>
      )}
      {chapterTitle && (
        <p style={{ color: 'rgba(255,255,255,0.72)', fontSize: '0.9375rem', margin: '0.5rem 0 0' }}>
          {t('player.chapter')}: {chapterTitle}
        </p>
      )}
      {shortDescription && (
        <p
          style={{
            color: 'rgba(255,255,255,0.84)',
            fontSize: '0.9375rem',
            lineHeight: 1.5,
            margin: '1rem 0 0',
            width: 'min(38rem, 43vw)',
            maxWidth: '100%',
            overflow: 'hidden',
            textOverflow: 'ellipsis',
            display: '-webkit-box',
            WebkitLineClamp: 5,
            WebkitBoxOrient: 'vertical',
          }}
        >
          {shortDescription}
        </p>
      )}
    </div>
  );
}
