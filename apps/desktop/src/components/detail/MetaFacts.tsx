import React from 'react';
import { t } from '../../i18n';
import { SectionLabel, space } from '../../design';
import type { Meta } from '../../core/types';
import { MS } from './detailStyles';
import { formatEpDate } from './EpisodePanel';
import { dedupedWatchProviders, WatchProviderLogo, openWatchProvidersLink } from './watchProvidersSection';

export function nameList(value: string[] | undefined): string[] {
  if (Array.isArray(value)) return value.filter((entry) => typeof entry === 'string' && entry.trim().length > 0);
  if (typeof value === 'string') return (value as string).split(',').map((entry) => entry.trim()).filter(Boolean);
  return [];
}

function Fact({ label, value }: { label: string; value: string }) {
  return (
    <div style={MS.factRow}>
      <span style={MS.factKey}>{label}</span>
      <span style={MS.factValue}>{value}</span>
    </div>
  );
}

export function MetaFactsSection({ displayMeta }: { displayMeta: Meta }) {
  const watchProviders = dedupedWatchProviders(displayMeta.watchProviders).slice(0, 6);
  const nextEpisode = displayMeta.nextEpisodeToAir;
  const language = displayMeta.originalLanguage?.toUpperCase();

  const facts: React.ReactNode[] = [];
  if (displayMeta.network) facts.push(<Fact key="network" label={t('detail.fact_network')} value={displayMeta.network} />);
  if (language) facts.push(<Fact key="language" label={t('detail.fact_language')} value={language} />);
  if (displayMeta.status) facts.push(<Fact key="status" label={t('detail.fact_status')} value={displayMeta.status} />);
  if (displayMeta.productionCountries?.length)
    facts.push(
      <Fact key="country" label={t('detail.fact_country')} value={displayMeta.productionCountries.slice(0, 2).join(', ')} />,
    );
  if (nextEpisode?.airDate)
    facts.push(
      <Fact
        key="next"
        label={t('detail.next_episode')}
        value={`${formatEpDate(nextEpisode.airDate)} · ${t('format.season_episode_short', nextEpisode.season, nextEpisode.episode)}`}
      />,
    );

  if (facts.length === 0 && watchProviders.length === 0) return null;

  return (
    <div style={MS.detailsSection}>
      <h3 style={MS.detailsSectionTitle}>{t('common.details')}</h3>
      {facts.length > 0 && <div style={MS.factGrid}>{facts}</div>}
      {watchProviders.length > 0 && (
        <div style={{ marginTop: facts.length > 0 ? space[4] : 0, display: 'flex', flexDirection: 'column', gap: space[2] }}>
          <SectionLabel>{t('detail.watch_providers')}</SectionLabel>
          <div style={{ display: 'flex', flexWrap: 'wrap', gap: space[1.5] }}>
            {watchProviders.map((provider) => (
              <WatchProviderLogo
                key={provider.name}
                name={provider.name}
                logo={provider.logo}
                size="2.25rem"
                onClick={() => openWatchProvidersLink(displayMeta.watchProviders?.link)}
              />
            ))}
          </div>
        </div>
      )}
    </div>
  );
}
