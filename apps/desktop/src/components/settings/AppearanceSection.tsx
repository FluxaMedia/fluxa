import React, { useRef, useState } from 'react';
import { t } from '../../i18n';
import { ActionTile, ChoiceTile, DownloadIcon, PaletteIcon, SliderTile, ToggleTile, SettingsSection, TrashIcon } from './SettingsUI';
import type { Prefs } from './settingsTypes';
import { BUILT_IN_THEMES } from '../../theme/defaults';
import { isValidThemePack, parseThemePacks } from '../../theme/adapter';
import type { ThemePack } from '../../theme/types';

function skinNavigationVisible(raw: string, route: string): boolean {
  try {
    const config = JSON.parse(raw) as { navigation?: { visible?: string[] } };
    return config.navigation?.visible?.includes(route) ?? true;
  } catch {
    return true;
  }
}

function setSkinNavigationVisible(raw: string, route: string, visible: boolean): string {
  let config: { navigation?: { visible?: string[]; order?: string[] } } = {};
  try {
    config = JSON.parse(raw) as typeof config;
  } catch {
    config = {};
  }
  const current = config.navigation?.visible ?? ['home', 'library', 'discover', 'calendar', 'settings'];
  const next = visible ? [...new Set([...current, route])] : current.filter((item) => item !== route);
  return JSON.stringify({ ...config, navigation: { ...config.navigation, visible: next } });
}

function skinHomeSectionVisible(raw: string, section: string): boolean {
  try {
    const config = JSON.parse(raw) as { home?: { hiddenSections?: string[] } };
    return !(config.home?.hiddenSections ?? []).includes(section);
  } catch {
    return true;
  }
}

function setSkinHomeSectionVisible(raw: string, section: string, visible: boolean): string {
  let config: { home?: { hiddenSections?: string[]; sectionOrder?: string[] } } = {};
  try {
    config = JSON.parse(raw) as typeof config;
  } catch {
    config = {};
  }
  const hiddenSections = config.home?.hiddenSections ?? [];
  const next = visible ? hiddenSections.filter((item) => item !== section) : [...new Set([...hiddenSections, section])];
  return JSON.stringify({ ...config, home: { ...config.home, hiddenSections: next } });
}

function skinHomeOrderValue(raw: string): string {
  try {
    const config = JSON.parse(raw) as { home?: { sectionOrder?: string[] } };
    const order = config.home?.sectionOrder ?? ['hero', 'continueWatching', 'catalogs'];
    return order.join('-');
  } catch {
    return 'hero-continueWatching-catalogs';
  }
}

function setSkinHomeOrder(raw: string, value: string): string {
  let config: { home?: { hiddenSections?: string[]; sectionOrder?: string[] } } = {};
  try {
    config = JSON.parse(raw) as typeof config;
  } catch {
    config = {};
  }
  return JSON.stringify({ ...config, home: { ...config.home, sectionOrder: value.split('-') } });
}

function themeLabel(theme: ThemePack): string {
  return theme.name ?? t(theme.nameKey);
}

function downloadTheme(theme: ThemePack): void {
  const blob = new Blob([JSON.stringify(theme, null, 2)], { type: 'application/json' });
  const url = URL.createObjectURL(blob);
  const anchor = document.createElement('a');
  anchor.href = url;
  anchor.download = `${theme.id}.json`;
  anchor.click();
  URL.revokeObjectURL(url);
}

export function AppearanceSection({ prefs, setPref }: { prefs: Prefs; setPref: <K extends keyof Prefs>(k: K, v: Prefs[K]) => void }) {
  const fileInputRef = useRef<HTMLInputElement>(null);
  const [themeImportError, setThemeImportError] = useState(false);
  const customThemes = parseThemePacks(prefs.customThemes);
  const allThemes = [...BUILT_IN_THEMES, ...customThemes];
  const selectedTheme = allThemes.find((theme) => theme.id === prefs.themeId) ?? BUILT_IN_THEMES[0];

  const importTheme = async (file: File | undefined) => {
    if (!file) return;
    setThemeImportError(false);
    try {
      if (file.size > 262144) throw new Error('theme-too-large');
      const value: unknown = JSON.parse(await file.text());
      if (!isValidThemePack(value) || BUILT_IN_THEMES.some((theme) => theme.id === value.id)) throw new Error('theme-invalid');
      const nextThemes = [...customThemes.filter((theme) => theme.id !== value.id), value].slice(-24);
      setPref('customThemes', JSON.stringify(nextThemes));
      setPref('themeId', value.id);
    } catch {
      setThemeImportError(true);
    }
    if (fileInputRef.current) fileInputRef.current.value = '';
  };

  const removeSelectedTheme = () => {
    if (!customThemes.some((theme) => theme.id === prefs.themeId)) return;
    setPref('customThemes', JSON.stringify(customThemes.filter((theme) => theme.id !== prefs.themeId)));
    setPref('themeId', 'fluxa-dark');
  };

  return (
    <>
      <SettingsSection title={t('auto.accent_color')} subtitle={t('auto.color_and_layout')}>
        <ChoiceTile
          title={t('settings.theme')}
          subtitle={t('settings.theme_desc')}
          options={allThemes.map((theme) => ({ value: theme.id, label: themeLabel(theme) }))}
          selected={prefs.themeId}
          onSelect={(v) => setPref('themeId', v)}
        />
        <ActionTile title={t('settings.theme_import')} subtitle={t('settings.theme_import_desc')} icon={<PaletteIcon />} onClick={() => fileInputRef.current?.click()} />
        <input ref={fileInputRef} type="file" accept="application/json,.json" hidden onChange={(event) => void importTheme(event.target.files?.[0])} />
        <ActionTile title={t('settings.theme_export')} subtitle={t('settings.theme_export_desc')} icon={<DownloadIcon />} onClick={() => downloadTheme(selectedTheme)} />
        {customThemes.some((theme) => theme.id === prefs.themeId) && (
          <ActionTile title={t('settings.theme_delete')} subtitle={t('settings.theme_delete_desc')} icon={<TrashIcon />} onClick={removeSelectedTheme} accent="var(--fluxa-error)" />
        )}
        {themeImportError && <p style={{ color: 'var(--fluxa-error)', margin: '0.75rem 1.125rem' }}>{t('settings.theme_import_error')}</p>}
        <ChoiceTile
          title={t('auto.accent_color')}
          subtitle={t('auto.color_and_layout')}
          options={[
            { value: '#FFFFFF', label: t('auto.white') },
            { value: '#E50914', label: t('auto.red') },
            { value: '#3F7CFF', label: t('auto.blue') },
            { value: '#54D17A', label: t('auto.green') },
            { value: '#FF8A3D', label: t('auto.orange') },
            { value: '#C084FC', label: t('auto.purple') },
          ]}
          selected={prefs.accentColorArgb}
          onSelect={(v) => setPref('accentColorArgb', v)}
        />
        <ToggleTile
          title={t('settings.skin_show_calendar')}
          subtitle={t('settings.skin_show_calendar_desc')}
          checked={skinNavigationVisible(prefs.skinConfig, 'calendar')}
          onToggle={(v) => setPref('skinConfig', setSkinNavigationVisible(prefs.skinConfig, 'calendar', v))}
        />
        {(['library', 'discover', 'settings'] as const).map((route) => (
          <ToggleTile
            key={route}
            title={t(`settings.skin_show_${route}`)}
            subtitle={t(`settings.skin_show_${route}_desc`)}
            checked={skinNavigationVisible(prefs.skinConfig, route)}
            onToggle={(v) => setPref('skinConfig', setSkinNavigationVisible(prefs.skinConfig, route, v))}
          />
        ))}
        <ToggleTile
          title={t('settings.skin_show_hero')}
          subtitle={t('settings.skin_show_hero_desc')}
          checked={skinHomeSectionVisible(prefs.skinConfig, 'hero')}
          onToggle={(v) => setPref('skinConfig', setSkinHomeSectionVisible(prefs.skinConfig, 'hero', v))}
        />
        <ToggleTile
          title={t('settings.skin_show_continue_watching')}
          subtitle={t('settings.skin_show_continue_watching_desc')}
          checked={skinHomeSectionVisible(prefs.skinConfig, 'continueWatching')}
          onToggle={(v) => setPref('skinConfig', setSkinHomeSectionVisible(prefs.skinConfig, 'continueWatching', v))}
        />
        <ToggleTile
          title={t('settings.skin_show_catalogs')}
          subtitle={t('settings.skin_show_catalogs_desc')}
          checked={skinHomeSectionVisible(prefs.skinConfig, 'catalogs')}
          onToggle={(v) => setPref('skinConfig', setSkinHomeSectionVisible(prefs.skinConfig, 'catalogs', v))}
        />
        <ChoiceTile
          title={t('settings.skin_home_order')}
          subtitle={t('settings.skin_home_order_desc')}
          options={[
            { value: 'hero-continueWatching-catalogs', label: t('settings.skin_home_order_default') },
            { value: 'hero-catalogs-continueWatching', label: t('settings.skin_home_order_catalogs_first') },
            { value: 'catalogs-continueWatching-hero', label: t('settings.skin_home_order_hero_last') },
          ]}
          selected={skinHomeOrderValue(prefs.skinConfig)}
          onSelect={(v) => setPref('skinConfig', setSkinHomeOrder(prefs.skinConfig, v))}
        />
      </SettingsSection>
      <SettingsSection title={t('settings.ui_scale')} subtitle={t('settings.ui_scale_desc')}>
        <SliderTile
          title={t('settings.ui_scale')}
          subtitle={t('settings.ui_scale_desc')}
          value={Number(prefs.uiScale)}
          min={75}
          max={150}
          step={5}
          format={(v) => `${v}%`}
          onChange={(v) => setPref('uiScale', String(v))}
        />
      </SettingsSection>
      <SettingsSection title={t('auto.interface_3c5ec842')} subtitle={t('settings.appearance_interface_desc')}>
        <ToggleTile
          title={t('settings.gif_autoplay')}
          subtitle={t('settings.gif_autoplay_desc')}
          checked={prefs.gifAutoplayEnabled}
          onToggle={(v) => setPref('gifAutoplayEnabled', v)}
        />
        <ToggleTile
          title="Reduced visual effects"
          subtitle="Disable blur and heavy shadows to improve performance on integrated GPUs."
          checked={prefs.reducedEffects}
          onToggle={(v) => setPref('reducedEffects', v)}
        />
        <ChoiceTile
          title={t('appearance.sidebar_layout')}
          subtitle={t('appearance.sidebar_layout_desc')}
          options={[
            { value: 'sidebar', label: 'Sidebar' },
            { value: 'topbar', label: 'Top Bar' },
          ]}
          selected={prefs.navLayout}
          onSelect={(v) => setPref('navLayout', v)}
        />
        {prefs.navLayout === 'sidebar' && (
          <ChoiceTile
            title={t('appearance.sidebar_mode')}
            subtitle={t('appearance.sidebar_mode_desc')}
            options={[
              { value: 'hover', label: t('appearance.sidebar_mode_hover') },
              { value: 'always', label: t('appearance.sidebar_mode_always') },
            ]}
            selected={prefs.navSidebarMode}
            onSelect={(v) => setPref('navSidebarMode', v)}
          />
        )}
        <ChoiceTile
          title={t('appearance.bar_rotation')}
          subtitle={t('appearance.bar_rotation_desc')}
          options={[
            { value: 'left', label: 'Left' },
            { value: 'right', label: 'Right' },
            { value: 'top', label: 'Top' },
            { value: 'bottom', label: 'Bottom' },
          ]}
          selected={prefs.navBarPosition}
          onSelect={(v) => setPref('navBarPosition', v)}
        />
        <ChoiceTile
          title={t('appearance.items_rotation')}
          subtitle={t('appearance.items_rotation_desc')}
          options={[
            { value: 'start', label: 'Left' },
            { value: 'center', label: 'Center' },
            { value: 'end', label: 'Right' },
          ]}
          selected={prefs.navItemsAlign}
          onSelect={(v) => setPref('navItemsAlign', v)}
        />
      </SettingsSection>
      <SettingsSection title={t('auto.posters')} subtitle={t('settings.appearance_posters_desc')}>
        <ChoiceTile
          title={t('auto.card_corners')}
          subtitle={t('auto.card_corners')}
          options={[
            { value: 'sharp', label: t('auto.sharp') },
            { value: 'classic', label: t('auto.classic') },
            { value: 'soft', label: t('auto.soft') },
            { value: 'rounded', label: t('auto.rounded') },
            { value: 'pill', label: t('auto.extra_rounded') },
          ]}
          selected={prefs.cardCornerPreset}
          onSelect={(v) => setPref('cardCornerPreset', v)}
        />
        <ChoiceTile
          title={t('auto.interface_density')}
          subtitle={t('auto.interface_density')}
          options={[
            { value: 'small', label: t('auto.small') },
            { value: 'medium', label: t('auto.medium') },
            { value: 'large', label: t('auto.large') },
          ]}
          selected={prefs.interfaceDensity}
          onSelect={(v) => setPref('interfaceDensity', v)}
        />
        <ChoiceTile
          title={t('auto.poster_width')}
          subtitle={t('auto.poster_width')}
          options={[
            { value: 'xsmall', label: t('auto.very_small') },
            { value: 'small', label: t('auto.small') },
            { value: 'medium', label: t('auto.medium') },
            { value: 'large', label: t('auto.large') },
            { value: 'xlarge', label: t('auto.very_large') },
          ]}
          selected={prefs.posterWidthPreset}
          onSelect={(v) => setPref('posterWidthPreset', v)}
        />
        <ToggleTile
          title={t('auto.horizontal')}
          subtitle={t('auto.poster')}
          checked={prefs.posterLandscapeMode}
          onToggle={(v) => setPref('posterLandscapeMode', v)}
        />
        <ToggleTile
          title={t('settings.poster_hover_preview')}
          subtitle={t('settings.poster_hover_preview_desc')}
          checked={prefs.posterHoverPreview}
          onToggle={(v) => setPref('posterHoverPreview', v)}
        />
        <ToggleTile
          title={t('auto.hide_titles')}
          subtitle={t('auto.hide_titles')}
          checked={prefs.posterHideTitles}
          onToggle={(v) => setPref('posterHideTitles', v)}
        />
        <ToggleTile
          title={t('settings.catalog_type_suffix')}
          subtitle={t('settings.catalog_type_suffix_desc')}
          checked={prefs.catalogTypeSuffixEnabled}
          onToggle={(v) => setPref('catalogTypeSuffixEnabled', v)}
        />
        <ChoiceTile
          title={t('auto.card_layout')}
          subtitle={t('auto.tune_language_and_visual_layout')}
          options={[
            { value: 'vertical', label: t('auto.vertical_layout') },
            { value: 'horizontal', label: t('auto.horizontal') },
          ]}
          selected={prefs.cardLayout}
          onSelect={(v) => setPref('cardLayout', v)}
        />
        <ChoiceTile
          title={t('auto.continue_watching_layout')}
          subtitle={t('auto.show_that_shelf_as_posters_or_episode_cards')}
          options={[
            { value: 'vertical', label: t('auto.vertical_layout') },
            { value: 'horizontal', label: t('auto.horizontal') },
            { value: 'inherit', label: t('auto.match_global') },
          ]}
          selected={prefs.continueWatchingLayout}
          onSelect={(v) => setPref('continueWatchingLayout', v)}
        />
        <ChoiceTile
          title={t('auto.series_artwork')}
          subtitle={t('auto.show_that_shelf_as_posters_or_episode_cards')}
          options={[
            { value: 'episode', label: t('auto.episode_cover') },
            { value: 'poster', label: t('auto.poster') },
            { value: 'background', label: t('auto.backdrop') },
          ]}
          selected={prefs.continueWatchingArtwork}
          onSelect={(v) => setPref('continueWatchingArtwork', v)}
        />
        <ChoiceTile
          title={t('settings.remaining_format')}
          subtitle={t('settings.remaining_format_desc')}
          options={[
            { value: 'time', label: t('settings.remaining_format_time') },
            { value: 'percent', label: t('settings.remaining_format_percent') },
          ]}
          selected={prefs.continueWatchingRemainingFormat}
          onSelect={(v) => setPref('continueWatchingRemainingFormat', v)}
        />
        <ChoiceTile
          title={t('settings.progress_direction')}
          subtitle={t('settings.progress_direction_desc')}
          options={[
            { value: 'remaining', label: t('settings.progress_direction_remaining') },
            { value: 'watched', label: t('settings.progress_direction_watched') },
          ]}
          selected={prefs.continueWatchingProgressDirection}
          onSelect={(v) => setPref('continueWatchingProgressDirection', v)}
        />
      </SettingsSection>
      <SettingsSection title={t('auto.continue_watching')} subtitle={t('auto.continue_watching')}>
        <ToggleTile
          title={t('auto.continue_watching')}
          subtitle={t('auto.continue_watching')}
          checked={prefs.continueWatchingEnabled}
          onToggle={(v) => setPref('continueWatchingEnabled', v)}
        />
        <ToggleTile
          title={t('settings.continue_watching_hide_titles')}
          subtitle={t('auto.hide_titles')}
          checked={prefs.continueWatchingHideTitles}
          onToggle={(v) => setPref('continueWatchingHideTitles', v)}
        />
        <ToggleTile
          title={t('settings.cw_keep_scheduled')}
          subtitle={t('settings.cw_keep_scheduled_desc')}
          checked={prefs.continueWatchingKeepScheduled}
          onToggle={(v) => setPref('continueWatchingKeepScheduled', v)}
        />
        <ToggleTile
          title={t('settings.cw_show_this_week')}
          subtitle={t('settings.cw_show_this_week_desc')}
          checked={prefs.continueWatchingShowThisWeek}
          onToggle={(v) => setPref('continueWatchingShowThisWeek', v)}
        />
      </SettingsSection>
      <SettingsSection
        title={t('settings.appearance_home_screen') || 'Ana Ekran'}
        subtitle={t('settings.appearance_home_screen_desc') || 'Ana ekrana özel görünüm ayarları'}
      >
        <ToggleTile
          title={t('settings.season_posters_on_hero') || "Hero'da Sezon Posterleri"}
          subtitle={t('settings.home_season_posters_on_hero_desc') || 'Serilerin hero bölümünde sezon posterlerini göster'}
          checked={prefs.homeSeasonPostersOnHero}
          onToggle={(v) => setPref('homeSeasonPostersOnHero', v)}
        />
        <ToggleTile
          title={t('settings.home_hero_autoplay_trailer')}
          subtitle={t('settings.home_hero_autoplay_trailer_desc')}
          checked={prefs.homeHeroAutoplayTrailer}
          onToggle={(v) => setPref('homeHeroAutoplayTrailer', v)}
        />
        {prefs.homeHeroAutoplayTrailer && (
          <ChoiceTile
            title={t('settings.home_hero_autoplay_trailer_delay')}
            subtitle={t('settings.home_hero_autoplay_trailer_delay_desc')}
            options={[
              { value: '2', label: '2s' },
              { value: '4', label: '4s' },
              { value: '6', label: '6s' },
              { value: '10', label: '10s' },
            ]}
            selected={prefs.homeHeroAutoplayTrailerDelaySecs}
            onSelect={(v) => setPref('homeHeroAutoplayTrailerDelaySecs', v)}
          />
        )}
      </SettingsSection>
      <SettingsSection title={t('settings.appearance_detail_screen')} subtitle={t('settings.appearance_detail_screen_desc')}>
        <ToggleTile
          title={t('settings.trailer_on_hero')}
          subtitle={t('settings.trailer_on_hero_desc')}
          checked={prefs.trailerOnHero}
          onToggle={(v) => setPref('trailerOnHero', v)}
        />
        <ToggleTile
          title={t('settings.detail_hero_autoplay_trailer')}
          subtitle={t('settings.detail_hero_autoplay_trailer_desc')}
          checked={prefs.detailHeroAutoplayTrailer}
          onToggle={(v) => setPref('detailHeroAutoplayTrailer', v)}
        />
        {prefs.detailHeroAutoplayTrailer && (
          <ChoiceTile
            title={t('settings.detail_hero_autoplay_trailer_delay')}
            subtitle={t('settings.detail_hero_autoplay_trailer_delay_desc')}
            options={[
              { value: '2', label: '2s' },
              { value: '4', label: '4s' },
              { value: '6', label: '6s' },
              { value: '10', label: '10s' },
            ]}
            selected={prefs.detailHeroAutoplayTrailerDelaySecs}
            onSelect={(v) => setPref('detailHeroAutoplayTrailerDelaySecs', v)}
          />
        )}
        <ToggleTile
          title={t('settings.blur_unwatched_episodes')}
          subtitle={t('settings.blur_unwatched_episodes_desc')}
          checked={prefs.blurUnwatchedEpisodes}
          onToggle={(v) => setPref('blurUnwatchedEpisodes', v)}
        />
        <ToggleTile
          title={t('settings.spoiler_hide_episode_info')}
          subtitle={t('settings.spoiler_hide_episode_info_desc')}
          checked={prefs.spoilerHideEpisodeInfo}
          onToggle={(v) => setPref('spoilerHideEpisodeInfo', v)}
        />
        <ToggleTile
          title={t('settings.season_posters_on_hero') || "Hero'da Sezon Posterleri"}
          subtitle={t('settings.detail_season_posters_on_hero_desc') || 'Detay ekranındaki hero bölümünde sezon posterlerini göster'}
          checked={prefs.detailSeasonPostersOnHero}
          onToggle={(v) => setPref('detailSeasonPostersOnHero', v)}
        />
        <ChoiceTile
          title={t('settings.season_selector') || 'Sezon Seçici'}
          subtitle={t('settings.season_selector_desc') || 'Sezon gezgininin görünümünü seç'}
          options={[
            { value: 'tabs', label: t('settings.season_selector_tabs') || 'Sekmeler' },
            { value: 'slider', label: t('settings.season_selector_slider') || 'Kaydırıcı' },
            { value: 'compact', label: t('settings.season_selector_compact') || 'Kompakt' },
          ]}
          selected={prefs.detailSeasonSelectorMode}
          onSelect={(v) => setPref('detailSeasonSelectorMode', v)}
        />
        <ChoiceTile
          title={t('settings.episode_cards_layout') || 'Bölüm Kartı Düzeni'}
          subtitle={t('settings.episode_cards_layout_desc') || 'Bölüm listesinin görünümünü seç'}
          options={[
            { value: 'standard', label: t('settings.episode_cards_standard') || 'Standart' },
            { value: 'wide', label: t('settings.episode_cards_wide') || 'Geniş' },
            { value: 'compact', label: t('settings.episode_cards_compact') || 'Kompakt' },
          ]}
          selected={prefs.episodeCardsLayout}
          onSelect={(v) => setPref('episodeCardsLayout', v)}
        />
      </SettingsSection>
    </>
  );
}
