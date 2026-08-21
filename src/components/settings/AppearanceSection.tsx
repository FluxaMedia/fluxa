import React from 'react';
import { t } from '../../i18n';
import { ChoiceTile, SliderTile, ToggleTile, SettingsSection } from './SettingsUI';
import type { Prefs } from './settingsTypes';

export function AppearanceSection({ prefs, setPref }: { prefs: Prefs; setPref: <K extends keyof Prefs>(k: K, v: Prefs[K]) => void }) {
  return (
    <>
      <SettingsSection title={t('auto.accent_color')} subtitle={t('auto.color_and_layout')}>
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
