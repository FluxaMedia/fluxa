import React, { useCallback, useEffect, useState } from 'react';
import { t } from '../i18n';
import { coreInvoke } from '../core/engine';
import type { Meta, MetaLink } from '../core/types';

export function DiscoverDetailPanel({ meta, onPlay, onDispatch }: { meta: Meta; onPlay: () => void; onDispatch: (a: string) => void }) {
  const [imgErr, setImgErr] = useState(false);
  const bgUrl = !imgErr ? (meta.background ?? meta.poster) : null;

  const [cast, setCast] = useState<string[]>([]);
  const [directors, setDirectors] = useState<string[]>([]);
  useEffect(() => {
    let cancelled = false;
    coreInvoke<{ cast: MetaLink[]; directors: MetaLink[] }>('classifyMetaLinks', JSON.stringify(meta.links ?? [])).then((result) => {
      if (cancelled) return;
      setCast((result?.cast ?? []).map((l) => l.name).slice(0, 4));
      setDirectors((result?.directors ?? []).map((l) => l.name).slice(0, 2));
    });
    return () => {
      cancelled = true;
    };
  }, [meta.links]);

  const handleToggleWatchlist = useCallback(
    () => onDispatch(JSON.stringify({ type: 'toggleWatchlistRequested', item: meta })),
    [onDispatch, meta],
  );

  return (
    <div style={DP.wrap}>
      {bgUrl && (
        <div style={DP.bg}>
          <img src={bgUrl} alt="" decoding="async" style={DP.bgImg} onError={() => setImgErr(true)} />
          <div style={DP.bgFade} />
        </div>
      )}

      <div style={DP.content}>
        <h1 style={DP.title}>{meta.name}</h1>

        <div style={DP.metaRow}>
          {meta.runtime && <span style={DP.metaItem}>{meta.runtime}</span>}
          {meta.releaseInfo && <span style={DP.metaItem}>{meta.releaseInfo}</span>}
          {meta.imdbRating && <span style={DP.imdbBadge}>IMDb {meta.imdbRating}</span>}
        </div>

        {meta.description && <p style={DP.desc}>{meta.description}</p>}

        {Array.isArray(meta.genres) && meta.genres.length > 0 && (
          <div style={DP.section}>
            <p style={DP.sectionLabel}>{t('auto.genres')}</p>
            <div style={{ display: 'flex', gap: '0.625rem', flexWrap: 'wrap' }}>
              {meta.genres.slice(0, 5).map((g) => (
                <span key={g} style={DP.genreChip}>
                  {g}
                </span>
              ))}
            </div>
          </div>
        )}

        {cast.length > 0 && (
          <div style={DP.section}>
            <p style={DP.sectionLabel}>{t('auto.cast')}</p>
            <div style={{ display: 'flex', flexWrap: 'wrap', gap: '0.25rem 0.875rem' }}>
              {cast.map((name) => (
                <span key={name} style={DP.castName}>
                  {name}
                </span>
              ))}
            </div>
          </div>
        )}

        {directors.length > 0 && (
          <div style={DP.section}>
            <p style={DP.sectionLabel}>{directors.length > 1 ? t('detail.directors') : t('detail.director')}</p>
            <div style={{ display: 'flex', flexWrap: 'wrap', gap: '0.25rem 0.875rem' }}>
              {directors.map((name) => (
                <span key={name} style={DP.castName}>
                  {name}
                </span>
              ))}
            </div>
          </div>
        )}

        <div style={DP.actions}>
          <button style={DP.detailsBtn} onClick={onPlay}>
            {t('common.details')}
            <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
              <path d="M10 6L8.59 7.41 13.17 12l-4.58 4.59L10 18l6-6z" />
            </svg>
          </button>
          <PanelIconBtn
            title={t('discover.add_to_list')}
            icon={
              <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                <path d="M19 13H13v6h-2v-6H5v-2h6V5h2v6h6v2z" />
              </svg>
            }
            onClick={handleToggleWatchlist}
          />
          <PanelIconBtn
            title={t('detail.mark_watched')}
            icon={
              <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                <path d="M12 4.5C7 4.5 2.73 7.61 1 12c1.73 4.39 6 7.5 11 7.5s9.27-3.11 11-7.5c-1.73-4.39-6-7.5-11-7.5zM12 17c-2.76 0-5-2.24-5-5s2.24-5 5-5 5 2.24 5 5-2.24 5-5 5zm0-8c-1.66 0-3 1.34-3 3s1.34 3 3 3 3-1.34 3-3-1.34-3-3-3z" />
              </svg>
            }
          />
        </div>
      </div>
    </div>
  );
}

function PanelIconBtn({ title, icon, onClick }: { title: string; icon: React.ReactNode; onClick?: () => void }) {
  const [hovered, setHovered] = useState(false);
  return (
    <button
      title={title}
      style={{
        width: '2.375rem',
        height: '2.375rem',
        borderRadius: '0.5rem',
        border: '1px solid rgba(255,255,255,0.15)',
        background: hovered ? 'rgba(255,255,255,0.1)' : 'rgba(255,255,255,0.05)',
        color: '#FFF',
        cursor: 'pointer',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        transition: 'background 0.15s',
        flexShrink: 0,
      }}
      onClick={onClick}
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
    >
      {icon}
    </button>
  );
}

const DP: Record<string, React.CSSProperties> = {
  wrap: { position: 'relative', display: 'flex', flexDirection: 'column', minHeight: '100%' },
  bg: { position: 'absolute', top: 0, left: 0, right: 0, height: '13.75rem', zIndex: 0, overflow: 'hidden' },
  bgImg: { width: '100%', height: '100%', objectFit: 'cover', objectPosition: 'center top', opacity: 0.4 },
  bgFade: { position: 'absolute', inset: 0, background: 'linear-gradient(to bottom, transparent 0%, #0C0D18 100%)' },
  content: { position: 'relative', zIndex: 1, padding: '11.25rem 1.25rem 2rem', display: 'flex', flexDirection: 'column', flex: 1 },
  title: { color: '#FFFFFF', fontSize: '1.625rem', fontWeight: 900, margin: '0 0 0.625rem', lineHeight: 1.1, letterSpacing: '-0.0187rem' },
  metaRow: { display: 'flex', alignItems: 'center', gap: '0.875rem', marginBottom: '0.875rem', flexWrap: 'wrap' },
  metaItem: { color: 'rgba(255,255,255,0.65)', fontSize: '0.8125rem', fontWeight: 600 },
  imdbBadge: {
    color: 'rgba(255,255,255,0.5)',
    fontSize: '0.75rem',
    fontWeight: 600,
    background: 'rgba(255,255,255,0.08)',
    borderRadius: '0.25rem',
    padding: '0.125rem 0.4375rem',
    border: '1px solid rgba(255,255,255,0.1)',
  },
  desc: {
    color: 'rgba(255,255,255,0.65)',
    fontSize: '0.8125rem',
    lineHeight: '1.1875rem',
    margin: '0 0 1rem',
    display: '-webkit-box',
    WebkitLineClamp: 5,
    WebkitBoxOrient: 'vertical',
    overflow: 'hidden',
  },
  section: { marginBottom: '0.875rem' },
  sectionLabel: {
    color: 'rgba(255,255,255,0.3)',
    fontSize: '0.625rem',
    fontWeight: 700,
    letterSpacing: '0.075rem',
    textTransform: 'uppercase',
    margin: '0 0 0.375rem',
  },
  genreChip: { color: 'rgba(255,255,255,0.65)', fontSize: '0.8125rem' },
  castName: { color: 'rgba(255,255,255,0.6)', fontSize: '0.8125rem' },
  actions: { display: 'flex', alignItems: 'center', gap: '0.5rem', marginTop: 'auto', paddingTop: '1.25rem', flexWrap: 'wrap' },
  detailsBtn: {
    display: 'inline-flex',
    alignItems: 'center',
    gap: '0.3125rem',
    height: '2.375rem',
    padding: '0 1.125rem',
    background: '#FFFFFF',
    color: '#000000',
    border: 'none',
    borderRadius: '0.5rem',
    fontSize: '0.8125rem',
    fontWeight: 700,
    cursor: 'pointer',
    flexShrink: 0,
  },
};
