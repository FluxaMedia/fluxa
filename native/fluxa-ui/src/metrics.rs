use super::*;
use std::sync::atomic::{AtomicU32, Ordering};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Scaled {
    pub ratio: f32,
    pub min: f32,
    pub max: f32,
    pub offset: f32,
}

impl Scaled {
    pub fn of(self, basis: f32) -> f32 {
        (basis * self.ratio + self.offset).clamp(self.min, self.max)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PosterSizes {
    pub mark: Scaled,
    pub badge: Scaled,
    pub tag: Scaled,
    pub caption: Scaled,
    pub meta_bar: Scaled,
    pub meta_text: Scaled,
    pub landscape_text: Scaled,
    pub score_bar: Scaled,
    pub score_minimal: Scaled,
    pub score_number: Scaled,
    pub score_caption: Scaled,
    pub sash: Scaled,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlayerInfo {
    pub left: f32,
    pub label_gap: f32,
    pub label_font: f32,
    pub title_font: f32,
    pub bottom: f32,
    pub scale: f32,
    pub description_rows: usize,
    pub width: Scaled,
    pub logo_width: f32,
    pub logo_height: f32,
}

/// Geometry consumed by every shared Rust screen.
///
/// These values are read from `shared/contracts/ui-tokens.json` through the
/// same parser used by the low-level renderer. The small fallbacks are only a
/// malformed-token safety net; they are not a second design system. Keeping
/// the lookup here prevents platform hosts from growing their own fixed UI
/// dimensions while still allowing the renderer to paint when a user ships a
/// partially migrated custom token document.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UiMetrics {
    pub compact: bool,
    pub platform: &'static str,
    pub window_class: &'static str,
    pub viewport_width: f32,
    pub background: Color32,
    pub surface: Color32,
    pub surface_raised: Color32,
    pub border: Color32,
    pub danger: Color32,
    pub dialog_radius: f32,
    pub motion_fast: f32,
    pub motion_normal: f32,
    pub motion_slow: f32,
    pub motion_curve: [f32; 4],
    pub navigation: Color32,
    pub text_primary: Color32,
    pub text_secondary: Color32,
    pub text_muted: Color32,
    pub accent: Color32,
    pub accent_foreground: Color32,
    pub button_primary: Color32,
    pub button_primary_text: Color32,
    pub pill_ratio: f32,
    pub button_border: u8,
    pub background_dim: u8,
    pub card_background: Color32,
    pub card_border: u8,
    pub card_logo_scrim: u8,
    pub input_background: Color32,
    pub input_border: Color32,
    pub input_border_focus: Option<Color32>,
    pub input_focus_width: f32,
    pub focus: Color32,
    pub page_padding: f32,
    pub screen_padding: f32,
    pub content_header_top_mobile: f32,
    pub content_header_top: f32,
    pub detail_header_top_mobile: f32,
    pub home_hero_content_max_width_desktop: f32,
    pub home_hero_logo_max_width_desktop: f32,
    pub home_hero_logo_height_desktop: f32,
    pub home_hero_synopsis_size_desktop: f32,
    pub library_search_reserved_width: f32,
    pub calendar_panel_width_desktop: f32,
    pub calendar_panel_width_tv: f32,
    pub settings_content_offset: f32,
    pub settings_nav_width: f32,
    pub settings_screen_padding_desktop: f32,
    pub settings_content_max_width_desktop: f32,
    pub settings_nav_item_height_desktop: f32,
    pub settings_nav_item_gap_desktop: f32,
    pub settings_section_title_size_desktop: f32,
    pub settings_row_label_size_desktop: f32,
    pub settings_row_value_size_desktop: f32,
    pub settings_extended_line_spacing: f32,
    pub settings_extended_line_spacing_tv: f32,
    pub settings_extended_line_inset: f32,
    pub settings_extended_input_height: f32,
    pub settings_extended_action_height: f32,
    pub settings_extended_action_gap: f32,
    pub settings_extended_addon_action_width: f32,
    pub button_height: f32,
    pub icon_button_size: f32,
    pub screen_control_height: f32,
    pub screen_control_radius: f32,
    pub labeled_button_icon: f32,
    pub labeled_button_text: f32,
    pub focus_ring_expand: f32,
    pub focus_ring_radius: f32,
    pub focus_ring_width: f32,
    pub card_content_padding: f32,
    pub settings_card_height_mobile: f32,
    pub settings_card_height_desktop: f32,
    pub settings_addon_card_padding: f32,
    pub settings_title_top: f32,
    pub settings_description_top: f32,
    pub settings_row_inset: f32,
    pub settings_row_height: f32,
    pub settings_row_spacing: f32,
    pub settings_toggle_width: f32,
    pub settings_toggle_height: f32,
    pub player_info: PlayerInfo,
    pub player_info_compact: PlayerInfo,
    pub player_tile_pad: f32,
    pub player_tile_pad_compact: f32,
    pub player_tile_width: f32,
    pub profile_page_inset_mobile: f32,
    pub profile_page_inset_desktop: f32,
    pub profile_avatars_max_width: f32,
    pub profile_form_max_width: f32,
    pub profile_picker_max_width: f32,
    pub player_tile_width_ratio_compact: f32,
    pub player_recommendations_width: f32,
    pub player_recommendations_inset: f32,
    pub player_recommendations_rows: usize,
    pub player_recommendations_rows_compact: usize,
    pub settings_group_title_alpha: u8,
    pub settings_group_title_alpha_compact: u8,
    pub settings_swatch_alpha: u8,
    pub settings_swatch_alpha_compact: u8,
    pub poster_sizes: PosterSizes,
    pub home_hero_height: Scaled,
    pub search_width: Scaled,
    pub paint_text_width: Scaled,
    pub paint_text_height: Scaled,
    pub welcome_qr: Scaled,
    pub welcome_text: Scaled,
    pub welcome_button: Scaled,
    pub welcome_wordmark: Scaled,
    pub detail_hero_height: Scaled,
    pub detail_info_width: Scaled,
    pub player_panel_wide: Scaled,
    pub player_panel_narrow: Scaled,
    pub player_panel_card: Scaled,
    pub home_hero_width: Scaled,
    pub ticket_width: Scaled,
    pub card_popover_min: Scaled,
    pub card_popover_max: Scaled,
    pub input_cell_height: Scaled,
    pub mini_player_width: Scaled,
    pub glass_refraction: Scaled,
    pub glass_bevel: Scaled,
    pub paint_hero_fade_from: f32,
    pub paint_hero_dark_at: f32,
    pub welcome_max_width: f32,
    pub sidebar_item_height: f32,
    pub shorts_scale: f32,
    pub shorts_button_height: f32,
    pub home_hero_bottom_inset: f32,
    pub player_recommendations_margin: f32,
    pub player_recommendations_font: f32,
    pub player_recommendations_logo_height: f32,
    pub player_recommendations_title_font: f32,
    pub settings_profile_rows: f32,
    pub profile_select_width: f32,
    pub home_hero_bottom_padding: f32,
    pub episodes_list_thumb_width: f32,
    pub profile_avatar_tile: f32,
    pub detail_hero_bottom_inset: f32,
    pub settings_nav_label_boost: f32,
    pub settings_nav_group_height: f32,
    pub settings_nav_group_gap: f32,
    pub settings_nav_heading_size: f32,
    pub settings_nav_icon: f32,
    pub settings_nav_row_inset: f32,
    pub settings_subtitle_boost: f32,
    pub settings_bottom_reserve: f32,
    pub episodes_gap: f32,
    pub episodes_list_gap: f32,
    pub episodes_grid_min: f32,
    pub detail_section_title: f32,
    pub detail_section_gap: f32,
    pub detail_season_row: f32,
    pub detail_season_tile_label: f32,
    pub detail_episode_width: f32,
    pub detail_cast_size: f32,
    pub detail_cast_text: f32,
    pub detail_trailer_width: f32,
    pub detail_trailer_width_mobile: f32,
    pub detail_hero_extra_mobile: f32,
    pub detail_info_label_scale_mobile: f32,
    pub library_downloads_status_reserve: f32,
    pub detail_trailer_text: f32,
    pub detail_info_row: f32,
    pub detail_info_label: f32,
    pub filmstrip_selected_scale: f32,
    pub filmstrip_min_gap: f32,
    pub filmstrip_wheel_step: f32,
    pub dropdown_dropdown_chrome: f32,
    pub seats_gap: f32,
    pub cards_preview_pad: f32,
    pub cards_preview_button_height: f32,
    pub downloads_change_size: f32,
    pub downloads_tab_center: f32,
    pub menu_popover_min_width: f32,
    pub menu_popover_pad: f32,
    pub streams_stream_row_pad: f32,
    pub streams_stream_action_size: f32,
    pub streams_stream_action_gap: f32,
    pub detail_modal_panel_max_width: f32,
    pub detail_modal_panel_max_height: f32,
    pub detail_modal_panel_margin: f32,
    pub detail_modal_button_height: f32,
    pub detail_modal_pad: f32,
    pub detail_modal_title_height: f32,
    pub detail_modal_episode_height: f32,
    pub detail_modal_episode_thumb_width: f32,
    pub detail_modal_cast_size: f32,
    pub detail_modal_similar_width: f32,
    pub settings_card_padding: f32,
    pub settings_logo: f32,
    pub settings_chip_height: f32,
    pub settings_action: f32,
    pub settings_card_gap: f32,
    pub settings_scraper_row: f32,
    pub settings_note_height: f32,
    pub settings_empty_height: f32,
    pub settings_plugin_risk_height: f32,
    pub settings_server_tabs_height: f32,
    pub settings_plex_hint_height: f32,
    pub settings_form_row: f32,
    pub settings_import_height: f32,
    pub settings_status_height: f32,
    pub settings_layout_preview_line: f32,
    pub settings_layout_preview_pad: f32,
    pub settings_logo_height: f32,
    pub settings_row_padding: f32,
    pub settings_name_gap: f32,
    pub settings_row: f32,
    pub settings_profile_header_height: f32,
    pub settings_profile_row_height: f32,
    pub settings_appearance_header_height: f32,
    pub settings_group_heading_height: f32,
    pub settings_group_gap: f32,
    pub settings_field_height: f32,
    pub settings_subtitle_preview_height: f32,
    pub settings_row_subtitle_extra: f32,
    pub settings_swatch_strip_extra: f32,
    pub settings_slider_extra: f32,
    pub settings_about_hero_height: f32,
    pub settings_about_action_height: f32,
    pub settings_about_tile_height: f32,
    pub settings_about_tile_gap: f32,
    pub settings_watch_ticket_max: f32,
    pub settings_compact_row: f32,
    pub settings_compact_text_left: f32,
    pub settings_group_title_inset: f32,
    pub settings_group_title_boost: f32,
    pub settings_swatch_gap: f32,
    pub settings_about_inset: f32,
    pub settings_service_row_extra: f32,
    pub settings_service_value_width: f32,
    pub settings_service_logo_height: f32,
    pub settings_service_logo_gap: f32,
    pub settings_text_inset: f32,
    pub settings_toggle_knob_radius: f32,
    pub calendar_grid_gap_mobile: f32,
    pub calendar_grid_gap: f32,
    pub calendar_cell_height_mobile: f32,
    pub calendar_cell_height_desktop: f32,
    pub calendar_cell_height_tv: f32,
    pub calendar_cell_radius: f32,
    pub calendar_empty_offset: f32,
    pub calendar_day_padding: f32,
    pub calendar_day_top: f32,
    pub section_gap: f32,
    pub control_gap: f32,
    pub card_radius: f32,
    pub card_progress_height: f32,
    pub filmstrip_min_frames: f32,
    pub filmstrip_frame_width: f32,
    pub card_title_size: f32,
    pub horizontal_card_width: f32,
    pub horizontal_card_height: f32,
    pub home_continue_card_width: f32,
    pub collection_poster_width: f32,
    pub collection_poster_height: f32,
    pub collection_wide_width: f32,
    pub collection_wide_height: f32,
    pub collection_square_size: f32,
    pub screen_margin: f32,
    pub nav_bar_top: f32,
    pub nav_item_height: f32,
    pub nav_bar_padding: f32,
    pub nav_pill_gap: f32,
    pub nav_avatar_radius: f32,
    pub member_avatar_radius: f32,
    pub member_ring_gap: f32,
    pub member_row_avatar_radius: f32,
    pub member_row_ring_gap: f32,
    pub member_row_icon: f32,
    pub surface_card_preview_logo_height_share: f32,
    pub surface_card_preview_logo_width_share: f32,
    pub surface_card_scrim_share: f32,
    pub surface_card_action_icon: f32,
    pub surface_menu_bleed: f32,
    pub surface_menu_scroll_inset: f32,
    pub surface_menu_scroll_min: f32,
    pub surface_menu_scroll_width: f32,
    pub surface_menu_handle_top: f32,
    pub surface_menu_handle_height: f32,
    pub surface_menu_handle_width: f32,
    pub surface_menu_min_rows: f32,
    pub surface_menu_split: f32,
    pub surface_menu_panel_width: f32,
    pub surface_menu_height_share: f32,
    pub surface_menu_pad_side: f32,
    pub surface_menu_pad_bottom: f32,
    pub surface_menu_pad_top: f32,
    pub surface_menu_rule_shrink: f32,
    pub surface_menu_header_inset: f32,
    pub surface_menu_row_inset: f32,
    pub surface_menu_row_radius: f32,
    pub surface_menu_icon: f32,
    pub surface_menu_header: f32,
    pub surface_menu_row: f32,
    pub surface_ticket_narrow_width: f32,
    pub surface_ticket_seat_overlap: f32,
    pub surface_ticket_seat_radius_narrow: f32,
    pub surface_ticket_seat_radius: f32,
    pub surface_ticket_dash_step: f32,
    pub surface_ticket_dash_length: f32,
    pub surface_ticket_dash_inset: f32,
    pub surface_ticket_logo_share: f32,
    pub surface_ticket_logo_max_height: f32,
    pub surface_ticket_field_max_width: f32,
    pub surface_ticket_field_height: f32,
    pub surface_ticket_stub_label: f32,
    pub surface_ticket_cell_label: f32,
    pub surface_ticket_cell_top: f32,
    pub surface_ticket_badge_row: f32,
    pub surface_ticket_badge_pad_y: f32,
    pub surface_ticket_badge_pad_x: f32,
    pub surface_ticket_inline_gap: f32,
    pub surface_ticket_host_radius: f32,
    pub surface_ticket_stub_pad_y: f32,
    pub surface_ticket_pad_y: f32,
    pub surface_ticket_pad_x: f32,
    pub surface_ticket_details: f32,
    pub surface_ticket_stub: f32,
    pub surface_ticket_notch: f32,
    pub surface_ticket_radius: f32,
    pub surface_detail_play_max_width: f32,
    pub surface_detail_similar_caption: f32,
    pub surface_detail_cast_label_height: f32,
    pub surface_detail_cast_label_width: f32,
    pub surface_detail_close_size: f32,
    pub surface_detail_action_icon: f32,
    pub surface_detail_glyph: f32,
    pub surface_detail_progress_height: f32,
    pub surface_detail_thumb_radius: f32,
    pub surface_detail_chip_height: f32,
    pub surface_detail_logo_height_share: f32,
    pub surface_detail_logo_width_share: f32,
    pub surface_detail_image_max_share: f32,
    pub surface_detail_fact_label_share: f32,
    pub surface_detail_text_share: f32,
    pub nav_label_boost: f32,
    pub navigation_icon: f32,
    pub settings_margin: f32,
    pub settings_top: f32,
    pub settings_card_height: f32,
    pub space: [f32; 12],
    pub radius: [f32; 2],
    pub icon: [f32; 3],
    pub sheet_max_width: f32,
    pub drawer_max_width: f32,
    pub sheet_handle: Vec2,
    pub scrim_alpha: u8,
    pub shade: [u8; 5],
    pub tone: [u8; 10],
    pub stroke: [f32; 3],
    pub control_sizes: [f32; 4],
    pub overlay_panel_alpha: u8,
    pub shadow_alpha: u8,
    pub shadow_blur: u8,
    pub shadow_offset: i8,
    pub profile_side_margin: f32,
    pub profile_avatar_radius: f32,
    pub profile_cell_width: f32,
    pub profile_gap: f32,
    pub profile_action_size: f32,
    pub profile_title_size: f32,
    pub profile_brand_height: f32,
    pub profile_header_gap: f32,
    pub overlay_margin: f32,
    pub toast_text_size: f32,
    pub toast_padding: f32,
    pub toast_bar_width: f32,
    pub detail_play_height: f32,
    pub detail_back_top: f32,
    pub detail_title_size: f32,
    pub calendar_cell_height: f32,
    pub calendar_panel_width: f32,
    pub settings_page_lead: f32,
    pub section_title_size: f32,
    pub calendar_beside_min_grid: f32,
    pub hero_width_cap: f32,
    pub hero_logo_height: f32,
    pub hero_logo_width_cap: f32,
    pub hero_title_scale: f32,
    pub hero_metadata_size: f32,
    pub hero_synopsis_size: f32,
    pub home_hero_content_max_width_tv: f32,
    pub home_hero_logo_height_tv: f32,
    pub home_hero_logo_max_width_tv: f32,
    pub home_hero_title_scale_tv: f32,
    pub home_hero_title_scale_desktop: f32,
    pub home_hero_metadata_boost_tv: f32,
    pub home_hero_synopsis_boost_tv: f32,
    pub settings_label_size: f32,
    pub episodes_cell_gap: f32,
    pub settings_page_lead_mobile: f32,
    pub episodes_gap_mobile: f32,
    pub row_heading_size: f32,
    pub grid_min_columns: f32,
    pub grid_max_gap: f32,
    pub settings_page_card_height: f32,
    pub settings_subpage_header_offset: f32,
    pub home_continue_card_height: f32,
    pub poster_card_width: f32,
    pub poster_card_height: f32,
    pub nav_vertical_padding: f32,
    pub nav_item_gap: f32,
    pub nav_label_size: f32,
    pub screen_title_size: f32,
    pub screen_body_size: f32,
    pub screen_section_title_size: f32,
    pub screen_section_title_size_tv: f32,
    pub screen_card_title_size: f32,
    pub screen_card_subtitle_size: f32,
    pub catalog_title_size: f32,
    pub horizontal_spacing: f32,
    pub vertical_spacing: f32,
    pub poster_height_ratio: f32,
    pub horizontal_card_height_ratio: f32,
    pub focused_scale: f32,
    pub card_lift: f32,
    pub gutter: f32,
    pub text: TypeScale,
    pub player_sources_hero_height: f32,
    pub player_sources_waiting_line: f32,
    pub player_sources_chip_shift: f32,
    pub player_sources_chip_shift_range: f32,
    pub player_sources_strip_fade: f32,
    pub player_sources_strip_lip: f32,
    pub player_recommendations_bottom: f32,
    pub player_recommendations_close_top: f32,
    pub player_recommendations_slide: f32,
    pub player_recommendations_dot_pitch: f32,
    pub player_recommendations_dot_hit: f32,
    pub player_recommendations_dot_radius: f32,
    pub player_recommendations_dot_radius_active: f32,
    pub player_recommendations_dots_gap: f32,
    pub player_recommendations_dots_bottom: f32,
    pub player_recommendations_solo_gap: f32,
    pub player_recommendations_eyebrow_height: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TypeScale {
    pub display: f32,
    pub title: f32,
    pub subtitle: f32,
    pub body: f32,
    pub meta: f32,
    pub label: f32,
}

const CONTINUE_CARD_SCALE: f32 = 1.05;

static TEXT_PRIMARY: AtomicU32 = AtomicU32::new(u32::MAX);
static BACKGROUND: AtomicU32 = AtomicU32::new(0xFF06_0606);
static SURFACE: AtomicU32 = AtomicU32::new(0xFF14_1416);
static SURFACE_RAISED: AtomicU32 = AtomicU32::new(0xFF1C_1C1F);
static BUTTON_HEIGHT: AtomicU32 = AtomicU32::new(0x4230_0000);
static ICON_BUTTON: AtomicU32 = AtomicU32::new(0x4220_0000);
static CONTROL_RADIUS: AtomicU32 = AtomicU32::new(0x4110_0000);
static DANGER: AtomicU32 = AtomicU32::new(0xFFFF_6B6B);
static ACCENT: AtomicU32 = AtomicU32::new(0);
static MOTION: [AtomicU32; 3] = [
    AtomicU32::new(0x3DF5_C28F),
    AtomicU32::new(0x3E38_51EC),
    AtomicU32::new(0x3E99_999A),
];

fn seconds(enabled: bool, ms: u64) -> f32 {
    if enabled { ms as f32 / 1000.0 } else { 0.0 }
}

pub(crate) fn motion_seconds(pace: usize) -> f32 {
    f32::from_bits(MOTION[pace].load(Ordering::Relaxed))
}

fn pack(color: Color32) -> u32 {
    u32::from_be_bytes(color.to_srgba_unmultiplied())
}

#[derive(Clone, Copy, PartialEq)]
struct MetricsKey {
    epoch: u64,
    width: u32,
    side_inset: u32,
    tv: bool,
    accent: u32,
    landscape: bool,
}

static ACTIVE: std::sync::RwLock<Option<UiMetrics>> = std::sync::RwLock::new(None);

pub(crate) fn active() -> UiMetrics {
    ACTIVE
        .read()
        .ok()
        .and_then(|slot| *slot)
        .unwrap_or_else(|| UiMetrics::for_viewport(Viewport::new(1280, 800, UiFormFactor::Desktop)))
}

fn publish_palette(metrics: &UiMetrics) {
    if let Ok(mut slot) = ACTIVE.write() {
        *slot = Some(*metrics);
    }
    TEXT_PRIMARY.store(pack(metrics.text_primary), Ordering::Relaxed);
    BACKGROUND.store(pack(metrics.background), Ordering::Relaxed);
    SURFACE.store(pack(metrics.surface), Ordering::Relaxed);
    SURFACE_RAISED.store(pack(metrics.surface_raised), Ordering::Relaxed);
    DANGER.store(pack(metrics.danger), Ordering::Relaxed);
    CONTROL_RADIUS.store(metrics.screen_control_radius.to_bits(), Ordering::Relaxed);
    BUTTON_HEIGHT.store(metrics.button_height.to_bits(), Ordering::Relaxed);
    ICON_BUTTON.store(metrics.icon_button_size.to_bits(), Ordering::Relaxed);
    crate::motion::set_curve(metrics.motion_curve);
    crate::style::set_shades(metrics.shade, metrics.overlay_panel_alpha);
    crate::style::set_tones(metrics.tone, metrics.stroke);
    crate::style::set_control_sizes(metrics.control_sizes);
    for (cell, value) in MOTION.iter().zip([
        metrics.motion_fast,
        metrics.motion_normal,
        metrics.motion_slow,
    ]) {
        cell.store(value.to_bits(), Ordering::Relaxed);
    }
}

fn unpack(cell: &AtomicU32) -> Color32 {
    let [r, g, b, a] = cell.load(Ordering::Relaxed).to_be_bytes();
    Color32::from_rgba_unmultiplied(r, g, b, a)
}

pub fn surface_color() -> Color32 {
    unpack(&SURFACE)
}

pub fn raised_color() -> Color32 {
    unpack(&SURFACE_RAISED)
}

pub fn button_height() -> f32 {
    f32::from_bits(BUTTON_HEIGHT.load(Ordering::Relaxed))
}

pub fn icon_button_size() -> f32 {
    f32::from_bits(ICON_BUTTON.load(Ordering::Relaxed))
}

pub fn control_radius() -> f32 {
    f32::from_bits(CONTROL_RADIUS.load(Ordering::Relaxed))
}

pub fn set_accent(color: Option<Color32>) {
    ACCENT.store(color.map_or(0, pack), Ordering::Relaxed);
}

pub fn accent() -> Color32 {
    match ACCENT.load(Ordering::Relaxed) {
        0 => text_primary(),
        _ => unpack(&ACCENT),
    }
}

pub fn danger_color() -> Color32 {
    unpack(&DANGER)
}

pub fn text_primary() -> Color32 {
    let [r, g, b, a] = TEXT_PRIMARY.load(Ordering::Relaxed).to_be_bytes();
    Color32::from_rgba_unmultiplied(r, g, b, a)
}

pub fn background() -> Color32 {
    let [r, g, b, a] = BACKGROUND.load(Ordering::Relaxed).to_be_bytes();
    Color32::from_rgba_unmultiplied(r, g, b, a)
}

pub fn ink(alpha: u8) -> Color32 {
    let [r, g, b, a] = TEXT_PRIMARY.load(Ordering::Relaxed).to_be_bytes();
    Color32::from_rgba_unmultiplied(r, g, b, (u16::from(a) * u16::from(alpha) / 255) as u8)
}

impl UiMetrics {
    pub fn profile_page_width(&self, viewport_width: f32, desktop_max: f32) -> f32 {
        if self.compact {
            viewport_width - self.profile_page_inset_mobile
        } else {
            (viewport_width - self.profile_page_inset_desktop).min(desktop_max)
        }
    }

    pub fn ink(&self, alpha: u8) -> Color32 {
        let [r, g, b, a] = self.text_primary.to_srgba_unmultiplied();
        Color32::from_rgba_unmultiplied(r, g, b, (u16::from(a) * u16::from(alpha) / 255) as u8)
    }

    pub fn for_viewport(viewport: Viewport) -> Self {
        thread_local! {
            static RECENT: std::cell::RefCell<Vec<(MetricsKey, UiMetrics)>> =
                const { std::cell::RefCell::new(Vec::new()) };
        }
        let key = MetricsKey {
            epoch: fluxa_theme::theme::active_epoch(),
            width: viewport.width.to_bits(),
            side_inset: viewport.side_inset.to_bits(),
            tv: viewport.is_tv(),
            accent: ACCENT.load(Ordering::Relaxed),
            landscape: poster_overlay::landscape(),
        };
        let hit = RECENT.with(|recent| {
            recent
                .borrow()
                .iter()
                .find(|(cached, _)| *cached == key)
                .map(|(_, metrics)| *metrics)
        });
        if let Some(metrics) = hit {
            publish_palette(&metrics);
            return metrics;
        }
        let metrics = Self::compute(viewport);
        RECENT.with(|recent| {
            let mut recent = recent.borrow_mut();
            if recent.len() >= 4 {
                recent.remove(0);
            }
            recent.push((key, metrics));
        });
        metrics
    }

    fn compute(viewport: Viewport) -> Self {
        let snapshot = active_snapshot(&fluxa_theme::theme::selected_theme_id())
            .expect("shared Fluxa theme must be valid");
        let tokens = &snapshot.tokens;
        let dark = &snapshot.theme;
        let number = |path: &[&str], fallback: f32| {
            let mut value = &tokens.layout;
            for key in path {
                let Some(next) = value.get(*key) else {
                    return fallback;
                };
                value = next;
            }
            value.as_f64().map(|value| value as f32).unwrap_or(fallback)
        };
        let platform = if viewport.is_compact() {
            "mobile"
        } else if viewport.is_tv() {
            "tv"
        } else {
            "desktop"
        };
        let window_class = match viewport.size_class() {
            SizeClass::Compact => "compact",
            SizeClass::Medium => "medium",
            SizeClass::Expanded => "expanded",
        };
        let roomy = !viewport.is_compact() && !viewport.is_tv();
        let pick = |roomy_value: f32, compact_value: f32| {
            if roomy { roomy_value } else { compact_value }
        };
        let common_dp = |key, fallback| number(&["common", "dp", key], fallback);
        let common_sp = |key, fallback| number(&["common", "sp", key], fallback);
        let platform_dp = |key, fallback| number(&["platforms", platform, "dp", key], fallback);
        let platform_scaled = |key: &str, ratio: f32, min: f32, max: f32| {
            tokens
                .layout
                .get("platforms")
                .and_then(|value| value.get(platform))
                .and_then(|value| value.get("scaled"))
                .and_then(|value| value.get(key))
                .and_then(|value| value.as_array())
                .and_then(|values| {
                    let at = |index: usize| values.get(index)?.as_f64().map(|v| v as f32);
                    Some(Scaled {
                        ratio: at(0)?,
                        min: at(1)?,
                        max: at(2)?,
                        offset: at(3).unwrap_or(0.0),
                    })
                })
                .unwrap_or(Scaled {
                    ratio,
                    min,
                    max,
                    offset: 0.0,
                })
        };
        let platform_sp = |key, fallback| number(&["platforms", platform, "sp", key], fallback);
        let window_dp =
            |key, fallback| number(&["windowClasses", window_class, "dp", key], fallback);
        let window_sp =
            |key, fallback| number(&["windowClasses", window_class, "sp", key], fallback);
        let color = |token: &str, fallback: Color32| {
            dark.color(token)
                .map(|value| {
                    Color32::from_rgba_unmultiplied(
                        (value.r * 255.0).round() as u8,
                        (value.g * 255.0).round() as u8,
                        (value.b * 255.0).round() as u8,
                        (value.a * 255.0).round() as u8,
                    )
                })
                .unwrap_or(fallback)
        };
        let mut metrics = Self {
            compact: viewport.is_compact(),
            platform,
            window_class,
            viewport_width: viewport.width,
            background: color("background", Color32::from_rgb(7, 7, 9)),
            surface: color("surface", Color32::from_rgb(20, 20, 22)),
            surface_raised: color("surfaceRaised", Color32::from_rgb(28, 28, 31)),
            border: color("border", Color32::from_white_alpha(20)),
            danger: color("error", Color32::from_rgb(255, 107, 107)),
            navigation: color("navigation", Color32::from_rgb(20, 20, 22)),
            text_primary: color("textPrimary", Color32::WHITE),
            text_secondary: color("textSecondary", Color32::from_rgb(184, 184, 190)),
            text_muted: color("textMuted", Color32::from_rgb(138, 138, 145)),
            accent: color("accent", Color32::from_rgb(239, 78, 52)),
            accent_foreground: color("accentForeground", Color32::WHITE),
            button_primary: color("buttonPrimary", color("textPrimary", Color32::WHITE)),
            button_primary_text: color(
                "buttonPrimaryText",
                color("background", Color32::from_rgb(7, 7, 9)),
            ),
            card_background: color(
                "cardBackground",
                color("surface", Color32::from_rgb(20, 20, 22)),
            ),
            card_border: (number(&["common", "alpha", "cardBorder"], 0.0) * 255.0).round() as u8,
            card_logo_scrim: (number(&["common", "alpha", "cardLogoScrim"], 0.745) * 255.0).round()
                as u8,
            input_background: color(
                "inputBackground",
                color("surfaceRaised", Color32::from_rgb(28, 28, 31)),
            ),
            input_border: color(
                "inputBorder",
                color("border", Color32::from_white_alpha(20)),
            ),
            input_border_focus: dark
                .color("inputBorderFocus")
                .is_ok()
                .then(|| color("inputBorderFocus", Color32::TRANSPARENT)),
            input_focus_width: number(&["common", "number", "inputFocusWidth"], 1.5),
            background_dim: (number(&["common", "alpha", "backgroundImageDim"], 0.55) * 255.0)
                .round() as u8,
            pill_ratio: number(&["common", "number", "pillRadiusRatio"], 0.5),
            button_border: (number(&["common", "alpha", "buttonBorder"], 0.07) * 255.0).round()
                as u8,
            focus: color("focus", Color32::WHITE),
            page_padding: match viewport.size_class() {
                _ if viewport.is_tv() => number(
                    &["windowClasses", "expanded", "dp", "pageHorizontalPadding"],
                    48.0,
                ),
                SizeClass::Expanded => dark.spacing.screen_padding,
                _ => window_dp("pageHorizontalPadding", 16.0),
            },
            screen_padding: dark.spacing.screen_padding,
            content_header_top_mobile: common_dp("contentHeaderTopMobile", 48.0),
            content_header_top: if viewport.side_inset > 0.0 {
                common_dp("contentHeaderTopSidebar", 48.0)
            } else {
                common_dp("contentHeaderTop", 110.0)
            },
            detail_header_top_mobile: common_dp("detailHeaderTopMobile", 48.0),
            home_hero_content_max_width_desktop: common_dp("homeHeroContentMaxWidthDesktop", 980.0),
            home_hero_logo_max_width_desktop: common_dp("homeHeroLogoMaxWidthDesktop", 820.0),
            home_hero_logo_height_desktop: common_dp("homeHeroLogoHeightDesktop", 148.0),
            home_hero_synopsis_size_desktop: common_sp("homeHeroSynopsisSizeDesktop", 16.0),
            library_search_reserved_width: common_dp("librarySearchReservedWidth", 150.0),
            calendar_panel_width_desktop: common_dp("calendarPanelWidthDesktop", 360.0),
            calendar_panel_width_tv: common_dp("calendarPanelWidthTv", 440.0),
            settings_content_offset: pick(
                common_dp("settingsNavWidthDesktop", 268.0)
                    + common_dp("settingsPanelGapDesktop", 32.0),
                common_dp("settingsContentOffset", 220.0),
            ),
            settings_nav_width: pick(
                common_dp("settingsNavWidthDesktop", 268.0),
                common_dp("settingsNavWidth", 196.0),
            ),
            settings_screen_padding_desktop: common_dp("settingsScreenPaddingDesktop", 48.0),
            settings_content_max_width_desktop: common_dp("settingsContentMaxWidthDesktop", 720.0),
            settings_nav_item_height_desktop: common_dp("settingsNavItemHeightDesktop", 44.0),
            settings_nav_item_gap_desktop: common_dp("settingsNavItemGapDesktop", 6.0),
            settings_section_title_size_desktop: common_sp("settingsSectionTitleSizeDesktop", 24.0),
            settings_row_label_size_desktop: common_sp("settingsRowLabelSizeDesktop", 18.0),
            settings_row_value_size_desktop: common_sp("settingsRowValueSizeDesktop", 15.0),
            settings_extended_line_spacing: common_dp("settingsExtendedLineSpacing", 34.0),
            settings_extended_line_spacing_tv: common_dp("settingsExtendedLineSpacingTv", 42.0),
            settings_extended_line_inset: common_dp("settingsExtendedLineInset", 20.0),
            settings_extended_input_height: common_dp("settingsExtendedInputHeight", 40.0),
            settings_extended_action_height: common_dp("settingsExtendedActionHeight", 38.0),
            settings_extended_action_gap: common_dp("settingsExtendedActionGap", 10.0),
            settings_extended_addon_action_width: common_dp("settingsAddonActionWidth", 148.0),
            button_height: platform_dp("buttonHeight", 44.0),
            icon_button_size: platform_dp("iconButtonSize", 40.0),
            screen_control_height: common_dp("screenControlHeight", 36.0),
            screen_control_radius: common_dp("screenControlRadius", 9.0),
            labeled_button_icon: common_dp("labeledButtonIconSize", 22.0),
            labeled_button_text: common_sp("labeledButtonTextSize", 12.0),
            focus_ring_expand: common_dp("focusRingExpand", 3.0),
            focus_ring_radius: common_dp("focusRingRadius", 10.0),
            focus_ring_width: common_dp("focusRingWidth", 2.5),
            card_content_padding: common_dp("cardContentPadding", 10.0),
            settings_card_height_mobile: common_dp("settingsCardHeightMobile", 168.0),
            settings_card_height_desktop: common_dp("settingsCardHeightDesktop", 154.0),
            settings_card_padding: pick(
                common_dp("settingsCardPaddingDesktop", 24.0),
                common_dp("settingsCardPadding", 16.0),
            ),
            settings_title_top: pick(
                common_dp("settingsTitleTopDesktop", 18.0),
                common_dp("settingsTitleTop", 14.0),
            ),
            settings_description_top: pick(
                common_dp("settingsDescriptionTopDesktop", 54.0),
                common_dp("settingsDescriptionTop", 39.0),
            ),
            settings_row_inset: pick(
                common_dp("settingsRowInsetDesktop", 24.0),
                common_dp("settingsRowInset", 14.0),
            ),
            settings_row_height: pick(
                common_dp("settingsRowHeightDesktop", 44.0),
                common_dp("settingsRowHeight", 32.0),
            ),
            settings_row_spacing: pick(
                common_dp("settingsRowSpacingDesktop", 56.0),
                common_dp("settingsRowSpacing", 38.0),
            ),
            settings_toggle_width: pick(
                common_dp("settingsToggleWidthDesktop", 48.0),
                common_dp("settingsToggleWidth", 40.0),
            ),
            settings_group_title_inset: platform_dp("settingsGroupTitleInset", 6.0),
            settings_group_title_boost: platform_dp("settingsGroupTitleBoost", 3.0),
            settings_swatch_gap: platform_dp("settingsSwatchGap", 14.0),
            settings_about_inset: platform_dp("settingsAboutInset", 28.0),
            settings_service_row_extra: platform_dp("settingsServiceRowExtra", 20.0),
            settings_service_value_width: platform_dp("settingsServiceValueWidth", 176.0),
            settings_service_logo_height: platform_dp("settingsServiceLogoHeight", 30.0),
            settings_service_logo_gap: platform_dp("settingsServiceLogoGap", 12.0),
            settings_text_inset: platform_dp("settingsTextInset", 8.0),
            settings_appearance_header_height: common_dp("settingsAppearanceHeaderHeight", 66.0),
            settings_group_heading_height: common_dp("settingsGroupHeadingHeight", 22.0),
            settings_group_gap: common_dp("settingsGroupGap", 22.0),
            settings_field_height: common_dp("settingsFieldHeight", 118.0),
            settings_subtitle_preview_height: common_dp("settingsSubtitlePreviewHeight", 168.0),
            settings_row_subtitle_extra: common_dp("settingsRowSubtitleExtra", 18.0),
            settings_swatch_strip_extra: common_dp("settingsSwatchStripExtra", 56.0),
            settings_slider_extra: common_dp("settingsSliderExtra", 46.0),
            settings_about_hero_height: common_dp("settingsAboutHeroHeight", 124.0),
            settings_about_action_height: common_dp("settingsAboutActionHeight", 44.0),
            settings_about_tile_height: common_dp("settingsAboutTileHeight", 76.0),
            settings_about_tile_gap: common_dp("settingsAboutTileGap", 12.0),
            settings_watch_ticket_max: common_dp("settingsWatchTicketMax", 560.0),
            settings_compact_row: common_dp("settingsCompactRow", 52.0),
            settings_compact_text_left: common_dp("settingsCompactTextLeft", 50.0),
            settings_addon_card_padding: common_dp("settingsAddonCardPadding", 16.0),
            settings_logo: common_dp("settingsLogo", 48.0),
            settings_chip_height: common_dp("settingsChipHeight", 26.0),
            settings_action: common_dp("settingsAction", 40.0),
            settings_card_gap: common_dp("settingsCardGap", 12.0),
            settings_scraper_row: common_dp("settingsScraperRow", 64.0),
            settings_note_height: common_dp("settingsNoteHeight", 48.0),
            settings_empty_height: common_dp("settingsEmptyHeight", 64.0),
            settings_plugin_risk_height: common_dp("settingsPluginRiskHeight", 76.0),
            settings_server_tabs_height: common_dp("settingsServerTabsHeight", 44.0),
            settings_plex_hint_height: common_dp("settingsPlexHintHeight", 48.0),
            settings_form_row: common_dp("settingsFormRow", 52.0),
            settings_import_height: common_dp("settingsImportHeight", 76.0),
            settings_status_height: common_dp("settingsStatusHeight", 24.0),
            settings_layout_preview_line: common_dp("settingsLayoutPreviewLine", 26.0),
            settings_layout_preview_pad: common_dp("settingsLayoutPreviewPad", 18.0),
            settings_logo_height: common_dp("settingsLogoHeight", 16.0),
            settings_row_padding: common_dp("settingsRowPadding", 14.0),
            settings_name_gap: common_dp("settingsNameGap", 4.0),
            settings_row: common_dp("settingsRow", 52.0),
            settings_profile_header_height: common_dp("settingsProfileHeaderHeight", 96.0),
            settings_profile_row_height: common_dp("settingsProfileRowHeight", 60.0),
            detail_modal_panel_max_width: common_dp("detailModalPanelMaxWidth", 980.0),
            detail_modal_panel_max_height: common_dp("detailModalPanelMaxHeight", 900.0),
            detail_modal_panel_margin: common_dp("detailModalPanelMargin", 24.0),
            detail_modal_button_height: common_dp("detailModalButtonHeight", 48.0),
            detail_modal_pad: common_dp("detailModalPad", 28.0),
            detail_modal_title_height: common_dp("detailModalTitleHeight", 48.0),
            detail_modal_episode_height: common_dp("detailModalEpisodeHeight", 132.0),
            detail_modal_episode_thumb_width: common_dp("detailModalEpisodeThumbWidth", 196.0),
            detail_modal_cast_size: common_dp("detailModalCastSize", 108.0),
            detail_modal_similar_width: common_dp("detailModalSimilarWidth", 128.0),
            streams_stream_row_pad: common_dp("streamsStreamRowPad", 20.0),
            streams_stream_action_size: common_dp("streamsStreamActionSize", 32.0),
            streams_stream_action_gap: common_dp("streamsStreamActionGap", 6.0),
            menu_popover_min_width: common_dp("menuPopoverMinWidth", 240.0),
            menu_popover_pad: common_dp("menuPopoverPad", 6.0),
            downloads_change_size: common_dp("downloadsChangeSize", 52.0),
            downloads_tab_center: common_dp("downloadsTabCenter", 19.0),
            cards_preview_pad: common_dp("cardsPreviewPad", 18.0),
            cards_preview_button_height: common_dp("cardsPreviewButtonHeight", 44.0),
            seats_gap: common_dp("seatsGap", 8.0),
            dropdown_dropdown_chrome: common_dp("dropdownDropdownChrome", 44.0),
            filmstrip_selected_scale: common_dp("filmstripSelectedScale", 1.08),
            filmstrip_min_gap: common_dp("filmstripMinGap", 0.06),
            filmstrip_wheel_step: common_dp("filmstripWheelStep", 40.0),
            detail_section_title: common_dp("detailSectionTitle", 44.0),
            detail_section_gap: common_dp("detailSectionGap", 36.0),
            detail_season_row: common_dp("detailSeasonRow", 54.0),
            detail_season_tile_label: common_dp("detailSeasonTileLabel", 30.0),
            detail_episode_width: common_dp("detailEpisodeWidth", 300.0),
            detail_cast_size: common_dp("detailCastSize", 92.0),
            detail_cast_text: common_dp("detailCastText", 48.0),
            detail_trailer_width: common_dp("detailTrailerWidth", 280.0),
            detail_trailer_width_mobile: common_dp("detailTrailerWidthMobile", 240.0),
            detail_hero_extra_mobile: common_dp("detailHeroExtraMobile", 250.0),
            detail_info_label_scale_mobile: number(
                &["common", "number", "detailInfoLabelScaleMobile"],
                0.7,
            ),
            library_downloads_status_reserve: common_dp("libraryDownloadsStatusReserve", 340.0),
            detail_trailer_text: common_dp("detailTrailerText", 34.0),
            detail_info_row: common_dp("detailInfoRow", 38.0),
            detail_info_label: common_dp("detailInfoLabel", 150.0),
            episodes_gap: common_dp("episodesGap", 16.0),
            episodes_list_gap: common_dp("episodesListGap", 8.0),
            episodes_grid_min: common_dp("episodesGridMin", 260.0),
            paint_hero_fade_from: platform_dp("paintHeroFadeFrom", 0.3),
            paint_hero_dark_at: platform_dp("paintHeroDarkAt", 0.8),
            welcome_max_width: platform_dp("welcomeMaxWidth", 400.0),
            sidebar_item_height: platform_dp("sidebarItemHeight", 44.0),
            shorts_scale: platform_dp("shortsScale", 1.0),
            shorts_button_height: platform_dp("shortsButtonHeight", 46.0),
            home_hero_bottom_inset: platform_dp("homeHeroBottomInset", 194.0),
            player_recommendations_margin: platform_dp("playerRecommendationsMargin", 56.0),
            player_recommendations_font: platform_dp("playerRecommendationsFont", 16.0),
            player_recommendations_logo_height: platform_dp(
                "playerRecommendationsLogoHeight",
                120.0,
            ),
            player_recommendations_title_font: platform_dp("playerRecommendationsTitleFont", 40.0),
            settings_profile_rows: platform_dp("settingsProfileRows", 2.0),
            profile_select_width: platform_dp("profileSelectWidth", 240.0),
            home_hero_bottom_padding: platform_dp("homeHeroBottomPadding", 48.0),
            episodes_list_thumb_width: platform_dp("episodesListThumbWidth", 200.0),
            profile_avatar_tile: platform_dp("profileAvatarTile", 84.0),
            detail_hero_bottom_inset: platform_dp("detailHeroBottomInset", 56.0),
            settings_nav_label_boost: platform_dp("settingsNavLabelBoost", 2.0),
            settings_nav_group_height: platform_dp("settingsNavGroupHeight", 28.0),
            settings_nav_group_gap: platform_dp("settingsNavGroupGap", 14.0),
            settings_nav_heading_size: platform_dp("settingsNavHeadingSize", 11.0),
            settings_nav_icon: platform_dp("settingsNavIcon", 18.0),
            settings_nav_row_inset: platform_dp("settingsNavRowInset", 12.0),
            settings_subtitle_boost: platform_dp("settingsSubtitleBoost", 3.5),
            settings_bottom_reserve: platform_dp("settingsBottomReserve", 0.0),
            search_width: platform_scaled("searchWidth", 0.36, 220.0, 420.0),
            paint_text_width: platform_scaled("paintSizeA", 0.022, 15.0, 26.0),
            paint_text_height: platform_scaled("paintSizeB", 0.052, 18.0, 60.0),
            welcome_qr: platform_scaled("welcomeQr", 0.34, 240.0, 420.0),
            welcome_text: platform_scaled("welcomeText", 0.32, 420.0, 680.0),
            welcome_button: platform_scaled("welcomeButton", 0.4, 320.0, 520.0),
            welcome_wordmark: platform_scaled("welcomeWordmark", 0.16, 56.0, 88.0),
            detail_hero_height: platform_scaled("detailHeroHeight", 0.84, 480.0, 980.0),
            detail_info_width: platform_scaled("detailInfoWidth", 0.42, 420.0, 560.0),
            player_panel_wide: platform_scaled("playerPanelA", 0.32, 380.0, 560.0),
            player_panel_narrow: platform_scaled("playerPanelB", 0.24, 260.0, 340.0),
            player_panel_card: platform_scaled("playerPanelC", 0.45, 280.0, 340.0),
            home_hero_width: platform_scaled("homeHeroWidth", 0.55, 420.0, 720.0),
            ticket_width: platform_scaled("ticketWidth", 0.62, 236.0, 272.0),
            card_popover_min: platform_scaled("cardPopoverMin", 1.5, 340.0, 420.0),
            card_popover_max: platform_scaled("cardPopoverMax", 2.4, 400.0, 500.0),
            input_cell_height: platform_scaled("inputCellHeight", 1.2, 42.0, 52.0),
            mini_player_width: platform_scaled("miniPlayerWidth", 0.22, 280.0, 400.0),
            glass_refraction: platform_scaled("glassRefraction", 0.3, 6.0, 22.0),
            glass_bevel: platform_scaled("glassBevel", 0.8, 8.0, 26.0),
            home_hero_height: platform_scaled("homeHeroHeight", 0.66, 728.0, 984.0),
            poster_sizes: PosterSizes {
                mark: platform_scaled("posterMark", 0.13, 14.0, 28.0),
                badge: platform_scaled("posterBadge", 0.26, 30.0, 64.0),
                tag: platform_scaled("posterTag", 0.065, 8.0, 14.0),
                caption: platform_scaled("posterCaption", 0.085, 9.0, 15.0),
                meta_bar: platform_scaled("posterMetaBar", 0.5, 11.0, 28.0),
                meta_text: platform_scaled("posterMetaText", 0.1, 11.0, 30.0),
                landscape_text: platform_scaled("posterLandscapeText", 0.08, 8.0, 13.0),
                score_bar: platform_scaled("posterScoreBar", 0.08, 8.0, 18.0),
                score_minimal: platform_scaled("posterScoreMinimal", 0.07, 8.0, 16.0),
                score_number: platform_scaled("posterScoreNumber", 0.2, 16.0, 56.0),
                score_caption: platform_scaled("posterScoreCaption", 0.075, 9.0, 18.0),
                sash: platform_scaled("posterSash", 0.07, 9.0, 17.0),
            },
            player_info: PlayerInfo {
                left: common_dp("playerInfoLeft", 0.0),
                label_gap: common_dp("playerInfoLabelGap", 0.0),
                label_font: common_dp("playerInfoLabelFont", 0.0),
                title_font: common_dp("playerInfoTitleFont", 0.0),
                bottom: common_dp("playerInfoBottom", 0.0),
                scale: common_dp("playerInfoScale", 0.0),
                description_rows: common_dp("playerInfoDescriptionRows", 0.0) as usize,
                width: Scaled {
                    ratio: common_dp("playerInfoWidthRatio", 0.0),
                    min: common_dp("playerInfoWidthMin", 0.0),
                    max: common_dp("playerInfoWidthMax", 0.0),
                    offset: 0.0,
                },
                logo_width: common_dp("playerInfoLogoWidth", 420.0),
                logo_height: common_dp("playerInfoLogoHeight", 96.0),
            },
            player_info_compact: PlayerInfo {
                left: common_dp("playerInfoLeftCompact", 0.0),
                label_gap: common_dp("playerInfoLabelGapCompact", 0.0),
                label_font: common_dp("playerInfoLabelFontCompact", 0.0),
                title_font: common_dp("playerInfoTitleFontCompact", 0.0),
                bottom: common_dp("playerInfoBottomCompact", 0.0),
                scale: common_dp("playerInfoScaleCompact", 0.0),
                description_rows: common_dp("playerInfoDescriptionRowsCompact", 0.0) as usize,
                width: Scaled {
                    ratio: common_dp("playerInfoWidthRatioCompact", 0.0),
                    min: common_dp("playerInfoWidthMin", 0.0),
                    max: common_dp("playerInfoWidthMax", 0.0),
                    offset: 0.0,
                },
                logo_width: common_dp("playerInfoLogoWidthCompact", 220.0),
                logo_height: common_dp("playerInfoLogoHeightCompact", 56.0),
            },
            player_tile_pad: common_dp("playerTilePad", 24.0),
            player_tile_pad_compact: common_dp("playerTilePadCompact", 16.0),
            player_tile_width: common_dp("playerTileWidth", 448.0),
            profile_page_inset_mobile: common_dp("profilePageInsetMobile", 32.0),
            profile_page_inset_desktop: common_dp("profilePageInsetDesktop", 56.0),
            profile_avatars_max_width: common_dp("profileAvatarsMaxWidth", 1100.0),
            profile_form_max_width: common_dp("profileFormMaxWidth", 520.0),
            profile_picker_max_width: common_dp("profilePickerMaxWidth", 1024.0),
            player_tile_width_ratio_compact: common_dp("playerTileWidthRatioCompact", 0.46),
            player_recommendations_width: common_dp("playerRecommendationsWidth", 496.0),
            player_recommendations_inset: common_dp("playerRecommendationsInset", 120.0),
            player_recommendations_rows: common_dp("playerRecommendationsRows", 4.0) as usize,
            player_recommendations_rows_compact: common_dp("playerRecommendationsRowsCompact", 3.0)
                as usize,
            settings_group_title_alpha: common_dp("settingsGroupTitleAlpha", 225.0) as u8,
            settings_group_title_alpha_compact: common_dp("settingsGroupTitleAlphaCompact", 150.0)
                as u8,
            settings_swatch_alpha: common_dp("settingsSwatchAlpha", 150.0) as u8,
            settings_swatch_alpha_compact: common_dp("settingsSwatchAlphaCompact", 100.0) as u8,
            settings_toggle_height: pick(
                common_dp("settingsToggleHeightDesktop", 26.0),
                common_dp("settingsToggleHeight", 20.0),
            ),
            settings_toggle_knob_radius: pick(
                common_dp("settingsToggleKnobRadiusDesktop", 9.0),
                common_dp("settingsToggleKnobRadius", 7.0),
            ),
            calendar_grid_gap_mobile: common_dp("calendarGridGapMobile", 5.0),
            calendar_grid_gap: common_dp("calendarGridGap", 8.0),
            calendar_cell_height_mobile: common_dp("calendarCellHeightMobile", 58.0),
            calendar_cell_height_desktop: common_dp("calendarCellHeightDesktop", 96.0),
            calendar_cell_height_tv: common_dp("calendarCellHeightTv", 112.0),
            calendar_cell_radius: common_dp("calendarCellRadius", 9.0),
            calendar_empty_offset: common_dp("calendarEmptyOffset", 150.0),
            calendar_day_padding: common_dp("calendarDayPadding", 8.0),
            calendar_day_top: common_dp("calendarDayTop", 7.0),
            section_gap: dark.spacing.section_gap,
            control_gap: dark.spacing.control_gap,
            card_radius: dark.shape.card_radius,
            dialog_radius: dark.shape.dialog_radius,
            motion_fast: seconds(dark.motion.enabled, dark.motion.fast_ms),
            motion_normal: seconds(dark.motion.enabled, dark.motion.normal_ms),
            motion_slow: seconds(dark.motion.enabled, dark.motion.slow_ms),
            motion_curve: [
                number(&["common", "number", "motionCurveX1"], 0.2),
                number(&["common", "number", "motionCurveY1"], 0.0),
                number(&["common", "number", "motionCurveX2"], 0.0),
                number(&["common", "number", "motionCurveY2"], 1.0),
            ],
            card_progress_height: common_dp("cardProgressBarHeight", 4.0),
            filmstrip_min_frames: platform_dp("seekFilmstripMinFrames", 6.0),
            filmstrip_frame_width: platform_dp("seekFilmstripFrameWidth", 150.0),
            card_title_size: common_sp("cardTitleSize", 12.0),
            horizontal_card_width: platform_dp("episodeCardWidth", 300.0),
            horizontal_card_height: platform_dp("episodeCardHeight", 180.0),
            // Home's Continue Watching row uses the same responsive size
            // presets as Compose horizontalCardWidth/Height, not the larger
            // episode-card dimensions used by other surfaces.
            home_continue_card_width: window_dp("horizontalCardBase", 196.0) * CONTINUE_CARD_SCALE,
            home_continue_card_height: window_dp("horizontalCardBase", 196.0)
                * CONTINUE_CARD_SCALE
                * number(&["common", "number", "horizontalCardHeightRatio"], 0.56),
            poster_card_width: platform_dp("posterCardWidth", 160.0),
            poster_card_height: platform_dp("posterCardHeight", 240.0),
            nav_vertical_padding: window_dp("navigationVerticalPadding", 12.0),
            nav_item_gap: window_dp("navigationItemSpacing", 8.0),
            nav_label_size: window_sp("navigationLabelSize", 14.0),
            screen_title_size: match platform {
                "mobile" => common_sp("screenTitleSizeMobile", 32.0),
                "tv" => common_sp("screenTitleSizeTv", 42.0),
                _ => common_sp("screenTitleSize", 34.0),
            },
            screen_body_size: if platform == "tv" {
                common_sp("screenBodySizeTv", 18.0)
            } else {
                common_sp("screenBodySize", 15.0)
            },
            screen_section_title_size: common_sp("screenSectionTitleSize", 20.0),
            screen_section_title_size_tv: common_sp("screenSectionTitleSizeTv", 24.0),
            screen_card_title_size: if platform == "tv" {
                common_sp("screenCardTitleSizeTv", 17.0)
            } else {
                platform_sp(
                    "screenCardTitleSize",
                    common_sp("screenCardTitleSize", 15.0),
                )
            },
            screen_card_subtitle_size: platform_sp(
                "screenCardSubtitleSize",
                common_sp("screenCardSubtitleSize", 12.0),
            ),
            catalog_title_size: window_sp("catalogTitleSize", 26.0),
            horizontal_spacing: window_dp("horizontalSpacing", 16.0),
            vertical_spacing: window_dp("verticalSpacing", 20.0),
            poster_height_ratio: number(&["common", "number", "posterHeightRatio"], 1.5),
            horizontal_card_height_ratio: number(
                &["common", "number", "horizontalCardHeightRatio"],
                0.56,
            ),
            focused_scale: number(&["common", "number", "cardFocusedScale"], 1.12),
            card_lift: number(&["common", "number", "cardLift"], 8.0),
            gutter: platform_dp("gutter", 16.0),
            text: TypeScale {
                display: platform_sp("display", 28.0),
                title: platform_sp("title", 20.0),
                subtitle: platform_sp("subtitle", 16.0),
                body: platform_sp("body", 14.5),
                meta: platform_sp("meta", 13.0),
                label: platform_sp("label", 12.0),
            },
            collection_poster_width: platform_dp("collectionPosterWidth", 156.0),
            collection_poster_height: platform_dp("collectionPosterHeight", 234.0),
            collection_wide_width: platform_dp("collectionWideWidth", 280.0),
            collection_wide_height: platform_dp("collectionWideHeight", 158.0),
            collection_square_size: platform_dp("collectionSquareSize", 150.0),
            screen_margin: 0.0,
            nav_bar_top: 0.0,
            nav_item_height: 0.0,
            nav_bar_padding: 0.0,
            nav_pill_gap: 0.0,
            nav_avatar_radius: 0.0,
            member_avatar_radius: 0.0,
            member_ring_gap: 0.0,
            member_row_avatar_radius: 0.0,
            member_row_ring_gap: 0.0,
            member_row_icon: 0.0,
            surface_card_preview_logo_height_share: 0.0,
            surface_card_preview_logo_width_share: 0.0,
            surface_card_scrim_share: 0.0,
            surface_card_action_icon: 0.0,
            surface_menu_bleed: 0.0,
            surface_menu_scroll_inset: 0.0,
            surface_menu_scroll_min: 0.0,
            surface_menu_scroll_width: 0.0,
            surface_menu_handle_top: 0.0,
            surface_menu_handle_height: 0.0,
            surface_menu_handle_width: 0.0,
            surface_menu_min_rows: 0.0,
            surface_menu_split: 0.0,
            surface_menu_panel_width: 0.0,
            surface_menu_height_share: 0.0,
            surface_menu_pad_side: 0.0,
            surface_menu_pad_bottom: 0.0,
            surface_menu_pad_top: 0.0,
            surface_menu_rule_shrink: 0.0,
            surface_menu_header_inset: 0.0,
            surface_menu_row_inset: 0.0,
            surface_menu_row_radius: 0.0,
            surface_menu_icon: 0.0,
            surface_menu_header: 0.0,
            surface_menu_row: 0.0,
            surface_ticket_narrow_width: 0.0,
            surface_ticket_seat_overlap: 0.0,
            surface_ticket_seat_radius_narrow: 0.0,
            surface_ticket_seat_radius: 0.0,
            surface_ticket_dash_step: 0.0,
            surface_ticket_dash_length: 0.0,
            surface_ticket_dash_inset: 0.0,
            surface_ticket_logo_share: 0.0,
            surface_ticket_logo_max_height: 0.0,
            surface_ticket_field_max_width: 0.0,
            surface_ticket_field_height: 0.0,
            surface_ticket_stub_label: 0.0,
            surface_ticket_cell_label: 0.0,
            surface_ticket_cell_top: 0.0,
            surface_ticket_badge_row: 0.0,
            surface_ticket_badge_pad_y: 0.0,
            surface_ticket_badge_pad_x: 0.0,
            surface_ticket_inline_gap: 0.0,
            surface_ticket_host_radius: 0.0,
            surface_ticket_stub_pad_y: 0.0,
            surface_ticket_pad_y: 0.0,
            surface_ticket_pad_x: 0.0,
            surface_ticket_details: 0.0,
            surface_ticket_stub: 0.0,
            surface_ticket_notch: 0.0,
            surface_ticket_radius: 0.0,
            surface_detail_play_max_width: 0.0,
            surface_detail_similar_caption: 0.0,
            surface_detail_cast_label_height: 0.0,
            surface_detail_cast_label_width: 0.0,
            surface_detail_close_size: 0.0,
            surface_detail_action_icon: 0.0,
            surface_detail_glyph: 0.0,
            surface_detail_progress_height: 0.0,
            surface_detail_thumb_radius: 0.0,
            surface_detail_chip_height: 0.0,
            surface_detail_logo_height_share: 0.0,
            surface_detail_logo_width_share: 0.0,
            surface_detail_image_max_share: 0.0,
            surface_detail_fact_label_share: 0.0,
            surface_detail_text_share: 0.0,
            nav_label_boost: 0.0,
            navigation_icon: 0.0,
            settings_margin: 0.0,
            settings_top: 0.0,
            settings_card_height: 0.0,
            radius: [common_dp("radiusMd", 12.0), common_dp("radiusSheet", 24.0)],
            icon: [
                common_dp("iconSm", 16.0),
                common_dp("iconMd", 22.0),
                common_dp("iconXl", 36.0),
            ],
            sheet_max_width: common_dp("sheetMaxWidth", 560.0),
            drawer_max_width: common_dp("drawerMaxWidth", 420.0),
            sheet_handle: Vec2::new(
                common_dp("sheetHandleWidth", 36.0),
                common_dp("sheetHandleHeight", 4.0),
            ),
            scrim_alpha: common_dp("scrimAlpha", 180.0) as u8,
            tone: [
                common_dp("toneFaint", 10.0) as u8,
                common_dp("toneSoft", 20.0) as u8,
                common_dp("toneRaised", 30.0) as u8,
                common_dp("toneStrong", 50.0) as u8,
                common_dp("toneDim", 90.0) as u8,
                common_dp("toneMuted", 130.0) as u8,
                common_dp("toneSubtle", 170.0) as u8,
                common_dp("toneBody", 210.0) as u8,
                common_dp("toneText", 235.0) as u8,
                common_dp("toneFull", 255.0) as u8,
            ],
            control_sizes: [
                number(&["common", "number", "iconButtonGlyphRatio"], 0.44),
                common_dp("sliderTrackHeight", 4.0),
                common_dp("sliderKnobRadius", 6.0),
                common_dp("sliderMarkHalf", 5.0),
            ],
            stroke: [
                common_dp("strokeHairline", 1.0),
                common_dp("strokeIcon", 1.6),
                common_dp("strokeEmphasis", 2.0),
            ],
            shade: [
                common_dp("shadeFaint", 90.0) as u8,
                common_dp("shadeVeil", 150.0) as u8,
                common_dp("shadeScrim", 180.0) as u8,
                common_dp("shadeStrong", 215.0) as u8,
                common_dp("shadeOpaque", 240.0) as u8,
            ],
            overlay_panel_alpha: common_dp("overlayPanelAlpha", 242.0) as u8,
            shadow_alpha: common_dp("shadowAlpha", 120.0) as u8,
            shadow_blur: common_dp("shadowBlur", 32.0) as u8,
            shadow_offset: common_dp("shadowOffset", 8.0) as i8,
            space: [
                common_dp("space2", 2.0),
                common_dp("space4", 4.0),
                common_dp("space6", 6.0),
                common_dp("space8", 8.0),
                common_dp("space10", 10.0),
                common_dp("space12", 12.0),
                common_dp("space14", 14.0),
                common_dp("space16", 16.0),
                common_dp("space20", 20.0),
                common_dp("space24", 24.0),
                common_dp("space32", 32.0),
                common_dp("space48", 48.0),
            ],
            profile_side_margin: 0.0,
            profile_brand_height: platform_dp("profileBrandHeight", 72.0),
            profile_header_gap: platform_dp("profileHeaderGap", 52.0),
            profile_avatar_radius: platform_dp("profileAvatarRadius", 65.0),
            profile_cell_width: platform_dp("profileCellWidth", 150.0),
            profile_gap: platform_dp("profileGap", 40.0),
            profile_action_size: platform_dp("profileActionSize", 28.0),
            profile_title_size: 0.0,
            overlay_margin: 0.0,
            toast_text_size: 0.0,
            toast_padding: 0.0,
            toast_bar_width: 0.0,
            detail_play_height: 0.0,
            detail_back_top: 0.0,
            detail_title_size: 0.0,
            calendar_cell_height: 0.0,
            calendar_panel_width: 0.0,
            settings_page_lead: 0.0,
            section_title_size: 0.0,
            calendar_beside_min_grid: common_dp("calendarBesideMinGrid", 640.0),
            hero_width_cap: 0.0,
            hero_logo_height: 0.0,
            hero_logo_width_cap: 0.0,
            hero_title_scale: 0.0,
            hero_metadata_size: 0.0,
            hero_synopsis_size: 0.0,
            home_hero_content_max_width_tv: common_dp("homeHeroContentMaxWidthTv", 980.0),
            home_hero_logo_height_tv: common_dp("homeHeroLogoHeightTv", 112.0),
            home_hero_logo_max_width_tv: common_dp("homeHeroLogoMaxWidthTv", 380.0),
            home_hero_title_scale_tv: number(&["common", "number", "homeHeroTitleScaleTv"], 2.0),
            home_hero_title_scale_desktop: number(
                &["common", "number", "homeHeroTitleScaleDesktop"],
                1.65,
            ),
            home_hero_metadata_boost_tv: common_sp("homeHeroMetadataBoostTv", 4.0),
            home_hero_synopsis_boost_tv: common_sp("homeHeroSynopsisBoostTv", 6.0),
            settings_label_size: 0.0,
            episodes_cell_gap: 0.0,
            settings_page_lead_mobile: common_dp("settingsPageLeadMobile", 8.0),
            episodes_gap_mobile: common_dp("episodesGapMobile", 8.0),
            row_heading_size: 0.0,
            grid_min_columns: platform_dp("gridMinColumns", 1.0),
            grid_max_gap: platform_dp("gridMaxGap", 0.0),
            settings_page_card_height: platform_dp("settingsPageCardHeight", 72.0),
            settings_subpage_header_offset: platform_dp("settingsSubpageHeaderOffset", 66.0),
            player_sources_hero_height: common_dp("playerSourcesHeroHeight", 180.0),
            player_sources_waiting_line: common_dp("playerSourcesWaitingLine", 30.0),
            player_sources_chip_shift: common_dp("playerSourcesChipShift", 42.0),
            player_sources_chip_shift_range: common_dp("playerSourcesChipShiftRange", 50.0),
            player_sources_strip_fade: common_dp("playerSourcesStripFade", 14.0),
            player_sources_strip_lip: common_dp("playerSourcesStripLip", 10.0),
            player_recommendations_bottom: common_dp("playerRecommendationsBottom", 56.0),
            player_recommendations_close_top: common_dp("playerRecommendationsCloseTop", 36.0),
            player_recommendations_slide: common_dp("playerRecommendationsSlide", 24.0),
            player_recommendations_dot_pitch: common_dp("playerRecommendationsDotPitch", 20.0),
            player_recommendations_dot_hit: common_dp("playerRecommendationsDotHit", 18.0),
            player_recommendations_dot_radius: common_dp("playerRecommendationsDotRadius", 3.5),
            player_recommendations_dot_radius_active: common_dp(
                "playerRecommendationsDotRadiusActive",
                4.5,
            ),
            player_recommendations_dots_gap: common_dp("playerRecommendationsDotsGap", 17.0),
            player_recommendations_dots_bottom: common_dp("playerRecommendationsDotsBottom", 9.0),
            player_recommendations_solo_gap: common_dp("playerRecommendationsSoloGap", 22.0),
            player_recommendations_eyebrow_height: common_dp(
                "playerRecommendationsEyebrowHeight",
                30.0,
            ),
        };
        metrics.row_heading_size = if platform == "tv" {
            metrics.catalog_title_size
        } else {
            metrics.catalog_title_size - 4.0
        };
        let (play_height, back_top, title_size) = match platform {
            "mobile" => (48.0, 12.0, 21.0),
            "tv" => (52.0, 14.0, 26.0),
            _ => (48.0, 14.0, 21.0),
        };
        let (side, title, overlay, toast) = match platform {
            "mobile" => (32.0, 28.0, 16.0, (15.0, 16.0, 84.0)),
            "tv" => (80.0, 38.0, 56.0, (26.0, 26.0, 160.0)),
            _ => (80.0, 38.0, 56.0, (17.0, 20.0, 120.0)),
        };
        metrics.profile_side_margin = side;
        metrics.profile_title_size = title;
        metrics.overlay_margin = overlay;
        metrics.toast_text_size = toast.0;
        metrics.toast_padding = toast.1;
        metrics.toast_bar_width = toast.2;
        (metrics.nav_label_boost, metrics.navigation_icon) = if platform == "tv" {
            (3.0, 25.0)
        } else {
            (0.0, 26.0)
        };
        metrics.detail_play_height = play_height;
        metrics.detail_back_top = back_top;
        metrics.detail_title_size = title_size;
        (
            metrics.settings_page_lead,
            metrics.settings_label_size,
            metrics.episodes_cell_gap,
        ) = if platform == "mobile" {
            (
                metrics.settings_page_lead_mobile,
                metrics.screen_body_size,
                metrics.episodes_gap_mobile,
            )
        } else {
            (
                metrics.settings_appearance_header_height,
                metrics.settings_row_label_size_desktop,
                metrics.episodes_gap,
            )
        };
        metrics.section_title_size = if platform == "tv" {
            metrics.screen_section_title_size_tv
        } else {
            metrics.screen_section_title_size
        };
        if platform == "tv" {
            metrics.hero_width_cap = metrics.home_hero_content_max_width_tv;
            metrics.hero_logo_height = metrics.home_hero_logo_height_tv;
            metrics.hero_logo_width_cap = metrics.home_hero_logo_max_width_tv;
            metrics.hero_title_scale = metrics.home_hero_title_scale_tv;
            metrics.hero_metadata_size =
                metrics.nav_label_size + metrics.home_hero_metadata_boost_tv;
            metrics.hero_synopsis_size =
                metrics.card_title_size + metrics.home_hero_synopsis_boost_tv;
        } else {
            metrics.hero_width_cap = metrics.home_hero_content_max_width_desktop;
            metrics.hero_logo_height = metrics.home_hero_logo_height_desktop;
            metrics.hero_logo_width_cap = metrics.home_hero_logo_max_width_desktop.min(460.0);
            metrics.hero_title_scale = metrics.home_hero_title_scale_desktop;
            metrics.hero_metadata_size = metrics.nav_label_size;
            metrics.hero_synopsis_size = metrics.home_hero_synopsis_size_desktop;
        }
        match platform {
            "mobile" => {
                metrics.content_header_top = metrics.content_header_top_mobile;
                metrics.calendar_grid_gap = metrics.calendar_grid_gap_mobile;
                metrics.calendar_cell_height = metrics.calendar_cell_height_mobile;
            }
            "tv" => {
                metrics.calendar_cell_height = metrics.calendar_cell_height_tv;
                metrics.calendar_panel_width = metrics.calendar_panel_width_tv;
            }
            _ => {
                metrics.calendar_cell_height = metrics.calendar_cell_height_desktop;
                metrics.calendar_panel_width = metrics.calendar_panel_width_desktop;
            }
        }
        let settings_margin = match platform {
            "mobile" => metrics.page_padding,
            "tv" => metrics.screen_padding.max(32.0),
            _ => metrics
                .settings_screen_padding_desktop
                .max(metrics.screen_padding),
        };
        metrics.settings_margin = settings_margin;
        metrics.settings_top = match platform {
            "mobile" => metrics.detail_header_top_mobile,
            "tv" => metrics.content_header_top,
            _ => settings_margin.max(76.0),
        };
        metrics.settings_card_height = if platform == "mobile" {
            metrics.settings_card_height_mobile
        } else {
            metrics.settings_card_height_desktop
        };
        metrics.nav_bar_top = common_dp("navBarTop", 14.0);
        metrics.nav_item_height = common_dp("navItemHeight", 44.0);
        metrics.nav_bar_padding = common_dp("navBarPadding", 6.0);
        metrics.nav_pill_gap = common_dp("navPillGap", 2.0);
        metrics.nav_avatar_radius = common_dp("navAvatarRadius", 15.0);
        metrics.member_avatar_radius = common_dp("memberAvatarRadius", 30.0);
        metrics.member_ring_gap = common_dp("memberRingGap", 5.0);
        metrics.member_row_avatar_radius = common_dp("memberRowAvatarRadius", 20.0);
        metrics.member_row_ring_gap = common_dp("memberRowRingGap", 3.5);
        metrics.member_row_icon = common_dp("memberRowIcon", 16.0);
        metrics.surface_card_preview_logo_height_share = common_dp("surfaceCardPreviewLogoHeightShare", 0.32);
        metrics.surface_card_preview_logo_width_share = common_dp("surfaceCardPreviewLogoWidthShare", 0.5);
        metrics.surface_card_scrim_share = common_dp("surfaceCardScrimShare", 0.4);
        metrics.surface_card_action_icon = common_dp("surfaceCardActionIcon", 20.0);
        metrics.surface_menu_bleed = common_dp("surfaceMenuBleed", 40.0);
        metrics.surface_menu_scroll_inset = common_dp("surfaceMenuScrollInset", 6.0);
        metrics.surface_menu_scroll_min = common_dp("surfaceMenuScrollMin", 24.0);
        metrics.surface_menu_scroll_width = common_dp("surfaceMenuScrollWidth", 3.0);
        metrics.surface_menu_handle_top = common_dp("surfaceMenuHandleTop", 10.0);
        metrics.surface_menu_handle_height = common_dp("surfaceMenuHandleHeight", 4.0);
        metrics.surface_menu_handle_width = common_dp("surfaceMenuHandleWidth", 36.0);
        metrics.surface_menu_min_rows = common_dp("surfaceMenuMinRows", 3.0);
        metrics.surface_menu_split = platform_dp("surfaceMenuSplit", 0.0);
        metrics.surface_menu_panel_width = platform_dp("surfaceMenuPanelWidth", 0.0);
        metrics.surface_menu_height_share = platform_dp("surfaceMenuHeightShare", 1.0);
        metrics.surface_menu_pad_side = platform_dp("surfaceMenuPadSide", 6.0);
        metrics.surface_menu_pad_bottom = platform_dp("surfaceMenuPadBottom", 6.0);
        metrics.surface_menu_pad_top = platform_dp("surfaceMenuPadTop", 6.0);
        metrics.surface_menu_rule_shrink = platform_dp("surfaceMenuRuleShrink", 16.0);
        metrics.surface_menu_header_inset = platform_dp("surfaceMenuHeaderInset", 12.0);
        metrics.surface_menu_row_inset = platform_dp("surfaceMenuRowInset", 12.0);
        metrics.surface_menu_row_radius = platform_dp("surfaceMenuRowRadius", 8.0);
        metrics.surface_menu_icon = platform_dp("surfaceMenuIcon", 18.0);
        metrics.surface_menu_header = platform_dp("surfaceMenuHeader", 0.0);
        metrics.surface_menu_row = platform_dp("surfaceMenuRow", 40.0);
        metrics.surface_ticket_narrow_width = common_dp("surfaceTicketNarrowWidth", 400.0);
        metrics.surface_ticket_seat_overlap = common_dp("surfaceTicketSeatOverlap", 5.0);
        metrics.surface_ticket_seat_radius_narrow =
            common_dp("surfaceTicketSeatRadiusNarrow", 14.0);
        metrics.surface_ticket_seat_radius = common_dp("surfaceTicketSeatRadius", 16.0);
        metrics.surface_ticket_dash_step = common_dp("surfaceTicketDashStep", 13.0);
        metrics.surface_ticket_dash_length = common_dp("surfaceTicketDashLength", 7.0);
        metrics.surface_ticket_dash_inset = common_dp("surfaceTicketDashInset", 10.0);
        metrics.surface_ticket_logo_share = common_dp("surfaceTicketLogoShare", 0.7);
        metrics.surface_ticket_logo_max_height = common_dp("surfaceTicketLogoMaxHeight", 54.0);
        metrics.surface_ticket_field_max_width = common_dp("surfaceTicketFieldMaxWidth", 392.0);
        metrics.surface_ticket_field_height = common_dp("surfaceTicketFieldHeight", 60.0);
        metrics.surface_ticket_stub_label = common_dp("surfaceTicketStubLabel", 24.0);
        metrics.surface_ticket_cell_label = common_dp("surfaceTicketCellLabel", 13.0);
        metrics.surface_ticket_cell_top = common_dp("surfaceTicketCellTop", 14.0);
        metrics.surface_ticket_badge_row = common_dp("surfaceTicketBadgeRow", 22.0);
        metrics.surface_ticket_badge_pad_y = common_dp("surfaceTicketBadgePadY", 3.0);
        metrics.surface_ticket_badge_pad_x = common_dp("surfaceTicketBadgePadX", 7.0);
        metrics.surface_ticket_inline_gap = common_dp("surfaceTicketInlineGap", 9.0);
        metrics.surface_ticket_host_radius = common_dp("surfaceTicketHostRadius", 15.0);
        metrics.surface_ticket_stub_pad_y = common_dp("surfaceTicketStubPadY", 20.0);
        metrics.surface_ticket_pad_y = common_dp("surfaceTicketPadY", 22.0);
        metrics.surface_ticket_pad_x = common_dp("surfaceTicketPadX", 24.0);
        metrics.surface_ticket_details = common_dp("surfaceTicketDetails", 46.0);
        metrics.surface_ticket_stub = common_dp("surfaceTicketStub", 124.0);
        metrics.surface_ticket_notch = common_dp("surfaceTicketNotch", 13.0);
        metrics.surface_ticket_radius = common_dp("surfaceTicketRadius", 26.0);
        metrics.surface_detail_play_max_width = common_dp("surfaceDetailPlayMaxWidth", 320.0);
        metrics.surface_detail_similar_caption = common_dp("surfaceDetailSimilarCaption", 56.0);
        metrics.surface_detail_cast_label_height = common_dp("surfaceDetailCastLabelHeight", 64.0);
        metrics.surface_detail_cast_label_width = common_dp("surfaceDetailCastLabelWidth", 40.0);
        metrics.surface_detail_close_size = common_dp("surfaceDetailCloseSize", 36.0);
        metrics.surface_detail_action_icon = common_dp("surfaceDetailActionIcon", 20.0);
        metrics.surface_detail_glyph = common_dp("surfaceDetailGlyph", 18.0);
        metrics.surface_detail_progress_height = common_dp("surfaceDetailProgressHeight", 4.0);
        metrics.surface_detail_thumb_radius = common_dp("surfaceDetailThumbRadius", 8.0);
        metrics.surface_detail_chip_height = common_dp("surfaceDetailChipHeight", 36.0);
        metrics.surface_detail_logo_height_share = common_dp("surfaceDetailLogoHeightShare", 0.3);
        metrics.surface_detail_logo_width_share = common_dp("surfaceDetailLogoWidthShare", 0.4);
        metrics.surface_detail_image_max_share = common_dp("surfaceDetailImageMaxShare", 0.55);
        metrics.surface_detail_fact_label_share = common_dp("surfaceDetailFactLabelShare", 0.38);
        metrics.surface_detail_text_share = common_dp("surfaceDetailTextShare", 0.62);
        metrics.screen_margin = match platform {
            "mobile" => metrics.page_padding,
            "tv" => metrics.screen_padding.max(32.0),
            _ => metrics.screen_padding,
        };
        let visible = platform_dp("posterVisibleCards", 0.0);
        if visible > 0.0 {
            let gap = metrics.horizontal_spacing;
            let content = viewport.width - metrics.page_padding * 2.0;
            let width = ((content - gap * (visible.ceil() - 1.0)) / visible).clamp(
                platform_dp("posterCardMin", 120.0),
                platform_dp("posterCardMax", 200.0),
            );
            metrics.poster_card_width = width;
            metrics.poster_card_height = width * metrics.poster_height_ratio;
        }
        let continue_visible = platform_dp("continueVisibleCards", 0.0);
        if continue_visible > 0.0 {
            let gap = metrics.horizontal_spacing;
            let content = viewport.width - metrics.page_padding * 2.0;
            let width = ((content - gap * (continue_visible.ceil() - 1.0)) / continue_visible)
                .clamp(
                    platform_dp("continueCardMin", 180.0),
                    platform_dp("continueCardMax", 320.0),
                );
            metrics.home_continue_card_width = width * CONTINUE_CARD_SCALE;
            metrics.home_continue_card_height =
                width * CONTINUE_CARD_SCALE * metrics.horizontal_card_height_ratio;
        }
        if poster_overlay::landscape() {
            metrics.poster_card_width = metrics.home_continue_card_width;
            metrics.poster_card_height = metrics.poster_card_width * 0.5625;
        }
        if ACCENT.load(Ordering::Relaxed) != 0 {
            metrics.accent = accent();
            metrics.accent_foreground =
                if crate::settings::accent_needs_dark_foreground(metrics.accent) {
                    metrics.background
                } else {
                    metrics.text_primary
                };
        }
        publish_palette(&metrics);
        metrics
    }

    pub fn navigation_label_size(self) -> f32 {
        self.nav_label_size + self.nav_label_boost
    }

    pub fn navigation_icon_size(self) -> f32 {
        self.navigation_icon
    }

    pub fn navigation_item_width(self, label: &str) -> f32 {
        let full = 44.0
            + self.navigation_icon_size()
            + 8.0
            + estimated_navigation_text_width(label, self.navigation_label_size());
        let compact = self.nav_item_height.max(self.navigation_icon_size() + 24.0);
        compact + (full - compact) * crate::navigation::top_label_t()
    }

    pub fn navigation_profile_width(self, profile_name: &str) -> f32 {
        let full = (36.0
            + self.nav_avatar_radius * 2.0
            + 8.0
            + estimated_navigation_text_width(profile_name, self.navigation_label_size()))
        .clamp(72.0, 200.0);
        let compact = self.nav_item_height;
        compact + (full - compact) * crate::navigation::top_label_t()
    }

    pub fn navigation_bar_width(self, viewport: Viewport, profile_name: &str) -> f32 {
        let compact = viewport.is_compact();
        let tv = viewport.is_tv();
        let margin = if compact {
            self.page_padding
        } else if tv {
            self.page_padding.max(32.0)
        } else {
            self.page_padding
        };
        let entries = crate::navigation::nav_entries();
        let nav_width = entries
            .iter()
            .map(|(_, label)| self.navigation_item_width(label))
            .sum::<f32>();
        (if compact {
            (viewport.width - margin * 2.0).max(280.0)
        } else {
            nav_width
                + self.nav_pill_gap * (entries.len() + 1) as f32
                + self.navigation_profile_width(profile_name)
        })
        .min((viewport.width - margin * 2.0).max(1.0))
    }
}
