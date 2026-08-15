import { useRef, useState } from 'react';
import { platformEmit as emit } from '../../platform/browser';
import { ChevronDown } from 'lucide-react';
import { t } from '../../i18n';
import { Popover } from '../ui/Popover';

export type EpisodeInfo = {
  id: string;
  name?: string;
  title?: string;
  season?: number;
  episode?: number;
  number?: number;
  thumbnail?: string;
  overview?: string;
};

export function epLabel(ep: EpisodeInfo): string {
  const s = ep.season != null ? `${t('format.season_abbrev')}${ep.season}` : '';
  const e = (ep.episode ?? ep.number) != null ? `${t('format.episode_abbrev')}${ep.episode ?? ep.number}` : '';
  const se = s || e ? `${s}${e} — ` : '';
  return se + (ep.name ?? ep.title ?? '');
}

function epKey(ep: EpisodeInfo): string {
  return `${ep.season ?? 1}:${ep.episode ?? ep.number ?? 0}`;
}

const iconBtn: React.CSSProperties = {
  background: 'none', border: 'none', color: '#fff', cursor: 'pointer',
  display: 'flex', alignItems: 'center', justifyContent: 'center',
  width: '2.75rem', height: '2.75rem', borderRadius: '0.5rem', padding: 0, flexShrink: 0,
};

interface EpisodePanelProps {
  episodes: EpisodeInfo[];
  currentEpisode: EpisodeInfo | null;
  onClose: () => void;
}

export function EpisodePanel({ episodes, currentEpisode, onClose }: EpisodePanelProps) {
  const seasonGroups = episodes.reduce<Record<number, EpisodeInfo[]>>((acc, ep) => {
    const s = ep.season ?? 1;
    if (!acc[s]) acc[s] = [];
    acc[s].push(ep);
    return acc;
  }, {});
  const seasons = Object.keys(seasonGroups).map(Number).sort((a, b) => a - b);

  const currentEpisodeKey = currentEpisode ? epKey(currentEpisode) : null;

  const [activeSeason, setActiveSeason] = useState(() => currentEpisode?.season ?? seasons[0] ?? 1);
  const [showSeasonDropdown, setShowSeasonDropdown] = useState(false);
  const seasonBtnRef = useRef<HTMLButtonElement>(null);

  return (
    <div
      style={{ position: 'absolute', top: 0, right: 0, bottom: 0, width: '23.75rem', background: 'rgba(10,12,18,0.97)', backdropFilter: 'blur(1.25rem)', borderLeft: '1px solid rgba(255,255,255,0.08)', zIndex: 6, display: 'flex', flexDirection: 'column', overflow: 'hidden' }}
      onClick={(e) => e.stopPropagation()}
    >
      <div style={{ padding: '1rem 1rem 0.625rem', display: 'flex', alignItems: 'center', justifyContent: 'space-between', flexShrink: 0 }}>
        <span style={{ color: '#fff', fontWeight: 700, fontSize: '0.9375rem' }}>{t('player.episodes')}</span>
        <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
          {seasons.length > 1 && (
            <div style={{ position: 'relative' }}>
              <button
                ref={seasonBtnRef}
                onClick={() => setShowSeasonDropdown((p) => !p)}
                style={{ background: 'rgba(255,255,255,0.1)', border: '1px solid rgba(255,255,255,0.15)', borderRadius: '0.375rem', color: '#fff', fontSize: '0.75rem', fontWeight: 600, padding: '0.25rem 0.5rem', cursor: 'pointer', display: 'flex', alignItems: 'center', gap: '0.25rem' }}
              >
                {t('player.season_n', activeSeason)}
                <ChevronDown size={12} />
              </button>
              <Popover open={showSeasonDropdown} onClose={() => setShowSeasonDropdown(false)} anchorRef={seasonBtnRef} placement="bottom-end" width="7.5rem">
                {seasons.map((s) => (
                  <button
                    key={s}
                    className="ui-popover-row"
                    onClick={() => { setActiveSeason(s); setShowSeasonDropdown(false); }}
                    style={{ display: 'block', width: '100%', background: s === activeSeason ? 'rgba(255,255,255,0.08)' : 'none', border: 'none', color: s === activeSeason ? '#fff' : 'rgba(255,255,255,0.65)', fontSize: '0.75rem', fontWeight: s === activeSeason ? 700 : 400, padding: '0.5rem 0.75rem', cursor: 'pointer', textAlign: 'left' }}
                  >
                    {t('player.season_n', s)}
                  </button>
                ))}
              </Popover>
            </div>
          )}
          <button
            onClick={onClose}
            style={{ ...iconBtn, width: '2rem', height: '2rem', opacity: 0.6 }}
          >
            ✕
          </button>
        </div>
      </div>
      <div style={{ flex: 1, overflowY: 'auto', padding: '0.25rem 0 0.5rem' }}>
        {(seasonGroups[activeSeason] ?? []).map((ep) => {
          const isCurrent = currentEpisodeKey === epKey(ep);
          return (
            <button
              key={ep.id}
              onClick={() => { void emit('native-player-play-episode', ep.id); onClose(); }}
              style={{ display: 'flex', alignItems: 'flex-start', gap: '0.75rem', width: '100%', background: isCurrent ? 'rgba(255,255,255,0.06)' : 'none', border: 'none', borderLeft: isCurrent ? '0.125rem solid #fff' : '0.125rem solid transparent', borderBottom: '1px solid rgba(255,255,255,0.05)', color: '#fff', padding: '0.75rem 1rem', cursor: 'pointer', textAlign: 'left' }}
              onMouseEnter={(e) => (e.currentTarget.style.background = 'rgba(255,255,255,0.08)')}
              onMouseLeave={(e) => (e.currentTarget.style.background = isCurrent ? 'rgba(255,255,255,0.06)' : 'none')}
            >
              <div style={{ flexShrink: 0, width: '8.875rem', height: '5rem', borderRadius: '0.375rem', overflow: 'hidden', background: '#1a1e28' }}>
                {ep.thumbnail
                  ? <img src={ep.thumbnail} alt="" style={{ width: '100%', height: '100%', objectFit: 'cover', display: 'block' }} />
                  : <div style={{ width: '100%', height: '100%', background: 'rgba(255,255,255,0.05)' }} />
                }
              </div>
              <div style={{ minWidth: 0, flex: 1, paddingTop: '0.125rem' }}>
                <div style={{ fontSize: '0.8125rem', fontWeight: 600, color: isCurrent ? '#fff' : 'rgba(255,255,255,0.92)', lineHeight: 1.3, marginBottom: '0.3125rem', display: '-webkit-box', WebkitLineClamp: 2, WebkitBoxOrient: 'vertical', overflow: 'hidden' }}>
                  {epLabel(ep)}{isCurrent ? ` · ${t('player.now_playing')}` : ''}
                </div>
                {ep.overview && (
                  <div style={{ fontSize: '0.6875rem', color: 'rgba(255,255,255,0.42)', lineHeight: 1.5, display: '-webkit-box', WebkitLineClamp: 3, WebkitBoxOrient: 'vertical', overflow: 'hidden' }}>
                    {ep.overview}
                  </div>
                )}
              </div>
            </button>
          );
        })}
      </div>
    </div>
  );
}
