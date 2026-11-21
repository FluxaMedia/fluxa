use super::*;
use std::sync::Arc;
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
    pub form: &'static str,
    pub window_class: &'static str,
    pub viewport_width: f32,
    pub viewport_height: f32,
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
    pub screen_poster_expand_seconds: f32,
    pub screen_poster_expand_aspect: f32,
    pub screen_hero_landscape_switch: f32,
    pub screen_hero_landscape_ratio: f32,
    pub screen_hero_landscape_max: f32,
    pub screen_hero_portrait_ratio: f32,
    pub screen_hero_portrait_max: f32,
    pub screen_hero_tv_height_ratio: f32,
    pub screen_hero_trim_ratio: f32,
    pub screen_hero_pinned_height_ratio: f32,
    pub screen_hero_pinned_min_height: f32,
    pub screen_hero_compact_logo_max_width: f32,
    pub screen_hero_desktop_fade: f32,
    pub screen_hero_width_ratio: f32,
    pub screen_hero_play_compact_width: f32,
    pub screen_hero_play_min_width: f32,
    pub screen_hero_play_card_ratio: f32,
    pub screen_hero_desktop_play_rise: f32,
    pub screen_nav_magnify: f32,
    pub screen_nav_jelly: f32,
    pub screen_nav_lens_span: f32,
    pub screen_nav_stretch: f32,
    pub calendar_panel_width_desktop: f32,
    pub calendar_panel_width_tv: f32,
    pub settings_content_offset: f32,
    pub settings_nav_width: f32,
    pub settings_screen_padding_desktop: f32,
    pub settings_content_max_width_desktop: f32,
    pub settings_row_label_size_desktop: f32,
    pub settings_extended_line_spacing_tv: f32,
    pub button_height: f32,
    pub icon_button_size: f32,
    pub screen_control_height: f32,
    pub screen_control_radius: f32,
    pub labeled_button_icon: f32,
    pub labeled_button_text: f32,
    pub focus_ring_expand: f32,
    pub focus_ring_radius: f32,
    pub focus_ring_width: f32,
    pub focus_ring_slide: f32,
    pub card_content_padding: f32,
    pub settings_card_height_mobile: f32,
    pub settings_card_height_desktop: f32,
    pub settings_row_height: f32,
    pub settings_toggle_width: f32,
    pub settings_toggle_height: f32,
    pub profile_page_inset_mobile: f32,
    pub profile_page_inset_desktop: f32,
    pub settings_swatch_alpha: u8,
    pub settings_swatch_alpha_compact: u8,
    pub poster_sizes: PosterSizes,
    pub home_hero_height: Scaled,
    pub search_width: Scaled,
    pub paint_text_width: Scaled,
    pub paint_text_height: Scaled,
    pub player_panel_wide: Scaled,
    pub player_panel_narrow: Scaled,
    pub home_hero_width: Scaled,
    pub ticket_width: Scaled,
    pub card_popover_min: Scaled,
    pub card_popover_max: Scaled,
    pub input_cell_height: Scaled,
    pub glass_refraction: Scaled,
    pub glass_bevel: Scaled,
    pub sidebar_item_height: f32,
    pub home_hero_bottom_inset: f32,
    pub home_hero_bottom_padding: f32,
    pub episodes_gap: f32,
    pub dropdown_dropdown_chrome: f32,
    pub downloads_change_size: f32,
    pub settings_about_char_share: f32,
    pub settings_about_dot: f32,
    pub settings_keys_share: f32,
    pub settings_preview_radius_share: f32,
    pub settings_preview_text_share: f32,
    pub settings_nav_progress_height: f32,
    pub settings_nav_progress_inset: f32,
    pub settings_nav_progress_radius: f32,
    pub settings_nav_gap_share_tv: f32,
    pub settings_nav_top_share: f32,
    pub settings_nav_max_share: f32,
    pub settings_compact_chevron_width: f32,
    pub settings_compact_chevron_height: f32,
    pub settings_nav_heading_share: f32,
    pub settings_tile_radius_share: f32,
    pub settings_page_title_boost: f32,
    pub settings_value_max_share: f32,
    pub settings_sheet_value_share: f32,
    pub settings_appearance_header_height: f32,
    pub settings_toggle_knob_radius: f32,
    pub calendar_grid_gap_mobile: f32,
    pub calendar_grid_gap: f32,
    pub guide_channel_width_mobile: f32,
    pub guide_channel_width_desktop: f32,
    pub guide_channel_width_tv: f32,
    pub guide_row_height_mobile: f32,
    pub guide_row_height_desktop: f32,
    pub guide_row_height_tv: f32,
    pub guide_hour_width_mobile: f32,
    pub guide_hour_width_desktop: f32,
    pub guide_hour_width_tv: f32,
    pub calendar_cell_height_mobile: f32,
    pub calendar_cell_height_desktop: f32,
    pub calendar_cell_height_tv: f32,
    pub section_gap: f32,
    pub control_gap: f32,
    pub card_radius: f32,
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
    pub control_cue_wrap_ratio: f32,
    pub control_subtitle_wrap_ratio: f32,
    pub control_subtitle_bottom_ratio: f32,
    pub control_hero_scrim_span: f32,
    pub control_skeleton_row_w: f32,
    pub control_skeleton_button_w: f32,
    pub control_skeleton_sub_w: f32,
    pub control_skeleton_title_w: f32,
    pub control_skeleton_top: f32,
    pub control_skeleton_unit_max: f32,
    pub control_skeleton_unit_min: f32,
    pub control_skeleton_unit_base: f32,
    pub control_downloads_speed_gap_ratio: f32,
    pub control_downloads_thumb_scrim: f32,
    pub control_downloads_aspect: f32,
    pub control_downloads_line_ratio: f32,
    pub control_downloads_scrim_ratio: f32,
    pub control_downloads_card_min: f32,
    pub control_downloads_row_gap_ratio: f32,
    pub control_downloads_pad_ratio: f32,
    pub control_downloads_icon_ratio: f32,
    pub control_code_halo: f32,
    pub control_code_edge: f32,
    pub control_send_stroke: f32,
    pub control_search_ring_offset: f32,
    pub control_search_trail_ratio: f32,
    pub control_release_thumb_ratio: f32,
    pub control_toast_height_ratio: f32,
    pub control_pill_progress_radius: f32,
    pub control_step_arm: f32,
    pub control_step_stroke: f32,
    pub control_swatch_hover_grow: f32,
    pub control_row_line_ratio: f32,
    pub control_segment_max_width: f32,
    pub control_chip_height_ratio: f32,
    pub control_profile_hover_scale: f32,
    pub control_profile_reveal_floor: f32,
    pub control_profile_lock_ratio: f32,
    pub surface_card_row_bar_max: f32,
    pub surface_card_row_gap_share: f32,
    pub surface_card_landscape_logo_height_share: f32,
    pub surface_card_landscape_logo_width_share: f32,
    pub surface_card_textless_logo_center: f32,
    pub surface_card_textless_logo_height_share: f32,
    pub surface_card_textless_logo_width_share: f32,
    pub surface_card_logo_scrim_share: f32,
    pub surface_card_badge_inset_share: f32,
    pub surface_card_badge_height_share: f32,
    pub surface_card_badge_pad_share: f32,
    pub surface_card_title_gap_share: f32,
    pub surface_card_bar_gap_share: f32,
    pub surface_card_line_height: f32,
    pub surface_card_continue_scrim_mid: f32,
    pub surface_card_continue_scrim_top: f32,
    pub surface_card_preview_logo_height_share: f32,
    pub surface_card_preview_logo_width_share: f32,
    pub surface_card_scrim_share: f32,
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
    pub surface_ticket_logo_share: f32,
    pub surface_ticket_field_max_width: f32,
    pub surface_ticket_stub: f32,
    pub surface_detail_play_max_width: f32,
    pub surface_detail_logo_height_share: f32,
    pub surface_detail_logo_width_share: f32,
    pub surface_detail_image_max_share: f32,
    pub surface_detail_fact_label_share: f32,
    pub surface_detail_text_share: f32,
    pub panel_sync_sheet_ratio: f32,
    pub panel_reserved_desktop: f32,
    pub panel_drawer_width: f32,
    pub panel_list_sheet_ratio: f32,
    pub panel_list_back_header_scale: f32,
    pub panel_list_head_scale: f32,
    pub panel_list_back_scale: f32,
    pub panel_list_radius_scale: f32,
    pub panel_list_group_lift: f32,
    pub panel_rich_aspect: f32,
    pub panel_sample_ratio: f32,
    pub panel_value_share: f32,
    pub panel_sync_width: f32,
    pub panel_sync_top: f32,
    pub panel_sync_bottom: f32,
    pub panel_sync_back: f32,
    pub panel_sync_back_font: f32,
    pub panel_sync_back_icon: f32,
    pub panel_sync_back_gap: f32,
    pub panel_sync_reset_font: f32,
    pub panel_sync_reset_pad: f32,
    pub panel_sync_reset_height: f32,
    pub panel_sync_head_height: f32,
    pub panel_sync_reading_height: f32,
    pub panel_sync_big_font: f32,
    pub panel_sync_unit_font: f32,
    pub panel_sync_unit_lift: f32,
    pub panel_sync_stepper_width: f32,
    pub panel_sync_stepper_height: f32,
    pub panel_sync_step_font: f32,
    pub panel_sync_status_height: f32,
    pub panel_sync_status_font: f32,
    pub panel_sync_status_drop: f32,
    pub panel_sync_gap: f32,
    pub panel_sync_auto_height: f32,
    pub panel_sync_auto_font: f32,
    pub panel_sync_auto_icon: f32,
    pub panel_sync_auto_gap: f32,
    pub panel_sync_hint_height: f32,
    pub panel_sync_hint_font: f32,
    pub panel_sync_hint_drop: f32,
    pub panel_sync_rule_gap: f32,
    pub panel_sync_method_height: f32,
    pub panel_sync_choice_height: f32,
    pub panel_sync_row_lead: f32,
    pub panel_sync_row_trail: f32,
    pub panel_sync_row_radius: f32,
    pub panel_sync_method_icon: f32,
    pub panel_sync_method_icon_gap: f32,
    pub panel_sync_method_chevron_gap: f32,
    pub panel_sync_label_font: f32,
    pub panel_sync_detail_font: f32,
    pub panel_sync_choice_font: f32,
    pub panel_sync_choice_inset: f32,
    pub panel_sync_check: f32,
    pub panel_sync_check_room: f32,
    pub panel_list_title_font: f32,
    pub panel_list_back_icon: f32,
    pub panel_list_group_font: f32,
    pub panel_list_heading_font: f32,
    pub panel_list_heading_font_sheet: f32,
    pub panel_list_label_font: f32,
    pub panel_list_subline_font: f32,
    pub panel_list_chip_font: f32,
    pub panel_list_chip_height: f32,
    pub panel_list_icon: f32,
    pub panel_list_check: f32,
    pub panel_list_chevron: f32,
    pub panel_rich_detail_font: f32,
    pub panel_rich_detail_rows: usize,
    pub panel_choice_font: f32,
    pub panel_step_font: f32,
    pub panel_step_glyph: f32,
    pub panel_value_font: f32,
    pub panel_badge_font: f32,
    pub panel_sync_edge: f32,
    pub panel_badge: Vec2,
    pub playerctl_loading_logo_width: f32,
    pub playerctl_loading_logo_height: f32,
    pub playerctl_track_boost: f32,
    pub playerctl_skip_min_width: f32,
    pub playerctl_thumb_aspect: f32,
    pub playerctl_next_play_weight: f32,
    pub playerctl_row_cull: f32,
    pub playerctl_modal_max_height: f32,
    pub playerctl_reco_logo_width: f32,
    pub playerctl_reco_right_inset: f32,
    pub playerctl_dismiss_inset: f32,
    pub playerctl_dismiss_top: f32,
    pub playerctl_sources_inset: f32,
    pub playerctl_sources_logo_width: f32,
    pub playerctl_badge_font: f32,
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
    pub touch_handle: f32,
    pub profile_title_size: f32,
    pub overlay_margin: f32,
    pub toast_text_size: f32,
    pub toast_padding: f32,
    pub toast_bar_width: f32,
    pub detail_play_height: f32,
    pub detail_back_top: f32,
    pub detail_title_size: f32,
    pub guide_channel_width: f32,
    pub guide_row_height: f32,
    pub guide_hour_width: f32,
    pub calendar_cell_height: f32,
    pub calendar_panel_width: f32,
    pub settings_page_lead: f32,
    pub section_title_size: f32,
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
static ACCENT: AtomicU32 = AtomicU32::new(u32::MAX);
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
pub(crate) struct MetricsKey {
    epoch: u64,
    width: u32,
    height: u32,
    side_inset: u32,
    tv: bool,
    mobile: bool,
    landscape: bool,
}

static INPUT: std::sync::atomic::AtomicU8 = std::sync::atomic::AtomicU8::new(0);

pub fn set_input(kind: &str) {
    let code = match kind {
        "touch" => 1,
        "pointer" => 2,
        "focus" => 3,
        _ => 0,
    };
    INPUT.store(code, Ordering::Relaxed);
}

pub(crate) fn input() -> &'static str {
    match INPUT.load(Ordering::Relaxed) {
        1 => "touch",
        2 => "pointer",
        3 => "focus",
        _ => match active().form {
            "mobile" => "touch",
            "tv" => "focus",
            _ => "pointer",
        },
    }
}

static ACTIVE: std::sync::RwLock<Option<(MetricsKey, std::sync::Arc<UiMetrics>)>> =
    std::sync::RwLock::new(None);

pub(crate) fn active_key() -> Option<MetricsKey> {
    ACTIVE
        .read()
        .ok()
        .and_then(|slot| slot.as_ref().map(|(key, _)| *key))
}

pub(crate) fn active() -> std::sync::Arc<UiMetrics> {
    if let Some((_, metrics)) = ACTIVE.read().ok().and_then(|slot| slot.clone()) {
        return metrics;
    }
    Arc::new(UiMetrics::for_viewport(Viewport::new(
        1280,
        800,
        UiFormFactor::Desktop,
    )))
}

fn publish_palette(key: MetricsKey, shared: &Arc<UiMetrics>) {
    let metrics: &UiMetrics = shared;
    if let Ok(mut slot) = ACTIVE.write() {
        *slot = Some((key, shared.clone()));
    }
    TEXT_PRIMARY.store(pack(metrics.text_primary), Ordering::Relaxed);
    BACKGROUND.store(pack(metrics.background), Ordering::Relaxed);
    SURFACE.store(pack(metrics.surface), Ordering::Relaxed);
    SURFACE_RAISED.store(pack(metrics.surface_raised), Ordering::Relaxed);
    ACCENT.store(pack(metrics.accent), Ordering::Relaxed);
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

pub fn accent() -> Color32 {
    unpack(&ACCENT)
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
        *Self::shared(viewport)
    }

    pub(crate) fn shared(viewport: Viewport) -> Arc<Self> {
        thread_local! {
            static RECENT: std::cell::RefCell<Vec<(MetricsKey, Arc<UiMetrics>)>> =
                const { std::cell::RefCell::new(Vec::new()) };
        }
        let key = MetricsKey {
            epoch: fluxa_theme::theme::active_epoch(),
            width: viewport.width.to_bits(),
            height: viewport.height.to_bits(),
            side_inset: viewport.side_inset.to_bits(),
            tv: viewport.is_tv(),
            mobile: viewport.form_factor == UiFormFactor::Mobile,
            landscape: poster_overlay::landscape(),
        };
        let metrics = RECENT.with(|recent| {
            let mut recent = recent.borrow_mut();
            if let Some((_, hit)) = recent.iter().find(|(cached, _)| *cached == key) {
                return hit.clone();
            }
            let computed = Arc::new(Self::compute(viewport));
            if recent.len() >= 4 {
                recent.remove(0);
            }
            recent.push((key, computed.clone()));
            computed
        });
        let published = ACTIVE
            .read()
            .ok()
            .is_some_and(|slot| slot.as_ref().is_some_and(|(active, _)| *active == key));
        if !published {
            publish_palette(key, &metrics);
        }
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
            form: match viewport.form_factor {
                UiFormFactor::Mobile => "mobile",
                UiFormFactor::Desktop => "desktop",
                UiFormFactor::Tv => "tv",
            },
            window_class,
            viewport_width: viewport.width,
            viewport_height: viewport.height,
            background: color("background", Color32::from_rgb(7, 7, 9)),
            surface: color("surface", Color32::from_rgb(20, 20, 22)),
            surface_raised: color("surfaceRaised", Color32::from_rgb(28, 28, 31)),
            border: color("border", Color32::from_white_alpha(20)),
            danger: color("error", Color32::from_rgb(255, 107, 107)),
            navigation: color("navigation", Color32::from_rgb(20, 20, 22)),
            text_primary: color("textPrimary", Color32::WHITE),
            text_secondary: color("textSecondary", Color32::from_rgb(184, 184, 190)),
            text_muted: color("textMuted", Color32::from_rgb(138, 138, 145)),
            accent: color("accent", Color32::WHITE),
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
            screen_poster_expand_seconds: common_dp("screenPosterExpandSeconds", 0.25),
            screen_poster_expand_aspect: common_dp("screenPosterExpandAspect", 1.7777778),
            screen_hero_landscape_switch: common_dp("screenHeroLandscapeSwitch", 1.15),
            screen_hero_landscape_ratio: common_dp("screenHeroLandscapeRatio", 0.4166667),
            screen_hero_landscape_max: common_dp("screenHeroLandscapeMax", 440.0),
            screen_hero_portrait_ratio: common_dp("screenHeroPortraitRatio", 1.3333334),
            screen_hero_portrait_max: common_dp("screenHeroPortraitMax", 560.0),
            screen_hero_tv_height_ratio: common_dp("screenHeroTvHeightRatio", 0.7),
            screen_hero_trim_ratio: common_dp("screenHeroTrimRatio", 0.93),
            screen_hero_pinned_height_ratio: common_dp("screenHeroPinnedHeightRatio", 0.55),
            screen_hero_pinned_min_height: common_dp("screenHeroPinnedMinHeight", 300.0),
            screen_hero_compact_logo_max_width: common_dp("screenHeroCompactLogoMaxWidth", 220.0),
            screen_hero_desktop_fade: common_dp("screenHeroDesktopFade", 176.0),
            screen_hero_width_ratio: common_dp("screenHeroWidthRatio", 0.58),
            screen_hero_play_compact_width: common_dp("screenHeroPlayCompactWidth", 108.0),
            screen_hero_play_min_width: common_dp("screenHeroPlayMinWidth", 160.0),
            screen_hero_play_card_ratio: common_dp("screenHeroPlayCardRatio", 0.46),
            screen_hero_desktop_play_rise: common_dp("screenHeroDesktopPlayRise", 194.0),
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
            settings_row_label_size_desktop: common_sp("settingsRowLabelSizeDesktop", 18.0),
            settings_extended_line_spacing_tv: common_dp("settingsExtendedLineSpacingTv", 42.0),
            button_height: platform_dp("buttonHeight", 44.0),
            icon_button_size: platform_dp("iconButtonSize", 40.0),
            screen_control_height: common_dp("screenControlHeight", 36.0),
            screen_control_radius: common_dp("screenControlRadius", 9.0),
            labeled_button_icon: common_dp("labeledButtonIconSize", 22.0),
            labeled_button_text: common_sp("labeledButtonTextSize", 12.0),
            focus_ring_expand: common_dp("focusRingExpand", 3.0),
            focus_ring_radius: common_dp("focusRingRadius", 10.0),
            focus_ring_width: common_dp("focusRingWidth", 2.5),
            focus_ring_slide: platform_dp("focusRingSlideMs", 0.0) / 1000.0,
            card_content_padding: common_dp("cardContentPadding", 10.0),
            settings_card_height_mobile: common_dp("settingsCardHeightMobile", 168.0),
            settings_card_height_desktop: common_dp("settingsCardHeightDesktop", 154.0),
            settings_row_height: pick(
                common_dp("settingsRowHeightDesktop", 44.0),
                common_dp("settingsRowHeight", 32.0),
            ),
            settings_toggle_width: pick(
                common_dp("settingsToggleWidthDesktop", 48.0),
                common_dp("settingsToggleWidth", 40.0),
            ),
            settings_appearance_header_height: common_dp("settingsAppearanceHeaderHeight", 66.0),
            downloads_change_size: common_dp("downloadsChangeSize", 52.0),
            dropdown_dropdown_chrome: common_dp("dropdownDropdownChrome", 44.0),
            episodes_gap: common_dp("episodesGap", 16.0),
            sidebar_item_height: platform_dp("sidebarItemHeight", 44.0),
            home_hero_bottom_inset: platform_dp("homeHeroBottomInset", 194.0),
            home_hero_bottom_padding: platform_dp("homeHeroBottomPadding", 48.0),
            search_width: platform_scaled("searchWidth", 0.36, 220.0, 420.0),
            paint_text_width: platform_scaled("paintSizeA", 0.022, 15.0, 26.0),
            paint_text_height: platform_scaled("paintSizeB", 0.052, 18.0, 60.0),
            player_panel_wide: platform_scaled("playerPanelA", 0.32, 380.0, 560.0),
            player_panel_narrow: platform_scaled("playerPanelB", 0.24, 260.0, 340.0),
            home_hero_width: platform_scaled("homeHeroWidth", 0.55, 420.0, 720.0),
            ticket_width: platform_scaled("ticketWidth", 0.62, 236.0, 272.0),
            card_popover_min: platform_scaled("cardPopoverMin", 1.5, 340.0, 420.0),
            card_popover_max: platform_scaled("cardPopoverMax", 2.4, 400.0, 500.0),
            input_cell_height: platform_scaled("inputCellHeight", 1.2, 42.0, 52.0),
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
            profile_page_inset_mobile: common_dp("profilePageInsetMobile", 32.0),
            profile_page_inset_desktop: common_dp("profilePageInsetDesktop", 56.0),
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
            guide_channel_width_mobile: common_dp("guideChannelWidthMobile", 96.0),
            guide_channel_width_desktop: common_dp("guideChannelWidthDesktop", 150.0),
            guide_channel_width_tv: common_dp("guideChannelWidthTv", 200.0),
            guide_row_height_mobile: common_dp("guideRowHeightMobile", 64.0),
            guide_row_height_desktop: common_dp("guideRowHeightDesktop", 72.0),
            guide_row_height_tv: common_dp("guideRowHeightTv", 96.0),
            guide_hour_width_mobile: common_dp("guideHourWidthMobile", 180.0),
            guide_hour_width_desktop: common_dp("guideHourWidthDesktop", 240.0),
            guide_hour_width_tv: common_dp("guideHourWidthTv", 320.0),
            calendar_cell_height_mobile: common_dp("calendarCellHeightMobile", 58.0),
            calendar_cell_height_desktop: common_dp("calendarCellHeightDesktop", 96.0),
            calendar_cell_height_tv: common_dp("calendarCellHeightTv", 112.0),
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
            settings_about_char_share: 0.0,
            settings_about_dot: 0.0,
            settings_keys_share: 0.0,
            settings_preview_radius_share: 0.0,
            settings_preview_text_share: 0.0,
            settings_nav_progress_height: 0.0,
            settings_nav_progress_inset: 0.0,
            settings_nav_progress_radius: 0.0,
            settings_nav_gap_share_tv: 0.0,
            settings_nav_top_share: 0.0,
            settings_nav_max_share: 0.0,
            settings_compact_chevron_width: 0.0,
            settings_compact_chevron_height: 0.0,
            settings_nav_heading_share: 0.0,
            settings_tile_radius_share: 0.0,
            settings_page_title_boost: 0.0,
            settings_value_max_share: 0.0,
            settings_sheet_value_share: 0.0,
            control_cue_wrap_ratio: 0.0,
            control_subtitle_wrap_ratio: 0.0,
            control_subtitle_bottom_ratio: 0.0,
            control_hero_scrim_span: 0.0,
            control_skeleton_row_w: 0.0,
            control_skeleton_button_w: 0.0,
            control_skeleton_sub_w: 0.0,
            control_skeleton_title_w: 0.0,
            control_skeleton_top: 0.0,
            control_skeleton_unit_max: 0.0,
            control_skeleton_unit_min: 0.0,
            control_skeleton_unit_base: 0.0,
            control_downloads_speed_gap_ratio: 0.0,
            control_downloads_thumb_scrim: 0.0,
            control_downloads_aspect: 0.0,
            control_downloads_line_ratio: 0.0,
            control_downloads_scrim_ratio: 0.0,
            control_downloads_card_min: 0.0,
            control_downloads_row_gap_ratio: 0.0,
            control_downloads_pad_ratio: 0.0,
            control_downloads_icon_ratio: 0.0,
            control_code_halo: 0.0,
            control_code_edge: 0.0,
            control_send_stroke: 0.0,
            control_search_ring_offset: 0.0,
            control_search_trail_ratio: 0.0,
            control_release_thumb_ratio: 0.0,
            control_toast_height_ratio: 0.0,
            control_pill_progress_radius: 0.0,
            control_step_arm: 0.0,
            control_step_stroke: 0.0,
            control_swatch_hover_grow: 0.0,
            control_row_line_ratio: 0.0,
            control_segment_max_width: 0.0,
            control_chip_height_ratio: 0.0,
            control_profile_hover_scale: 0.0,
            control_profile_reveal_floor: 0.0,
            control_profile_lock_ratio: 0.0,
            surface_card_row_bar_max: 0.0,
            surface_card_row_gap_share: 0.0,
            surface_card_landscape_logo_height_share: 0.0,
            surface_card_landscape_logo_width_share: 0.0,
            surface_card_textless_logo_center: 0.0,
            surface_card_textless_logo_height_share: 0.0,
            surface_card_textless_logo_width_share: 0.0,
            surface_card_logo_scrim_share: 0.0,
            surface_card_badge_inset_share: 0.0,
            surface_card_badge_height_share: 0.0,
            surface_card_badge_pad_share: 0.0,
            surface_card_title_gap_share: 0.0,
            surface_card_bar_gap_share: 0.0,
            surface_card_line_height: 0.0,
            surface_card_continue_scrim_mid: 0.0,
            surface_card_continue_scrim_top: 0.0,
            surface_card_preview_logo_height_share: 0.0,
            surface_card_preview_logo_width_share: 0.0,
            surface_card_scrim_share: 0.0,
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
            surface_ticket_logo_share: 0.0,
            surface_ticket_field_max_width: 0.0,
            surface_ticket_stub: 0.0,
            surface_detail_play_max_width: 0.0,
            surface_detail_logo_height_share: 0.0,
            surface_detail_logo_width_share: 0.0,
            surface_detail_image_max_share: 0.0,
            surface_detail_fact_label_share: 0.0,
            surface_detail_text_share: 0.0,
            panel_sync_sheet_ratio: 0.0,
            panel_reserved_desktop: 0.0,
            panel_drawer_width: 0.0,
            panel_list_sheet_ratio: 0.0,
            panel_list_back_header_scale: 0.0,
            panel_list_head_scale: 0.0,
            panel_list_back_scale: 0.0,
            panel_list_radius_scale: 0.0,
            panel_list_group_lift: 0.0,
            panel_rich_aspect: 0.0,
            panel_sample_ratio: 0.0,
            panel_value_share: 0.0,
            panel_sync_width: 0.0,
            panel_sync_top: 0.0,
            panel_sync_bottom: 0.0,
            panel_sync_back: 0.0,
            panel_sync_back_font: 0.0,
            panel_sync_back_icon: 0.0,
            panel_sync_back_gap: 0.0,
            panel_sync_reset_font: 0.0,
            panel_sync_reset_pad: 0.0,
            panel_sync_reset_height: 0.0,
            panel_sync_head_height: 0.0,
            panel_sync_reading_height: 0.0,
            panel_sync_big_font: 0.0,
            panel_sync_unit_font: 0.0,
            panel_sync_unit_lift: 0.0,
            panel_sync_stepper_width: 0.0,
            panel_sync_stepper_height: 0.0,
            panel_sync_step_font: 0.0,
            panel_sync_status_height: 0.0,
            panel_sync_status_font: 0.0,
            panel_sync_status_drop: 0.0,
            panel_sync_gap: 0.0,
            panel_sync_auto_height: 0.0,
            panel_sync_auto_font: 0.0,
            panel_sync_auto_icon: 0.0,
            panel_sync_auto_gap: 0.0,
            panel_sync_hint_height: 0.0,
            panel_sync_hint_font: 0.0,
            panel_sync_hint_drop: 0.0,
            panel_sync_rule_gap: 0.0,
            panel_sync_method_height: 0.0,
            panel_sync_choice_height: 0.0,
            panel_sync_row_lead: 0.0,
            panel_sync_row_trail: 0.0,
            panel_sync_row_radius: 0.0,
            panel_sync_method_icon: 0.0,
            panel_sync_method_icon_gap: 0.0,
            panel_sync_method_chevron_gap: 0.0,
            panel_sync_label_font: 0.0,
            panel_sync_detail_font: 0.0,
            panel_sync_choice_font: 0.0,
            panel_sync_choice_inset: 0.0,
            panel_sync_check: 0.0,
            panel_sync_check_room: 0.0,
            panel_list_title_font: 0.0,
            panel_list_back_icon: 0.0,
            panel_list_group_font: 0.0,
            panel_list_heading_font: 0.0,
            panel_list_heading_font_sheet: 0.0,
            panel_list_label_font: 0.0,
            panel_list_subline_font: 0.0,
            panel_list_chip_font: 0.0,
            panel_list_chip_height: 0.0,
            panel_list_icon: 0.0,
            panel_list_check: 0.0,
            panel_list_chevron: 0.0,
            panel_rich_detail_font: 0.0,
            panel_rich_detail_rows: 0,
            panel_choice_font: 0.0,
            panel_step_font: 0.0,
            panel_step_glyph: 0.0,
            panel_value_font: 0.0,
            panel_badge_font: 0.0,
            panel_sync_edge: 0.0,
            panel_badge: Vec2::ZERO,
            playerctl_loading_logo_width: 0.0,
            playerctl_loading_logo_height: 0.0,
            playerctl_track_boost: 0.0,
            playerctl_skip_min_width: 0.0,
            playerctl_thumb_aspect: 0.0,
            playerctl_next_play_weight: 0.0,
            playerctl_row_cull: 0.0,
            playerctl_modal_max_height: 0.0,
            playerctl_reco_logo_width: 0.0,
            playerctl_reco_right_inset: 0.0,
            playerctl_dismiss_inset: 0.0,
            playerctl_dismiss_top: 0.0,
            playerctl_sources_inset: 0.0,
            playerctl_sources_logo_width: 0.0,
            playerctl_badge_font: 0.0,
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
            touch_handle: platform_dp("touchHandle", 0.0),
            profile_title_size: 0.0,
            screen_nav_magnify: 0.0,
            screen_nav_jelly: 0.0,
            screen_nav_lens_span: 0.0,
            screen_nav_stretch: 0.0,
            overlay_margin: 0.0,
            toast_text_size: 0.0,
            toast_padding: 0.0,
            toast_bar_width: 0.0,
            detail_play_height: 0.0,
            detail_back_top: 0.0,
            detail_title_size: 0.0,
            guide_channel_width: 0.0,
            guide_row_height: 0.0,
            guide_hour_width: 0.0,
            calendar_cell_height: 0.0,
            calendar_panel_width: 0.0,
            settings_page_lead: 0.0,
            section_title_size: 0.0,
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
                metrics.guide_channel_width = metrics.guide_channel_width_mobile;
                metrics.guide_row_height = metrics.guide_row_height_mobile;
                metrics.guide_hour_width = metrics.guide_hour_width_mobile;
            }
            "tv" => {
                metrics.calendar_cell_height = metrics.calendar_cell_height_tv;
                metrics.guide_channel_width = metrics.guide_channel_width_tv;
                metrics.guide_row_height = metrics.guide_row_height_tv;
                metrics.guide_hour_width = metrics.guide_hour_width_tv;
                metrics.calendar_panel_width = metrics.calendar_panel_width_tv;
            }
            _ => {
                metrics.calendar_cell_height = metrics.calendar_cell_height_desktop;
                metrics.guide_channel_width = metrics.guide_channel_width_desktop;
                metrics.guide_row_height = metrics.guide_row_height_desktop;
                metrics.guide_hour_width = metrics.guide_hour_width_desktop;
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
        metrics.screen_nav_magnify = common_dp("screenNavMagnify", 0.18);
        metrics.screen_nav_jelly = common_dp("screenNavJelly", 0.1);
        metrics.screen_nav_lens_span = common_dp("screenNavLensSpan", 0.9);
        metrics.screen_nav_stretch = common_dp("screenNavStretch", 0.6);
        metrics.member_avatar_radius = common_dp("memberAvatarRadius", 30.0);
        metrics.member_ring_gap = common_dp("memberRingGap", 5.0);
        metrics.member_row_avatar_radius = common_dp("memberRowAvatarRadius", 20.0);
        metrics.member_row_ring_gap = common_dp("memberRowRingGap", 3.5);
        metrics.member_row_icon = common_dp("memberRowIcon", 16.0);
        metrics.settings_about_char_share = common_dp("settingsAboutCharShare", 0.58);
        metrics.settings_about_dot = common_dp("settingsAboutDot", 3.5);
        metrics.settings_keys_share = common_dp("settingsKeysShare", 0.9);
        metrics.settings_preview_radius_share = common_dp("settingsPreviewRadiusShare", 0.6);
        metrics.settings_preview_text_share = common_dp("settingsPreviewTextShare", 0.17);
        metrics.settings_nav_progress_height = common_dp("settingsNavProgressHeight", 2.5);
        metrics.settings_nav_progress_inset = common_dp("settingsNavProgressInset", 1.5);
        metrics.settings_nav_progress_radius = common_dp("settingsNavProgressRadius", 1.25);
        metrics.settings_nav_gap_share_tv = common_dp("settingsNavGapShareTv", 0.75);
        metrics.settings_nav_top_share = common_dp("settingsNavTopShare", 1.5);
        metrics.settings_nav_max_share = common_dp("settingsNavMaxShare", 0.32);
        metrics.settings_compact_chevron_width = common_dp("settingsCompactChevronWidth", 4.5);
        metrics.settings_compact_chevron_height = common_dp("settingsCompactChevronHeight", 5.5);
        metrics.settings_nav_heading_share = common_dp("settingsNavHeadingShare", 0.6);
        metrics.settings_tile_radius_share = common_dp("settingsTileRadiusShare", 0.3);
        metrics.settings_page_title_boost = platform_dp("settingsPageTitleBoost", 3.0);
        metrics.settings_value_max_share = common_dp("settingsValueMaxShare", 0.5);
        metrics.settings_sheet_value_share = common_dp("settingsSheetValueShare", 0.4);
        metrics.control_cue_wrap_ratio = common_dp("controlCueWrapRatio", 0.8);
        metrics.control_subtitle_wrap_ratio = common_dp("controlSubtitleWrapRatio", 0.6);
        metrics.control_subtitle_bottom_ratio = common_dp("controlSubtitleBottomRatio", 0.1);
        metrics.control_hero_scrim_span = common_dp("controlHeroScrimSpan", 0.7);
        metrics.control_skeleton_row_w = common_dp("controlSkeletonRowW", 180.0);
        metrics.control_skeleton_button_w = common_dp("controlSkeletonButtonW", 120.0);
        metrics.control_skeleton_sub_w = common_dp("controlSkeletonSubW", 200.0);
        metrics.control_skeleton_title_w = common_dp("controlSkeletonTitleW", 280.0);
        metrics.control_skeleton_top = common_dp("controlSkeletonTop", 0.2);
        metrics.control_skeleton_unit_max = common_dp("controlSkeletonUnitMax", 1.6);
        metrics.control_skeleton_unit_min = common_dp("controlSkeletonUnitMin", 0.7);
        metrics.control_skeleton_unit_base = common_dp("controlSkeletonUnitBase", 900.0);
        metrics.control_downloads_speed_gap_ratio = common_dp("controlDownloadsSpeedGapRatio", 1.5);
        metrics.control_downloads_thumb_scrim = common_dp("controlDownloadsThumbScrim", 0.5);
        metrics.control_downloads_aspect = common_dp("controlDownloadsAspect", 0.6);
        metrics.control_downloads_line_ratio = common_dp("controlDownloadsLineRatio", 1.2);
        metrics.control_downloads_scrim_ratio = common_dp("controlDownloadsScrimRatio", 0.3);
        metrics.control_downloads_card_min = common_dp("controlDownloadsCardMin", 300.0);
        metrics.control_downloads_row_gap_ratio = common_dp("controlDownloadsRowGapRatio", 1.5);
        metrics.control_downloads_pad_ratio = common_dp("controlDownloadsPadRatio", 1.2);
        metrics.control_downloads_icon_ratio = common_dp("controlDownloadsIconRatio", 0.4);
        metrics.control_code_halo = common_dp("controlCodeHalo", 2.5);
        metrics.control_code_edge = common_dp("controlCodeEdge", 1.5);
        metrics.control_send_stroke = common_dp("controlSendStroke", 2.2);
        metrics.control_search_ring_offset = common_dp("controlSearchRingOffset", 1.5);
        metrics.control_search_trail_ratio = common_dp("controlSearchTrailRatio", 0.6);
        metrics.control_release_thumb_ratio = common_dp("controlReleaseThumbRatio", 0.7);
        metrics.control_toast_height_ratio = common_dp("controlToastHeightRatio", 2.6);
        metrics.control_pill_progress_radius = common_dp("controlPillProgressRadius", 1.5);
        metrics.control_step_arm = common_dp("controlStepArm", 0.2);
        metrics.control_step_stroke = common_dp("controlStepStroke", 1.6);
        metrics.control_swatch_hover_grow = common_dp("controlSwatchHoverGrow", 0.1);
        metrics.control_row_line_ratio = common_dp("controlRowLineRatio", 1.3);
        metrics.control_segment_max_width = common_dp("controlSegmentMaxWidth", 120.0);
        metrics.control_chip_height_ratio = common_dp("controlChipHeightRatio", 0.8);
        metrics.control_profile_hover_scale = common_dp("controlProfileHoverScale", 1.0);
        metrics.control_profile_reveal_floor = common_dp("controlProfileRevealFloor", 0.9);
        metrics.control_profile_lock_ratio = common_dp("controlProfileLockRatio", 0.7);
        metrics.surface_card_row_bar_max = common_dp("surfaceCardRowBarMax", 220.0);
        metrics.surface_card_row_gap_share = common_dp("surfaceCardRowGapShare", 1.5);
        metrics.surface_card_landscape_logo_height_share =
            common_dp("surfaceCardLandscapeLogoHeightShare", 0.3);
        metrics.surface_card_landscape_logo_width_share =
            common_dp("surfaceCardLandscapeLogoWidthShare", 0.5);
        metrics.surface_card_textless_logo_center =
            common_dp("surfaceCardTextlessLogoCenter", 0.27);
        metrics.surface_card_textless_logo_height_share =
            common_dp("surfaceCardTextlessLogoHeightShare", 0.2);
        metrics.surface_card_textless_logo_width_share =
            common_dp("surfaceCardTextlessLogoWidthShare", 0.76);
        metrics.surface_card_logo_scrim_share = common_dp("surfaceCardLogoScrimShare", 0.5);
        metrics.surface_card_badge_inset_share = common_dp("surfaceCardBadgeInsetShare", 0.06);
        metrics.surface_card_badge_height_share = common_dp("surfaceCardBadgeHeightShare", 1.4);
        metrics.surface_card_badge_pad_share = common_dp("surfaceCardBadgePadShare", 0.45);
        metrics.surface_card_title_gap_share = common_dp("surfaceCardTitleGapShare", 0.35);
        metrics.surface_card_bar_gap_share = common_dp("surfaceCardBarGapShare", 0.5);
        metrics.surface_card_line_height = common_dp("surfaceCardLineHeight", 1.25);
        metrics.surface_card_continue_scrim_mid = common_dp("surfaceCardContinueScrimMid", 0.5);
        metrics.surface_card_continue_scrim_top = common_dp("surfaceCardContinueScrimTop", 0.2);
        metrics.surface_card_preview_logo_height_share =
            common_dp("surfaceCardPreviewLogoHeightShare", 0.32);
        metrics.surface_card_preview_logo_width_share =
            common_dp("surfaceCardPreviewLogoWidthShare", 0.5);
        metrics.surface_card_scrim_share = common_dp("surfaceCardScrimShare", 0.4);
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
        metrics.surface_ticket_logo_share = common_dp("surfaceTicketLogoShare", 0.7);
        metrics.surface_ticket_field_max_width = common_dp("surfaceTicketFieldMaxWidth", 392.0);
        metrics.surface_ticket_stub = common_dp("surfaceTicketStub", 124.0);
        metrics.surface_detail_play_max_width = common_dp("surfaceDetailPlayMaxWidth", 320.0);
        metrics.surface_detail_logo_height_share = common_dp("surfaceDetailLogoHeightShare", 0.3);
        metrics.surface_detail_logo_width_share = common_dp("surfaceDetailLogoWidthShare", 0.4);
        metrics.surface_detail_image_max_share = common_dp("surfaceDetailImageMaxShare", 0.55);
        metrics.surface_detail_fact_label_share = common_dp("surfaceDetailFactLabelShare", 0.38);
        metrics.surface_detail_text_share = common_dp("surfaceDetailTextShare", 0.62);
        metrics.panel_sync_sheet_ratio = common_dp("panelSyncSheetRatio", 0.88);
        metrics.panel_reserved_desktop = common_dp("panelReservedDesktop", 120.0);
        metrics.panel_drawer_width = common_dp("panelDrawerWidth", 300.0);
        metrics.panel_list_sheet_ratio = common_dp("panelListSheetRatio", 0.78);
        metrics.panel_list_back_header_scale = common_dp("panelListBackHeaderScale", 1.25);
        metrics.panel_list_head_scale = common_dp("panelListHeadScale", 0.8);
        metrics.panel_list_back_scale = common_dp("panelListBackScale", 0.85);
        metrics.panel_list_radius_scale = common_dp("panelListRadiusScale", 0.25);
        metrics.panel_list_group_lift = common_dp("panelListGroupLift", 0.24);
        metrics.panel_rich_aspect = common_dp("panelRichAspect", 1.7778);
        metrics.panel_sample_ratio = common_dp("panelSampleRatio", 0.05);
        metrics.panel_value_share = common_dp("panelValueShare", 0.4);
        metrics.panel_sync_width = platform_dp("panelSyncWidth", 485.6);
        metrics.panel_sync_top = platform_dp("panelSyncTop", 11.04);
        metrics.panel_sync_bottom = platform_dp("panelSyncBottom", 25.76);
        metrics.panel_sync_back = platform_dp("panelSyncBack", 38.4);
        metrics.panel_sync_back_font = platform_dp("panelSyncBackFont", 18.4);
        metrics.panel_sync_back_icon = platform_dp("panelSyncBackIcon", 21.6);
        metrics.panel_sync_back_gap = platform_dp("panelSyncBackGap", 11.2);
        metrics.panel_sync_reset_font = platform_dp("panelSyncResetFont", 15.2);
        metrics.panel_sync_reset_pad = platform_dp("panelSyncResetPad", 19.2);
        metrics.panel_sync_reset_height = platform_dp("panelSyncResetHeight", 35.2);
        metrics.panel_sync_head_height = platform_dp("panelSyncHeadHeight", 48.0);
        metrics.panel_sync_reading_height = platform_dp("panelSyncReadingHeight", 57.6);
        metrics.panel_sync_big_font = platform_dp("panelSyncBigFont", 45.6);
        metrics.panel_sync_unit_font = platform_dp("panelSyncUnitFont", 20.8);
        metrics.panel_sync_unit_lift = platform_dp("panelSyncUnitLift", 17.6);
        metrics.panel_sync_stepper_width = platform_dp("panelSyncStepperWidth", 137.6);
        metrics.panel_sync_stepper_height = platform_dp("panelSyncStepperHeight", 41.6);
        metrics.panel_sync_step_font = platform_dp("panelSyncStepFont", 14.88);
        metrics.panel_sync_status_height = platform_dp("panelSyncStatusHeight", 41.6);
        metrics.panel_sync_status_font = platform_dp("panelSyncStatusFont", 15.2);
        metrics.panel_sync_status_drop = platform_dp("panelSyncStatusDrop", 3.2);
        metrics.panel_sync_gap = platform_dp("panelSyncGap", 17.6);
        metrics.panel_sync_auto_height = platform_dp("panelSyncAutoHeight", 52.8);
        metrics.panel_sync_auto_font = platform_dp("panelSyncAutoFont", 16.8);
        metrics.panel_sync_auto_icon = platform_dp("panelSyncAutoIcon", 22.4);
        metrics.panel_sync_auto_gap = platform_dp("panelSyncAutoGap", 11.2);
        metrics.panel_sync_hint_height = platform_dp("panelSyncHintHeight", 40.0);
        metrics.panel_sync_hint_font = platform_dp("panelSyncHintFont", 14.08);
        metrics.panel_sync_hint_drop = platform_dp("panelSyncHintDrop", 8.8);
        metrics.panel_sync_rule_gap = platform_dp("panelSyncRuleGap", 8.8);
        metrics.panel_sync_method_height = platform_dp("panelSyncMethodHeight", 60.8);
        metrics.panel_sync_choice_height = platform_dp("panelSyncChoiceHeight", 43.2);
        metrics.panel_sync_row_lead = platform_dp("panelSyncRowLead", 8.0);
        metrics.panel_sync_row_trail = platform_dp("panelSyncRowTrail", 11.2);
        metrics.panel_sync_row_radius = platform_dp("panelSyncRowRadius", 14.4);
        metrics.panel_sync_method_icon = platform_dp("panelSyncMethodIcon", 22.4);
        metrics.panel_sync_method_icon_gap = platform_dp("panelSyncMethodIconGap", 17.6);
        metrics.panel_sync_method_chevron_gap = platform_dp("panelSyncMethodChevronGap", 25.6);
        metrics.panel_sync_label_font = platform_dp("panelSyncLabelFont", 16.48);
        metrics.panel_sync_detail_font = platform_dp("panelSyncDetailFont", 14.08);
        metrics.panel_sync_choice_font = platform_dp("panelSyncChoiceFont", 15.68);
        metrics.panel_sync_choice_inset = platform_dp("panelSyncChoiceInset", 48.0);
        metrics.panel_sync_check = platform_dp("panelSyncCheck", 20.0);
        metrics.panel_sync_check_room = platform_dp("panelSyncCheckRoom", 32.0);
        metrics.panel_list_title_font = platform_dp("panelListTitleFont", 17.6);
        metrics.panel_list_back_icon = platform_dp("panelListBackIcon", 22.4);
        metrics.panel_list_group_font = platform_dp("panelListGroupFont", 12.8);
        metrics.panel_list_heading_font = platform_dp("panelListHeadingFont", 13.6);
        metrics.panel_list_heading_font_sheet = platform_dp("panelListHeadingFontSheet", 12.48);
        metrics.panel_list_label_font = platform_dp("panelListLabelFont", 17.28);
        metrics.panel_list_subline_font = platform_dp("panelListSublineFont", 12.48);
        metrics.panel_list_chip_font = platform_dp("panelListChipFont", 12.8);
        metrics.panel_list_chip_height = platform_dp("panelListChipHeight", 19.2);
        metrics.panel_list_icon = platform_dp("panelListIcon", 22.4);
        metrics.panel_list_check = platform_dp("panelListCheck", 19.2);
        metrics.panel_list_chevron = platform_dp("panelListChevron", 17.6);
        metrics.panel_rich_detail_font = platform_dp("panelRichDetailFont", 13.6);
        metrics.panel_rich_detail_rows = platform_dp("panelRichDetailRows", 3.0) as usize;
        metrics.panel_choice_font = platform_dp("panelChoiceFont", 14.4);
        metrics.panel_step_font = platform_dp("panelStepFont", 15.2);
        metrics.panel_step_glyph = platform_dp("panelStepGlyph", 19.2);
        metrics.panel_value_font = platform_dp("panelValueFont", 14.4);
        metrics.panel_badge_font = platform_dp("panelBadgeFont", 9.92);
        metrics.panel_sync_edge = platform_dp("panelSyncEdge", 12.4);
        metrics.panel_badge = Vec2::new(
            common_dp("panelBadgeWidth", 24.0),
            common_dp("panelBadgeHeight", 16.0),
        );
        metrics.playerctl_loading_logo_width = common_dp("playerctlLoadingLogoWidth", 480.0);
        metrics.playerctl_loading_logo_height = common_dp("playerctlLoadingLogoHeight", 160.0);
        metrics.playerctl_track_boost = common_dp("playerctlTrackBoost", 0.5);
        metrics.playerctl_skip_min_width = common_dp("playerctlSkipMinWidth", 150.0);
        metrics.playerctl_thumb_aspect = common_dp("playerctlThumbAspect", 0.5625);
        metrics.playerctl_next_play_weight = common_dp("playerctlNextPlayWeight", 1.63);
        metrics.playerctl_row_cull = common_dp("playerctlRowCull", 240.0);
        metrics.playerctl_modal_max_height = common_dp("playerctlModalMaxHeight", 900.0);
        metrics.playerctl_reco_logo_width = platform_dp("playerctlRecoLogoWidth", 314.0);
        metrics.playerctl_reco_right_inset = platform_dp("playerctlRecoRightInset", 120.0);
        metrics.playerctl_dismiss_inset = platform_dp("playerctlDismissInset", 28.0);
        metrics.playerctl_dismiss_top = platform_dp("playerctlDismissTop", 16.0);
        metrics.playerctl_sources_inset = platform_dp("playerctlSourcesInset", 24.0);
        metrics.playerctl_sources_logo_width = platform_dp("playerctlSourcesLogoWidth", 480.0);
        metrics.playerctl_badge_font = platform_dp("playerctlBadgeFont", 12.5);
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
