import React, { useState } from 'react';
import { t } from '../i18n';
import type { Meta } from '../core/types';

export function DiscoverDetailPanel({
  meta,
  onPlay,
  onDispatch,
}: {
  meta: Meta;
  onPlay: () => void;
  onDispatch: (a: string) => void;
}) {
  const [imgErr, setImgErr] = useState(false);
  const bgUrl = !imgErr ? (meta.background ?? meta.poster) : null;

  const cast = (meta.links ?? [])
    .filter((l) => l.category.toLowerCase().includes('cast') || l.category.toLowerCase() === 'actor')
    .map((l) => l.name)
    .slice(0, 4);

  const directors = (meta.links ?? [])
    .filter((l) => l.category.toLowerCase().includes('director'))
    .map((l) => l.name)
    .slice(0, 2);

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
          {meta.imdbRating && (
            <div style={DP.imdbBadge}>
              <span style={DP.imdbLabel}>IMDb</span>
              <span style={DP.imdbVal}>{meta.imdbRating}</span>
            </div>
          )}
        </div>

        {meta.description && <p style={DP.desc}>{meta.description}</p>}

        {meta.genres && meta.genres.length > 0 && (
          <div style={DP.section}>
            <p style={DP.sectionLabel}>{t('auto.genres')}</p>
            <div style={{ display: 'flex', gap: 10, flexWrap: 'wrap' }}>
              {meta.genres.slice(0, 5).map((g) => (
                <span key={g} style={DP.genreChip}>{g}</span>
              ))}
            </div>
          </div>
        )}

        {cast.length > 0 && (
          <div style={DP.section}>
            <p style={DP.sectionLabel}>{t('auto.cast')}</p>
            <div style={{ display: 'flex', flexWrap: 'wrap', gap: '6px 20px' }}>
              {cast.map((name) => (
                <span key={name} style={DP.castName}>{name}</span>
              ))}
            </div>
          </div>
        )}

        {directors.length > 0 && (
          <div style={DP.section}>
            <p style={DP.sectionLabel}>{directors.length > 1 ? t('detail.directors') : t('detail.director')}</p>
            <div style={{ display: 'flex', flexWrap: 'wrap', gap: '6px 20px' }}>
              {directors.map((name) => (
                <span key={name} style={DP.castName}>{name}</span>
              ))}
            </div>
          </div>
        )}

        <div style={DP.actions}>
          <button style={DP.playBtn} onClick={onPlay}>
            <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor"><path d="M8 5v14l11-7z" /></svg>
            {t('common.details')}
          </button>
          <PanelIconBtn
            title={t('discover.add_to_list')}
            icon={<svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor"><path d="M19 13H13v6h-2v-6H5v-2h6V5h2v6h6v2z" /></svg>}
            onClick={() => onDispatch(JSON.stringify({ type: 'toggleWatchlistRequested', item: meta }))}
          />
          <PanelIconBtn
            title={t('detail.mark_watched')}
            icon={<svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor"><path d="M12 4.5C7 4.5 2.73 7.61 1 12c1.73 4.39 6 7.5 11 7.5s9.27-3.11 11-7.5c-1.73-4.39-6-7.5-11-7.5zM12 17c-2.76 0-5-2.24-5-5s2.24-5 5-5 5 2.24 5 5-2.24 5-5 5zm0-8c-1.66 0-3 1.34-3 3s1.34 3 3 3 3-1.34 3-3-1.34-3-3-3z" /></svg>}
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
        width: 38, height: 38,
        borderRadius: '50%',
        border: '1px solid rgba(255,255,255,0.15)',
        background: hovered ? 'rgba(255,255,255,0.14)' : 'rgba(255,255,255,0.06)',
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
  bg: { position: 'absolute', top: 0, left: 0, right: 0, height: 220, zIndex: 0, overflow: 'hidden' },
  bgImg: { width: '100%', height: '100%', objectFit: 'cover', objectPosition: 'center top', opacity: 0.4 },
  bgFade: { position: 'absolute', inset: 0, background: 'linear-gradient(to bottom, transparent 0%, #0C0D18 100%)' },
  content: { position: 'relative', zIndex: 1, padding: '180px 20px 32px', display: 'flex', flexDirection: 'column', flex: 1 },
  title: { color: '#FFFFFF', fontSize: 26, fontWeight: 900, margin: '0 0 10px', fontFamily: 'sans-serif', lineHeight: 1.1, letterSpacing: '-0.3px' },
  metaRow: { display: 'flex', alignItems: 'center', gap: 14, marginBottom: 14, flexWrap: 'wrap' },
  metaItem: { color: 'rgba(255,255,255,0.65)', fontSize: 13, fontWeight: 600, fontFamily: 'sans-serif' },
  imdbBadge: { display: 'flex', alignItems: 'center', gap: 4, background: '#F5C518', borderRadius: 4, padding: '2px 6px' },
  imdbLabel: { color: '#000', fontSize: 10, fontWeight: 900, fontFamily: 'sans-serif' },
  imdbVal: { color: '#000', fontSize: 11, fontWeight: 900, fontFamily: 'sans-serif' },
  desc: { color: 'rgba(255,255,255,0.65)', fontSize: 13, lineHeight: '19px', margin: '0 0 16px', fontFamily: 'sans-serif', display: '-webkit-box', WebkitLineClamp: 5, WebkitBoxOrient: 'vertical', overflow: 'hidden' },
  section: { marginBottom: 14 },
  sectionLabel: { color: 'rgba(255,255,255,0.3)', fontSize: 10, fontWeight: 700, letterSpacing: '1.2px', textTransform: 'uppercase', margin: '0 0 6px', fontFamily: 'sans-serif' },
  genreChip: { color: 'rgba(255,255,255,0.65)', fontSize: 13, fontFamily: 'sans-serif' },
  castName: { color: 'rgba(255,255,255,0.6)', fontSize: 13, fontFamily: 'sans-serif' },
  actions: { display: 'flex', alignItems: 'center', gap: 10, marginTop: 'auto', paddingTop: 20, flexWrap: 'wrap' },
  playBtn: { display: 'inline-flex', alignItems: 'center', gap: 7, height: 38, padding: '0 18px', background: '#FFFFFF', color: '#000000', border: 'none', borderRadius: 8, fontSize: 13, fontWeight: 900, cursor: 'pointer', fontFamily: 'sans-serif', flexShrink: 0 },
};
