import React, { useEffect, useRef, useState } from 'react';
import { coreBuildMetadataFeedOptions, coreEffectiveMetadataFeedSelection, coreToggleMetadataFeedLimited } from '../../core/engine';
import type { AddonDescriptor } from '../../core/types';
import { addonKey } from '../../core/addons';
import { t } from '../../i18n';
import { InfoTile, InputTile, SettingsSection, StorageIcon, ToggleTile, isFeedEnabled } from './SettingsUI';
import type { Prefs } from './settingsTypes';
import { setRpdbApiKey, validateRpdbApiKey } from '../../core/rpdb';
import { color } from '../../design/tokens';

function applyFeedOrder(feeds: { key: string; label: string }[], order: string[]): { key: string; label: string }[] {
  if (!order.length) return feeds;
  const ordered: typeof feeds = [];
  for (const key of order) {
    const feed = feeds.find((f) => f.key === key);
    if (feed) ordered.push(feed);
  }
  for (const feed of feeds) {
    if (!order.includes(feed.key)) ordered.push(feed);
  }
  return ordered;
}

function moveFeedInOrder(feeds: { key: string; label: string }[], order: string[], key: string, delta: -1 | 1): string[] {
  const orderedFeeds = applyFeedOrder(feeds, order);
  const allKeys = orderedFeeds.map((f) => f.key);
  const idx = allKeys.indexOf(key);
  if (idx === -1) return order;
  const newIdx = Math.max(0, Math.min(allKeys.length - 1, idx + delta));
  if (newIdx === idx) return order;
  const newKeys = [...allKeys];
  [newKeys[idx], newKeys[newIdx]] = [newKeys[newIdx], newKeys[idx]];
  return newKeys;
}

function FeedToggleList({
  title,
  subtitle,
  feeds,
  selected,
  order,
  maxEnabled,
  defaultAll = true,
  onChange,
  onOrderChange,
}: {
  title: string;
  subtitle: string;
  feeds: { key: string; label: string }[];
  selected: string[];
  order?: string[];
  maxEnabled?: number;
  defaultAll?: boolean;
  onChange: (value: string[]) => void;
  onOrderChange?: (value: string[]) => void;
}) {
  const availableKeys = feeds.map((feed) => feed.key);
  const defaultKeys = defaultAll ? availableKeys.slice(0, maxEnabled) : [];
  const effective = selected.length === 0 ? defaultKeys : selected;
  const orderedFeeds = order ? applyFeedOrder(feeds, order) : feeds;

  const toggleFeed = async (key: string, enabled: boolean) => {
    let next: string[] | null = null;
    const toggleBase = selected.length === 0 ? effective : selected;
    if (maxEnabled) {
      next = await coreToggleMetadataFeedLimited(toggleBase, availableKeys, key, maxEnabled);
    } else {
      const effectiveSelection =
        selected.length === 0 ? effective : ((await coreEffectiveMetadataFeedSelection(selected, availableKeys)) ?? effective);
      next = enabled
        ? [...new Set([...effectiveSelection, key])].filter((value) => availableKeys.includes(value))
        : effectiveSelection.filter((value) => value !== key);
    }
    if (!next) {
      next = enabled ? [...effective, key] : effective.filter((value) => value !== key);
      next = [...new Set(next)].filter((value) => availableKeys.includes(value));
      if (maxEnabled) next = next.slice(-maxEnabled);
    }
    onChange(next);
  };

  return (
    <SettingsSection title={title} subtitle={subtitle}>
      {feeds.length === 0 ? (
        <InfoTile title={title} value="Install metadata addons to choose feeds" icon={<StorageIcon />} />
      ) : (
        orderedFeeds.map((feed, idx) => (
          <div key={feed.key} style={{ display: 'flex', alignItems: 'center' }}>
            <div style={{ flex: 1, minWidth: 0 }}>
              <ToggleTile
                title={feed.label}
                subtitle={maxEnabled ? `Enabled feeds are capped at ${maxEnabled}` : 'Included in this catalog group'}
                checked={effective.includes(feed.key)}
                onToggle={(v) => void toggleFeed(feed.key, v)}
              />
            </div>
            {onOrderChange && (
              <div style={{ display: 'flex', flexDirection: 'column', gap: '0.125rem', padding: '0 0.5rem', flexShrink: 0 }}>
                <button
                  disabled={idx === 0}
                  onClick={() => onOrderChange(moveFeedInOrder(feeds, order ?? [], feed.key, -1))}
                  style={{
                    background: 'none',
                    border: 'none',
                    cursor: idx === 0 ? 'default' : 'pointer',
                    color: idx === 0 ? 'rgba(255,255,255,0.2)' : 'rgba(255,255,255,0.55)',
                    padding: '0.125rem 0.25rem',
                    lineHeight: 1,
                  }}
                  title={t('common.move_up')}
                >
                  <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
                    <path d="M7 14l5-5 5 5z" />
                  </svg>
                </button>
                <button
                  disabled={idx === orderedFeeds.length - 1}
                  onClick={() => onOrderChange(moveFeedInOrder(feeds, order ?? [], feed.key, 1))}
                  style={{
                    background: 'none',
                    border: 'none',
                    cursor: idx === orderedFeeds.length - 1 ? 'default' : 'pointer',
                    color: idx === orderedFeeds.length - 1 ? 'rgba(255,255,255,0.2)' : 'rgba(255,255,255,0.55)',
                    padding: '0.125rem 0.25rem',
                    lineHeight: 1,
                  }}
                  title={t('common.move_down')}
                >
                  <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
                    <path d="M7 10l5 5 5-5z" />
                  </svg>
                </button>
              </div>
            )}
          </div>
        ))
      )}
    </SettingsSection>
  );
}

export function ContentSection({
  prefs,
  setPref,
  installedAddons,
  disabledAddonKeys = [],
}: {
  prefs: Prefs;
  setPref: <K extends keyof Prefs>(k: K, v: Prefs[K]) => void;
  installedAddons: AddonDescriptor[];
  disabledAddonKeys?: string[];
}) {
  const [feeds, setFeeds] = useState<{ key: string; label: string }[]>([]);
  const activeAddons = installedAddons.filter((a) => !disabledAddonKeys.includes(addonKey(a)));

  useEffect(() => {
    let cancelled = false;
    coreBuildMetadataFeedOptions(activeAddons).then((items) => {
      if (cancelled) return;
      const next = ((items ?? []) as Array<{ key?: unknown; label?: unknown }>)
        .map((item) => ({
          key: typeof item.key === 'string' ? item.key : '',
          label: typeof item.label === 'string' ? item.label : '',
        }))
        .filter((item) => item.key && item.label);
      setFeeds(next);
    });
    return () => {
      cancelled = true;
    };
  }, [installedAddons, disabledAddonKeys]);

  const [rpdbKeyStatus, setRpdbKeyStatus] = useState<'idle' | 'checking' | 'valid' | 'invalid'>('idle');
  const rpdbCheckTimer = useRef<ReturnType<typeof setTimeout> | null>(null);

  useEffect(() => {
    if (!prefs.rpdbApiKey) {
      setRpdbKeyStatus('idle');
      return;
    }
    setRpdbKeyStatus('checking');
    if (rpdbCheckTimer.current) clearTimeout(rpdbCheckTimer.current);
    rpdbCheckTimer.current = setTimeout(() => {
      void validateRpdbApiKey(prefs.rpdbApiKey).then((valid) => setRpdbKeyStatus(valid ? 'valid' : 'invalid'));
    }, 500);
    return () => {
      if (rpdbCheckTimer.current) clearTimeout(rpdbCheckTimer.current);
    };
  }, [prefs.rpdbApiKey]);

  return (
    <>
      <SettingsSection title={t('settings.hero_catalogs')} subtitle={t('settings.show_hero_section_desc')}>
        <ToggleTile
          title={t('settings.show_hero_section')}
          subtitle={t('settings.show_hero_section_desc')}
          checked={prefs.showHeroSection}
          onToggle={(v) => setPref('showHeroSection', v)}
        />
      </SettingsSection>
      <FeedToggleList
        title={t('settings.hero_catalogs')}
        subtitle={t('settings.show_hero_section_desc')}
        feeds={feeds}
        selected={prefs.heroFeedToggles}
        order={prefs.heroFeedOrder}
        maxEnabled={2}
        onChange={(v) => setPref('heroFeedToggles', v)}
        onOrderChange={(v) => setPref('heroFeedOrder', v)}
      />
      <FeedToggleList
        title={t('settings.home_catalogs')}
        subtitle={t('settings.home_catalogs_desc')}
        feeds={feeds}
        selected={prefs.homeFeedToggles}
        order={prefs.homeFeedOrder}
        onChange={(v) => setPref('homeFeedToggles', v)}
        onOrderChange={(v) => setPref('homeFeedOrder', v)}
      />
      <FeedToggleList
        title={t('settings.top_10_catalogs')}
        subtitle={t('settings.top_10_catalogs_desc')}
        feeds={feeds.filter((feed) => isFeedEnabled(prefs.homeFeedToggles, feed.key))}
        selected={prefs.topTenFeedToggles}
        defaultAll={false}
        onChange={(v) => setPref('topTenFeedToggles', v)}
      />
      <SettingsSection title={t('settings.integrations')} subtitle={t('settings.integrations_desc')}>
        <InputTile
          title={t('settings.tmdb_api_key')}
          subtitle={t('settings.tmdb_api_key_desc')}
          value={prefs.tmdbApiKey}
          placeholder={t('settings.api_key_placeholder')}
          onChange={(v) => setPref('tmdbApiKey', v)}
          status={
            <p style={{ fontSize: '0.75rem', marginTop: '0.375rem', color: prefs.tmdbApiKey ? color.success : color.textMuted }}>
              {prefs.tmdbApiKey ? t('settings.tmdb_metadata_source_active') : t('settings.tmdb_metadata_source_inactive')}
            </p>
          }
        />
        {prefs.tmdbApiKey && (
          <ToggleTile
            title={t('settings.tmdb_prefer_over_addons')}
            subtitle={t('settings.tmdb_prefer_over_addons_desc')}
            checked={prefs.tmdbPreferOverAddons}
            onToggle={(v) => setPref('tmdbPreferOverAddons', v)}
          />
        )}
        <InputTile
          title={t('settings.rpdb_api_key')}
          subtitle={t('settings.rpdb_api_key_desc')}
          value={prefs.rpdbApiKey}
          placeholder={t('settings.api_key_placeholder')}
          onChange={(v) => {
            void setPref('rpdbApiKey', v);
            setRpdbApiKey(v);
          }}
          status={
            rpdbKeyStatus !== 'idle' && (
              <p
                style={{
                  fontSize: '0.75rem',
                  marginTop: '0.375rem',
                  color: rpdbKeyStatus === 'invalid' ? 'var(--primary-accent-color)' : color.textMuted,
                }}
              >
                {rpdbKeyStatus === 'checking'
                  ? t('settings.rpdb_key_checking')
                  : rpdbKeyStatus === 'valid'
                    ? t('settings.rpdb_key_valid')
                    : t('settings.rpdb_key_invalid')}
              </p>
            )
          }
        />
        <InputTile
          title={t('settings.omdb_api_key')}
          subtitle={t('settings.omdb_api_key_desc')}
          value={prefs.omdbApiKey}
          placeholder={t('settings.api_key_placeholder')}
          onChange={(v) => setPref('omdbApiKey', v)}
        />
        <InputTile
          title={t('settings.mdblist_api_key')}
          subtitle={t('settings.mdblist_api_key_desc')}
          value={prefs.mdblistApiKey}
          placeholder={t('settings.api_key_placeholder')}
          onChange={(v) => setPref('mdblistApiKey', v)}
        />
        <InputTile
          title={t('settings.fanart_api_key')}
          subtitle={t('settings.fanart_api_key_desc')}
          value={prefs.fanartApiKey}
          placeholder={t('settings.api_key_placeholder')}
          onChange={(v) => setPref('fanartApiKey', v)}
        />
      </SettingsSection>
      {prefs.tmdbApiKey && (
        <SettingsSection title={t('settings.tmdb_enrichment')} subtitle={t('settings.tmdb_enrichment_desc')}>
          <ToggleTile
            title={t('settings.tmdb_enrich_artwork')}
            subtitle={t('settings.tmdb_enrich_artwork_desc')}
            checked={prefs.tmdbEnrichArtworkEnabled}
            onToggle={(v) => setPref('tmdbEnrichArtworkEnabled', v)}
          />
          <ToggleTile
            title={t('settings.tmdb_enrich_description')}
            subtitle={t('settings.tmdb_enrich_description_desc')}
            checked={prefs.tmdbEnrichDescriptionEnabled}
            onToggle={(v) => setPref('tmdbEnrichDescriptionEnabled', v)}
          />
          <ToggleTile
            title={t('settings.tmdb_enrich_genres_keywords')}
            subtitle={t('settings.tmdb_enrich_genres_keywords_desc')}
            checked={prefs.tmdbEnrichGenresKeywordsEnabled}
            onToggle={(v) => setPref('tmdbEnrichGenresKeywordsEnabled', v)}
          />
          <ToggleTile
            title={t('settings.tmdb_enrich_cast_crew')}
            subtitle={t('settings.tmdb_enrich_cast_crew_desc')}
            checked={prefs.tmdbEnrichCastCrewEnabled}
            onToggle={(v) => setPref('tmdbEnrichCastCrewEnabled', v)}
          />
          <ToggleTile
            title={t('settings.tmdb_enrich_network')}
            subtitle={t('settings.tmdb_enrich_network_desc')}
            checked={prefs.tmdbEnrichNetworkEnabled}
            onToggle={(v) => setPref('tmdbEnrichNetworkEnabled', v)}
          />
          <ToggleTile
            title={t('settings.tmdb_enrich_ratings')}
            subtitle={t('settings.tmdb_enrich_ratings_desc')}
            checked={prefs.tmdbRatingsEnabled}
            onToggle={(v) => setPref('tmdbRatingsEnabled', v)}
          />
          <ToggleTile
            title={t('settings.tmdb_enrich_collection')}
            subtitle={t('settings.tmdb_enrich_collection_desc')}
            checked={prefs.tmdbCollectionInfoEnabled}
            onToggle={(v) => setPref('tmdbCollectionInfoEnabled', v)}
          />
          <ToggleTile
            title={t('settings.tmdb_enrich_status_schedule')}
            subtitle={t('settings.tmdb_enrich_status_schedule_desc')}
            checked={prefs.tmdbEnrichStatusScheduleEnabled}
            onToggle={(v) => setPref('tmdbEnrichStatusScheduleEnabled', v)}
          />
          <ToggleTile
            title={t('settings.tmdb_enrich_origin_titles')}
            subtitle={t('settings.tmdb_enrich_origin_titles_desc')}
            checked={prefs.tmdbEnrichOriginTitlesEnabled}
            onToggle={(v) => setPref('tmdbEnrichOriginTitlesEnabled', v)}
          />
          <ToggleTile
            title={t('settings.tmdb_enrich_watch_providers')}
            subtitle={t('settings.tmdb_enrich_watch_providers_desc')}
            checked={prefs.tmdbEnrichWatchProvidersEnabled}
            onToggle={(v) => setPref('tmdbEnrichWatchProvidersEnabled', v)}
          />
          <ToggleTile
            title={t('settings.tmdb_enrich_episode_stills')}
            subtitle={t('settings.tmdb_enrich_episode_stills_desc')}
            checked={prefs.tmdbEpisodeImagesEnabled}
            onToggle={(v) => setPref('tmdbEpisodeImagesEnabled', v)}
          />
        </SettingsSection>
      )}
    </>
  );
}
