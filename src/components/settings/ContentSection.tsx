import React, { useEffect, useState } from 'react';
import { coreBuildMetadataFeedOptions, coreEffectiveMetadataFeedSelection, coreToggleMetadataFeedLimited } from '../../core/engine';
import type { AddonDescriptor } from '../../core/types';
import { addonKey } from '../../core/addons';
import { t } from '../../i18n';
import { InfoTile, SettingsSection, StorageIcon, ToggleTile, isFeedEnabled } from './SettingsUI';
import type { Prefs } from './settingsTypes';

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
    </>
  );
}
