import React from 'react';
import { t } from '../../i18n';
import { ChoiceTile, ToggleTile, SettingsSection } from './SettingsUI';
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
      <SettingsSection title={t('auto.interface_3c5ec842')} subtitle={t('auto.language_theme_startup')}>
        <ChoiceTile
          title="Sidebar Layout"
          subtitle="Choose navigation style"
          options={[{ value: 'sidebar', label: 'Sidebar' }, { value: 'topbar', label: 'Top Bar' }]}
          selected={prefs.navLayout}
          onSelect={(v) => setPref('navLayout', v)}
        />
        <ChoiceTile
          title="Bar Rotation"
          subtitle="Move the navigation bar around the screen"
          options={[{ value: 'left', label: 'Left' }, { value: 'right', label: 'Right' }, { value: 'top', label: 'Top' }, { value: 'bottom', label: 'Bottom' }]}
          selected={prefs.navBarPosition}
          onSelect={(v) => setPref('navBarPosition', v)}
        />
        <ChoiceTile
          title="Items Rotation"
          subtitle="Align navigation items inside the bar"
          options={[{ value: 'start', label: 'Left' }, { value: 'center', label: 'Center' }, { value: 'end', label: 'Right' }]}
          selected={prefs.navItemsAlign}
          onSelect={(v) => setPref('navItemsAlign', v)}
        />
        <ToggleTile title={t('settings.amoled')} subtitle={t('auto.makes_the_background_and_bottom_bar_pure_bla')} checked={prefs.amoledMode} onToggle={(v) => setPref('amoledMode', v)} />
      </SettingsSection>
      <SettingsSection title={t('auto.posters')} subtitle={t('auto.poster_width')}>
        <ChoiceTile
          title={t('auto.card_corners')}
          subtitle={t('auto.card_corners')}
          options={[{ value: 'sharp', label: t('auto.sharp') }, { value: 'classic', label: t('auto.classic') }, { value: 'soft', label: t('auto.soft') }, { value: 'rounded', label: t('auto.rounded') }, { value: 'pill', label: t('auto.extra_rounded') }]}
          selected={prefs.cardCornerPreset}
          onSelect={(v) => setPref('cardCornerPreset', v)}
        />
        <ChoiceTile
          title={t('auto.interface_density')}
          subtitle={t('auto.interface_density')}
          options={[{ value: 'small', label: t('auto.small') }, { value: 'medium', label: t('auto.medium') }, { value: 'large', label: t('auto.large') }]}
          selected={prefs.interfaceDensity}
          onSelect={(v) => setPref('interfaceDensity', v)}
        />
        <ChoiceTile
          title={t('auto.poster_width')}
          subtitle={t('auto.poster_width')}
          options={[{ value: 'xsmall', label: t('auto.very_small') }, { value: 'small', label: t('auto.small') }, { value: 'medium', label: t('auto.medium') }, { value: 'large', label: t('auto.large') }, { value: 'xlarge', label: t('auto.very_large') }]}
          selected={prefs.posterWidthPreset}
          onSelect={(v) => setPref('posterWidthPreset', v)}
        />
        <ToggleTile title={t('auto.horizontal')} subtitle={t('auto.poster')} checked={prefs.posterLandscapeMode} onToggle={(v) => setPref('posterLandscapeMode', v)} />
        <ToggleTile title={t('auto.hide_titles')} subtitle={t('auto.hide_titles')} checked={prefs.posterHideTitles} onToggle={(v) => setPref('posterHideTitles', v)} />
        <ChoiceTile
          title={t('auto.card_layout')}
          subtitle={t('auto.tune_language_and_visual_layout')}
          options={[{ value: 'vertical', label: t('auto.vertical_layout') }, { value: 'horizontal', label: t('auto.horizontal') }]}
          selected={prefs.cardLayout}
          onSelect={(v) => setPref('cardLayout', v)}
        />
        <ChoiceTile
          title={t('auto.continue_watching_layout')}
          subtitle={t('auto.show_that_shelf_as_posters_or_episode_cards')}
          options={[{ value: 'vertical', label: t('auto.vertical_layout') }, { value: 'horizontal', label: t('auto.horizontal') }, { value: 'inherit', label: t('auto.match_global') }]}
          selected={prefs.continueWatchingLayout}
          onSelect={(v) => setPref('continueWatchingLayout', v)}
        />
        <ChoiceTile
          title={t('auto.series_artwork')}
          subtitle={t('auto.show_that_shelf_as_posters_or_episode_cards')}
          options={[{ value: 'episode', label: t('auto.episode_cover') }, { value: 'poster', label: t('auto.poster') }, { value: 'background', label: t('auto.backdrop') }]}
          selected={prefs.continueWatchingArtwork}
          onSelect={(v) => setPref('continueWatchingArtwork', v)}
        />
      </SettingsSection>
      <SettingsSection title={t('auto.continue_watching')} subtitle={t('auto.continue_watching')}>
        <ToggleTile title={t('auto.continue_watching')} subtitle={t('auto.continue_watching')} checked={prefs.continueWatchingEnabled} onToggle={(v) => setPref('continueWatchingEnabled', v)} />
        <ToggleTile title={t('settings.continue_watching_hide_titles')} subtitle={t('auto.hide_titles')} checked={prefs.continueWatchingHideTitles} onToggle={(v) => setPref('continueWatchingHideTitles', v)} />
      </SettingsSection>
      <SettingsSection title={t('settings.appearance_home_screen') || 'Ana Ekran'} subtitle={t('settings.appearance_home_screen_desc') || 'Ana ekrana özel görünüm ayarları'}>
        <ToggleTile
          title={t('settings.season_posters_on_hero') || "Hero'da Sezon Posterleri"}
          subtitle={t('settings.home_season_posters_on_hero_desc') || 'Serilerin hero bölümünde sezon posterlerini göster'}
          checked={prefs.homeSeasonPostersOnHero}
          onToggle={(v) => setPref('homeSeasonPostersOnHero', v)}
        />
      </SettingsSection>
      <SettingsSection title={t('settings.appearance_detail_screen')} subtitle={t('settings.appearance_detail_screen_desc')}>
        <ToggleTile title={t('settings.trailer_on_hero')} subtitle={t('settings.trailer_on_hero_desc')} checked={prefs.trailerOnHero} onToggle={(v) => setPref('trailerOnHero', v)} />
        <ToggleTile title={t('settings.blur_unwatched_episodes')} subtitle={t('settings.blur_unwatched_episodes_desc')} checked={prefs.blurUnwatchedEpisodes} onToggle={(v) => setPref('blurUnwatchedEpisodes', v)} />
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
        <ChoiceTile
          title={t('settings.detail_page_view_mode')}
          subtitle={t('settings.detail_page_view_mode_desc')}
          options={[{ value: 'modern', label: t('settings.view_mode_modern') }, { value: 'legacy', label: t('settings.view_mode_legacy') }]}
          selected={prefs.detailEpisodeViewMode}
          onSelect={(v) => setPref('detailEpisodeViewMode', v)}
        />
      </SettingsSection>
    </>
  );
}
