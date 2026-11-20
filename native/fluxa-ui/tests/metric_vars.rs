use fluxa_ui::{UiFormFactor, UiMetrics, Viewport, reload_styles};

#[test]
fn css_root_metric_vars_override_token_values() {
    let viewport = Viewport::new(1280, 800, UiFormFactor::Desktop);
    reload_styles(":root { --metric-focus-ring-slide-ms: 250; }").unwrap();
    let after = UiMetrics::for_viewport(viewport).focus_ring_slide;
    reload_styles("").unwrap();
    let restored = UiMetrics::for_viewport(viewport).focus_ring_slide;
    assert_eq!(after, 0.25);
    assert_ne!(restored, after);
}
