//! Engine defaults for layout and component metrics.
//!
//! These are the numbers the layout code reads by name through `vars::px` and
//! `vars::scaled`. They sit underneath the bundled skin and any user skin, so a
//! skin overrides one by declaring `--name` in `:root` (optionally inside a
//! `@media (size: ...)` block) and otherwise inherits the value below. Design
//! tokens (colors, spacing and radius scales, text sizes, motion) stay in the
//! skin stylesheet.

/// A typed default. Rendered to the same text a skin would write.
#[derive(Clone, Copy)]
pub(super) enum V {
    /// A length in logical pixels.
    Px(f32),
    /// A unitless number: ratio, share, count, 0-255 alpha or seconds.
    Num(f32),
    /// A `clamp(min, basis%, max)` expression resolved against the element basis.
    Clamp(&'static str),
}

impl V {
    fn render(self) -> String {
        match self {
            Self::Px(value) => format!("{value}px"),
            Self::Num(value) => value.to_string(),
            Self::Clamp(text) => text.to_owned(),
        }
    }
}

/// Defaults that apply under every query, then those under one size class.
/// Later layers win, matching the order they had in the stylesheet.
pub(super) fn layers() -> Vec<(Option<&'static str>, Vec<(String, String)>)> {
    let render = |items: &[(&str, V)]| {
        items
            .iter()
            .map(|(name, value)| ((*name).to_owned(), value.render()))
            .collect::<Vec<_>>()
    };
    vec![
        (None, render(BASE)),
        (Some("compact"), render(COMPACT)),
        (Some("medium"), render(MEDIUM)),
        (Some("tv"), render(TV)),
    ]
}

const BASE: &[(&str, V)] = &[
    ("root-font-size", V::Px(16.0)),
    ("pill-ratio", V::Num(0.5)),
    ("button-border", V::Px(18.0)),
    ("background-dim", V::Px(140.0)),
    ("card-logo-scrim", V::Px(190.0)),
    ("input-focus-width", V::Px(1.5)),
    ("screen-padding", V::Px(24.0)),
    ("content-header-top-mobile", V::Px(48.0)),
    ("content-header-top", V::Px(110.0)),
    ("library-search-reserved-width", V::Px(150.0)),
    ("screen-poster-expand-seconds", V::Num(0.25)),
    ("screen-poster-expand-aspect", V::Num(1.77778)),
    ("screen-hero-landscape-switch", V::Num(1.15)),
    ("screen-hero-landscape-ratio", V::Num(0.416667)),
    ("screen-hero-landscape-max", V::Num(440.0)),
    ("screen-hero-portrait-ratio", V::Num(1.33333)),
    ("screen-hero-portrait-max", V::Num(560.0)),
    ("screen-hero-tv-height-ratio", V::Num(0.7)),
    ("screen-hero-trim-ratio", V::Num(0.93)),
    ("screen-hero-pinned-height-ratio", V::Num(0.55)),
    ("screen-hero-pinned-min-height", V::Px(300.0)),
    ("screen-hero-compact-logo-max-width", V::Px(220.0)),
    ("screen-hero-desktop-fade", V::Px(176.0)),
    ("screen-hero-width-ratio", V::Num(0.58)),
    ("screen-hero-play-compact-width", V::Px(108.0)),
    ("screen-hero-play-min-width", V::Px(160.0)),
    ("screen-hero-play-card-ratio", V::Num(0.46)),
    ("screen-hero-desktop-play-rise", V::Px(194.0)),
    ("calendar-panel-width-tv", V::Px(440.0)),
    ("settings-content-offset", V::Num(300.0)),
    ("settings-nav-width", V::Px(268.0)),
    ("settings-content-max-width-desktop", V::Px(720.0)),
    ("settings-row-label-size-desktop", V::Px(18.0)),
    ("settings-extended-line-spacing-tv", V::Px(42.0)),
    ("button-height", V::Px(44.0)),
    ("icon-button-size", V::Px(40.0)),
    ("screen-control-height", V::Px(36.0)),
    ("screen-control-radius", V::Px(9.0)),
    ("labeled-button-icon", V::Px(22.0)),
    ("labeled-button-text", V::Px(12.0)),
    ("focus-ring-expand", V::Px(3.0)),
    ("focus-ring-radius", V::Px(10.0)),
    ("focus-ring-width", V::Px(2.5)),
    ("focus-ring-slide", V::Px(0.0)),
    ("card-content-padding", V::Px(10.0)),
    ("settings-row-height", V::Px(52.0)),
    ("settings-toggle-width", V::Px(48.0)),
    ("settings-toggle-height", V::Px(26.0)),
    ("profile-page-inset-mobile", V::Px(32.0)),
    ("profile-page-inset-desktop", V::Px(56.0)),
    ("settings-swatch-alpha", V::Num(150.0)),
    ("settings-swatch-alpha-compact", V::Num(100.0)),
    ("poster-sizes-mark", V::Clamp("clamp(14px, 13%, 28px)")),
    ("poster-sizes-badge", V::Clamp("clamp(30px, 26%, 64px)")),
    ("poster-sizes-tag", V::Clamp("clamp(8px, 6.5%, 14px)")),
    ("poster-sizes-caption", V::Clamp("clamp(9px, 8.5%, 15px)")),
    ("poster-sizes-meta-bar", V::Clamp("clamp(11px, 50%, 28px)")),
    ("poster-sizes-meta-text", V::Clamp("clamp(11px, 10%, 30px)")),
    (
        "poster-sizes-landscape-text",
        V::Clamp("clamp(8px, 8%, 13px)"),
    ),
    ("poster-sizes-score-bar", V::Clamp("clamp(8px, 8%, 18px)")),
    (
        "poster-sizes-score-minimal",
        V::Clamp("clamp(8px, 7%, 16px)"),
    ),
    (
        "poster-sizes-score-number",
        V::Clamp("clamp(16px, 20%, 56px)"),
    ),
    (
        "poster-sizes-score-caption",
        V::Clamp("clamp(9px, 7.5%, 18px)"),
    ),
    ("poster-sizes-sash", V::Clamp("clamp(9px, 7%, 17px)")),
    ("paint-text-width", V::Clamp("clamp(15px, 2.2%, 26px)")),
    ("paint-text-height", V::Clamp("clamp(18px, 5.2%, 60px)")),
    ("player-panel-wide", V::Clamp("clamp(380px, 32%, 560px)")),
    ("player-panel-narrow", V::Clamp("clamp(260px, 24%, 340px)")),
    ("card-popover-min", V::Clamp("clamp(340px, 150%, 420px)")),
    ("card-popover-max", V::Clamp("clamp(400px, 240%, 500px)")),
    ("input-cell-height", V::Clamp("clamp(42px, 120%, 52px)")),
    ("glass-refraction", V::Clamp("clamp(6px, 30%, 22px)")),
    ("glass-bevel", V::Clamp("clamp(8px, 80%, 26px)")),
    ("sidebar-item-height", V::Px(44.0)),
    ("settings-toggle-knob-radius", V::Px(9.0)),
    ("calendar-grid-gap-mobile", V::Px(5.0)),
    ("calendar-grid-gap", V::Px(8.0)),
    ("calendar-cell-height-mobile", V::Px(58.0)),
    ("calendar-cell-height-desktop", V::Px(96.0)),
    ("calendar-cell-height-tv", V::Px(112.0)),
    ("control-gap", V::Px(12.0)),
    ("card-title-size", V::Px(12.0)),
    ("horizontal-card-width", V::Px(300.0)),
    ("horizontal-card-height", V::Px(180.0)),
    ("collection-poster-width", V::Px(156.0)),
    ("collection-poster-height", V::Px(234.0)),
    ("collection-wide-width", V::Px(280.0)),
    ("collection-wide-height", V::Px(158.0)),
    ("collection-square-size", V::Px(150.0)),
    ("screen-margin", V::Px(24.0)),
    ("control-cue-wrap-ratio", V::Num(0.8)),
    ("control-subtitle-wrap-ratio", V::Num(0.6)),
    ("control-subtitle-bottom-ratio", V::Num(0.06)),
    ("control-downloads-aspect", V::Num(0.5625)),
    ("control-downloads-line-ratio", V::Num(1.25)),
    ("control-downloads-card-min", V::Num(300.0)),
    ("control-downloads-row-gap-ratio", V::Num(1.5)),
    ("control-downloads-pad-ratio", V::Num(1.25)),
    ("control-code-halo", V::Px(2.5)),
    ("control-code-edge", V::Px(1.5)),
    ("control-send-stroke", V::Px(2.2)),
    ("control-search-ring-offset", V::Num(1.5)),
    ("control-search-trail-ratio", V::Num(0.6)),
    ("control-step-arm", V::Px(0.2)),
    ("control-swatch-hover-grow", V::Px(0.06)),
    ("control-row-line-ratio", V::Num(1.3)),
    ("control-segment-max-width", V::Px(120.0)),
    ("control-chip-height-ratio", V::Num(0.8)),
    ("surface-card-landscape-logo-height-share", V::Px(0.3)),
    ("surface-card-landscape-logo-width-share", V::Px(0.5)),
    ("surface-card-textless-logo-center", V::Px(0.27)),
    ("surface-card-textless-logo-height-share", V::Px(0.2)),
    ("surface-card-textless-logo-width-share", V::Px(0.76)),
    ("surface-card-logo-scrim-share", V::Num(0.5)),
    ("surface-card-preview-logo-height-share", V::Px(0.32)),
    ("surface-card-preview-logo-width-share", V::Px(0.5)),
    ("surface-ticket-narrow-width", V::Px(400.0)),
    ("toggle-knob-inset", V::Px(2.0)),
    ("caret-width", V::Px(1.6)),
    ("caret-gap", V::Px(1.0)),
    ("input-dash-stroke", V::Px(2.0)),
    ("touch-slop", V::Px(10.0)),
    ("touch-field-reach", V::Px(4.0)),
    ("player-video-radius", V::Px(18.0)),
    ("player-watch-inset", V::Px(40.0)),
    ("player-thumb-radius", V::Px(6.0)),
    ("player-thumb-backdrop", V::Num(200.0)),
    ("player-cue-lift", V::Px(150.0)),
    ("player-cue-lift-hidden", V::Px(56.0)),
    ("overlay-panel-alpha", V::Num(242.0)),
    ("shadow-alpha", V::Num(120.0)),
    ("shadow-blur", V::Px(32.0)),
    ("shadow-offset", V::Num(8.0)),
    ("touch-handle", V::Px(0.0)),
    ("calendar-cell-height", V::Px(96.0)),
    ("row-heading-size", V::Px(24.0)),
    ("nav-label-boost", V::Num(0.0)),
    ("navigation-icon", V::Px(26.0)),
    ("nav-label-size", V::Px(15.0)),
    ("screen-title-size", V::Px(34.0)),
    ("screen-body-size", V::Px(15.0)),
    ("screen-section-title-size", V::Px(20.0)),
    ("screen-card-title-size", V::Px(16.0)),
    ("screen-card-subtitle-size", V::Px(14.0)),
    ("catalog-title-size", V::Px(28.0)),
    ("horizontal-spacing", V::Px(20.0)),
    ("vertical-spacing", V::Px(24.0)),
    ("poster-height-ratio", V::Num(1.5)),
    ("horizontal-card-height-ratio", V::Num(0.56)),
    ("focused-scale", V::Num(1.12)),
    ("card-lift", V::Px(8.0)),
    ("gutter", V::Px(48.0)),
    ("poster-visible-cards", V::Num(7.0)),
    ("poster-card", V::Clamp("clamp(140px, 100%, 200px)")),
    ("continue-visible-cards", V::Num(6.2)),
    ("continue-card", V::Clamp("clamp(240px, 100%, 340px)")),
    ("sheet-handle-width", V::Px(36.0)),
    ("sheet-handle-height", V::Px(4.0)),
    ("icon-button-glyph-ratio", V::Num(0.44)),
    ("slider-track-height", V::Px(4.0)),
    ("slider-knob-radius", V::Px(6.0)),
    ("slider-mark-half", V::Px(5.0)),
    ("player-pill-mobile", V::Num(42.0)),
    ("player-icon-mobile", V::Num(42.0)),
    ("player-volume-width-mobile", V::Num(88.0)),
    ("player-stats-rows-mobile", V::Num(16.0)),
    ("player-stats-columns-mobile", V::Num(2.0)),
    ("player-margin-mobile", V::Clamp("clamp(14px, 3.5%, 64px)")),
    ("player-pill-desktop", V::Num(44.0)),
    ("player-icon-desktop", V::Num(44.0)),
    ("player-volume-width-desktop", V::Num(88.0)),
    ("player-stats-rows-desktop", V::Num(26.0)),
    ("player-stats-columns-desktop", V::Num(3.0)),
    ("player-margin-desktop", V::Clamp("clamp(22px, 3.5%, 64px)")),
    ("player-pill-tv", V::Num(60.0)),
    ("player-icon-tv", V::Num(56.0)),
    ("player-volume-width-tv", V::Num(128.0)),
    ("player-stats-rows-tv", V::Num(20.0)),
    ("player-stats-columns-tv", V::Num(3.0)),
    ("player-margin-tv", V::Clamp("clamp(48px, 5%, 4096px)")),
    ("navigation-rail-width-desktop", V::Px(76.0)),
    ("navigation-rail-expanded-width-desktop", V::Px(232.0)),
    ("navigation-rail-width-tv", V::Px(104.0)),
    ("navigation-rail-expanded-width-tv", V::Px(300.0)),
    ("tv-zoom", V::Px(2.0)),
];

const COMPACT: &[(&str, V)] = &[
    ("content-header-top", V::Px(48.0)),
    ("settings-content-offset", V::Num(220.0)),
    ("settings-nav-width", V::Px(196.0)),
    ("settings-row-height", V::Px(46.0)),
    ("settings-toggle-width", V::Px(52.0)),
    ("settings-toggle-height", V::Px(32.0)),
    ("poster-sizes-tag", V::Clamp("clamp(10px, 6.5%, 14px)")),
    ("poster-sizes-sash", V::Clamp("clamp(10px, 7%, 17px)")),
    ("settings-toggle-knob-radius", V::Px(13.0)),
    ("calendar-grid-gap", V::Px(5.0)),
    ("horizontal-card-width", V::Px(244.0)),
    ("horizontal-card-height", V::Px(148.0)),
    ("collection-poster-width", V::Px(117.0)),
    ("collection-poster-height", V::Px(176.0)),
    ("collection-wide-width", V::Px(210.0)),
    ("collection-wide-height", V::Px(119.0)),
    ("collection-square-size", V::Px(113.0)),
    ("screen-margin", V::Px(16.0)),
    ("touch-handle", V::Px(24.0)),
    ("calendar-cell-height", V::Px(58.0)),
    ("row-heading-size", V::Px(20.0)),
    ("nav-label-size", V::Px(13.0)),
    ("screen-title-size", V::Px(32.0)),
    ("screen-card-title-size", V::Px(15.0)),
    ("screen-card-subtitle-size", V::Px(12.0)),
    ("catalog-title-size", V::Px(24.0)),
    ("horizontal-spacing", V::Px(12.0)),
    ("vertical-spacing", V::Px(12.0)),
    ("gutter", V::Px(16.0)),
    ("poster-visible-cards", V::Num(2.6)),
    ("poster-card", V::Clamp("clamp(110px, 100%, 170px)")),
    ("continue-visible-cards", V::Num(1.6)),
    ("continue-card", V::Clamp("clamp(180px, 100%, 240px)")),
];

const MEDIUM: &[(&str, V)] = &[
    ("row-heading-size", V::Px(22.0)),
    ("nav-label-size", V::Px(14.0)),
    ("catalog-title-size", V::Px(26.0)),
    ("horizontal-spacing", V::Px(16.0)),
    ("vertical-spacing", V::Px(20.0)),
];

const TV: &[(&str, V)] = &[
    ("settings-content-offset", V::Num(220.0)),
    ("settings-nav-width", V::Px(196.0)),
    ("button-height", V::Px(56.0)),
    ("icon-button-size", V::Px(52.0)),
    ("focus-ring-slide", V::Px(0.17)),
    ("settings-row-height", V::Px(46.0)),
    ("settings-toggle-width", V::Px(52.0)),
    ("settings-toggle-height", V::Px(32.0)),
    ("sidebar-item-height", V::Px(56.0)),
    ("settings-toggle-knob-radius", V::Px(13.0)),
    ("horizontal-card-width", V::Px(356.0)),
    ("horizontal-card-height", V::Px(208.0)),
    ("screen-margin", V::Px(32.0)),
    ("calendar-cell-height", V::Px(112.0)),
    ("row-heading-size", V::Px(28.0)),
    ("nav-label-boost", V::Num(3.0)),
    ("navigation-icon", V::Px(25.0)),
    ("screen-title-size", V::Px(42.0)),
    ("screen-body-size", V::Px(18.0)),
    ("screen-card-title-size", V::Px(17.0)),
    ("screen-card-subtitle-size", V::Px(12.0)),
    ("gutter", V::Px(64.0)),
    ("poster-visible-cards", V::Num(6.0)),
    ("poster-card", V::Clamp("clamp(120px, 100%, 180px)")),
    ("continue-visible-cards", V::Num(6.5)),
    ("continue-card", V::Clamp("clamp(240px, 100%, 320px)")),
];

#[cfg(test)]
mod tests {
    use super::{V, layers};

    #[test]
    fn values_render_like_skin_text() {
        assert_eq!(V::Px(18.0).render(), "18px");
        assert_eq!(V::Px(1.5).render(), "1.5px");
        assert_eq!(V::Num(0.5).render(), "0.5");
        assert_eq!(
            V::Clamp("clamp(1px, 2%, 3px)").render(),
            "clamp(1px, 2%, 3px)"
        );
    }

    #[test]
    fn size_layers_only_restate_base_names() {
        let all = layers();
        let base = &all[0].1;
        for (key, vars) in &all[1..] {
            for (name, _) in vars {
                assert!(
                    base.iter().any(|(known, _)| known == name),
                    "{key:?} overrides --{name} that has no base default"
                );
            }
        }
    }
}
